# Tutorials — a hall for each class

**Built 2026-10-23, unplayed.** `--tutorial <class>` (`?tutorial=reaver`), or
the Esc menu's **Tutorials** row. Code: `crates/sim/src/tutorial/` (the
tables and the tally), `crates/sim/src/arena/hall.rs` (the halls), and
`crates/game/src/tutorial.rs` (the panel). Held by
`crates/sim/tests/tutorial.rs`.

## 1 · What it is

Each class on offer has a **hall**: a long, wide gorge of rooms walked down
`+x`, a doorway between each two. Walking into a room opens it — which room
you are in is how far down the hall you stand, nothing else. The panel on the
right says the room's title, what it is for, and a checklist.

- **The first room is the premise**: what the class is, how it plays, how it
  wins, in plain words — and a jump and a dodge to get your feet.
- **Then one room per click** — left, middle, right — with every way that
  click is thrown, in order: on the floor, the string or the hold, held to
  the end, in the air, off a jump (space + click).
- **Then the keys**: Q, E, shift and whatever moves her somewhere — a dash, a
  blink, a teleport, a send.
- **Then the advanced rooms, one technique each**, floored in dressed stone
  and tagged ADVANCED. The first one says so in as many words: these get the
  most from the class, and it plays well without any of them.

Basic rooms ask for **each move once**, to get a feel for it; together they
are a few minutes. Every task has a **checkbox**, ticked the first time, and a
**count** (`×3`) beside it every time after, so practising in a room is
counted too. A task that should strike the dummy waits a little after the move
begins (`tutorial::LANDING`, 75 frames — long enough for a bolt to cross a
room) and says **Hit** or **that missed the dummy**.

**The dummy follows you.** It stands a third of the way into the room you are
in (with somebody else in the hall too, the first of you by seat), facing the door you came through — or aside by the north wall, in a room
about getting somewhere — and is stood back up to a full bar every frame. You
are topped up whenever you run low. Nobody is knocked out of a lesson, and no
round ends in a hall (`World::step_hall`). One room has a dummy that
**thrusts a spear** on a count the panel shows in tenths: the Champion's
clash (`tutorial::dummy_input`).

**Tab walks you into the next class's hall**: a hall always teaches the class
you are playing (`game::tutorial::steer`). What you have done is kept per
class for the session, beside the fight rather than in it.

## 1b · A shared space (2026-10-25)

The owner, from a replay of two people playing: "make it so that the
tutorials are a shared space, one person starting one doesnt teleport
everyone there but two that each go there can see each other, and green
checks work per person even in multiplayer."

- **The halls are on the valley's map**: a row of them past its north edge
  (`valley::layout::hall_at`), each on its own floor inside its own rim, with
  no door (`valley::Kind::Hall`). A hall is a place in the same world as
  everybody else, not a world of its own.
- **Going to one takes only you.** In the valley a hall's trip, V and a
  cairn from the menu move the seat that asked and nobody else
  (`World::go_alone`): two people who each go to the Champion's hall are in
  it together. `--tutorial` still starts in the hall, and so does a hall's
  trip from outside the valley -- now as the valley, arriving there.
- **One dummy a hall, shared.** A hall somebody is in takes the first seat
  nobody plays as its dummy (`World::dummies`, `World::step_hall`); it
  stands in the room of the first of them (by seat) and thrusts at them in
  the clash room. Its buttons are the simulation's own
  (`tutorial::dummy_input`), so every machine drives it alike. Two halls in
  use, two dummies. A room for four has no seat to spare, and a hall there
  has no dummy: what does not have to strike one still ticks.
- **Who may hurt whom is asked of the pair** (`World::foes`): a hall's
  dummy and the people learning in that hall, and nobody else -- two people
  in a hall never hurt each other.
- **Your list is yours.** `Progress::observe` takes the seat it tallies, and
  the game feeds it this machine's seat, offline and online (it was fed
  only offline, and only seat one: online, nothing ticked). Each frame is
  counted the first time it is made, as it is sounded.

