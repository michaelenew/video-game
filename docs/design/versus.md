---
status: built 2026-10-21, unplayed
---

# Versus modes — floating stages and King of the Hill

Versus was one rule in one place: last one standing, in the proving ground.
This adds **two modes and nine places built for fighting each other**:

- **Four floating duel stages**: small islands hanging over open air. Knock
  your opponent off the edge and you win the round, the way Smash's ring-out
  works.
- **King of the Hill**, Team Fortress 2's way, on **five maps**: one
  capture point in the middle; you hold it until the other side takes it
  back; the side that holds it long enough wins.

All of it is `sim::versus` (the rules), `sim::arena::stages` and
`sim::arena::hills` (the places), and is tested by `crates/sim/tests/versus.rs`.

**How to get there:** the Esc menu's **Versus stages** section has a button
for each; `Shift+N` in game steps through all nine (`N` alone is still the
jump courses); and `--arena <name>` / `?arena=<name>` starts on one:
`anvil`, `ziggurat`, `caldera`, `twin_isles`, `quarry`, `glacier`, `delta`,
`gorge`, `henge`. The mode comes from the map, as it does in Team Fortress
2: there is no separate mode switch.

## 1 · The rules

### A stage is a table of rules beside its arena

`versus::Stage` names, for an arena built for versus, its **kind** (duel or
hill), a one-line **theme**, a **pit** and a **hill**. `versus::of(arena)` is
the lookup, the way `course::of` is for the jump courses. An arena with no
stage plays exactly as before.

### The pit: off the edge is out

On a stage with a pit, a fighter whose feet go below it has their health set
to zero that frame, in the fight's own knockout check -- so the round ends
exactly as it does for a knockout, last one standing. Every duel stage's
islands hang at forty metres with the pit at twenty-five; nothing stands
below it (`a_floating_stage_hangs_wholly_above_its_pit_and_starts_you_on_it`).

### King of the Hill

The state is `World::hill` (`versus::Koth`), about thirty bytes, hashed only
on a map with a hill, so every other fight hashes as it always did.

| Rule | Built as | Knob ("Match" family) | First guess |
| --- | --- | --- | --- |
| The point is **locked** at the start of a round, so both sides can reach it | counts down from the round's start | King of the hill, point unlocks after | 10 s |
| **On the point** is anywhere over its disc, **in the air too** -- a column, so hopping and fighting on it keeps capturing -- but never standing on a bridge over it (each map caps the column under its own walkway, `Hill::room`) | | King of the hill, the point counts up to (m) | 7 m, over the Dual mage's 6 m hop |
| One side **alone** on the point **captures** it | progress climbs a frame at a time | King of the hill, a capture takes | 6 s |
| A **second** fighter of that side speeds it up | each one after the first adds a share | King of the hill, a second capper adds (%) | 50 % |
| **Both sides** on it: **contested**, nothing moves | | | |
| **Nobody** on it: a capture under way **drains** slowly | | King of the hill, an empty point drains (%) | 25 % of one capper |
| The **holders** standing on their own point push an enemy capture back | at their capture rate | | |
| The other side stepping onto a part-taken point first **takes it back to nothing**, then starts its own | | | |
| **Holding** the point runs your side's **clock** down | each side has its own | King of the hill, each side's clock | 2:00 |
| Your clock out and nobody challenging: **you win the round** | | | |
| Your clock out while anybody of the other side is on the point: **overtime**, until the last of them steps or is knocked off -- then you win that frame, whatever capture they had under way -- or they take it. Contested in overtime moves nothing either way, as contested always does | | | |
| The **knocked out** stand back up, fresh, at their side's start | after a wait | King of the hill, the dead are back after | 5 s |

Team Fortress 2 runs three-minute clocks and a thirty-second lock for twelve
a side. Fights here are one against one or two against two, and a duel's
time to kill is about a minute, so every number above is shorter and every
one is a guess.

### Sides, and who can hurt whom

**Seats alternate between the two sides** (`versus::side`): seats one and
three are **blue**, two and four **orange** -- the colours of seats one and
two, who lead them. So a hill game with two players is seat one against seat
two, and with four it is two against two.

On a hill **a side cannot hurt its own** (`World::foes`): blows, the shadow's
copy, shields in flight, the Rush's shoulder, fields and thralls all ask it,
and shots -- bolts, debris, the Elementalist's air, the aimed lines -- pass
through team-mates (`aim::Targets::spare`, from `World::allies`). On a duel
stage it is everybody against everybody, as versus always was.

