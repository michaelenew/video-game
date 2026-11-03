---
status: built 2026-10-21; the stages rebuilt bigger 2026-11-02 (fight-spaces.md), unplayed
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
`gorge`, `henge` (and each hill map's `bigger_` and `biggest_`). The mode comes from the map, as it does in Team Fortress
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

**Rebuilt 2026-11-02 to [fight-spaces.md](fight-spaces.md)**: each main
island is still under a reach across (25 m), because the edge is the point of
a ring-out stage. What they gained is *pieces*: islands across real gaps,
spread over about 47 m, starts 21–22 m apart, a high level 8.4 m over the
starts, lifts and ferries that move, and **a recovery**: two narrow updrafts
per stage beside an edge, which a fighter knocked off can steer into and ride
back up. Every island hangs (a ceiling under it as well as a floor on it),
the marks stand on the island's top (`Mark::up_cm`), and each is turned half
a turn about its middle, so both starts are the same start. The renderer
draws them over a drop (`drop: true`), with land and woods far below.

| Stage | Theme | What is in it |
| --- | --- | --- |
| **Anvil** (`anvil`) | A basalt anvil over a sea of embers at dusk | A 23 × 14 m face with a narrower waist under it; a low boss in the middle; a 3 m basalt block each side for cover. **A hammer** over each half: a 6 × 5.5 m slab 8.4 m up, climbed by two hovering stones (2.7 m, then 5.4 m) or reached by **a lift** off the face's long side. **The horn** off each end: its own rock across a 5.5 m gap, a metre lower. **A recovery updraft** beside each long side |
| **Ziggurat** (`ziggurat`) | A temple's top risen into the desert sky | A 23 × 17 m court; three 2.7 m tiers to a 5 m crown 8.4 m up; **a dais** each side, a slab flat on the court that lifts you to the crown's height, then a level jump onto it. An obelisk at two corners. **A stele** off each long side: an island 7 m out and 3 m down, jumped down to, with **a ferry** across the gap as the way back. **A recovery updraft** off each end |
| **Caldera** (`caldera`) | A volcano's crater torn loose | A 22 m square ash floor inside a 2 m lip that stands only at the corners: 8 m openings on four sides. **The plug blown out**: two slabs 8.4 m up with 5 m of sky between them over the middle. **The vent still breathes**: an updraft out of the crater floor throws you up past them. Each slab is climbed from a corner of the lip by a hovering boulder, or reached by **a lift** out of the side gap. **An ejecta island** 6 m off the east and west gaps, 2 m down. **A recovery updraft** beside the corners |
| **Twin Isles** (`twin_isles`) | Two snowy islands at dawn | Two 10.5 × 15 m isles 8 m apart, joined by a 3 m bridge. **An ice arch** 8.4 m over the bridge, climbed by two seracs on each isle (2.7 m and 5.4 m, cover too). Hung stones north and south of the bridge with 3.3 m of air under them, and under them **an ice floe** drifting across the gap. **A skerry** 5 m off each far end, a metre down. A low knoll on each isle. **A recovery updraft** off each isle's outer side |

## 3 · The King of the Hill maps (`arena/hills/`)

**Rebuilt 2026-11-02 to [fight-spaces.md](fight-spaces.md)**, after the
owner: "everyone just stands in the middle and spams abilities". Every map
is now **128 × 92 m** (five reaches by three and a half), its starts **58 m**
from the point, with **three levels** -- the floor, a middle level at 6.5–8 m,
a top at 14.5–16 m -- **updrafts and lifts** to climb by, and structure in
every ring round the point: the point itself, the perches that can hit it,
and cover and high ground out past a reach. Each map is still turned half a
turn about its middle ((x, z) to (-x, -z)), so each side has exactly what the
other has; one file a map, its table the middle's solids and one half's.

**Every map comes in three sizes** (`arena::hills::sizes`): as built
(*big*), **bigger** (stretched across the floor by five quarters: 160 ×
115 m, starts 72 m out) and **biggest** (by three halves: 192 × 138 m, starts
87 m out). Every height is kept, so every stair, ledge and level is what it
was; the point stays four metres across; a walkway over the point keeps its
width; and every way the map promises is walked on all three sizes. `Shift+N`
steps through a map's three sizes before the next map, and the Esc menu has a
row a map: *big · bigger · biggest*. `--arena bigger_quarry`,
`biggest_henge` and so on. The plans of all fifteen are
[gallery/hill-sizes.png](gallery/hill-sizes.png).

What each is built to have, from Team Fortress 2's King of the Hill maps and
nothing copied from any one of them, now held by `tests/fight_space.rs`:

- **High ground over the point**, and higher ground past a reach from it.
- **A way in nobody sees**: a tunnel, a cave, a covered channel.
- **At least three ways onto the point**, of different kinds, and ways off it.
- **Something that moves you**: an updraft or a lift.
- **No buildings**: the cover is rocks, cut blocks, stones and banks.
- **Every way walked by all five movers** (`sim::class::MOVERS`). Until
  2026-11-02 every climb was sized to the Bulwark's 2.7 m jump; it is not on
  offer, and the maps could not be tall while it was the yardstick.

### The Quarry (`quarry`)

A terraced stone quarry. **The point** is on a cut plinth on **the pit
floor**, under open sky. North and south, the quarry's faces climb in two
benches the length of the pit: **the lower bench** at 6.5 m and **the upper
bench** at 14.5 m against the rim. In the middle each face **juts** out toward
the point: its lower bench 14 m from the point, the contest's high ground; its
upper bench 26 m back, just out of a reach, overlooking everything. Through
the foot of each jut runs **the cutting**, a tunnel 4 × 4.5 m and 40 m long;
halfway along **the slot** opens straight onto the point -- the hidden way in,
and a hole to drop through from the bench. **A crane jib** sticks out from the
upper bench over the slot, 14.5 m up, to be knocked off. **The hoist** on each
jut lifts you from the lower bench to the upper on an 11-second cycle. **Two
gantries** cross the pit from jut to jut 16 m either side of the point, 6.5 m
up, each on a pier to hide behind. Each side climbs the face on its left by
**the steps** at its own end, and the face on its right by **a spoil ramp**
and **an updraft** up the upper bench; an updraft or a ledge on each jut's
face is the quick way up off the point. Out past a reach: a great cut block
to climb, a tor, spoil heaps, a low wall before each start.

### The Glacier (`glacier`)

A snowfield under a glacier's snout. The point is on **the dome**, a hump of
old ice 4.6 m up, climbed by a stair of ice at each side's corner, a ledge in
the middle of the north and south faces, and a step down from the gate serac.
Under each end of the dome runs **a melt cave**, 4 m tall -- the hidden way --
ending under a hole in the dome's top beside the point, where **the geyser**,
an updraft, throws whoever steps in up onto the point. Each side's high
ground is **the snout** on its left: **the shelf** 8 m up (a rock stair from
the start, **the cornice** to catch, or **a calving lift**) and **the ridge**
behind it 16 m up (a stair of ice on the shelf, or **the moulin**, a shaft
through the shelf whose updraft carries you to the ridge's lip). From the
shelf's front **an ice bridge** crosses 7.2 m over the approach to **the gate
serac** (8 m); beside it **its twin** (11 m), a 2 m chimney to kick up
between; from the gate a 5.6 m step drops toward the dome -- the high road
onto the point. **A floe** docks at the shelf's east end and slides out over
the dome's flank and back. South of the dome stands **the sentinel**, a 9 m
serac with a 5 m shoulder and an updraft up its face. Out past a reach: a
leaning serac, a block of ice, a moraine boulder, a snow wall before each
start, drifts of snow, and crevasses in the corners.

### The Delta (`delta`)

A river mouth with its waterworks in ruins. The point is on a low island,
ringed by four standing stones taller than a jump, under open sky. Eighteen
metres out each side runs **the dyke**, an earth bank 6.5 m high the width of
the map -- the middle level -- cut in its middle by **the breach**, ten metres
of open floor straight at the point. Across the breach's mouth **the sluice
gate**, a timber gate that sinks flush with the floor and rises on a 12.5 s
clock, shuts the breach and carries whoever stands on it up onto the dyke; the
two sides' gates run half a cycle apart. Inside the dyke's south length runs
**the sluice**, a covered channel 4.2 m tall -- the hidden way -- fed by **the
old lock**, an 8 m trench from the start's flank with an updraft in it. Over
the north half runs **the aqueduct**, the high road, at 14.5 m, from the top of
**the water tower** out of reach near the start, across **a 3 m break** to
jump, on over the dyke and the point's flank on tall piers with arches to
fight under, to a broken end past the point. The tower is climbed by an
updraft up its back or by **the water-lift**, a stone float rising against its
face; the aqueduct by steps from the dyke or by **the spout**, an updraft up
the pier nearest the point -- the way up and out of the point. Mud banks, a
knoll and reed islands roll the floor; a fallen span is cover by the point.

### The Gorge (`gorge`)

A river canyon fought up and down three levels. The floor is the gorge
bottom, 28 m wide, the river down its middle and a side's start at each end;
**the point** is on a gravel bar in the river, barely proud of the water.
West of it stand **the stacks**, three rock pillars 2.4, 4.2 and 5.6 m tall,
a perch line over the point. Seven metres up **a terrace** 14 m deep runs
along each cliff; **the plank bridge** crosses from terrace to terrace
straight over the point -- the one thing over it -- and **a rock arch**
crosses again 28 m out at each side's own end. Each terrace is climbed by
**a stair** of blocks, by **the hoist**, a timber crib that rises out of the
shingle 12 m from the point to the terrace lip, and on the north by **the
cave's skylight**. **The cliff-tops** are 15 m up and 28 m out: high ground
past a reach, with a tor each, climbed by **a thermal** up the cliff from the
bay beside a start, another from the terrace, and a stair of blocks. **A
cable car** crosses between the cliff-tops at each side's end, high over the
gorge. **The cave** runs inside the north terrace from the bay by the west
start, 35 m, 5 m wide and 4.5 m tall, out of a mouth beside the point: the
hidden way, with a skylight to drop through and an updraft back up.

### The Henge (`henge`)

A giant's stone circle on a rolling grass down. The point is a worn altar
slab nearly flush with the chalk. Twenty metres out six great stones, seven
metres tall, carry **a ring road of lintels**, four metres wide with
eight-metre gaps under it to walk through. North and south the ring is broken
by **two giant trilithons**, neutral ground: fifteen-metre uprights a gate
apart, their tops the highest ground on the map. One **long lintel** crosses
over the point from the west stone to the east: the high road onto the point,
to drop from or be knocked off. The ring road is climbed by a two-block stair
on your own west stone, or by an updraft against a stone's outer face; the
giants' tops by **corbels** jutting from a giant at one end of your arc, a
**ley-line updraft** up a giant's face at the other, and a ley line on each
giant's outer face straight from the down. Out past a reach: an avenue of
standing stones from each start, an outer ring of smaller stones, **a long
barrow** on each side's south flank whose 4.4 m passage comes out by the
circle -- the hidden way -- and whose roof is a perch, and **a dolmen** on
the north flank, climbed by an updraft or a propped stone.

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
- **every way every stage promises walked** (`Stage::ways`) -- each climb,
  lift, tunnel and hidden way onto the point, on all three sizes of every
  hill map, and every height and lift on the floating stages -- by every
  class that moves; and every stage held to the fight-space rules
  (`tests/fight_space.rs`).

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

**Since 2026-10-27 a room whose link is the valley plays in groups**
(`net::group::Room`, [exploration 0029](exploration/0029_networking_at_scale.md)
§14): everybody starts alone where the link puts them, and whoever is near
shares a world with them; nobody is seated and nobody is sent a world. What
follows is a room for one fight -- an arena or a hunt -- which plays as it
always has. A group that starts a match from the valley plays it among its
own members, as a group.

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
