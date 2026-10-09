# The valley — a world to climb with a friend

*Built 2026-10-08, from [exploration/0006](exploration/0006_valley.md), and
**unplayed**. Where this and the note differ, this is what was built and
the note is how it was argued.*

***One map since 2026-10-09.*** *The places are no longer joined by seams you
stand in to be teleported: they are laid out on one map, and you walk from
the town to the Saddle and into every room. How, and the tiles and distance
loading that make it cheap, is [atlas.md](atlas.md); what changed here is
marked below.*

* The code is `crates/sim/src/valley.rs` (the
rules), `crates/sim/src/arena/{hearth,ring,mouth,bank,shelves,pinewood,saddle,highlands}.rs`
(the places), and `crates/game/src/valley.rs` (what is drawn and said).*

**The valley is where the game starts.** `cargo run -p game` puts both
players in Hearth's square. `--versus` is the proving ground as it always
was; `--hunt <creature>` and `--arena <name>` start what they always
started. In the browser the query string does the same (`?versus`).

| Key or flag | What it does |
| --- | --- |
| *(nothing)* | Start in Hearth |
| `V` | Back to Hearth from anywhere, both of you, with the journey kept |
| `--dev` | Also shows how much of the map is loaded, under the place's name |
| `--versus` | Start in the proving ground instead |
| `--open` | Every waystone lit, as if every creature had been beaten: for walking the whole valley without the fights |
| `--arena mouth` (and `bank`, `shelves`, `pinewood`, `saddle`, `hearth`, `ring`) | Start in that place, in the valley |

![The valley, from above](gallery/valley.png)

*The plan sheet: `cargo run -p look --example valley` draws every place from
above in one picture, the climbs' heights as shades and the seams as
colours.*

---

## 1 · The shape: a line, with rooms off it

