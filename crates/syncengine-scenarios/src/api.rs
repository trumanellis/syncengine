//! Lua API bindings for scenario scripts.
//!
//! Exposes functions like launch(), kill(), connect(), after(), etc. to Lua.

#![allow(dead_code)]

use mlua::{Function, Lua, Result as LuaResult, Table};
use std::time::Duration;

use crate::instance::SharedInstanceManager;
use crate::scheduler::SharedScheduler;

/// Create the `ctx` table that gets passed to scenario callbacks
pub fn create_context_table(
    lua: &Lua,
    instances: SharedInstanceManager,
    scheduler: SharedScheduler,
) -> LuaResult<Table> {
    let ctx = lua.create_table()?;

    // ctx.launch(name, opts)
    // opts can include:
    //   - profile (string): Profile name for the instance
    //   - connect_to (array): List of instance names to auto-connect to
    //   - total (number): Expected total instance count for proper window tiling
    let instances_clone = instances.clone();
    let launch_fn = lua.create_function(move |_lua, args: (String, Option<Table>)| {
        let (name, opts) = args;

        let profile = opts
            .as_ref()
            .and_then(|t| t.get::<String>("profile").ok())
            .unwrap_or_else(|| capitalize(&name));

        // Get connect_to peers if specified
        let connect_peers: Option<Vec<String>> = opts.as_ref().and_then(|t| {
            t.get::<Vec<String>>("connect_to").ok()
        });

        // Get expected total for proper window tiling (for dynamic scenarios)
        let total_expected: Option<u8> = opts.as_ref().and_then(|t| {
            t.get::<u8>("total").ok()
        });

        let instances = instances_clone.clone();

        // Use block_in_place to safely block from within async context
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut mgr = instances.write().await;
                mgr.launch_with_connect(&name, &profile, connect_peers.clone(), total_expected)
                    .map_err(|e| mlua::Error::runtime(e.to_string()))
            })
        })?;

        if let Some(ref peers) = connect_peers {
            tracing::info!(name = %name, profile = %profile, connect_to = ?peers, "Launched instance with auto-connect");
        } else {
            tracing::info!(name = %name, profile = %profile, "Launched instance");
        }
        Ok(())
    })?;
    ctx.set("launch", launch_fn)?;

    // ctx.kill(name)
    let instances_clone = instances.clone();
    let kill_fn = lua.create_function(move |_, name: String| {
        let instances = instances_clone.clone();
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut mgr = instances.write().await;
                mgr.kill(&name)
                    .map_err(|e| mlua::Error::runtime(e.to_string()))
            })
        })?;
        Ok(())
    })?;
    ctx.set("kill", kill_fn)?;

    // ctx.restart(name)
    let instances_clone = instances.clone();
    let restart_fn = lua.create_function(move |_, name: String| {
        let instances = instances_clone.clone();
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut mgr = instances.write().await;
                mgr.restart(&name)
                    .map_err(|e| mlua::Error::runtime(e.to_string()))
            })
        })?;
        Ok(())
    })?;
    ctx.set("restart", restart_fn)?;

    // ctx.connect(a, b) - Connect instance B to instance A's invite
    // This reads A's .invite file and writes it to B's .connect file
    // Instance B's bootstrap watcher will detect the .connect file and process it
    let instances_clone = instances.clone();
    let connect_fn = lua.create_function(move |_, (a, b): (String, String)| {
        let instances = instances_clone.clone();
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mgr = instances.read().await;
                let bootstrap_dir = mgr.bootstrap_dir();

                // Read A's invite file
                let a_invite_path = bootstrap_dir.join(format!("{}.invite", a.to_lowercase()));
                let invite_str = std::fs::read_to_string(&a_invite_path)
                    .map_err(|e| mlua::Error::runtime(format!(
                        "Cannot read {}.invite: {} (is '{}' running?)", a, e, a
                    )))?;

                // Write to B's connect file (B's watcher will pick it up)
                let b_connect_path = bootstrap_dir.join(format!("{}.connect", b.to_lowercase()));
                std::fs::write(&b_connect_path, &invite_str)
                    .map_err(|e| mlua::Error::runtime(format!(
                        "Cannot write {}.connect: {}", b, e
                    )))?;

                tracing::info!(
                    from = %a,
                    to = %b,
                    invite_path = %a_invite_path.display(),
                    connect_path = %b_connect_path.display(),
                    "Wrote connection request to {}.connect", b
                );

                Ok::<_, mlua::Error>(())
            })
        })?;
        Ok(())
    })?;
    ctx.set("connect", connect_fn)?;

    // ctx.connect_mesh(instances...) - Connect all instances to each other
    let connect_mesh_fn = lua.create_function(move |_, names: Vec<String>| {
        tracing::info!(instances = ?names, "Creating mesh topology");
        // Each pair gets connected
        for i in 0..names.len() {
            for j in (i + 1)..names.len() {
                tracing::info!(a = %names[i], b = %names[j], "Mesh connection");
            }
        }
        Ok(())
    })?;
    ctx.set("connect_mesh", connect_mesh_fn)?;

    // ctx.connect_to_all(name) - Connect name to all other running instances
    let instances_clone = instances.clone();
    let connect_to_all_fn = lua.create_function(move |_, name: String| {
        let instances = instances_clone.clone();
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mgr = instances.read().await;
                let all_instances = mgr.list_instances();
                for other in all_instances {
                    if other != name {
                        tracing::info!(from = %name, to = %other, "Connect to all");
                    }
                }
                Ok::<_, mlua::Error>(())
            })
        })?;
        Ok(())
    })?;
    ctx.set("connect_to_all", connect_to_all_fn)?;

    // ctx.after(seconds, callback)
    let scheduler_clone = scheduler.clone();
    let after_fn = lua.create_function(move |lua, (seconds, callback): (f64, Function)| {
        let scheduler = scheduler_clone.clone();
        let delay = Duration::from_secs_f64(seconds);

        // Store callback in registry so it survives
        let key = lua.create_registry_value(callback)?;

        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut sched = scheduler.write().await;
                sched.schedule_after(delay, key);
            });
        });

        tracing::debug!(seconds = seconds, "Scheduled after callback");
        Ok(())
    })?;
    ctx.set("after", after_fn)?;

    // ctx.every(seconds, callback)
    let scheduler_clone = scheduler.clone();
    let every_fn = lua.create_function(move |lua, (seconds, callback): (f64, Function)| {
        let scheduler = scheduler_clone.clone();
        let interval = Duration::from_secs_f64(seconds);

        let key = lua.create_registry_value(callback)?;

        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut sched = scheduler.write().await;
                sched.schedule_every(interval, key);
            });
        });

        tracing::debug!(seconds = seconds, "Scheduled every callback");
        Ok(())
    })?;
    ctx.set("every", every_fn)?;

    // ctx.random(min, max)
    let random_fn = lua.create_function(|_, (min, max): (i32, i32)| {
        use rand::Rng;
        let mut rng = rand::rng();
        Ok(rng.random_range(min..=max))
    })?;
    ctx.set("random", random_fn)?;

    // ctx.random_instance()
    let instances_clone = instances.clone();
    let random_instance_fn = lua.create_function(move |_, ()| {
        let instances = instances_clone.clone();
        let result = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mgr = instances.read().await;
                mgr.random_instance()
            })
        });
        Ok(result)
    })?;
    ctx.set("random_instance", random_instance_fn)?;

    // ctx.log(message)
    let log_fn = lua.create_function(|_, msg: String| {
        tracing::info!(target: "scenario", "{}", msg);
        Ok(())
    })?;
    ctx.set("log", log_fn)?;

    // ctx.sleep(seconds) - Synchronous sleep
    let sleep_fn = lua.create_function(|_, seconds: f64| {
        std::thread::sleep(Duration::from_secs_f64(seconds));
        Ok(())
    })?;
    ctx.set("sleep", sleep_fn)?;

    // ctx.instances() - List all instance names
    let instances_clone = instances.clone();
    let list_fn = lua.create_function(move |_, ()| {
        let instances = instances_clone.clone();
        let result = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mgr = instances.read().await;
                mgr.list_instances()
            })
        });
        Ok(result)
    })?;
    ctx.set("instances", list_fn)?;

    // ctx.send_packet(from_node, to_node, content)
    // Simulates sending a DirectMessage packet from one instance to another
    // This writes a packet file that can be checked by check_received
    let instances_clone = instances.clone();
    let send_packet_fn = lua.create_function(move |_, (from, to, content): (String, String, String)| {
        let instances = instances_clone.clone();

        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mgr = instances.read().await;

                // Verify from instance exists
                if !mgr.is_running(&from) {
                    return Err(mlua::Error::runtime(format!("Instance '{}' not running", from)));
                }

                // Get data directories
                let from_data_dir = mgr.get_data_dir(&from)
                    .ok_or_else(|| mlua::Error::runtime(format!("No data dir for '{}'", from)))?;

                // Create "outbox" directory in sender's data dir
                let outbox_dir = from_data_dir.join("outbox");
                std::fs::create_dir_all(&outbox_dir)
                    .map_err(|e| mlua::Error::runtime(format!("Failed to create outbox dir: {}", e)))?;

                // Write packet file with timestamp to avoid collisions
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| mlua::Error::runtime(e.to_string()))?
                    .as_millis();

                let packet_filename = format!("{}_{}_to_{}.packet", timestamp, from, to);
                let packet_path = outbox_dir.join(&packet_filename);

                // Write packet data (can be enhanced with actual packet serialization later)
                let packet_data = format!("FROM: {}\nTO: {}\nTIMESTAMP: {}\nCONTENT: {}", from, to, timestamp, content);
                std::fs::write(&packet_path, packet_data)
                    .map_err(|e| mlua::Error::runtime(format!("Failed to write packet: {}", e)))?;

                tracing::info!(
                    from = %from,
                    to = %to,
                    content = %content,
                    path = %packet_path.display(),
                    "Packet written to outbox"
                );

                Ok::<_, mlua::Error>(())
            })
        })?;

        tracing::info!(from = %from, to = %to, "send_packet: packet queued for delivery");
        Ok(())
    })?;
    ctx.set("send_packet", send_packet_fn)?;

    // ctx.check_received(node, from_node, content_substring)
    // Returns true if node has received a packet from from_node containing content_substring
    let instances_clone = instances.clone();
    let check_received_fn = lua.create_function(move |_, (node, from, content_substring): (String, String, String)| {
        let instances = instances_clone.clone();

        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mgr = instances.read().await;

                // Get node's data directory
                let node_data_dir = mgr.get_data_dir(&node)
                    .ok_or_else(|| mlua::Error::runtime(format!("No data dir for '{}'", node)))?;

                // Check "inbox" directory (packets forwarded to this node)
                let inbox_dir = node_data_dir.join("inbox");

                if !inbox_dir.exists() {
                    tracing::debug!(node = %node, "No inbox directory yet");
                    return Ok(false);
                }

                // Search for packets from the specified sender containing the substring
                let entries = std::fs::read_dir(&inbox_dir)
                    .map_err(|e| mlua::Error::runtime(format!("Failed to read inbox dir: {}", e)))?;

                for entry in entries {
                    let entry = entry.map_err(|e| mlua::Error::runtime(e.to_string()))?;
                    let path = entry.path();

                    // Check if filename indicates it's from the expected sender
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        // Filename format: "{timestamp}_{from}_to_{to}.packet"
                        if !filename.contains(&format!("{}_to_", from)) {
                            continue;
                        }

                        // Read and check content
                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                if content.contains(&content_substring) {
                                    tracing::info!(
                                        node = %node,
                                        from = %from,
                                        filename = %filename,
                                        "Found matching packet in inbox"
                                    );
                                    return Ok(true);
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to read packet {}: {}", filename, e);
                            }
                        }
                    }
                }

                // Also check outbox of sender (for packets that haven't been relayed yet)
                let sender_data_dir = mgr.get_data_dir(&from);
                if let Some(sender_dir) = sender_data_dir {
                    let outbox_dir = sender_dir.join("outbox");
                    if outbox_dir.exists() {
                        if let Ok(entries) = std::fs::read_dir(&outbox_dir) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                                    if filename.contains(&format!("_to_{}", node)) {
                                        if let Ok(content) = std::fs::read_to_string(&path) {
                                            if content.contains(&content_substring) {
                                                tracing::info!(
                                                    node = %node,
                                                    from = %from,
                                                    "Found packet in sender's outbox (pending relay)"
                                                );
                                                // Found in outbox but not relayed yet
                                                return Ok(false);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                Ok(false)
            })
        })
    })?;
    ctx.set("check_received", check_received_fn)?;

    Ok(ctx)
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}
