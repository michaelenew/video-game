---
status: building -- the shapes, the Great Wood, the Needle, ranges (the roost among them) and the Deep built 2026-10-15; the High Plane next; the shorter road held (§4 below)
started: 2026-10-15
---

> **The owner's answers, 2026-10-15.** VALE-1 yes, the road shortens and the
> world grows up and down. VALE-2 **fall damage applies** in the Wood. VALE-3
> a beaten roaming creature comes back **after a while**. VALE-4 provoking a
> peaceful one **depends on the creature**: give the world variety. VALE-5 the
> Spire's top is **a fight**, and no pool at its foot. VALE-6 **both** a tepui
> and floating islands for the plane above. VALE-7 **more shapes** rather than
> boxes, unless that costs performance: built as round and turned solids
> ([../atlas.md](../atlas.md) §"Round and turned solids"), which cost a box's
> query and a square root.
>
> **Built so far**: the shapes, the Great Wood and the Spire -- named **the
> Needle** in the game, because the jump course is already called the Spire --
> as [../valley.md](../valley.md) §1a has them. Where they differ from this
> note: the Needle stands on a plaza just outside the town's south wall, not
> in the square; its shaft is 315 m, so the ruins are 353 m up; and the climb
> through the dome is the shaft's own vine running on into the chimney's.
>
> **Ranges** are built as §3c has them, but simpler than the clock: a
> sleeping range keeps nothing at all -- its creature is put on its marks when
> you walk in, as a room's is -- so nothing needed a closed-form wander. Six of
> them, temper 0 by the town to 3 on the Saddle, with four temperaments
> (hunting, territorial, peaceful, skittish); the roost is one of them,
> stacked over the Needle's plaza. [../valley.md](../valley.md) §1b.
>
> **The Deep** is §3b's answer, built: a pit sunk 140 m with a lid at the
> road's height, three generated warrens and the Caverns under it, the
> pit's own floor and its rock walls the cave, and ranges stacked on every
> level. [../valley.md](../valley.md) §1c.
>
> **The shorter road is held.** The rooms either side of the road are
> already 30 to 50 metres apart (the Mire and the Pan, the Den and the
> Highlands), so the road cannot be scaled without walking rooms into each
> other; the stretches with nothing on them add up to about 120 m, an
> eighth. Halving it means moving rooms -- across the road, or up the
> slopes on shelves of their own -- which is a layout of its own, and wants
> the owner's say on which rooms go where.

# 0019 — The Spire, the Great Wood, the planes above and below, and a valley that is lived in

