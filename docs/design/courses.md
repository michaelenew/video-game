---
status: built; the courses and the instrument are measuring tools, nothing here is decided
proposed: 2026-10-03
built: 2026-10-03
---

# Jump courses — the movement envelope, and courses built against it

[world.md](world.md) §2 describes **trails**: "short traversal places with no
creature, where the challenge is the movement system itself". These are the
first ones. They are **measuring instruments as much as content**: what each
class can actually do with its feet, played in the simulation, and six courses
built against those numbers. Every hop in them is measured for every class.

```
cargo run -p game -- --arena climb --p1 reaver     # desktop; N steps to the next course
?arena=climb&p1=reaver                              # the browser build, the same
cargo run --release -p sim --bin envelope          # §1: what each class can do
cargo run --release -p sim --bin courses           # §4-5: every hop, every class, the margins
cargo test -p sim --test courses                   # the routes this document promises, played
```

## 0 · What these are for

The owner's goal, as given on 2026-10-03:

- **The game is hard, and gated by skill**, Dark-Souls fashion. Movement is
  part of that challenge, not only the fights: fights will sit in the middle
  or at the end of long movement-gated stretches of wilderness.
- So **the main route is difficult and every class can finish it**, even if
  some find it easier. No class's tools should make a hard section trivial, and
  no class should find one impossible. Only the *barely possible* routes may
  end up belonging to a class.
- **The Bulwark is out of this exercise** by the owner's choice: it may not fit
  an athletic, agile hunter game. It is not measured here and nothing is built
  for it. Five classes: the Reaver, the Elementalist, the Blood mage, the Dual
  mage and the Champion.

This document is diagnosis: which tools trivialise which hops, and which
classes cannot do what the others find merely hard (§6). What to do about either
is the owner's call.

## 1 · The envelope

`cargo run --release -p sim --bin envelope`, played in **the lab**
(`--arena lab`, `sim::arena::lab`): an open floor, a 20 m runway with a real
edge, and eight ledges 2 to 20 m tall. Everything is scripted input into a
`World` (`sim::envelope`); nothing is read off the prose.

### The jump

| Class | Run | Short hop | Full hop | Long jump, level | + airdodge |
| --- | --- | --- | --- | --- | --- |
| Shadow Reaver | 7.0 m/s | 1.2 m | 5.1 m, 1.0 s | 7.4 m | 8.7 m |
| Elementalist | 7.0 | 1.2 | 5.0, 1.0 s | 7.6 | 8.9 |
| Blood mage | 7.0 | 1.0 | 4.0, 0.9 s | 6.3 | 7.7 |
| Dual mage | 7.0 | 1.4 | 5.9, 1.2 s | 8.7 | 9.5 |
| Champion | 7.0 | 0.9 | 3.9, 0.8 s | 6.2 | 7.6 |

Every class runs at the same 7 m/s and reaches it at once, so a standing and a
running long jump are the same jump.

### Gap and rise

The widest gap a running jump lands across, edge to edge, onto a ledge this far
above (+) or below (−) the takeoff: **plain / with the airdodge at its best
frame**. Read from a real trajectory off the runway's edge, replayed against a
ledge at every gap and rise through `Arena::resolve` — the simulation's own rule
for whether a body lands on a top or meets a face. Space is pressed on the last
frame before the edge.

| Class | −8 m | −4 m | −2 m | level | +1 m | +2 m | +3 m | +4 m | +5 m |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Shadow Reaver | 11.0/11.0 | 9.4/10.0 | 8.5/9.6 | 7.7/9.1 | 7.4/8.7 | 6.9/8.2 | 6.4/7.6 | 5.8/6.8 | 4.8/5.2 |
| Elementalist | 11.6/11.6 | 9.8/10.3 | 8.8/9.9 | 8.0/9.3 | 7.6/8.9 | 7.1/8.4 | 6.6/7.8 | 5.9/6.9 | 4.8/5.2 |
| Blood mage | 9.8/10.2 | 8.2/9.5 | 7.5/8.9 | 6.8/8.1 | 6.3/7.6 | 5.8/7.0 | 5.3/6.2 | 4.1/4.5 | none |
| Dual mage | 12.6/12.6 | 10.8/10.8 | 10.0/10.3 | 8.9/9.9 | 8.5/9.6 | 8.0/9.4 | 7.6/8.9 | 7.1/8.3 | 6.4/7.4 |
| Champion | 9.7/10.3 | 8.1/9.3 | 7.4/8.7 | 6.7/8.0 | 6.2/7.5 | 5.7/6.9 | 5.1/6.1 | 3.9/4.3 | none |
| Dual mage, second jump | 30.3 | 26.1 | 24.1 | 22.0 | 21.0 | 20.0 | 18.9 | 17.9 | 16.9 |

