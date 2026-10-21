---
status: built 2026-10-21, unplayed
---

# Versus modes — floating stages and King of the Hill

Versus was one rule in one place: last one standing, in the proving ground.
This adds **two modes and seven places built for fighting each other**:

- **Four floating duel stages**: small islands hanging over open air. Knock
  your opponent off the edge and you win the round, the way Smash's ring-out
  works.
- **King of the Hill**, Team Fortress 2's way, on **three maps**: one
  capture point in the middle; you hold it until the other side takes it
  back; the side that holds it long enough wins.

All of it is `sim::versus` (the rules), `sim::arena::stages` and
`sim::arena::hills` (the places), and is tested by `crates/sim/tests/versus.rs`.

**How to get there:** `Shift+N` in game steps through the seven stages (`N`
alone is still the jump courses), and `--arena <name>` / `?arena=<name>`
starts on one: `anvil`, `ziggurat`, `caldera`, `twin_isles`, `quarry`,
`glacier`, `delta`. The mode comes from the map, as it does in Team Fortress
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
| One side **alone** on the point **captures** it | progress climbs a frame at a time | King of the hill, a capture takes | 6 s |
| A **second** fighter of that side speeds it up | each one after the first adds a share | King of the hill, a second capper adds (%) | 50 % |
| **Both sides** on it: **contested**, nothing moves | | | |
| **Nobody** on it: a capture under way **drains** slowly | | King of the hill, an empty point drains (%) | 25 % of one capper |
| The **holders** standing on their own point push an enemy capture back | at their capture rate | | |
| The other side stepping onto a part-taken point first **takes it back to nothing**, then starts its own | | | |
| **Holding** the point runs your side's **clock** down | each side has its own | King of the hill, each side's clock | 2:00 |
| Your clock out and nobody challenging: **you win the round** | | | |
| Your clock out while the other side is on the point or part way to taking it: **overtime**, for as long as they keep at it | | | |
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

**Not done:** the fighters are still coloured by seat, not side, so seat
three is green and seat four violet while playing for blue and orange. The
HUD names the sides by colour; the bodies should follow.

## 2 · The floating duel stages (`arena/stages.rs`)

Each is **tight** -- twenty-odd metres across, so nobody runs for long -- has
**a few changes of height** of a metre or two, and is turned half a turn
about its middle, so both starts are the same start. Every island hangs (a
ceiling under it as well as a floor on it, as the jump courses' do), and the
marks stand on the island's top (`Mark::up_cm`). The renderer draws them
over a drop (`drop: true`), with land and woods far below.

| Stage | Theme | What is in it |
| --- | --- | --- |
| **Anvil** (`anvil`) | A basalt slab over a sea of embers at dusk | 20 × 13 m face; a low boss in the middle; two stone slabs hovering 2.2 m over it to fight down from; at each end **the horn**, a tongue a metre lower than the face, out over the drop on three sides |
| **Ziggurat** (`ziggurat`) | A temple's top risen into the desert sky | Three tiers a metre each -- court, terrace, a 4 m crown -- so whoever holds the crown fights downhill; an obelisk at two corners; a stepping stone hung off each long side |
| **Caldera** (`caldera`) | A volcano's crater torn loose | 20 m square ash floor inside a 1.2 m lip that stands only at the corners: six metres of nothing at the middle of each side, four ways out; a drum of rock in the middle |
| **Twin Isles** (`twin_isles`) | Two snowy islands at dawn | Two 10 × 14 m isles five metres apart, a 3 m bridge between them; a knoll on each; a stone hung over the gap north and south of the bridge, a way across that is not the bridge |

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

Each is about 75 × 55 m, inside a rim.

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

## 4 · Checked by

`crates/sim/tests/versus.rs`:

- every stage is registered, named, and the right kind; `Shift+N`'s order
  visits all seven;
- a floating stage hangs wholly above its pit and starts everybody on it; a
  fall off it ends the round; standing still on it does not;
- the point is standable at its middle, and no start is on it;
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
- **The point unlocks after ten seconds**, not thirty: the maps are small
  and the walk to the point is about ten seconds.
- **The Caldera's plug was a cone 1.8 m high** and came down to a drum a
  metre and a quarter: bodies pressed against a cone's slope wall-jumped off
  it instead of climbing it. And a cairn in the middle of the Twin Isles'
  bridge was taken out: it tripped bodies and added nothing.
- **The sides' bodies are not yet coloured by side** (§1).
- **Nothing here has been played.** Every size, every clock and every height
  is a first guess, judged by the walking test and by screenshots.