**On a hill the fighters wear their side's colour** (`hud::seat_colour`):
seats one and three blue, two and four orange, bodies and health bars
alike. Everywhere else each seat keeps its own colour, as before.

## 2 · The floating duel stages (`arena/stages.rs`)

Each is **tight** -- twenty-odd metres across, so nobody runs for long -- has
**a few changes of height** of a metre or two, and is turned half a turn
about its middle, so both starts are the same start. Every island hangs (a
ceiling under it as well as a floor on it, as the jump courses' do), and the
marks stand on the island's top (`Mark::up_cm`). The renderer draws them
over a drop (`drop: true`), with land and woods far below.

| Stage | Theme | What is in it |
| --- | --- | --- |
| **Anvil** (`anvil`) | A basalt slab over a sea of embers at dusk | 20 × 13 m face; a low boss in the middle; two stone slabs hovering with 3.3 m of air under them to fight down from; off each end **the horn**, a rock of its own across a 4 m gap and a metre lower than the face |
| **Ziggurat** (`ziggurat`) | A temple's top risen into the desert sky | Three tiers a metre each -- court, terrace, a 4 m crown -- so whoever holds the crown fights downhill; an obelisk at two corners; a stepping stone hung 3.5 m off each long side, 1.5 m down |
| **Caldera** (`caldera`) | A volcano's crater torn loose | 20 m square ash floor inside a 1.2 m lip that stands only at the corners: six metres of nothing at the middle of each side, four ways out; a drum of rock in the middle |
| **Twin Isles** (`twin_isles`) | Two snowy islands at dawn | Two 10 × 14 m isles seven metres apart, a 3 m bridge between them; a knoll on each; a stone hung over the gap north and south of the bridge with 3.3 m of air under it, a way across that is not the bridge |

## 3 · The King of the Hill maps (`arena/hills.rs`)

Every map is **turned half a turn about its middle** ((x, z) to (-x, -z)), so
each side has exactly what the other has; its table is the middle's solids
and one half's, and `hills::whole` turns the half for the other side. What
each is built to have, from Team Fortress 2's King of the Hill maps and
nothing copied from any one of them:

- **High ground over the point**, close enough to fight down from.
- **A way in nobody sees**: a tunnel or covered passage -- the one kind of
  confined space on the map.
- **Open air everywhere else**: from either start to the point and to every
  high place there is a way that never goes under a roof, through at most one
  or two narrow places.
- **No buildings**: the cover is rocks, cut blocks, stones and banks.
- **Every climb a jump the shortest jumper makes** (2.7 m at the top of a
  held jump): a team game shuts nobody out of high ground. This is not a
  movement section (CLAUDE.md's Bulwark rule is about those), so the
  Bulwark walks every route in the test.

Each is about 75 × 55 m, inside a rim. Where high ground backs onto the
rim (the Glacier's shelves, the Gorge's cliff-tops) the rim's bank starts at
that height, so walking off the back of it is walking onto the bank and not
into a crease below it.

### The Quarry (`quarry`)

A worked-out stone quarry. **The point** is on a cut plinth in the middle of
the pit's floor. North and south, **two benches** -- terraces four metres up,
twelve from the point -- are the high ground: each is climbed by a two-step
stair at its own side's end, and by a ledge on its face toward the point.
Through each bench at its foot runs **the cutting**, a tunnel the length of
it (3 × 3 m, 24 m long), and halfway along it opens onto the point through
**the slot**, a gap cut down through the bench: the hidden way onto the
point, and a hole to drop through from the top. Behind each bench a lane runs
in the open; the floor between the benches is open and strewn with cut
blocks; a **tor** by each start is a 4.5 m perch.

### The Glacier (`glacier`)

A snowfield under a glacier's snout. The point is on **the dome**, a hump of
ice three metres up in the middle, climbed by an ice stair at one end of each
long face and a ledge in the middle of each. Under the dome runs **the melt
cave**, east to west -- a sneak from one half of the field to the other
beneath the defenders' feet. Each side's own high ground is **an ice shelf**
five and a half metres up along the rim on its left, climbed from its start,
whose edge looks down on the dome across a gap you can drop from; under each
shelf **another cave** runs from the start out toward the dome. Seracs,
boulders and blocks of ice stand about the open field.

### The Delta (`delta`)

A river's mouth among reeds. The point is on a low island ringed by **four
standing stones**. Each side's half is crossed by **a dyke**, an earth bank
three metres high -- open at a lane at each end and at **the breach** in its
middle, the map's one choke, straight at the point -- climbed by steps on
either face. The dyke's south length hides **the sluice**, a covered channel
inside it from the south lane to the breach: the hidden way onto the point.
Over the north lane runs a broken **aqueduct**, the side's high road, 5.7 m
up from its start to a broken end over the point's north flank. Reeds, a
hummock and a fallen arch are cover.

### The Gorge (`gorge`)

A river gorge twenty metres wide between cliffs seven metres high, the
sides' starts at its two ends. The point is on a gravel bar in the river,
and **a plank bridge** crosses the gorge straight over it, four body heights
up (its planks 6.3–7 m over the bar), from one cliff-top to the other: a
walkway to fight down onto the point from, and to be knocked off. Each
cliff-top is a long terrace of open high ground, climbed out of the gorge by
two ledges at each end. Inside the north cliff runs **the cave**, in from a
bay by the west start and out of a mouth in the cliff beside the point --
the hidden way; the south cliff's is the east side's. The gorge floor is
open, with boulders.

### The Henge (`henge`)

A giant's stone circle on a grass down. Four standing stones seven metres
tall stand nine metres from the point, north, south, east and west, and
**two lintels** cross between them over it in a cross, four body heights
up: a walkway over the point from every side. Each stone is climbed by a
stair of three blocks on its outer face; the west stone is the west side's,
the east the east side's, north and south nobody's. Out by each start a
**long barrow** has a passage through it that comes out beside the south (or
north) stone -- the hidden way. A ring of smaller stones stands about for
cover.

## 4 · Checked by

`crates/sim/tests/versus.rs`:

- every stage is registered, named, and the right kind; `Shift+N`'s order
  visits all nine;
- a floating stage hangs wholly above its pit and starts everybody on it; a
  fall off it ends the round; standing still on it does not;
- the point is standable at its middle, and no start is on it;
- the point is a column: a hop over it counts, and nothing hanging over
  it can be stood on inside the column -- standing on the Gorge's bridge or
  the Henge's lintels is not on the point; **every class hops on the point
  for a whole capture without once leaving it**;
- the hill's rules one at a time: locked at the start, captured, held, the
  clock run, the round won; contested; a capture taken back before the other
  side starts its own; overtime; a second capper faster; the knocked out
  stood back up at their start; a side cannot hurt its own; determinism;
- **every way on every hill map walked** -- each climb, each tunnel and the
  hidden way onto the point -- by the Bulwark, the class with the shortest
  jump; and every height on the floating stages by every class that moves.

The arenas also pass every check every arena does (`tests/arena.rs`,
`four.rs`, `rim.rs`). The game's side is `game::hill` (the point drawn on its
surface in the holder's colour, with the capture growing out of its middle),
`hud::hill_line` (both clocks and the point's state under the bars), the
dressing in `game::arenas::stages`, and a sky each in `look::skies`.

## 5 · Decided while building, for a person to review

- **The mode comes from the map**, not a menu: a hill map is always King of
  the Hill and a floating stage always a duel. A duel on a hill map, or a
  hill on a floating stage, is a line in a `Stage` away.
- **Sides alternate by seat** rather than first half against second half, so
  the two-player game is unchanged: seat one against seat two.
- **A round of King of the Hill is one game**: winning it adds a round win to
  everybody on the side. Rounds are counted as versus counts them.
- **The point is a column seven metres tall** (from play): the first cut
  counted three metres over the surface, and a high jump left it.
- **The Glacier's shelves backed onto a bank that started at the floor**,
  so stepping off a shelf's back dropped you into a crease under a
  five-metre wall; the bank starts at the shelves' height now.
- **The point unlocks after ten seconds**, not thirty: the maps are small
  and the walk to the point is about ten seconds.
- **The Caldera's plug was a cone 1.8 m high** and came down to a drum a
  metre and a quarter: bodies pressed against a cone's slope wall-jumped off
  it instead of climbing it. And a cairn in the middle of the Twin Isles'
  bridge was taken out: it tripped bodies and added nothing.
- **The sides' bodies are their side's colour on a hill** (§1); the third
  and fourth seats' class buttons keep their seat's colour, as a way of
  telling the two team-mates apart in the HUD.
- **More room between the floating stages' pieces** (from play): every
  separate piece is across a real gap now -- the Anvil's horns 4 m out,
  the Ziggurat's stones 3.5 m, the Twin Isles 7 m apart -- and everything
  hovering has 3.3 m of air under it, where it had 2.2. Every one is still
  reached by every class that moves (`tests/versus.rs`).
- **Nothing here has been played.** Every size, every clock and every height
  is a first guess, judged by the walking test and by screenshots.
