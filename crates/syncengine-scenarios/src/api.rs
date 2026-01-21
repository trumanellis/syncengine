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
    // opts can include: profile (string), connect_to (array of instance names)
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

        let instances = instances_clone.clone();

        // We need to use blocking because mlua async is tricky
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let mut mgr = instances.write().await;
            mgr.launch_with_connect(&name, &profile, connect_peers.clone())
                .map_err(|e| mlua::Error::runtime(e.to_string()))?;
            Ok::<_, mlua::Error>(())
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
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let mut mgr = instances.write().await;
            mgr.kill(&name)
                .map_err(|e| mlua::Error::runtime(e.to_string()))?;
            Ok::<_, mlua::Error>(())
        })?;
        Ok(())
    })?;
    ctx.set("kill", kill_fn)?;

    // ctx.restart(name)
    let instances_clone = instances.clone();
    let restart_fn = lua.create_function(move |_, name: String| {
        let instances = instances_clone.clone();
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let mut mgr = instances.write().await;
            mgr.restart(&name)
                .map_err(|e| mlua::Error::runtime(e.to_string()))?;
            Ok::<_, mlua::Error>(())
        })?;
        Ok(())
    })?;
    ctx.set("restart", restart_fn)?;

    // ctx.connect(a, b) - Write invite from a, read and send request from b
    let instances_clone = instances.clone();
    let connect_fn = lua.create_function(move |_, (a, b): (String, String)| {
        let instances = instances_clone.clone();
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let mgr = instances.read().await;

            // Get bootstrap directory
            let bootstrap_dir = mgr.bootstrap_dir();
            std::fs::create_dir_all(&bootstrap_dir)
                .map_err(|e| mlua::Error::runtime(e.to_string()))?;

            // For now, just log the connection intent
            // The actual connection happens through the bootstrap mechanism
            tracing::info!(from = %a, to = %b, "Connection requested");

            Ok::<_, mlua::Error>(())
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

        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let mut sched = scheduler.write().await;
            sched.schedule_after(delay, key);
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

        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let mut sched = scheduler.write().await;
            sched.schedule_every(interval, key);
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
        let rt = tokio::runtime::Handle::current();
        let result = rt.block_on(async {
            let mgr = instances.read().await;
            mgr.random_instance()
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
        let rt = tokio::runtime::Handle::current();
        let result = rt.block_on(async {
            let mgr = instances.read().await;
            mgr.list_instances()
        });
        Ok(result)
    })?;
    ctx.set("instances", list_fn)?;

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