Held by `tests/halls_together.rs` and, under rollback,
`net/tests/ggrs_synctest.rs` (`going_alone_into_the_halls`).

## 2 · How a task is read

A task is a function of **two frames of the world** (`tutorial::Seen`): the
same move `sound::cue` and the fight report make — read the world twice, see
what changed — so nothing about teaching is in the step and the tally changes
no fight. The usual questions are on `Seen`:

| Question | What it reads |
| --- | --- |
| `began(kind)` | the move's windup began. A hold's windup comes when the hold is done, so a channel that becomes a tap is the tap beginning. |
| `out(kind)` | its first live frame. The Champion's taps, the Blood mage's lash and sweep: their tap and hold share a windup, so only the live frame is certain. |
| `channelled(kind)` | a hold began. |
| `shadow()`, `effects(kind)`, `appeared(kind)` | the Reaver's shadow; an effect of hers counted before and after. |
| `struck(dummy)` | its bar is short of full by more than a curse's tick, a stun began, or it was marked. |

Where the code has no flag for a thing, a task reads what it leaves behind,
and these are the proxies chosen (from reading each class's code, 2026-10-23):

- **The Champion's clash**: `fury.clash_by` lives less than a frame, so a
  clash is his `parried` rising to `PARRY_FLOURISH`, his health unchanged and
  the dummy staggered. A **Rush** is the recharge jumping: a finisher's drive
  sets the run without spending one.
- **The Reaver's dash jump** is the shadow's `flip` starting; **Tumble into
  Plunge** is a Plunge begun aloft with the trip's takeoff spent; a **cash**
  is the dummy's marks going to nothing on a frame it was struck with a
  finisher out (a fade takes one, and does not hurt).
- **The Elementalist's structure jump** is her feet crossing 8 m (the double:
  25 m) with no wind of hers under her. A plain jump is 5 m, off a stone's
  top 6.8; the rehearsal's timing reaches 25.9. A **raise** is a slot holding
  a newer stone than it did.
- **The Blood mage's teleport** is a dodge that moved her more than
  `BLINKED` in a frame; a **drink** is red rising as grey falls; a **strong**
  move is one begun on her ground with `thrown_at` above one.
- **The Dual mage's blink** is a dodge that moved her more than a metre in a
  frame; a **purge** is three curses or more gone at once on a struck frame
  with a Light move out.

## 3 · The halls

Room by room; the tables in `crates/sim/src/tutorial/<class>.rs` are the
truth, with the exact keys each task asks for.

**Champion** (11 rooms) — premise · sword (three-hit string, Crescent,
Thousand cuts, air, Rising cut) · hammer (string, Shockwave, Meteor, air,
Uppercut) · spear (string, Repel, Heaven-piercer, air, Pole drive) · E (Rush,
a Rush attack, Rush out of a swing) · *advanced:* on the beat (true links) ·
momentum (fill it, spend it on a finisher) · the pole vault (a shelf to try
for) · **jump, then right click** (the Pole drive as a second jump, then the
air spear) · the clash (a swinging dummy) · Rush into the string (shoulder,
the refund, Rush out).

**Shadow Reaver** (10) — premise · finishers (Reave, Plunge, Gallows) ·
thrown blades (Shuriken, Tumble, Fan) · the shadow (send, send again, a
copied swing, Whirl, Rising whirl, walk onto it) · Q, E and the dash
(lotus, Executioner, dash, dash from the air) · *advanced:* **the dash jump**
(over a gap) · **Tumble into Plunge** (the midair jump, then left click) ·
marks and cashing them (three, two or more cashed, all five) · dragging the
lotus · cutting a recovery short with right click.

