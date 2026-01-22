-- offline-relay.lua
-- Store-and-forward relay test with REAL offline behavior
--
-- This scenario demonstrates true store-and-forward:
-- 1. All three nodes start and establish connections
-- 2. Peace goes OFFLINE (killed)
-- 3. Love sends a message to Peace (Joy stores it)
-- 4. Love goes OFFLINE too (killed) - sender is now gone!
-- 5. Peace comes back ONLINE (restarted)
-- 6. Joy forwards the stored message to Peace (even though Love is gone)
--
-- Topology Timeline:
--   t=0s:   [love] ─── [joy] ─── [peace]  (all connected)
--   t=10s:  [love] ─── [joy]    [PEACE]   (peace killed)
--   t=14s:  Love sends message to Peace
--   t=16s:  [LOVE]     [joy]    [PEACE]   (love killed - sender gone!)
--   t=20s:  [LOVE]     [joy] ─── [peace]  (peace restarted)
--   t=27s:  Peace receives relayed message (from Joy, even though Love is gone)

scenario {
    name = "offline-relay",
    description = "Store-and-forward with both sender and recipient going offline",

    instances = {
        {name = "love", profile = "Love"},
        {name = "joy", profile = "Joy"},
        {name = "peace", profile = "Peace"},
    },

    -- Manual topology - we orchestrate hub-and-spoke through Joy
    topology = "none",

    on_start = function(ctx)
        ctx.log("=== Offline Relay Test ===")
        ctx.log("")
        ctx.log("This test demonstrates TRUE store-and-forward:")
        ctx.log("  - Peace goes offline")
        ctx.log("  - Love sends a message (Joy stores it)")
        ctx.log("  - Love ALSO goes offline")
        ctx.log("  - Peace comes back and receives the message from Joy")
        ctx.log("  - (Even though Love is no longer online!)")
        ctx.log("")

        -- Phase 1: Establish connections (hub-and-spoke through Joy)
        ctx.after(2.0, function()
            ctx.log("Phase 1: Establishing connections...")
            ctx.log("         Creating hub-and-spoke: Love <-> Joy <-> Peace")
            ctx.connect("joy", "love")   -- Love connects to Joy
            ctx.connect("joy", "peace")  -- Peace connects to Joy
        end)

        ctx.after(4.0, function()
            -- Reverse connections for bidirectional contacts
            ctx.connect("love", "joy")
            ctx.connect("peace", "joy")
        end)

        -- Phase 2: Kill Peace (simulate going offline)
        ctx.after(10.0, function()
            ctx.log("")
            ctx.log("Phase 2: Peace is going OFFLINE...")
            ctx.log("         (Killing Peace instance)")
            ctx.kill("peace")
        end)

        -- Phase 3: Love sends message while Peace is offline
        ctx.after(14.0, function()
            ctx.log("")
            ctx.log("Phase 3: Love sending message to Peace...")
            ctx.log("         (Peace is OFFLINE - Joy stores for relay)")
            ctx.send_packet("love", "peace", "Hello Peace! You were offline when I sent this. ~Love")
        end)

        -- Phase 4: Kill Love too (sender goes offline!)
        ctx.after(16.0, function()
            ctx.log("")
            ctx.log("Phase 4: Love is ALSO going OFFLINE...")
            ctx.log("         (Killing Love instance - sender is now gone!)")
            ctx.log("         Only Joy remains to relay the message.")
            ctx.kill("love")
        end)

        -- Phase 5: Peace comes back online
        ctx.after(20.0, function()
            ctx.log("")
            ctx.log("Phase 5: Peace is coming back ONLINE...")
            ctx.log("         (Restarting Peace instance)")
            ctx.log("         Note: Love is still offline!")
            ctx.restart("peace")
        end)

        -- Phase 6: Wait for relay and show results
        ctx.after(27.0, function()
            ctx.log("")
            ctx.log("=== Test Complete ===")
            ctx.log("")
            ctx.log("Expected result:")
            ctx.log("  1. Peace was offline when Love sent the message")
            ctx.log("  2. Joy stored the message for relay")
            ctx.log("  3. Love went offline BEFORE Peace came back")
            ctx.log("  4. When Peace reconnected, Joy forwarded the message")
            ctx.log("  5. Peace should see Love's message - even though Love is gone!")
            ctx.log("")
            ctx.log("This demonstrates TRUE store-and-forward relay.")
            ctx.log("Check Peace's chat to verify!")
            ctx.log("")
            ctx.log("Press Ctrl+C to stop remaining instances")
        end)
    end,
}
