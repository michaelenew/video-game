---
status: proposed 2026-11-02; the King of the Hill maps and the floating stages rebuilt to it, unplayed
---

# Fight spaces — room to fight

The owner, 2026-11-02:

> I need the coop modes to have more battle area. On koth, everyone just
> stands in the middle and spams abilities and it becomes chaotic. I think
> that's more a function of the level design than the classes, the koth
> levels are just tiny relative to each class's mobility options, as are the
> 1v1 stages and most fights. I need us to build towards a level design
> principle distilled from revisions of the current levels. Make them
> bigger, wider, taller, and more intriguing types of structures to interact
> with.

This is that principle: what was wrong, measured; the rules a fight space is
built to, in the game's own units; and what each level became. The code is
`sim::fightspace` (the measures), `cargo run --release -p sim --bin
fightspace` (every fight's numbers, one line each) and
`crates/sim/tests/fight_space.rs` (the rules, held for every versus stage).

## 1 · Measure a place in moves, not metres

A metre means nothing to a fight. What a fight feels is **how many of its
moves a distance is**: thirty metres is "out of reach" to one game and "a
step" to another. So a fight space is measured in two yardsticks taken off
the classes themselves, worked out by running them (`sim::fightspace`):

- **R, a reach: 25 m.** How far a fight acts: the second-longest of the five
  movers' longest pointed reaches (a skillshot, a grounded cast). The longest
  is the Elementalist's Gale at 32 m, one class's outlier; the Blood mage's
  vines and the Champion's Heaven-piercer reach 25. Somewhere further than R
  from you is somewhere only one move in the game arrives.
- **J, a jump: 3.9 m.** How high a fight climbs: the full held jump of the
  mover that jumps least (the Champion). Two heights less than **a jump and
  a half (5.9 m)** apart are one level you hop between; further apart they
  are two levels, and the higher one is *held*.

Both move when the kits do, so the rules move with them: a kit change that
lengthens the pokes makes every map that was big enough a little small, and
`tests/fight_space.rs` says so.

## 2 · What was wrong, measured

The audit before any change (2026-11-02, `--bin fightspace`):

| Place | Size | Start → middle | Levels | Structure, core / contest / out of reach | Moves you |
| --- | --- | --- | --- | --- | --- |
| Quarry | 76 × 58 m, 3.0 × 2.3 R | 33 m, 1.3 R | 1 | 4 / 20 / 6 | nothing |
| Glacier | 76 × 56 m | 33 m, 1.3 R | 1 | 3 / 10 / 14 | nothing |
| Delta | 80 × 56 m | 36 m, 1.4 R | 1 | 4 / 18 / 10 | nothing |
| Gorge | 76 × 52 m | 33 m, 1.3 R | 2 (7 m) | 1 / 26 / 2 | nothing |
| Henge | 76 × 56 m | 34 m, 1.3 R | 2 (7 m) | 3 / 20 / 8 | nothing |
| Floating stages | 20-odd m of islands | 6–9 m | 1 | nothing past 0.7 R | nothing |
| Proving ground | 28 × 28 m, 1.1 R | -- | 1 | -- | nothing |
| Creature rooms | 30–54 m, 1.0–2.1 R | -- | 1 (the Cliffs 3, the Hollows 2) | -- | nothing |

Three things, and none of them is the classes:

1. **One level.** Every hill map's high ground was 3–5.5 m up -- a little
   over one jump -- so it was hopped onto, not held. The fight was one plane
   round the point, with bumps.
2. **Everything interesting was inside one reach of the point.** The
   benches, dykes, stones and bridges stood 9–15 m from it (0.4–0.6 R): from
   the point, every perch was in range, and from every perch, the point was.
   Past a reach there was empty floor, so leaving the middle bought nothing
   but being shot at with nothing to stand behind.
3. **The starts were 1.3 reaches out.** A fighter respawned, walked five
   seconds and was in the fight again; there was no "away" to be in, and no
   approach to choose.

The duel stages have the same three at a smaller scale: twenty metres of
island is less than one reach across, so a ranged class covers all of it
from the middle, and a metre or two of height is no height at all.

## 3 · The rules

**F1 · An "away" is a place.** Past a reach from the objective there is
somewhere worth standing: high ground, cover, a route in. A fight needs
places out of each other's range, or positioning is not a choice. A hill
map's starts are at least **2.2 R** from the point by a straight line; a
duel stage's pieces span at least **1.8 R**.

**F2 · Three rings, each with something in it.** Round the objective:

- **The core** (within 0.4 R, 10 m): the point itself, melee range. Kept
  small and mostly open -- **the objective is smaller than the fight** --
  with a little cover on it. At most a fifth of the map's structure.
- **The contest** (0.4–1 R): the perches and flanks that can hit the point,
  and be hit from it. At least a third of the structure.
- **Out of reach** (past R): where the approach is chosen, where a beaten
  side regroups, where a flank fight happens away from the point. At least a
  quarter of the structure.

**F3 · Levels are more than a jump apart.** Heights are counted in jumps,
and two levels are at least **a jump and a half** apart, so the higher one
is climbed by a route and held, not hopped onto. A hill map has **three
levels at least** -- the floor, a middle level, a top -- with its top at
least **3.5 J (14 m)** over the starts; a duel stage two, with its top at
least **2 J** over the starts.

**F4 · Every level is reached two ways, of two kinds.** A stair of steps
under a jump; a ledge to catch; a wall to kick off; an updraft; a lift. One
kind is a corridor; two is a choice, and the second way up is the one the
defenders are not watching.

**F5 · Every structure is a verb.** A box is worth its place if it is
something to *do*: climb it (a ledge catches), kick up between two of it,
ride it (an updraft, a lift, a ferry), drop through it (a slot, a hole),
cross over the fight on it (a bridge, a lintel), duck under it (an arch,
a tunnel), be knocked off it (an edge over a drop), hide behind it (taller
than a jump, wider than a body). **Every map has at least one thing that
moves you** -- an updraft or a drift -- and the engine's whole kit of verbs
is open to it: since 2026-11-02 a versus stage carries its own updrafts and
drifts (`versus::Stage::vents`, `Stage::drifts`), exactly the valley's.

**F6 · Three ways in, and ways out.** At least three ways from each start to
the point, of different kinds -- the open floor, the high road, the hidden
way -- and as many from the core to out of reach. "Everybody stands in the
middle" is half a failure of the ways *out*: when leaving the point leads
nowhere, nobody leaves.

**F7 · Lanes fit the movement.** An open run is long enough for a charge,
a dash or a Rush to mean something -- twenty metres and more -- and a lane
is wide enough to sidestep a skillshot in, eight metres and more. Over the
core the sky is open, but for one deliberate overpass; a tunnel is the
exception, and tall enough to jump in.

**F8 · Within what the engine holds.** A versus stage is fought alone, so
it is read box by box: at most 64 solids (`arena::MAX_SOLIDS`), 32 floor
regions, a dozen relief bumps, four drifts, and a diagonal under 170 m
(lengths over 181 m saturate in fixed point). So **spend boxes on big
shapes** and let the cheap things do the rest: relief for hills and ramps
(no boxes), updrafts for climbs (no boxes), drifts for crossings (raised
only near a fighter).

## 4 · What each kind of fight is held to

`tests/fight_space.rs` checks these for every versus stage:

| | King of the Hill | Floating duel stage |
| --- | --- | --- |
| Size | ≥ 5 R long, ≥ 3.5 R wide | pieces span ≥ 1.8 R |
| Start | ≥ 2.2 R from the point | starts ≥ 0.8 R apart |
| Levels | ≥ 3, top ≥ 3.5 J | ≥ 2, top ≥ 2 J |
| Structure | core ≤ 1/5, contest ≥ 1/3, out of reach ≥ 1/4 | -- |
| Ways | ≥ 3 to the point | -- |
| Moves you | an updraft or a drift | an updraft or a drift |

A floating stage keeps its **main island under a reach across**: the edge is
the point of a ring-out stage, and an island too big to be pushed off is a
hill map with a pit. What it gains is *pieces*: islands across real gaps, at
real heights, and something that moves you -- and an updraft that a fighter
knocked off can steer into and ride back, which is a recovery, Smash's
second half, that the stages did not have.

## 5 · Decided while building, for a person to review

- **The hill maps' ways are walked by the five movers, not the Bulwark.**
  `versus.md` §3 sized every climb to the Bulwark's 2.7 m jump so a team
  game shut nobody out; the Bulwark has not been on offer since 2026-10-21,
  and a map sized to it could not be tall. Every way on every map is walked
  by all five movers (`tests/versus.rs`); the steps are sized to the
  Champion's 3.9 m.
- **R is the second-longest reach, not the longest.** The Elementalist's
  Gale (32 m) would make every map a third bigger for one move.
- **The point stays 4 m across.** The objective is meant to be smaller than
  the fight; it is the room round it that grew.
- **The unlock stays at ten seconds.** The walk from a start is about eight
  now; first play will say whether that wants to be longer.
- **Nothing here has been played.** Every size is a first guess at the
  rules, checked by walking every way with every class and by screenshots.

## 6 · The revisions

What each level became is in [versus.md](versus.md) §2–§3; the numbers
before and after are `--bin fightspace`'s, and are kept here.

*(Filled in as each map is rebuilt.)*

## 7 · Next

- **The proving ground** is 1.1 R square, and is where every measuring test
  stands (`arenas.md` §1a), so it is not resized; a 1v1 that wants room is a
  floating stage, or the Ring.
- **The Ring**, the valley's place for fighting each other, is the proving
  ground's square in sand, and on the map: it grows when the valley's layout
  is next opened.
- **The creature rooms** (1.0–2.1 R) are sized to their creatures' AI and
  sit on the valley's map; each is its own revision, with its creature's
  harness run after (`review.md`). The Cliffs and the Last Valley already
  pass F1 and F3.
