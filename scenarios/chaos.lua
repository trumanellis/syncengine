-- chaos.lua
-- Random node failures and restarts to test resilience
--
-- Behavior:
--   - Starts 5 nodes in mesh topology
--   - Randomly kills nodes every 10-30 seconds
--   - Restarts killed nodes after 5 seconds
--   - Tests network resilience and reconnection

scenario {
    name = "chaos",
    description = "Random node failures to test network resilience",

    instances = {
        {name = "faith", profile = "Faith"},
        {name = "hope", profile = "Hope"},
        {name = "grace", profile = "Grace"},
        {name = "truth", profile = "Truth"},
        {name = "wisdom", profile = "Wisdom"},
    },

    topology = "mesh",

    on_running = function(ctx)
        ctx.log("Chaos mode activated - random failures will occur")

        -- Schedule random failures
        local function schedule_chaos()
            local delay = random(10, 30)
            ctx.after(delay, function()
                local victim = ctx.random_instance()
                if victim then
                    ctx.log("CHAOS: Killing " .. victim)
                    ctx.kill(victim)

                    -- Restart after 5 seconds
                    ctx.after(5, function()
                        ctx.log("CHAOS: Restarting " .. victim)
                        ctx.restart(victim)
                    end)
                end

                -- Schedule next chaos event
                schedule_chaos()
            end)
        end

        -- Start the chaos loop
        schedule_chaos()
    end
}
