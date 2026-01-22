-- mesh-messaging.lua
-- Full mesh topology - tests direct messaging between all connected peers
--
-- Topology (full mesh):
--   love --- joy
--     \     /
--      \   /
--      peace
--
-- All pairs are directly connected, so ctx.send_packet works for any pair.
-- Unlike offline-relay.lua (hub-and-spoke through Joy), this scenario
-- ensures every node has every other node in their contacts.

scenario {
    name = "mesh-messaging",
    description = "Full mesh with direct messaging test",

    instances = {
        {name = "love", profile = "Love"},
        {name = "joy", profile = "Joy"},
        {name = "peace", profile = "Peace"},
    },

    -- Full mesh - everyone connected to everyone
    -- This ensures all name resolutions succeed
    topology = "mesh",

    on_start = function(ctx)
        ctx.log("=== Direct Messaging Test (Full Mesh) ===")
        ctx.log("Topology: love <-> joy <-> peace <-> love")
        ctx.log("All nodes are connected to each other")
        ctx.log("")

        -- Wait for mesh connections to establish
        -- topology = "mesh" triggers auto-connect, but we need time for:
        --   1. All .invite files to be written (1-2s)
        --   2. Connections to be processed by watchers (1s polling)
        --   3. Contact requests to be accepted
        ctx.after(8.0, function()
            ctx.log("Step 1: Love sends message to Peace...")
            ctx.send_packet("love", "peace", "Hello Peace! Direct from Love.")
        end)

        ctx.after(10.0, function()
            ctx.log("Step 2: Peace sends message to Joy...")
            ctx.send_packet("peace", "joy", "Hi Joy! From Peace.")
        end)

        ctx.after(12.0, function()
            ctx.log("Step 3: Joy sends message to Love...")
            ctx.send_packet("joy", "love", "Hey Love! Joy here.")
        end)

        -- Bidirectional test - ensure both directions work
        ctx.after(14.0, function()
            ctx.log("Step 4: Testing reverse directions...")
            ctx.send_packet("peace", "love", "Reply from Peace!")
            ctx.send_packet("joy", "peace", "Reply from Joy!")
            ctx.send_packet("love", "joy", "Reply from Love!")
        end)

        ctx.after(17.0, function()
            ctx.log("")
            ctx.log("=== Test Complete ===")
            ctx.log("Each instance should have received 2 messages:")
            ctx.log("  - Love: from Joy and Peace")
            ctx.log("  - Joy: from Love and Peace")
            ctx.log("  - Peace: from Love and Joy")
            ctx.log("")
            ctx.log("Check packet count in each instance's Network tab")
            ctx.log("Press Ctrl+C to stop all instances")
        end)
    end,
}