The second-jump row has her bars **set** at 80 (the second tier). Earning it
with nothing to hit is in the next table.

### Each class's own tools

| Class | Tool | Measured |
| --- | --- | --- |
| Reaver | Send shadow | 9.0 m along the floor when sent at nothing |
| | Dash to it, then the dash jump | the dash jump carries **10.6 m** past the shadow, 5.1 m up |
| | Shadow onto a ledge, dash | onto **every** lab ledge, 2 to 20 m tall, from 9 m back (next table) |
| Elementalist | One-stone jump | **25.9 m** straight up |
| | Double stone jump | **44.7 m** straight up |
| | Updraft, and a jump in it | 5.0 m: nothing past her own full hop (`elemental lift`: 2.6 m alone) |
| | A stone | 1.8 m tall, raised up to 6 m away |
| Dual mage | Earning tiers with nothing to hit (autos alternated on the spot) | blink at **11.6 s**, second jump at **17.2 s**, wings at **22.0 s** |
| | Blink | 3.2 m standing, 2.1 m airborne, and it stops her horizontal speed |
| | Ascending (wings), a beat every 12 / 20 / 30 frames | **65 / 51 / 33 m up**, 60–64 m along, before the feet are down |
| Champion | One Rush | 6.3 m |
| | Pole vault | 7.2 m up, 14.2 m along |
| Blood mage | Pool blink | **not measured**: a pool comes from cutting somebody, and a course has nobody to cut |
| | Dodge-as-blink | the flag is off (9 m if it were on) |
| Everyone | Airdodge | in the gap table: +0.4 to +1.4 m on a level gap |

Up onto a ledge with the tool: **the furthest back she can stand, still, from a
ledge's face and end on its top** (half-metre steps to 30 m; `30+` reached the
end of the scan; `–` from nowhere):

| Tool | 2 m | 4 m | 6 m | 8 m | 10 m | 12 m | 16 m | 20 m |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Reaver, shadow + dash | 9.0 | 9.0 | 9.0 | 9.0 | 9.0 | 9.0 | 9.0 | 9.0 |
| Elementalist, stone jump | 3.5 | 3.0 | 2.5 | 2.5 | 2.5 | 2.0 | 1.5 | 1.0 |
| Elementalist, double stone jump | 17.0 | 4.0 | 4.0 | 3.5 | 3.5 | 3.5 | 3.5 | 3.0 |
| Elementalist, Updraft | 2.0 | 0.5 | – | – | – | – | – | – |
| Elementalist, stone stair | 30+ | 30+ | 12.5 | – | – | – | – | – |
| Dual mage, second jump (bars set) | 30+ | 30+ | 15.0 | 13.0 | – | – | – | – |
| Champion, Rush + pole vault | 17.5 | 16.5 | 15.0 | – | – | – | – | – |
| Champion, takeoff (space + a weapon) | 30+ | 17.5 | 16.0 | 14.0 | – | – | – | – |
| Blood mage, Grasp haul | 8.5 | 4.0 | – | – | – | – | – | – |

Two readings worth having. **The stone jumps are nearly vertical**: her air
steering is the weakest of the five, so out of a 44 m rise she drifts about
3 m, and the stone has to go down where she wants to come up. **The Grasp hauls
to a face, not a top**: the arms leave her chest, so aimed at a top above her
chest they meet the face first and pull her to the foot of it; only thrown
from a jump does it put her on a 2 or 4 m top.

## 2 · The courses

**The look** is the Hallelujah Mountains' climb to the banshee rookery:
islands of rock hanging in open air at staggered heights over a long drop into
cloud, stepping-stone chains of small islands, a long arch, an overhang, and
the finish on the last island, a nest with eggs in it — the rookery the
Galewing would live in. Every island but the first hangs (its bottom is above
the floor); the first is **the foot**, a spire standing on the floor, because a
round starts on the ground under a mark. Grass-topped rock, stone stepping
stones, a wooden nest; a sheet of mist a metre over the dark floor; far
floating peaks in the haze; a cairn on each checkpoint.

