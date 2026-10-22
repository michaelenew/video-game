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

### Every versus round opens with a pick (2026-10-22)

From the owner: a class change used to restart the whole fight, which
against people is everybody's round stopping because one player pressed
Tab. Now **a class change never resets anybody**, in any mode: the fighter
who asked is recast where they stand (`World::recast_in_place`) -- the
same spot, facing, wins and share of their bar, so it is not a heal -- and
whatever their old class had out in the world is taken back.

In versus that alone would let a player swap to whatever counters the
fight in front of them, so **a versus round opens with a pick**
(`Phase::Picking`): fifteen seconds ("Versus, time to choose classes before
a round", Match family) in which everybody stands on their marks, moves
about and tries their moves, nobody can be hurt (`World::pvp` is false),
and a fall off a stage is back on your mark. `Tab` and the class pickers
work; `Enter` says you are done (`Travel::ready`), and the round starts as
soon as every seat is ready or the clock runs out -- fresh on the marks,
with **the classes locked** for the round (`World::classes_locked`: the
request is not heard). The next round opens with the next pick. At one
keyboard, `Enter` readies both seats, since the dummy cannot press it.

It applies to the duel in the proving ground, the floating stages and the
hills -- not to a hunt, a jump course or the valley, where a class can be
changed whenever anybody asks. It is on in a world the game plays
(`World::drafting`, `draft=1` on a replay's start line) and off in one
built bare, which is every test and harness duel that fights from its first
frame, and the game's own scripted runs (`DEMO=1`, `SHOT_FRAME`). Pinned by
`crates/sim/tests/recast.rs`.

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

## 6 · A room you come and go from

*Built 2026-10-22, from the owner: "players should be able to join and leave a
room at will -- a 4 player room is max 4 players, and players leaving doesn't
mess everyone else up. The match starts when everyone currently joined clicks
Start match; people joining during the vs phase go to whichever team is
smaller, randomly if they're the same size, dropped right into the fight on
whatever class they joined the room as."*

**What a player sees.** A room's link carries its size (`players=`, chosen
beside *Create a room*). The first two in start playing together at once, in
the world the link describes; anybody else who opens the link is let into
the match going on, up to the room's size, and a fifth is told the room is
full. Everybody in the fight presses **Start match** in the Esc menu (it says
how many are ready); the frame the last of them does, the match starts on
every machine at once -- a fresh round of the versus stage they are on, or
the first floating duel from anywhere else. Somebody joining after that
drops into the fight on the smaller side, as what they came in as. Somebody
leaving -- Leave, or closing the page -- is let go within a second, and
their seat is the next newcomer's.

**How it works.**

- **The world's half** (`sim::state::World`): `ready`, a flag per seat flipped
  by `Travel::START`, and `all_starting` starting the match inside the step, so
  the frame is agreed by the inputs themselves. `open_seat` picks a free seat
  on the smaller side (`versus::side`, seats alternate), a coin when level;
  `seat_join` stands a newcomer where a fresh start of the same fight would
  and touches nobody else. `Travel::admit(seat, class)` carries it on the
  wire. `crates/sim/tests/join.rs`.
- **The room's half** (`net::lobby`): a lobby that stays open for the room's
  life. The **lineup** -- who is in which seat, numbered by an epoch -- is the
  host's (the lowest id still in it, never the newcomer it is letting in),
  posted on the board. Every pair in it has a line, and lines outlive a
  session (`Shared`). Notes are version three; a hello says whether its
  sender is already in a lineup, so a newcomer never makes a first lineup of
  its own.
- **The session's half** (`net::drop_in`): one GGRS session per lineup.
  GGRS's player `k` is world seat `lineup.members[k].seat`, and a seat
  nobody in the session plays reads as left (`handle_requests_seated`). When
  somebody comes, the host's game picks their seat and the lobby posts the
  new lineup; once the newcomer has a line to everybody, the host's own
  input carries the admission, so every machine seats them on the same
  frame. When that frame is confirmed, everybody starts the next session
  from the world at it -- a hitch of the few frames played past it -- and the
  host sends that world to the newcomer in chunks until they say they have
  it, checked against its checksum. Each session's GGRS messages carry its
  epoch (`Mesh::for_epoch`): one peer's new session and another's old one
  share the lines for a moment, and GGRS indexed past its players on a
  message from a session of another size.
- `crates/net/tests/drop_in.rs` drives four peers the way the game does --
  two start, two more drop in on the smaller side, everybody starts the
  match and plays on in step; one of four leaves and the next takes the seat;
  a fifth is turned away; a newcomer with the lowest id of all still joins.

**Found on the way, in four desktop copies over a local broker**:

- A desktop line read as gone on any error its WebRTC library reported -- a
  stray packet's included -- so every desktop match dropped a friend a few
  seconds in, after the leave fix of #219. Open, a line is gone now only if
  its channel closed or the connection died (`net::native`).
- A desktop line dropped without a word, so a player pressing Leave held
  everybody for the session's fifteen silent seconds; it closes politely
  now, and the others let the seat go in about half a second.
- The lobby, looking for the world on a line before its session ran, threw
  away a peer's GGRS handshake, and GGRS's retry timer could put it off for
  good; packets it reads that are not the world are set aside for the
  session now.
- A crowd aggregate a cell's leader sent could stand in for one a machine
  summed itself, depending on when it arrived -- a desync the frame a match
  started. Within one session every live region is summed by everybody, so
  a leader's word on a live region is not taken
  (`regions::Ledger::offer_heard`).

**Decided while building, for a person to review.** The match starts with
two, not when the room is full. The newcomer's side, when the two are level,
is the low bit of their id. A session switch rewinds the frames played past
the admission -- at most the rollback window, twelve. A newcomer who joins
from a desktop into a page's room relies on the world's layout being the
same on both; the checksum check says so plainly if it ever is not. Somebody
whose game vanishes without closing (a crash, a lost network) is still let
go only after GGRS's fifteen silent seconds, which are there for a page
drawing an arena for the first time.
