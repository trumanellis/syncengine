-- offline-relay.lua
-- Store-and-forward relay test with REAL offline behavior
--
-- Forked from mesh-messaging.lua - uses same reliable mesh topology
--
-- Topology (full mesh):
--   love --- joy
--     \     /
--      \   /
--      peace
--
-- Test sequence:
--   1. All three nodes connect (full mesh)
--   2. Peace goes OFFLINE (killed)
--   3. Love sends a message to Peace (stored, waiting for delivery)
--   4. Love goes OFFLINE too (sender is now gone!)
--   5. Peace comes back ONLINE (restarted)
--   6. Peace should receive Love's message (via Joy relay)

scenario {
    name = "offline-relay",
    description = "Store-and-forward with sender and recipient going offline",

    instances = {
        {name = "love", profile = "Love"},
        {name = "joy", profile = "Joy"},
        {name = "peace", profile = "Peace"},
    },

    -- Full mesh - everyone connected to everyone
    topology = "mesh",

    on_start = function(ctx)
        ctx.log("=== Offline Relay Test (Full Mesh) ===")
        ctx.log("Topology: love <-> joy <-> peace <-> love")
        ctx.log("All nodes start connected to each other")
        ctx.log("")

        -- Wait for mesh connections to establish (same as mesh-messaging)
        ctx.after(8.0, function()
            ctx.log("Phase 1: Mesh established, killing Peace...")
            ctx.kill("peace")
        end)

        ctx.after(12.0, function()
            ctx.log("Phase 2: Love sending message to Peace (who is offline)...")
            ctx.send_packet("love", "peace", "Hello Peace! You were offline when I sent this. ~Love")
        end)

        ctx.after(24.0, function()
            ctx.log("Phase 3: Killing Love (message should have synced to Joy)...")
            ctx.kill("love")
        end)

        ctx.after(28.0, function()
            ctx.log("Phase 4: Restarting Peace...")
            ctx.restart("peace")
        end)

        ctx.after(40.0, function()
            ctx.log("")
            ctx.log("=== Test Complete ===")
            ctx.log("Expected: Peace received Love's message via Joy")
            ctx.log("Check Peace's Network tab for the message")
            ctx.log("Press Ctrl+C to stop")
        end)
    end,
}