![](gallery/course-climb.jpg)

| Course | `--arena` | Tier | What it is |
| --- | --- | --- | --- |
| The Stair | `stair` | easy | Six islands, each up to 2.5 m higher, gaps 2.5–4 m |
| The Causeway | `causeway` | easy | Read horizontally, drifting down: level gaps, two stepping stones, a 20 m bridge |
| The Climb | `climb` | hard | The ascent to the rookery: every hop near the airdodged edge of the shortest jumpers; stepping stones; a gap under an overhang; the arch |
| The Drift | `drift` | hard | The crossing: long gaps at about one height, drifting down, every one past a plain jump for the shortest jumpers; three stones 1.2 m across |
| The Spire | `spire` | barely possible | Built against the Elementalist: the rookery's spire 32 m above the launch, its face half a metre out |
| The Gulf | `gulf` | barely possible | Built against the Reaver: a shadow on a stone a metre across, the dash jump off it across 9 m, and a last island 5 m up and 8.2 m out |

**How they were built.** Each course is a list of hops — a direction, the gap
from the last island's edge, the rise from its top, the size of the next — and
the arena table (`sim::arena::climb`) and its dressing
(`game::arenas::climb`) were generated from those lists, so the numbers in a
route (`Step::gap`, `Step::rise`) are what stands in the world. The hard courses
put every hop **just inside the airdodged jump of the shortest jumpers** (the
Champion and the Blood mage, §1), so that they finish and nobody has much to
spare.

**Falling.** The **pit** is a height below every island (`Course::pit`). Below
it, the course stands you on the middle of **the last checkpoint you reached**,
fresh — full health, your stones, shadow and shield taken back, as a round
starts — and the clock keeps running. The start, the finish, and the islands the
route marks are checkpoints; a cairn stands on each.

**What the sim keeps** (`sim::course`): one `Run` per fighter, twelve bytes —
the furthest gate reached, the falls, when you left the start and how long you
took. In the snapshot (a fall moves a body, so both peers must do it on the same
frame) and hashed only in a course, so every other fight hashes as before.
`World` is 3,984 bytes against the 4,096 cap.

**Playing one.** `--arena <name>` (`?arena=<name>` in the browser), with
`--p1 <class>` (`?p1=`). **`N`** steps to the next course in order of
difficulty, and from anywhere else to the first; it travels on the wire like
`Shift+H` (`Travel::arena`), so online both players go. `Backspace` restarts.
The panel on the right lists the courses and shows the current one's tier, the
last checkpoint, the clock and the falls; the clock stops at the nest.

## 3 · The hops, as built

| Course | Hop | Gap | Rise | Ask |
| --- | --- | --- | --- | --- |
| Stair | 1–6 | 2.5–4.0 m | 0 to +2.5 m | plain jumps |
| Causeway | 1, 2 | 4.5, 5.0 | 0, −1 | |
| | 3, 4 | 4.0, 4.0 | 0 | stepping stones 2 m across |
| | 5, 6, 7 | 5.5, 3.0, 5.0 | −2, 0, 0 | the bridge, the far island |
| Climb | 1 | 6.0 | +2 | Champion and Blood mage: past the plain jump (5.7, 5.8), inside the airdodge (6.9, 7.0) |
| | 2, 3, 4 | 4.5 each | +1 each | stepping stones 1.5 m across |
| | 5 | 3.6 | +4 | Champion 3.9/4.3, Blood mage 4.1/4.5: the tallest rise either can take |
| | 6 | 7.0 | 0 | past both shortest jumpers' plain 6.7/6.8; inside the airdodge 8.0/8.1 |
| | 7 | 2.5 | 0 | **under an overhang**: a slab 3 m over the takeoff, from 1.5 m before the edge to 1.5 m past the far one |
| | 8 | 5.0 | −2 | onto the arch, 24 m long |
| | 9 | 5.0 | +3 | the rookery: Champion 5.1/6.1 |
| Drift | 1, 3 | 7.2, 7.4 | 0 | past the plain jump of the Champion and Blood mage |
| | 2 | 7.8 | −2 | |
| | 4, 5, 6 | 5.5 each | 0 | stepping stones 1.2 m across |
| | 7 | 8.5 | −4 | |
| | 8, 9 | 6.0, 7.0 | +2, −1 | |
| Spire | 1, 2 | 5.0 | +1 | to the launch |
| | 3 | 0.5 | **+32** | the spire: only the double stone jump (and wings) |
| Gulf | 1 | 6.0 | 0 | |
| | 2 | 7.5 | −0.5 | onto a stone a metre across |
| | 3 | 9.0 | +1.5 | off the stone, no run-up: the dash jump |
| | 4, 5 | 7.5, 8.0 | −3, +1 | onto a stone and off it |
| | 6 | 8.2 | **+5** | higher than any jump but the Dual mage's, nearer than the shadow's 9 m |