**Elementalist** (10) — premise · earth (Raise, Fissure, Earth jump,
Landfall) · fire (pillar, Strike, carpet, fountain) · wind (small and grown
air balls, Gale, Updraft on the floor and in the air) · Q, E and the dodge
(kick a stone, Cataclysm, break through a stone) · *advanced:* **the
structure jump** and the double · **exploding a stone** (shards, a burning
stone, the shatter jump) · **wind through fire** (a burning Gale) · the
thermal (over a gap) · pushing out of a recovery.

**Blood mage** (11) — premise · thorns (lash, Bramble, Vines, air,
Thornfall) · spikes (one, a ring, Cascade, Stake, Upheaval) · the scythe
(Sweep, the third sweep, Reap, Harvest, air, Rising reap) · Q, E and shift
(Coagulate, a heart, teleport, swim) · *advanced:* fight on your ground ·
bleed then spike · drink it back · spend your grey · swim and surface ·
hearts and thralls.

**Dual mage** (11) — premise · Dark (the ball floor and air, the hop, Chain
lightning, Veil) · both at once (Binary, Phase, the Flurry dash, Equinox) ·
Light (the ball, the Disk ray, Sanctuary) · the keys (Devour, Smite, Abyss,
Judgement) · *advanced:* switch hands · vent a runaway bar · curse then purge
· the hop chain · what the lower bar unlocks (blink, second jump, wings) · the
Flurry dash jump (over a gap).

**The Bulwark has no hall**: it is not on offer (`class::ROTATION`), and
putting it back is a table in `tutorial/` and a line in `tutorial::ALL`.

## 4 · The hall's ground

`arena::hall` builds each hall from its class's rooms, so the two cannot
disagree: 24 m a room, 18 m wide, posts either side of a 9 m doorway, a rim
of land too steep to climb along both sides and both ends (`arena::rim`, as
round every fight). A room's `Bay` is what it has besides its floor — a
**shelf** across its far end, a **wall** across its middle, or a **gap**
between two low shelves — each with a lane round it at the north side so
nobody is stuck below it. `tests/tutorial.rs` walks every hall end to end
with every mover.

## 5 · Open, and decided on the owner's behalf

- **Everything is unplayed.** The texts were written from the kits and the
  code; the move names are the code's.
- **Basic rooms ask for the move, not the hit, where a hit is a matter of
  aim that a first try should not fail on** — Gallows against a dummy on the
  floor (it drives up past it), the Fan, the air hammer and spear, most
  takeoffs. Every move that plainly should land on a dummy in front of you
  asks it to.
- **The shelf in the pole vault room is 4.5 m** and nothing asks you to reach
  it; a vault's apex against it has not been measured. The same for the gaps
  (1.5 m high, 8 m across).
- **The Elementalist's double is passed at 25 m**, not the rehearsal's 44.8:
  the code itself calls the perfect double a thirty-three-millisecond double
  tap nobody does by hand.
- **The Dual mage's bars are not seeded** on entering a room: the bars do not
  drain on their own, so the switch-hands room builds what the unlocks room
  spends, and wings take about sixteen alternating taps a hand.
- **The clash room's dummy thrusts a spear, not a sword** (2026-10-23, from
  play). A clash is his finisher live as their blow lands; if his finisher
  reaches them first they are simply hit and their swing is taken back. A
  quick sword up close left one frame in a hundred and forty where the two
  met; the spear, from three or four metres, leaves four or five -- a hold
  begun about a fifth of a second before the thrust
  (`tests/tutorial.rs`, `the_clash_room_can_be_timed_by_hand`).
- **Progress is kept for the session**, not saved to disk.
- **Docs found stale while writing the halls**: the Reaver's kit and
  exploration 0014 say Gallows blinks to the shadow and bursts it (a comment
  at `state.rs`'s `burst_the_collected` too) — since 2026-10-18 it is a drive
  up that never touches it. The Champion's kit says the Rush is "six metres
  in twenty frames, whichever way you hold"; the code is twelve frames at 32
  m/s toward the crosshair. A comment in `moves::champion` says a tap at the
  top momentum tier throws a finisher; no code does.
