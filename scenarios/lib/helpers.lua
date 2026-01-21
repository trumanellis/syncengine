-- Scenario Helper Library
-- Common functions for scenario scripts

-- Generate a list of instance configurations
-- Usage: generate_instances(3) -> {{name="node1", profile="Node1"}, ...}
function generate_instances(count, prefix)
    prefix = prefix or "node"
    local instances = {}
    for i = 1, count do
        local name = prefix .. i
        table.insert(instances, {
            name = name,
            profile = capitalize(name)
        })
    end
    return instances
end

-- Create mesh topology connections (all-to-all)
-- Usage: mesh_topology(ctx, {"love", "joy", "peace"})
function mesh_topology(ctx, names)
    ctx.log("Creating mesh topology for " .. #names .. " nodes")
    for i = 1, #names do
        for j = i + 1, #names do
            ctx.connect(names[i], names[j])
        end
    end
end

-- Create hub-spoke topology (hub connects to all spokes)
-- Usage: hub_spoke_topology(ctx, "hub", {"spoke1", "spoke2", "spoke3"})
function hub_spoke_topology(ctx, hub, spokes)
    ctx.log("Creating hub-spoke topology: " .. hub .. " -> " .. #spokes .. " spokes")
    for _, spoke in ipairs(spokes) do
        ctx.connect(hub, spoke)
    end
end

-- Create chain topology (A -> B -> C -> D)
-- Usage: chain_topology(ctx, {"a", "b", "c", "d"})
function chain_topology(ctx, names)
    ctx.log("Creating chain topology for " .. #names .. " nodes")
    for i = 1, #names - 1 do
        ctx.connect(names[i], names[i + 1])
    end
end

-- Create ring topology (A -> B -> C -> D -> A)
-- Usage: ring_topology(ctx, {"a", "b", "c", "d"})
function ring_topology(ctx, names)
    ctx.log("Creating ring topology for " .. #names .. " nodes")
    chain_topology(ctx, names)
    if #names > 2 then
        ctx.connect(names[#names], names[1])
    end
end

-- Launch instances with staggered delays
-- Usage: staggered_launch(ctx, instances, 2) -- 2 second delay between each
function staggered_launch(ctx, instances, delay_seconds)
    delay_seconds = delay_seconds or 1
    for i, inst in ipairs(instances) do
        ctx.after((i - 1) * delay_seconds, function()
            ctx.launch(inst.name, inst)
        end)
    end
end

-- Sacred names for SyncEngine instances
SACRED_NAMES = {
    "love", "joy", "peace", "patience", "kindness",
    "goodness", "faithfulness", "gentleness", "self-control",
    "wisdom", "truth", "grace", "hope", "faith",
    "unity", "harmony", "balance", "serenity", "clarity"
}

-- Get random sacred names
-- Usage: random_sacred_names(5) -> {"love", "grace", "peace", "hope", "joy"}
function random_sacred_names(count)
    local available = {}
    for _, name in ipairs(SACRED_NAMES) do
        table.insert(available, name)
    end

    local selected = {}
    for i = 1, math.min(count, #available) do
        local idx = random(1, #available)
        table.insert(selected, available[idx])
        table.remove(available, idx)
    end

    return selected
end

-- Create instances from sacred names
-- Usage: sacred_instances(3) -> instances with random sacred names
function sacred_instances(count)
    local names = random_sacred_names(count)
    local instances = {}
    for _, name in ipairs(names) do
        table.insert(instances, {
            name = name,
            profile = capitalize(name)
        })
    end
    return instances
end

-- Print scenario banner
function banner(name, description)
    print("╔════════════════════════════════════════════════════════════╗")
    print("║  SCENARIO: " .. string.upper(name) .. string.rep(" ", 48 - #name) .. "║")
    if description then
        print("║  " .. description .. string.rep(" ", 58 - #description) .. "║")
    end
    print("╚════════════════════════════════════════════════════════════╝")
end

-- Return module table for require()
return {
    generate_instances = generate_instances,
    mesh_topology = mesh_topology,
    hub_spoke_topology = hub_spoke_topology,
    chain_topology = chain_topology,
    ring_topology = ring_topology,
    staggered_launch = staggered_launch,
    random_sacred_names = random_sacred_names,
    sacred_instances = sacred_instances,
    banner = banner,
    SACRED_NAMES = SACRED_NAMES,
}
