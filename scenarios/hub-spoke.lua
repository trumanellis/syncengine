-- hub-spoke.lua
-- Central hub with satellite nodes
--
-- Topology:
--        harmony
--           |
--   balance-+-unity-+-serenity
--           |
--        clarity

scenario {
    name = "hub-spoke",
    description = "Central hub with satellite nodes",

    on_start = function(ctx)
        -- Unity is the hub (center)
        ctx.launch("unity", {profile = "Unity"})
        ctx.log("Hub 'unity' launched")

        -- Spokes connect only to hub
        local spokes = {"harmony", "balance", "serenity", "clarity"}

        for i, name in ipairs(spokes) do
            ctx.after(i * 2, function()
                ctx.launch(name, {profile = capitalize(name)})
                ctx.connect(name, "unity")
                ctx.log("Spoke '" .. name .. "' connected to hub")
            end)
        end
    end
}