## 4 · The instrument

`cargo run --release -p sim --bin courses` (`sim::coursecheck`) plays **every
hop for every class**: she stands on one island, and a pilot takes her to the
next with one technique; a hop counts when she is standing on the next top. For
each hop and class it reports two **margins**, because a hop can be lost two
ways:

- **Timing — `J` and `D`.** `J` is how many of sixteen takeoff frames land a
  plain running jump (space pressed 0 to 15 frames before the last frame on the
  island). `D` is how many frames the airdodge can be thrown on and land, at the
  best takeoff. `J16` does not care when you jump; `J1` is frame-perfect; `J0`
  means the plain jump does not do it.
- **Distance — `a/b`.** How much wider the gap could be, at the same rise, and
  still be landed: plain / with the airdodge (the §1 trajectories against the
  hop's rise). Negative is short.

Then the class's own tools on the same hop, with how many of their timings
work (`shadowx4`, `2 stonesx12`). Then **the whole course**, each hop with the
first technique that clears it, in the order a player reaches for them: the
jump, the airdodge, the class's tool — with one step of looking back, so the
Reaver can take two hops at once with the dash jump off a stone.

The pilot steers the Quake way — the stick at the next island's middle, which
carves the velocity round to it — and lets go once over it. Its techniques:
the running jump (16 timings), the airdodge (59 timings × 4 takeoffs), and per
class: the shadow and dash, from the floor or thrown from a hop, and the dash
jump; one and two stones under her feet (25 timings each); the second jump after
**boxing up to the tier on the spot** (the time counts) and ascending (wings,
after boxing to the top); the three takeoffs. **Not in it**: the Champion's pole
vault and Rush, the Blood mage's Grasp (it hauls to faces, §1) and pool blink,
short hops, and air strafing other than homing. A `J0` on a stepping stone for
the strong jumpers is the pilot overshooting a 1.2–2 m top with a full hop — a
player would short-hop it; the airdodge (which wipes the rise) lands it.

## 5 · The matrix

Measured, every cell: `cargo run --release -p sim --bin courses`. The
whole-course routes for the two hard courses and the two barely possible ones
are pinned in `crates/sim/tests/courses.rs`.

### Who finishes, and how

| Course | Reaver | Elementalist | Blood mage | Dual mage | Champion |
| --- | --- | --- | --- | --- | --- |
| Stair (easy) | jumps, 8.4 s | jumps, 8.6 s | jumps, 7.6 s | jumps, 9.5 s | jumps, 7.5 s |
| Causeway (easy) | 2 airdodges, 10.9 s | 2 airdodges, 11.0 s | 1 airdodge, 10.8 s | 2 airdodges, 11.3 s | 1 airdodge, 10.8 s |
| **Climb (hard)** | 1 airdodge, 14.1 s | 1 airdodge, 14.3 s | **3 airdodges**, 13.7 s | 3 airdodges, 14.0 s | **2 airdodges + a takeoff**, 13.7 s |
| **Drift (hard)** | 1 airdodge, 13.5 s | 1 airdodge, 13.7 s | **4 airdodges**, 13.0 s | 3 airdodges, 13.9 s | **5 airdodges**, 13.1 s |
| Spire (edge) | no — stuck at the spire | **double stone jump**, 6.2 s | no | **wings**, 33.6 s (22 s of it boxing) | no |
| Gulf (edge) | **dash jump + shadow**, 8.0 s | no — the last island | no — off the stone | no — the last island | no — off the stone |

The airdodges on the easy courses and on the Dual mage's hard runs are all on
stepping stones: the pilot's full hop overshoots a small top (§4).

### The hard hops, class by class

The timing window of the plain jump and of the airdodge, and how much wider the
gap could be with the airdodge (`J/D/+b`). Bold: the plain jump does not do it.

| Hop | Reaver | Elementalist | Blood mage | Dual mage | Champion |
| --- | --- | --- | --- | --- | --- |
| Climb 1: 6 m, +2 | J10 D34 +2.2 | J11 D35 +2.4 | J1 D32 +1.0 | J16 D36 +3.4 | **J0** D17 +0.9 |
| Climb 2–4: stones, 4.5 m, +1 | J4–0 D16 | J1–0 D17 | J12 D15 +3.1 | J0 D11 | J12 D15 +3.0 |
| Climb 5: 3.6 m, +4 | J16 D42 +3.2 | J16 D41 +3.3 | J2 D35 +0.9 | J16 D43 +4.7 | **J0 D0**: takeoff only |
| Climb 6: 7 m level | J8 D29 +2.1 | J10 D30 +2.3 | **J0** D20 +1.1 | J16 D32 +2.9 | **J0** D19 +1.0 |
| Climb 7: under the overhang | J3 D58 | J4 D58 | J2 D57 | J4 D58 | J2 D57 |
| Climb 9: 5 m, +3 | J15 D39 +2.6 | J16 D39 +2.8 | J4 D37 +1.2 | J16 D40 +3.9 | J3 D36 +1.1 |
| Drift 1: 7.2 m level | J7 D28 +1.9 | J9 D29 +2.1 | **J0** D19 +0.9 | J16 D31 +2.7 | **J0** D18 +0.8 |
| Drift 2: 7.8 m, −2 | J8 D26 +1.8 | J11 D27 +2.1 | **J0** D22 +1.1 | J16 D29 +2.5 | **J0** D22 +0.9 |
| Drift 3: 7.4 m level | J5 D26 +1.7 | J7 D26 +1.9 | **J0** D16 +0.7 | J16 D29 +2.5 | **J0** D15 +0.6 |
| Drift 4–6: stones, 5.5 m | J5–0 D9–10 | J3–0 D9–10 | J10–14 D24–25 | J0 D8–9 | J10–15 D25 |
| Drift 7: 8.5 m, −4 | J9 D24 +1.5 | J16 D25 +1.8 | **J0** D20 +1.0 | J16 D28 +2.3 | **J0** D19 +0.8 |
| Drift 8: 6 m, +2 | J10 D34 +2.2 | J11 D34 +2.4 | J1 D31 +1.0 | J16 D36 +3.4 | **J0** D17 +0.9 |
| Drift 9: 7 m, −1 | J11 D31 +2.4 | J14 D32 +2.7 | J3 D29 +1.5 | J16 D33 +3.1 | J2 D28 +1.4 |

**Tools that also clear the hard hops** (every one of them, unless listed):
the Reaver's shadow and dash (all but Climb 7, under the overhang); the
Elementalist's one or two stones (all but Climb 5, 7 and 9); the Dual mage's
wings after 22 s of boxing (all but four of the stones), and her second jump after 17 s
on four; the Champion's takeoffs (all but the stones).

## 6 · Observations

Facts and measurements, for the owner to decide on. Nothing here has been
played by a person.

**The Dual mage's plain jump trivialises the hard courses.** Every hard hop but
the stepping stones and the overhang she clears with a plain jump from any of
the sixteen takeoff frames (`J16`), with 2.3 to 4.7 m to spare after the
airdodge; the other four have 0.6 to 3.3 m, and the Champion and Blood mage
0.6 to 1.5 m off the stones. Her jump is 5.9 m against 3.9–5.1, her level long
jump 8.7 m against 6.2–7.6. This is before her tiers.

**The tools that make a hard section trivial**, measured, candidates to weaken:

- **The Reaver's shadow and dash: 9 m, at any height, with no timing.** Onto a
  20 m ledge from 9 m back. It clears every hard hop but the one under the
  overhang, and the dash is not resolved against the world on the way. Its
  limits are line of sight and that the top must be under the crosshair or
  under the 9 m reach sphere, which a hanging island far above her is not.
- **The Elementalist's double stone jump: 44.7 m up** (single 25.9), against a
  5.0 m full hop. It clears every hard hop but three (Climb 5, 7 and 9), and a 9 m gap with 1.5 m
  rise off a stone a metre across with no run-up (the Gulf's crossing). Its
  limit is drift: about 3 m sideways out of a 44 m rise.
- **The Reaver's dash jump**: 10.6 m along and 5.1 m up, out of a dash that has
  already crossed 9 m — the Gulf's 9 m crossing off a stone.
- **The Dual mage's tiers, earned on the spot with nothing to hit.** Boxing
  the air for 17 s buys a second jump worth 22 m level; 22 s buys wings worth
  65 m up and 60 m along, enough for the Spire. The tiers decay (the calm, 2 a
  second), so each hop is boxed for again. This is slow, not hard: the cost is
  time, not skill.
- **The Champion's takeoffs**: up 8 m from 14 m back; it is the only way he
  clears Climb 5 (3.6 m, +4), and it clears the Gulf's last island (8.2 m, +5)
  that the Elementalist and the Dual mage cannot.

**The classes that cannot do what the others find merely hard**, candidates for
more mobility:

- **The Blood mage has no movement tool that works here.** The Grasp hauls her
  to a face rather than onto it (onto a 2 m top from 8.5 m only when thrown
  from a jump; a 4 m top from 4 m; nothing taller); the pool blink needs a
  pool, which needs a victim; dodge-as-blink is a flag, off. She finishes the
  hard courses on the airdodge alone, with **no plain jump on five hard hops**
  and 0.7 to 1.1 m to spare on them. At +5 m nothing she has lands at any gap.
- **The Champion is the shortest jumper** (3.9 m hop, 6.2 m long jump). Without
  the takeoffs he could not do Climb 5 (`J0 D0`); with them he is fine. He needs
  the airdodge on seven of the eighteen hard hops, at 0.6 to 1.0 m to spare.
- **The Updraft adds nothing to a jump**: 5.0 m, her own full hop.

**Equally skill-gated, where it is.** The overhang (Climb 7) is the one hop
where every class has nearly the same narrow window (`J2`–`J4`): a ceiling
caps every jump at the same height. Stepping stones flatten the field the other
way — the short jumpers land them from more frames than the long ones do.

**The barely possible courses are not quite locked.** The Spire is the
Elementalist's, and the Dual mage's after 22 s of boxing. The Gulf is the
Reaver's alone, but each of its three hard hops is cleared by somebody else
(the stone crossing by the Elementalist, the 8 m off the stone by the Dual mage,
the last island by the Champion's takeoff).

## 7 · Props a shared world could give every class

Things the Hallelujah Mountains scene has that the simulation does not, not
built, with what each would do to the matrix:

- **Hanging vines and roots, climbable or swingable by anyone.** The vertical
  answer the Blood mage and the Champion lack; it would flatten the +4/+5 m
  rows, where they have nothing, and make a climb a route rather than a class
  test. It would also make the Elementalist's stone jumps less special wherever
  a vine hangs.
- **Natural updraft vents.** A column anyone can ride: the same as giving every
  class the Elementalist's Updraft, which today adds nothing to her own jump
  (§1) — so a vent strong enough to matter would outclass her Updraft and
  compete with her stone jumps. It would help the short jumpers most.
- **Drifting rocks.** Islands that move on a fixed path: a timing challenge the
  same for every class, which is the kind of gate §0 wants. They would cut
  against the Reaver's shadow (it waits where it was sent, on a rock that has
  moved on) and the Elementalist's stones (raised on a moving top), and the
  sim has no moving solids yet — a moving island is a new kind of thing.

## 8 · Not done, and next

- **Margins against geometry, not just the bench**: the distance margin is the
  §1 trajectory against an infinitely deep ledge; it does not see a small top
  or a ceiling. The timing windows do (they are played on the real course).
- **Short hops and real air strafing in the pilot**: the stepping-stone `J0`s
  for the long jumpers are the pilot's, not the class's.
- **The Champion's pole vault and Rush, and the Blood mage's Grasp**, in the
  pilot: measured in the lab (§1), not on the courses.
- **Pools for the Blood mage**: a course with something to cut would let her
  blink; nothing here measures it.
- **Per-checkpoint times and a best-time record**; a finish that does more than
  stop the clock.
- **The shadow's 9 m at any height**: whether a shadow should be able to stand
  on a ledge it could not see the top of is an aiming question for
  [aiming.md](aiming.md), not a course one.
