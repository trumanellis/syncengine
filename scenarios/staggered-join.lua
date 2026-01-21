-- staggered-join.lua
-- Nodes join the network at different times
--
-- Timeline:
--   0s:  love starts
--   5s:  joy joins, connects to love
--   10s: peace joins, connects to all

scenario {
    name = "staggered-join",
    description = "Nodes join at different times to test late-join sync",

    on_start = function(ctx)
        -- First node starts immediately
        ctx.launch("love", {profile = "Love"})
        ctx.log("love started - the field is now active")

        -- Second node joins after 5 seconds
        ctx.after(5, function()
            ctx.launch("joy", {profile = "Joy"})
            ctx.connect("joy", "love")
            ctx.log("joy joined - connected to love")
        end)

        -- Third node joins after 10 seconds
        ctx.after(10, function()
            ctx.launch("peace", {profile = "Peace"})
            ctx.connect_to_all("peace")
            ctx.log("peace joined - connected to all existing nodes")
        end)

        -- Log milestone at 15 seconds
        ctx.after(15, function()
            ctx.log("All nodes should now be synchronized")
        end)
    end
}
