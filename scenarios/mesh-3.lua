-- mesh-3.lua
-- Three nodes in full mesh topology
--
-- Topology:
--   love ─── joy
--     \     /
--      \   /
--      peace

scenario {
    name = "mesh-3",
    description = "Three nodes in full mesh topology",

    instances = {
        {name = "love", profile = "Love"},
        {name = "joy", profile = "Joy"},
        {name = "peace", profile = "Peace"},
    },

    topology = "mesh",
}
