-- offline-relay.lua
-- Store-and-forward relay test with REAL offline behavior
--
-- This scenario demonstrates message delivery when the recipient goes offline:
-- 1. All three nodes start and establish connections (full mesh for direct messaging)
-- 2. Peace goes OFFLINE (killed)
-- 3. Love sends a message to Peace (stored locally, waiting for delivery)
-- 4. Love goes OFFLINE too (sender is now gone!)
-- 5. Peace comes back ONLINE (restarted)
-- 6. Peace should receive Love's message (via sync or relay)
--
-- Topology: Full mesh (all connected to all) so Love can resolve Peace's name
--
-- Timeline:
--   t=0s:   Launch all instances
--   t=3s:   Start mesh connections (6 total: each pair connects bidirectionally)
--   t=12s:  Kill Peace (goes offline)
--   t=16s:  Love sends message to Peace
--   t=18s:  Kill Love (sender goes offline)
--   t=22s:  Peace restarts
--   t=30s:  Test complete

scenario {
    name = "offline-relay",
    description = "Store-and-forward with both sender and recipient going offline",

    instances = {
        {name = "love", profile = "Love"},
        {name = "joy", profile = "Joy"},
        {name = "peace", profile = "Peace"},
    },

    -- Use mesh topology for auto-connect - ensures all nodes know each other
    topology = "mesh",

    on_start = function(ctx)
        ctx.log("=== Offline Relay Test ===")
        ctx.log("")
        ctx.log("This test demonstrates message delivery with offline nodes:")
        ctx.log("  - All nodes connect in a mesh (so Love knows Peace)")
        ctx.log("  - Peace goes offline")
        ctx.log("  - Love sends a message to Peace")
        ctx.log("  - Love ALSO goes offline")
        ctx.log("  - Peace comes back and receives the message")
        ctx.log("")

        -- Phase 1: Wait for mesh connections to establish
        -- topology = "mesh" handles auto-connect, but we need time for exchanges
        ctx.after(10.0, function()
            ctx.log("")
            ctx.log("Phase 1: Connections should be established")
            ctx.log("         Waiting 2 more seconds to ensure stability...")
        end)

        -- Phase 2: Kill Peace (simulate going offline)
        ctx.after(12.0, function()
            ctx.log("")
            ctx.log("Phase 2: Peace is going OFFLINE...")
            ctx.log("         (Killing Peace instance)")
            ctx.kill("peace")
        end)

        -- Phase 3: Love sends message while Peace is offline
        ctx.after(16.0, function()
            ctx.log("")
            ctx.log("Phase 3: Love sending message to Peace...")
            ctx.log("         (Peace is OFFLINE - message stored for later)")
            ctx.send_packet("love", "peace", "Hello Peace! You were offline when I sent this. ~Love")
        end)

        -- Phase 4: Kill Love too (sender goes offline!)
        ctx.after(18.0, function()
            ctx.log("")
            ctx.log("Phase 4: Love is ALSO going OFFLINE...")
            ctx.log("         (Killing Love instance - sender is now gone!)")
            ctx.log("         Only Joy remains online.")
            ctx.kill("love")
        end)

        -- Phase 5: Peace comes back online
        ctx.after(22.0, function()
            ctx.log("")
            ctx.log("Phase 5: Peace is coming back ONLINE...")
            ctx.log("         (Restarting Peace instance)")
            ctx.log("         Note: Love is still offline!")
            ctx.restart("peace")
        end)

        -- Phase 6: Wait for sync and show results
        ctx.after(30.0, function()
            ctx.log("")
            ctx.log("=== Test Complete ===")
            ctx.log("")
            ctx.log("Expected result:")
            ctx.log("  1. Peace was offline when Love sent the message")
            ctx.log("  2. Message was stored (locally or on Joy)")
            ctx.log("  3. Love went offline BEFORE Peace came back")
            ctx.log("  4. When Peace reconnected and synced, it received the message")
            ctx.log("  5. Peace should see Love's message - even though Love is gone!")
            ctx.log("")
            ctx.log("Check Peace's chat to verify!")
            ctx.log("")
            ctx.log("Press Ctrl+C to stop remaining instances")
        end)
    end,
}