One climb from Hearth's gate to the Saddle, **one map** (since 2026-10-09:
it was loaded a reach at a time). The creatures' arenas are **rooms** off
the reaches, entered through notches in the walls, so every fight is the
arena it was tuned in. Hearth sits at the bottom of the climb *and* at the
end of the Long Valley, and the last fight walks toward the town it started
in. (It was a loop, with the Long Valley's far end opening off the Saddle;
on one map the climb runs east and the Long Valley west, so it is reached
from Hearth's west gate only.)

| Place | Size | Top | The climb | Rooms off it | The way on |
| --- | --- | --- | --- | --- | --- |
| **Hearth** | 90 × 72 m | — | A stair up the inside of the east wall to the lookout | **the Ring** (north door) | the valley gate east; the west gate to the Long Valley, under the fifth waystone |
| **The Mouth** | 212 × 56 m | 20 m | Scree, then the Step (three shelves), a 6 m gap, a three-ledge traverse up the Lip's face over a 15 m slot | the low meadow (Hornback herd) | the Lip |
| **The Bank** | 172 × 48 m | 24.5 m | Ten shelves zig-zagging up the bank, a snow balcony, then the gully | the den (Gnawers), through a notch in the north wall (it was a cave in the bank's face, with no room behind it for a room) | **first waystone**: one of the herd, the den |
| **The Shelves** | 220 × 56 m | 32 m | A ledge along the north wall 20 m over the hollow, broken by 5 m and 6 m gaps; then a four-ledge chimney | the Mire (Mireback), the Pan behind a 7 m bluff (Sandmaw) | **second waystone**: one of the Pan, the Mire |
| **The Pinewood** | 200 × 64 m | 26 m | Trunks; an eleven-ledge chimney between two giant trunks; the band | the Den (the Pair), the Hollows under a root roof (Broodmother), the Highlands up a six-block stair (Ridgeback) | **third waystone**: two of the Pair, the Broodmother, the Ridgeback |
| **The Saddle** | 170 × 64 m | 22 m; the Shrine 36 m | The ridge, three metres wide and 20 m up, broken twice over updrafts; a six-ledge chimney between two spires | the Cliffs (Galewing), the Ashwood (Veilstalker); the Shrine (Mantis), over a bridge from the second spire's top through the north wall, behind the **fourth waystone**: one of the Cliffs, the Ashwood | none: the east passage is a dead end now, its fifth waystone dark |
| **The Long Valley** | the Siegeshell's own | — | — | it *is* the room: the Siegeshell walks it toward Hearth | out of Hearth's west gate, under the **fifth waystone**: the Mantis |

The tiers are world.md §3's, and the gates are exactly what that table says
(`valley::tier`). **The Ridgeback has a room of its own**, the Highlands: the
proving ground's floor plan on a moor, so meeting it in the valley is the
fight it was tuned in, and `arena::for_species` still sends `--hunt ridgeback`
to the proving ground.

Not built from world.md §2: the Crossing (the herd's escort mode stays a
`--hunt hornback-escort` fight), and looking into a room from the trail to
see the creature's idle life. A room is behind its seam until you walk in.

## 2 · The rules

All of them are in `World::step_valley` and `sim::valley::open`, and all
of them only run when the world is the valley (`Journey::on`). Outside it
nothing here happens and nothing here is hashed, so every pinned fight
hashes as it did.

- **A seam** is the mouth of a passage or a notch where two places meet, and
  seams come in pairs (`tests/valley.rs` checks every pair). Since
  2026-10-09 it is a doorway, not a trip: **you walk through it**. The world
  runs in the coordinates of the place the first fighter on their feet is
  in, and moves into the next place's when they cross (atlas.md). Every
  doorway is walked through by `tests/valley.rs`.
- **You no longer have to go together.** The rule that a seam fired when
  everybody stood in it, or one had held it for `seam_hold`, went with the
  teleport: there is nothing to go *to* that the other cannot walk to after
  you. The two of you can be apart; one live hunt at a time.
- **A waystone** gates a seam that leads on. It is lit when enough of its
  tier have been beaten, counted from the journey's bitmask. An unlit one's
  doorway is **a wall**, drawn as a dark slab, and the screen says what it is
  waiting for; it stands only while nobody is on its far side, so going back
  down is never gated.
- **What has been beaten** is the journey's, and a creature gets there two
  ways: a hunt won in the valley, and each player's trophies. A trophy is
  off the snapshot, so the game sends each one on the wire as a trip byte
  (`Destination::Credit`) and both peers credit it on the same frame. The
  pair's journey is the union of both players' trophies: the note's "the
  more travelled of the two", and world.md §5's "a locked place is locked to
  the lone player, not to the pair".
- **A cairn** is a snow top. Touch one and you are rested (health back) and
  it is your checkpoint.
- **Nobody dies in a reach.** A fighter whose health runs out — a fall
  costs what the fall rule says, free to 9 m and 25 a metre past it — is
  stood on their last cairn, or on the place's arrival marks if they have
  touched none, with their health back.
- **Peace.** In the town and the reaches the two of you cannot hurt each
  other: `World::pvp()` is false, and every place that used to ask "is this
  versus" asks that instead. **The Ring is the one place for fighting each
  other** (the ask was "make sure Hearth has an arena for PvP"): the proving
  ground's square to the centimetre, sand for earth, rounds as versus plays
  them, the dummy or the sparring bot as player two when nobody has the
  second keys. Hop its south wall and walk into the door to leave.
- **A room starts its hunt** when anybody walks into it. Win it and the
  place goes quiet: the round does not restart, and you walk out the way you
  came. Lose it and you wake outside, at the seam you went in by. You can
  run: when nobody on their feet is left in the room, the hunt is over,
  unwon, and the creature is gone; walk back in and it starts again.
- **A room has ground round it.** Most rooms are walled with low walls,
  which ended the world when a room was all there was. On one map a room has
  six metres of ground round its walls and a cliff round that.
- **Player two, unplayed,** is not in the valley: kept a kilometre under the
  place's floor where nothing reaches it, rather than a body lying at every
  seam. In the Ring and in a room the second seat plays as it always has.

The journey is a few bytes of the snapshot: whether this is the valley, the
beaten bitmask, the seam you came in by, and each fighter's last cairn (a box
of the map, by its index there).

## 3 · Climbing, as built

The note's vocabulary — scree, shelf, stair, traverse, gap, chimney, bluff,
cairn — is spelt in boxes and in **relief that climbs**: a reach's floor
rises along its length by ramps (`relief::Ramp`, a smoothstep between two
lines), so the scree is a slope you run up and the Mouth's river terrace is
lower than its meadow. Terrain-sized boxes are drawn as cliffs (forms.md),
roughened in metres so their edges stay where their collision is.

**The route has two speeds, and nobody is ever stuck.** Every climb a class
might not make has a slow way round: a vine up the face, a stair, the
hollow. The hops that have no way round are listed in `tests/valley.rs`
(`REQUIRED`), and every class's plain running jump is checked against each
one with `sim::envelope`. The Bulwark's jump is what sized them: its 2.7 m
apex and 5.2 m reach on the level set the Mouth's first traverse jump at
2.6 m out and 1.5 m up.

Two props from [courses.md](courses.md) §4 are built, because the note
predicted the Bulwark, the Blood mage and the Champion would have no answer
to a bluff:

- **Vines.** A strand of green down a face. Against one, hold jump to climb
  at `vine_speed` (2.5 m/s), crouch to slide down, otherwise cling. A vine
  counts as footing, so letting go is a fall from where you let go, not from
  where you started. Fourteen of them across the five reaches.
- **Updrafts.** A column of pale rings. A body in the air inside one rises
  at `vent_rise` (8 m/s) up to its top, then falls as usual. The Saddle's
  ridge is crossed by them, and two off the Saddle's floor are how you get
  back up after falling off it.

Both are tables in the place's file (`PLACE.vines`, `PLACE.vents`); the
motion is `World::climb_and_ride`, before the body moves, and it is the
same for every class.

## 4 · What is drawn and said

`crates/game/src/valley.rs` draws, for a place that is in the valley:

- every seam as a glow on its floor and a column of light, coloured lit or
  unlit by the look crate (`Palette::beacon`). A room's exit has the glow and
  no column, because the hunters arrive standing in it. None of it shows
  outside the valley;
- a dark waystone's doorway as a slab of its grey, while it is shut;
- each waystone's light on its stone's top;
- vines and updrafts;
- from Hearth's wall, the reaches themselves, as far as the fog lets you
  see. (The painted ridges that stood in for them went when the valley
  became one map.) Only what is within the sky's reach is drawn
  (`game::stream`, atlas.md);
- one line under the scoreboard: the place, how many creatures the journey
  has beaten, and while somebody stands in a seam, where it leads or which
  waystone is dark.

## 5 · Measuring it

The note said the instrument is a pair's replay, not a search, and the
harness reads one now. `cargo run -p hunt --bin replay -- <tape>` on a tape
recorded in the valley prints **THE VALLEY**: every place visited, how long
the pair spent there, falls, and how each was left — by which seam, and
whether both were in the next place when the world moved into it
("together") or not ("apart"). `crates/hunt/tests/replay.rs` walks two fighters out of
Hearth's gate and into the Mouth and checks the report says so.

The target was **three minutes end to end for a runner who knows it, five
for a pair who does not.** The five reaches are 974 m end to end with about
125 m of climbing, close to the note's arithmetic for three minutes. Nobody has run it yet.

## 6 · Open

1. **Nobody has played it.** Every height, gap and hold is a first guess
   sized against the jump envelope. Record a pair's run (`Y`) and judge it.
2. **The Bulwark sets the gaps.** Every required hop is within its plain
   jump, which makes them easy for everyone else. The mechanics are the
   shortcuts (a stone, a shadow, a Grasp over a bluff), but the gaps are not
   yet wide enough to *need* one anywhere. Whether some climbs should have
   only the slow way for some classes is the next tuning question.
3. **The Siegeshell does not come to you.** World.md §3 wanted the bells to
   ring in Hearth when the Shrine's waystone lights. It is walked to, by the
   fifth waystone, from either end.
4. **The armoury, the bot's corner and the hunter's notes** in Hearth are
   solids with nothing in them yet.
5. **The seam hold** is gone with the teleports (2026-10-09), and its knob
   with it.
6. **One map**: everything [atlas.md](atlas.md) §Open lists -- how the joins
   read, the sky changing at a doorway, the Long Valley's loop.
