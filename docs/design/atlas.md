# The atlas — one map, read a tile at a time

*Built 2026-10-09. The code is `crates/sim/src/atlas.rs` (the map and its
tiles), `crates/sim/src/valley/layout.rs` (where the valley's places go),
`crates/sim/src/valley/open.rs` (the rules that make it play as one piece),
`crates/sim/src/arena/mod.rs` (`Terrain`, which every query goes through) and
`crates/game/src/stream.rs` (distance loading). Tests: `sim/tests/atlas.rs`,
`sim/tests/valley.rs`, `net/tests/ggrs_synctest.rs`.*

![The valley as one map, from above](gallery/atlas.png)

*`cargo run -p look --example atlas`: every place where the layout put it,
doorways outlined (gold open, grey behind a waystone).*

## Why

An arena is one table of boxes, and every query walked the whole table:
collision, the floor under a point, the aiming ray, a creature's sight. That is
why an arena was capped at sixty-four boxes, and why the valley was a string of
separate rooms joined by teleports ("seams"). The ask was the opposite: **the
whole valley, its fights and Hearth as one contiguous map**, built so that it
can grow to millions of boxes, and it has to work with two players over
rollback netcode.

So there are two problems, and they are separate:

1. **A frame must cost what is near the bodies, not what is in the world.**
   That is a *spatial index*: the map is cut into square tiles, and a question
   reads only the tiles near it.
2. **The screen must hold only what is near the camera.** That is *distance
   loading*: pieces are drawn when the camera comes within reach and dropped
   when it goes away again.

## The map (`sim::atlas`)

- **Places on one map.** Each place is still an arena, drawn in its own
  coordinates exactly as before. The map puts each one at an offset, cuts the
  doorways where two places meet, and adds the few boxes that belong to no
  place (a cliff round a room, a door's sill). The valley's map is 18 places,
  17 doorways and about 600 boxes.
- **Tiles.** The map is cut into 16 m squares. Each tile lists the boxes that
  touch it, in map order, and the places whose footprint does. A query over a
  box reads the tiles under it and merges their lists in map order, each box
  once. The busiest tile in the valley holds 18 boxes, so a body's collision
  reads a few dozen boxes however large the map grows. `tests/atlas.rs` checks
  that a query never misses a box (against walking every box, 2,000 random
  queries over a thousand scattered boxes), and that a map a hundred times
  bigger costs a body-sized query no more.
- **Order is kept**, because the collision resolve pushes a body out of one
  box at a time and the order is part of the answer. Map order is the same on
  every machine and does not depend on where a question is asked from.
- **Built once, never in a frame.** The map is composed the first time it is
  asked for, which is when a valley world is built, and never changes after.
  It is not in the snapshot: both peers build the same map from the same
  tables, the same way they read the same arena tables. A frame that reads it
  allocates nothing (`tests/budget.rs` has two valley scenarios).
- **The 64 cap now means something narrower.** `arena::MAX_SOLIDS` holds for
  an arena that is fought on *alone* (a creature's own arena, a course),
  because alone it is still one table walked end to end. A place that only
  ever lives on a map — the town and the reaches — is not held to it.

## The land (`sim::valley::land`) — since the second pass, 2026-10-09

The first pass laid the old rooms end to end: a valley of vertical walls and
right angles, every reach a corridor of boxes, every room behind a notch.
It worked and it read as rooms. **The second pass replaced the reaches with
land**: the ground between places is a height function, and the valley is
shaped by it rather than walled by boxes. The aim was the field of a Guild
Wars or Final Fantasy map — a road winding along a valley floor, foothills
you can wander up, mountains as the edge of the world, a river, a tarn,
woods, rocks, and the fights in clearings off the road.

![The Pinewood](gallery/valley-pinewood.png)

- **A way** is a line of points, each with a floor height and a half-width:
  the floor is level across its width and eases from point to point. The
  road is each reach's `WAY` in turn, wide in the meadows (fifty metres) and
  narrow at the passes; each room has a side path to it. Off a floor's edge
  the land rises: **foothills** first, a third as steep as they are long for
  sixteen metres, and then the **mountainside**, turning to a slope of 1.4.
- **A pad** is a place that keeps its own floor: the town, the Ring, every
  room. Inside it the floor is the place's own relief, exactly as tuned;
  outside, the land rises off its edge the way it does off a way -- or, for
  a room with a **rim** ([arenas.md](arenas.md) §1a, since the third pass),
  as its rim's bank, straight off its walls: held up over the valley round
  it for the bank and four metres past its crest, then falling away three in
  one to whatever is round it; cut only by a trodden way within twenty
  metres of the room's own door (`land::Pad::door`), so a clearing has an
  edge and one way in (`tests/rim.rs`).
- **The ground is the lowest any way or pad would make it, and the highest any
  trodden way or pad holds it up to.** The first carves valleys: two meet as
  a saddle, a side path is a notch in a hillside. The second is an
  embankment: a path that climbs above the valley it leaves (to the
  Highlands, to the Shrine) holds the ground up under it, falling away off
  its edge.
- **Water** is a way with a surface over its floor: the Mouth's river, the
  Shelves' tarn.
- **Noise** over all of it: a hand's breadth on a floor, metres up a
  mountainside, and ridged crests on the high faces. Fixed point and hashed
  integers, so both machines raise the same mountains.

**Steep ground is a wall.** A step whose ground rises faster than
`terrain_steepest` (the Oven, 0.85) is taken back, or slid along; ground that
steep is no footing, so a body standing on it slides down
(`terrain_slide`) and cannot jump off it. That is what makes a mountainside
the edge of the world without a box in sight (`tests/valley.rs`,
`a_mountainside_is_a_wall`). Only on land: a place's own floor answers as it
always did, so every pinned fight is unchanged.

**What hangs** is relative to the land now: a box hangs (is a ceiling) if its
bottom is above the floor under it, not above zero. A tree or a crag stands
on ground metres up and is not a ceiling for being there.

## Where things go (`sim::valley::layout`)

- **The town and the reaches share the map's coordinates**: a reach's origin
  is the map's, so its road, seams and crags are written where they are, and
  walking from one reach to the next moves nothing.
- **A room** sits in a clearing off the road, at the end of a side path from
  a junction to the middle of its near side. Its rim's bank rises straight
  off its walls (a room without one has six metres of level ground round it)
  and its doorway is cut through its wall where the path arrives
  (`Plan::carves`). The Cliffs sit sunk with their plateau at the road's
  height, reached over a bridge across the ravine round them; the Shrine is
  at the top of a switchback twenty metres up the Saddle's north-east slope.
- **A waystone's door** is a box across a pass, shut while the stone is dark.
- **Crags, cairns, trees and boulders** are made by the layout, standing on
  the land wherever it is under them: a crag is a pillar of rock with ledges
  up one face a jump apart, a vine down another and a cairn on top; a tree's
  trunk is a box you cannot walk through, which the renderer draws as a tree.

Three things changed in the places themselves on the first pass and stay:
the Bank's den is off the north of the road (it was a cave in a bank face);
the Shrine is not on a spire; the Long Valley is reached from Hearth's west
gate only.

## Playing it as one piece (`sim::valley::open`)

- **The world runs in the coordinates of the place it is in** (`World::arena`).
  Every position in the snapshot is in that place's coordinates, so a
  creature's fight is exactly the fight it was tuned in, floor at zero and
  all. The rest of the map is around it, solid, through `Terrain`: a query
  moves its box onto the map, reads the tiles, and moves each box it finds
  back into the place's coordinates.
- **The frame follows the fighters.** When the first fighter on their feet
  walks into another place, the world moves into that place's coordinates:
  every position it keeps shifts by the difference between the two places'
  origins, a whole number of fixed-point units, on the same frame on both
  machines. `tests/valley.rs` (`a_change_of_frame_changes_nothing`) moves a
  world of every class, with every mechanic out, into another place's
  coordinates and checks it plays on and comes back identical; a field the
  shift forgot would show up there.
- **Walking into a room starts its hunt.** The creature is put on its marks,
  noticing you as it always has. A hunt holds the world in its room while
  anybody on their feet is in it; when nobody is, the hunt is over, unwon,
  and the creature is gone. Won, the room goes quiet; lost, you wake outside
  its door, as before.
- **A dark waystone's door is a wall**: a box the shape of the doorway, raised
  on the ground the way a slag mound is, unless somebody already stands on
  its far side (so starting further up with `--arena` never strands you).
- **Cairns** are counted by their index on the map, so a checkpoint is one
  box of the map wherever the world's frame is.

## Two players

Both peers simulate the whole snapshot, and the snapshot holds only what is
near the party: the two fighters, what they have thrown, and the one live
hunt. The map itself is static data both peers already have, so nothing about
it goes on the wire. `net/tests/ggrs_synctest.rs` walks two fighters out of
Hearth's gate and into the Highlands under SyncTest's forced rollbacks with
random buttons pressed, which re-simulates the frame the world changes place
over and over; neither desyncs.

**What this does not do yet** is let the two players be far apart *and* both
be in fights: there is one live hunt at a time, in the room the world is in.
That, and more than two players, is the overlapping-region design on the
unmerged branch `claude/brave-cray-d08zzb` (`regions.md` there,
[exploration/0005](exploration/0005_open_world_netcode.md) when it lands):
hexagonal regions with their own checksums, so peers only exchange inputs
with peers near them. The map's tiles are not those regions and do not need
to be — tiles are how a query finds boxes; regions are who must agree about
a frame — but a region layer would sit on top of this map unchanged.

## Distance loading (`game::stream`, `game::land`)

**The land is drawn a tile at a time**: each 16 m tile of the map is a mesh at
the simulation's own heights, a metre a vertex near the camera and coarser
further off (two, four and eight metres), with a skirt hanging from its edges
so two tiles drawn at different fineness never show a crack. Its colour is
`look::Palette::land` for what the ground is there — rock as it steepens,
snow high up a mountain — with the road worn in (`Palette::trodden`), and
patches across a meadow. Water is a surface over the tiles under it. A tile
is built, rebuilt finer or coarser, or dropped as the camera moves, nearest
first, up to 24 a frame and no more than 3 ms of it; the land is drawn out to 300–700 m (the sky's reach, a
fifth more). Trees are three shared meshes — a trunk, a pine's cone, a
broadleaf's crown — and a few materials, so a wood of a thousand trees costs
little.


In the valley, the renderer draws two kinds of piece, each loaded when the
camera comes within reach and dropped when it goes further than that by
another 60 m (so the edge never flickers):

- **A place**: its floor, the patches on it, its props, what is scattered on
  its floor, its seams and vines, under one entity.
- **A box**: each box of the map on its own, found through the tiles round
  the camera, so loading reads the tiles near the camera and not the map.

Building is spread over frames: the nearest place and the 48 nearest boxes
still wanted, each frame, until everything in reach is drawn -- and boxes and
tiles stop for the frame once 3 ms has gone on building (`BUILD_BUDGET_MS`),
at least one of each, because a tile in full detail is half a millisecond and
twenty-four of them was a dropped frame every few steps.

**A wall of dressed stone is drawn in two fineness** (`forms::build`'s
`near`): in full relief, every block standing proud of the core, when the
nearest edge of its box is within 60 m of the camera; past that (and 12 m
more before a near wall goes back), the same courses in the same colours with
each block only its face -- two triangles a block instead of ten, and a
relief of five centimetres that is under a pixel there anyway. Either way
**the blocks cast no shadow**; the core does, which is the same shadow
(`forms::build_parts`). See `architecture.md` §"The valley's frame" for what
it cost before.

The reach is the current sky's own (`look::Sky::reach`, clamped to 160–600 m):
past it the fog is the horizon's colour, so a piece loading there cannot be
seen arriving. Everything hangs off one root at the map's origin as seen from
the current place, so when the world moves into another place's coordinates
only the root moves; the interpolator's previous frame and the camera's
smoothed focus are moved with it (`same_frame`, `CameraRig::shift`). `--dev`
shows how much is loaded under the place's name.

## Toward millions of boxes

What is built scales; what will need doing when the content arrives:

- **Fixed-point range.** Positions are 16.16, so about ±32 km from the place
  the world is in, and the map is refused past 16 km each way
  (`atlas::MAX_SPAN_M`) so every place can still see every other. The valley
  is 1.4 km. A bigger world needs map coordinates in 64 bits, with only the
  tiles near the current place converted into its 16.16 frame.
- **Where the boxes come from.** The valley's map is composed from Rust
  tables at start-up. A million boxes wants a baked file loaded once (the
  simulation does no I/O, so the game hands it over), with the same tiles;
  nothing in a query would change.
- **Streaming the map's data**, not just its meshes, is only needed if the
  map stops fitting in memory, which at 28 bytes a box is far past a million.
- **Mesh building on the main thread.** Loading is spread over frames,
  nearest first; with very dense content a task pool would take it off the
  main thread altogether.

## Open

1. **Nobody has walked it.** Every doorway is walked by a test; nobody has
   looked at how the joins read, in particular the rooms' aprons and cliffs.
2. **The Long Valley's loop** (above): whether a return trail from the Saddle
   to Hearth's west is worth building.
3. **One live hunt at a time**, and two players: both are the region layer's
   (above, §Two players), not this map's.

## Fixed, 2026-10-09 (the same day)

- **The sky changed in one jump at a doorway.** Each place still has its own
  sky, but the air now blends between the two over 24 m either side of the
  nearest doorway, half-way at the doorway itself, so nothing jumps as the
  camera crosses and nothing jumps when the world moves into the next
  place's coordinates: dome, fog, clear colour, the sky's light and the
  sun's bearing (`look::Sky::toward`, `game::sky::weather`).
- **A fall into a place lower than the world's was free.** The fall rule
  measured a drop down to zero, the floor of every arena alone; on the map a
  place below the current one has floors under zero. It now measures down to
  the map's void (`Terrain::lowest`), and an arena alone still uses zero, so
  every pinned fight is unchanged (`tests/valley.rs`,
  `a_fall_into_a_lower_place_costs_the_same`).
- **`seam_hold`** left the Oven: nothing has read it since seams stopped
  being teleports. Pinned fights hash without the tuning, so none moved.
- **A map too wide for 16.16 is refused as it is built** (`MAX_SPAN_M`,
  16 km), instead of a sum saturating mid-frame and a wall turning up
  somewhere else.
- **Loading built everything in reach on one frame.** It builds nearest
  first, one place and 48 boxes a frame, and comes back next frame for the
  rest.