*The owner, 2026-10-15: Hearth gets an extremely tall, narrow tower with a
very large inverted dome on top (Korin's tower, without the rounded cap),
ruins on the dome and flying monsters roosting on it. The valley gets more
compact, with monsters roaming it -- easy ones near the town, harder the
further in. Near Hearth, a wood of giant trees, at least two body heights
across, with branches you can stand on climbing to the tops: a playground for
seeing what a class can do, and "however big you're thinking of, triple it",
because every developed class already clears half of a jump map in one go.
Routes up to a separate plane, over floating stones and vine-hung stones with
big gaps and small stones for precise jumping; routes down into caves that
twist into a maze and keep going down until they open into great caverns of
crystal. And not every creature should want a fight: a herd of peaceful
Ridgebacks, say.*

This note sizes each piece against what the fighters can actually do, finds
where the engine as built stops being able to say it, and proposes an order
to build in. Open questions are at the end, numbered VALE-1 onward.

## 1 · The ruler: what a fighter can do

Everything below is measured with these, from `sim::envelope`,
[courses.md](../courses.md) and the Oven:

| | |
| --- | --- |
| A body | 1.8 m tall |
| A plain running jump | 7–9 m level; a full hop's top 2.7 m (Bulwark) to 6.0 m (Dual mage) |
| With the class mechanics | 15 m past at four metres down for everyone (round one's search); the Reach's lanes go to 50 m and its ledges to 45 m, and the owner says those are being cleared |
| A vine | anyone, 2.5 m/s holding jump |
| A fall | free to 9 m, then 25 health a metre, halved landed slowly |
| The valley now | 974 m of road, ~125 m of climb, a road 50 m wide in the meadows |

The owner's correction is the important number: a jump course built for the
plain jump is half-cleared by the mechanics. So this note sizes the playground
in **three bands**, and every route has all three:

- **Walking band** -- what the plain jump does: rises of 2–3 m, gaps of 4–7 m.
  The route anyone can do, slowly.
- **Mechanic band** -- rises of 6–15 m, gaps of 12–25 m. One mechanic each.
- **Expert band** -- gaps of 30–50 m, tree to tree, stone to stone, where a
  class's mechanic has to be *chained* (a stone off a stone, a shadow off a
  dash) and missing costs a long fall.

The first thought for the giant trees was 60 m tall and 6 m thick. Tripled,
as asked: **150–200 m tall and 12–20 m across.** Everything else here follows
that scale.

## 2 · The pieces

### 2a · The Spire, in Hearth

**The shape.** A shaft of pale stone **8 m across and 320 m tall** -- forty
to one, which is what reads as "impossibly thin" from the square -- rising out
of the square beside the well. On its top, **an inverted dome 90 m across and
35 m deep**: its point on the shaft, its rim wider than the whole town, so
from inside the walls you look up at its underside the way you look up at
Korin's. The top is flat, a stone plain with **a ruined village** on it: broken
walls, a fallen arch, a cistern, column stumps -- the town's own building
tables, ruined (`sim::arena::hearth`'s parts, cut short and toppled). On the
rim and the ruins' high points, **roosts**: flying creatures sitting, turning
their heads, lifting off when somebody arrives.

**The way up** is a climb, not a stair, in the three bands:

- **A vine the whole height** -- the walking band: anybody, 320 m at 2.5 m/s,
  about two minutes. Slow on purpose, with **ledges every 40 m** to rest on and
  a cairn on every other one.
- **A corkscrew of stones jutting from the shaft** -- the mechanic band: each
  6–10 m above the last and a quarter-turn round, so the jump is up and across.
- **Through the dome, not round it.** The rim overhangs the shaft by 41 m,
  which no jump in the game goes round from below. So the shaft goes on up
  *through* the dome and opens in the middle of the ruins, as Korin's does: the
  last 35 m is a chimney inside the dome's stem. The overhang is the spectacle,
  not the obstacle.

**Coming down** is the fastest way back into the valley: a 355 m drop past the
rim. The fall rule makes that a death (it is 9 m free), and "nobody dies in a
reach" stands you on your cairn -- which is a fine answer for a mistake and a
poor one for a choice. Proposed: **water at the foot** (the well widened into a
pool the town's fountain feeds) that eats a fall into it, as a landing slowly
halves one. Diving off the dome into the pool is then a thing to do. VALE-5.

**Drawing it.** The renderer drops anything past the sky's reach (160–600 m)
and fogs toward it, and the Spire is meant to be seen from the whole valley.
It wants to be **a landmark**: drawn at any distance, as a silhouette in the
fog past the reach (one mesh, so it costs nothing), the way the mountains are.
Its dome also shades the town for part of the day, which is worth seeing in a
screenshot before deciding it is a feature.

### 2b · The Great Wood, beside Hearth

Out of Hearth's north or east gate, before the road climbs: **a grove of
twelve to fifteen giant trees in a clearing about 400 m across**, trunks
**12–20 m across** and **150–200 m tall**, 40–70 m apart. Under them, a floor of
moss and roots; over them, a canopy you can stand on.

On each tree:

- **A spiral of branches** up the trunk, each **4–8 m wide and 20–40 m long**,
  the walking band between them near the ground and the mechanic band higher
  up, so a beginner gets thirty metres up and a good player gets to the top.
- **Bridges between trees**: long branches that meet a neighbour's, the
  expert band where they do not quite meet (a 30–45 m gap at 120 m up).
- **A crown platform** at the top with a cairn: a view of the whole valley, and
  of the Spire across the town.
- **Vines** down some trunks, so nobody is stranded at the bottom of a
  climb they cannot make.

**Is a fall here free?** The Wood is meant to be practice ground: somewhere to
try a 40 m leap a hundred times. The fall rule would make every miss from the
canopy a walk back from a cairn. Proposed: **the moss is soft** -- a floor
material that takes the fall rule off (or a much longer free fall), as
water would. Then the Wood is a playground and the rest of the valley keeps its
teeth. VALE-2.

**What it costs.** About 35 boxes a tree, 450–550 for the Wood: the valley's
whole map is about 600 today. The tiles mean a body's collision reads only
what is near it, but a 16 m tile at a trunk will hold 40-odd boxes against the
18 in the busiest tile now. Within budget by the atlas's own arithmetic, and
`tests/budget.rs` will say.

### 2c · The routes up, to the High Plane

**What it is.** A separate plane, 250–300 m up: a **tepui** -- a table mountain
with sheer sides and a flat top a kilometre across -- standing beyond the
valley's south mountains, so its cliff is the wall of the valley at one point
and its top is a world of its own: a sky meadow, wind-bent trees, its own
creatures.

**The climbs to it** (two or three, from different reaches), in the owner's
words: floating stones and stones held by vines, big gaps, small stones for
precision. As built that is:

- **Floating stones** -- the floating island form (`game::forms`, turf on a lump
  of rock) at every size from 1.5 m precision stones to 15 m rests;
- **Stones on vines** -- a stone with vines up from it to an overhang of the
  cliff: drawn hanging, and the vine itself climbable, so it is a way up as
  well as a look;
- **Spacing** in the mechanic and expert bands: this is the treacherous route
  the owner asked for, with **cairns** at rests every 40–60 m of height, so a
  miss costs a stretch and not the climb.

The routes are long: 250 m of height at a rest every 50 m is five stretches of
a dozen jumps.

### 2d · The routes down, to the Deep

**What it is.** Caves that twist into **a maze that keeps descending** --
four or five levels, 30–50 m apart -- until it opens into **the Caverns**:
a hall about 300 by 200 m under a vault 80 m high, crystal columns, crystal
shards hung in the air as stepping stones, an underground lake. 200–250 m
under the valley floor at its deepest.

**Ways in.** A **sinkhole** in a meadow (a round pit, open to the sky, its
floor the first level of the maze) and a **cave mouth** at the foot of a
mountain. The maze is not a puzzle so much as a warren: loops, dead ends that
look down on the next level through a crack, the occasional drop that is the
fast way down and a long way back up.

**Sizes.** Passages **8–12 m wide and 8 m high** at the narrowest -- these
characters cannot fight, or even move, in a corridor built for walking -- and
chambers of 30–60 m. The camera already keeps the eye under a vault
(`camera::eye_under`), which the Hollows proved.

### 2e · A valley that is lived in

**Roaming.** Creatures out on the land, not only in rooms: a pack of Gnawers
rooting along a riverbank, a Hornback herd moving across the Mouth's meadow, a
Galewing on the Spire, something in the Deep. **Easy near Hearth and harder
further in**: by species (the tiers already rank them) and by temper (a temper
is a cleverer creature, not a tougher one, `sim::temper`), so the same Gnawers
are a temper higher in the Pinewood than in the Mouth.

**Not every creature is a fight.** Each roaming group gets a **disposition**:

| Disposition | What it does | Example |
| --- | --- | --- |
| **Grazing** | Moves along its range, ignores you, scatters if struck or crowded | A herd of Ridgebacks on the Highlands' moor |
| **Wary** | Watches, keeps its distance, flees if approached | Fliers roosting on the Spire |
| **Territorial** | Ignores you until you come within its inner ring; then fights, and stops when you leave | The Mireback in its marsh |
| **Hunting** | Comes for you when it notices you | The Deep's Gnawers |

A grazing herd struck hard enough could turn territorial for a while; that is
VALE-4.

### 2f · More compact

The valley's footprint is ~974 m of road. **Compact, as read here, means less
running between things worth seeing, not a smaller world:** the road shortens
to about **500–600 m**, the meadows narrow, and what is taken out of the
horizontal comes back in the vertical: the Spire 355 m up, the Wood's canopy
200 m up, the High Plane 300 m up, the Deep 250 m down. A minute's walk from
any sight to the next. VALE-1 checks the reading.

## 3 · Where the engine as built stops

Five places where what exists cannot say what is asked. Each has a cheap
answer and a proper one.

### 3a · Every solid is an upright box

A solid is an axis-aligned box: no round trunks, no domes, no diagonal
branches. A 16 m trunk drawn round inside a 16 m box has 3.3 m of invisible
corner sticking out at each diagonal -- which is exactly the rule
"a roof you see is a roof you land on" exists to forbid.

- **Cheap (zero engine change):** build round things out of **stacks of
  boxes**. A trunk is four overlapping boxes per tree (a stepped disc), out of
  true by about 15% at its notches; a dome is eight slabs widening upward; a
  diagonal branch is a staircase of short boxes, each step under the 0.45 m a
  body walks up. The renderer draws the round thing over the union.
- **Proper:** a second shape of solid, **an upright cylinder**, in `math.rs`
  beside the box, with resolve, ray and ceiling queries for it. That touches
  every query in `arena::Terrain` and `aim.rs`'s raycast, and the one-aim rule
  says it is done once, in those two files.

**Proposal:** stepped boxes first; a cylinder only if play says the notches
catch. A body sliding round a stepped trunk meets corners of a few
centimetres, which the step-over and the slide already handle.

### 3b · The land is one surface, so it cannot have a tunnel

The land is a height function -- one ground height for every point of the map
-- so nothing can go *under* it. A cave cannot run under a mountain.

- **Answer:** the Deep is **beside** the valley, not under it: a region of the
  map past the mountains whose land is sunk 250 m (a sunk pad, as the Cliffs'
  ravine is), with **a lid of rock boxes** at the height the mountains would be.
  The way in from the valley is a **gorge with a roof**: a way down whose land
  is carved open (the land does that already) and whose top is closed by boxes,
  so from inside it is a tunnel and from the valley a cleft under fallen rock.
  The maze's levels are box floors inside the sunk region; its bottom is the
  pad's own floor, which is the Caverns.
- **Also needed:** the map's void is at −100 m (`layout::VOID`) and the fall
  rule measures down to it; it goes to about −400 m.
- **Light.** The renderer lights everything by the sun. Under a lid, the boxes
  shade it, but a cavern lit only by crystals needs the renderer's own work
  (emissive crystals, a cave's ambient light), and is checked with a
  screenshot, not a test.

The High Plane is the same trick upward: a **pad held up**, 300 m high
(a pad with a rim already holds the land up round it), with its sides as cliff.

### 3c · One live hunt, two creature slots

The snapshot holds **two full creatures and 32 critters**, and is at 19,000 of
its 20,480 bytes. A valley with creatures everywhere cannot have them all in
it, and should not: rollback copies the snapshot every frame.

- **The reframing: a range is a room without walls.** Today walking into a room
  puts its creature on its marks, and leaving ends the hunt. A **range** is the
  same thing on open land: a circle on the map, a group of creatures, a
  disposition and a temper. It wakes when a fighter comes within its edge plus
  a margin, and is put away when nobody is.
- **Where is a creature nobody is watching?** On a clock. A sleeping range's
  creatures are not simulated at all: where each one *would be* is a pure
  function of the world's frame number (a slow loop round its range, a herd's
  drift across a meadow). Waking the range puts them there. Both peers get the
  same answer from the same frame, so a sleeping range costs **no snapshot
  bytes**, and from far off you still see the herd where it should be.
- **What that limits:** at most two big creatures awake at once (the slots),
  plus critters. Ranges are spaced so two awake is the most a pair meets.
  A herd of Ridgebacks is more than two big bodies; so a herd is **critters
  with the Ridgeback's body and mesh** (a critter is a box on its feet, 56
  bytes, and can be large), not full creatures. A grazing herd is never fought
  as the Ridgeback is, so it does not need the Ridgeback's skeleton.
- **The one-hunt rule** becomes: one *fight* at a time near the party. More
  than that is the region layer's ([regions.md](../regions.md)), which is
  where two players far apart, each in a fight, already lives.

### 3d · Fliers do not roost

The Galewing is a fight in its own room. A roost is a disposition (wary) and
an idle -- sitting, preening, taking off -- the creature does not have yet;
and small fliers would be a flying critter, which does not exist (a critter is
a body on its feet). Proposed: the Spire's roost is **Galewings at a temper**,
wary until you stand on the dome, territorial once you do: one awake at a
time, the others on the clock.

### 3e · The map's size is fine

The valley is 1.4 km across; the map is refused past 16 km
(`atlas::MAX_SPAN_M`). Adding the Wood, the High Plane and the Deep keeps it
under 3 km. Height is the same fixed point: ±32 km. Nothing to do.

## 4 · An order to build in

Each step is playable alone and the next does not undo it.

1. **The Great Wood**, beside Hearth: stepped trunks, branches, bridges,
   crowns, vines, soft moss. New forms for bark and branch. No engine change;
   the most play for the least risk, and it is where the owner said to start
   looking ("a big playground").
2. **The Spire**: the shaft, the vine and corkscrew, the dome as slabs, the
   ruins from the town's tables, the pool, the landmark drawing.
3. **Ranges**: the clock, waking and putting away, dispositions, the tier and
   temper by distance from Hearth, a grazing herd of Ridgeback-bodied
   critters. Sim work, with a test that a range woken on frame N puts every
   creature in the same place on both peers.
4. **Compaction**: re-lay the road shorter. Every road and room is walked end
   to end by `tests/valley.rs`, so this is safe to do and slow to finish.
5. **The Deep**: the void lowered, the gorge and the sinkhole, the maze's
   levels, the Caverns, crystals, the renderer's cave light.
6. **The High Plane**: the tepui, its climbs, its creatures.

## 5 · Open questions

- **VALE-1 · Compact.** Is "compact" the road's length -- about half, with the
  world growing up and down instead -- or should the whole valley also get
  smaller, and the new places be smaller with it?
- **VALE-2 · Falls in the Wood.** Soft moss (falls there are free), or the
  fall rule as everywhere (a miss from the canopy stands you on a cairn)?
- **VALE-3 · A roaming creature beaten.** Back the next time its range wakes
  (as a room's is when you walk back in), back after a while, or gone for the
  session?
- **VALE-4 · Provoking the peaceful.** Hit a grazing Ridgeback: does the herd
  scatter, turn on you, or ignore it?
- **VALE-5 · The Spire.** Should its top be a fight (Galewings defending the
  roost), or only something to see with creatures that leave as you arrive?
  And a pool at its foot that makes jumping off survivable -- yes?
- **VALE-6 · The planes' places.** A tepui beyond the mountains for the High
  Plane, and the Deep beside the valley under a lid (both forced by the land
  being one surface, §3b) -- or would floating islands *over* the valley read
  better for the plane above, as an archipelago in the sky?
- **VALE-7 · Round things.** Accept trunks and the dome built from stacked
  boxes, a few centimetres out of round, for now?
