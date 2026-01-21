-- offline-relay.lua
-- Three nodes for testing store-and-forward relay
--
-- Automated Test Flow:
-- 1. All nodes start (topology = none, no auto-connect)
-- 2. Love and Joy connect (Peace stays disconnected)
-- 3. Love sends Peace a message via Joy (Joy stores it)
-- 4. Peace connects to Joy
-- 5. Peace should receive Love's relayed message
--
-- Topology (orchestrated):
--   love ─── joy ─── peace

scenario {
    name = "offline-relay",
    description = "Three nodes for relay testing",

    instances = {
        {name = "love", profile = "Love"},
        {name = "joy", profile = "Joy"},
        {name = "peace", profile = "Peace"},
    },

    -- No auto-connect - we orchestrate connections manually
    topology = "none",

    on_start = function(ctx)
        ctx.log("=== Offline Relay Test ===")
        ctx.log("Topology: love ─── joy ─── peace")
        ctx.log("")

        -- Step 1: Connect Love <-> Joy (Peace stays offline)
        -- Wait 2s for all instances to write their .invite files
        ctx.after(2.0, function()
            ctx.log("Step 1: Connecting Joy to Love...")
            ctx.connect("love", "joy")  -- Joy connects to Love's invite
        end)

        -- Step 2: After connection stabilizes, Love sends message to Peace
        -- Wait 4s for the connection to be established (watcher polls every 1s)
        ctx.after(6.0, function()
            ctx.log("Step 2: Love sending message to Peace (via Joy relay)...")
            ctx.send_packet("love", "peace", "Hello Peace! Relayed through Joy.")
        end)

        -- Step 3: Connect Peace to Joy (should trigger relay forwarding)
        ctx.after(10.0, function()
            ctx.log("Step 3: Connecting Peace to Joy...")
            ctx.log("(Peace should receive Love's relayed message)")
            ctx.connect("joy", "peace")  -- Peace connects to Joy's invite
        end)

        -- Step 4: Log completion
        ctx.after(15.0, function()
            ctx.log("")
            ctx.log("=== Test Complete ===")
            ctx.log("Check Peace's instance - it should have received Love's message")
            ctx.log("Press Ctrl+C to stop all instances")
        end)
    end,
}
