---
status: built; hand-authored for play; difficulty is the author's guess, unplayed
proposed: 2026-10-03
built: 2026-10-03; rebuilt 2026-10-04 (round four, hand-authored); big jumps and the Reach 2026-10-04 (round five)
---

# Jump courses

[world.md](world.md) §2 describes **trails**: "short traversal places with no
creature, where the challenge is the movement system itself". These are the
first ones. Since round four they are **built to be played**, by a level
designer's judgement rather than by measurement: you play them and decide what
is good, bad and possible.

```
cargo run -p game -- --arena falls --p1 reaver    # desktop; N steps to the next course, Backspace restarts
cargo run -p game -- --arena reach --p1 elementalist   # the proving ground
?arena=reach&p1=elementalist                      # the browser build, the same
cargo test -p sim --test courses --test corner --test arena
```

`--arena` takes any course name below; `--p1` any of `reaver`, `elementalist`,
`blood`, `dual`, `champion`.

## 0 · What these are for

The owner's goal, as given on 2026-10-03:

- **The game is hard, and gated by skill**, Dark-Souls fashion. Movement is
  part of that challenge: fights will sit in the middle or at the end of long
  movement-gated stretches of wilderness.
- **The main route is difficult and every class can finish it**, even if some
  find it easier. Only the *barely possible* routes may belong to a class.
- **The Bulwark is out of this exercise** by the owner's choice. Five classes.

**Decided, 2026-10-03: a movement tool's reach is paid for in execution --
timing, aim, precision -- never in waiting, and never free.** (Also in
[README.md](README.md) §1.)

**Decided directions, not built:** the Dual mage's movement needs fixing
(approach undecided); the Reaver's shadow is to be made less trivial; the Blood
mage wants a way to make a pool to blink to without an enemy.

**Round four, 2026-10-04.** The owner, on round three's roofed courses: "Your
simulations are not really close to a useful representation of what a player
can or wants to do right now, but you're tweaking as if they are. Remove most of
the ceilings. That sucks. Build courses so that I can play and decide what's
good, bad and possible." And: "You build, I play." So the courses are
hand-authored; the tools of rounds two and three (§3) stay in the repository
but are **not design inputs**.

## 1 · The courses

Islands of turf on rock hang **sixty metres and more** over hills and woods
hazed by the air between (a floor drawn nearly black until 2026-10-09), with
far floating peaks and, far below, spires of rock crowned with trees for the
eye to measure the drop by. Since 2026-10-09 every box is drawn as what it is
([forms.md](forms.md), the fifth pass): a path island a **floating island**
with a lip of soil and a few roots, a stepping stone a fallen **column's
drum**, the nest a **scaffold** of planks on braced posts, the rock that
stands on the floor a banded **spire**. The collision is the boxes, as it
always was: every gap measured above is unchanged. The path is read by material: **grass** is a
path island, **sand** a stepping stone, **snow** a checkpoint (with a cairn on
it), **wood** the nest at the end, and grey **rock** is scenery, never the
route. Fall below the pit (six metres under the lowest island) and you are
stood on your last checkpoint; the clock runs from leaving the start to the
nest. `N` cycles all ten, in this order.

**Every tier is a guess. None of these has been played.**

| Course | `--arena` | Tier (guess) | What it is |
| --- | --- | --- | --- |
| The Stair | `stair` | easy | Six islands, each up to 2.5 m higher, gaps 2.5–4 m. |
| The Causeway | `causeway` | easy | Level gaps, two stepping stones, a 20 m bridge. |
| The Spiral | `spiral` | hard | Twenty ledges climbing round a 10 m pillar: a turn of short grass hops (+0.75 m each), then a turn of sand stones with longer gaps (+0.9 m), a checkpoint every four, the nest on the crown. Two tall shortcuts. |
| The Falls | `falls` | hard | A meadow, a stone stair to a 22 m arch, a required 14 m leap down off its end onto a wide landing, a zig-zag of six small stones stepping down 1.3 m each, a pool to rest on, the nest. |
| The Slalom | `slalom` | hard | Eight 2.2 m stones zig-zagging across a line of tall grey pillars, then islands under a cave mouth -- the one roof in the set, two metres over a head -- and a required 13 m leap out of it to the nest. |
| The Fork | `fork` | hard | From a hub, a high road up an 8 m wall and along the tops, or a low road of eight stones a metre down; they rejoin, then a required 15 m leap down and a runway to the nest. |
| The Waterfall | `waterfall` | hard | The valley's basalt cliff, 22 m tall, alone: three stones out of a pool, up the right of the falls, three level leaps *behind* the falling water, up the left to the lip, the nest on the top (§1c). |
| The Spire | `spire` | barely possible (Elementalist) | The summit 32 m above the launch, half a metre out. |
| The Gulf | `gulf` | barely possible | Open-air long jumps off a runway: 18 m level, 14 m and 3 m up, 22 m and 4 m down to the nest; a high line off to the right. |
| The Reach | `reach` | proving ground | Not a route: gap lanes of 10–50 m, ledges 5–45 m up, two long-and-up targets and two Grasp faces, all from one hub (§1b). |

Each start has a screenshot in the [gallery](gallery/README.md#the-jump-courses).

Every course has room for every class's tools: open air round the islands for
the Reaver's shadow and the Champion's takeoffs, tops wide enough for the
Elementalist's stones and the Blood mage's pools, and walls (the Spiral's
pillar, the Slalom's columns) to dash or vault against.

## 1a · Big jumps, for the class mechanics (round five)

The owner, 2026-10-04: "I would like jumps that are substantially beyond a
standard run and jump incorporated in some places. I want to try the class
mechanics." A plain run and jump reaches about 7–9 m level. Each big jump
below lands on a big top with a checkpoint just before it, so a miss is cheap
to retry. The course panel lists each course's big jumps. **Which mechanic each
was placed for is a guess, from round one's and two's reach figures used only
for scale. None has been tried.**

| Course | Big jump | Route? | Placed for |
| --- | --- | --- | --- |
| Spiral | **The chimney**: 17 m straight up off the pad (left of the start) to a balcony half a metre out, a hop from the crown | shortcut, skips the whole climb | Elementalist's stone launch |
| Spiral | **The shelf**: 9 m up and 6 m out from ledge 5's checkpoint, then back onto ledge 17 | shortcut, skips eleven ledges | Blood mage's Grasp, Champion's vault |
| Falls | **The long way up**: from the end of the meadow (a 12 m runway) to the arch, 19 m and 6 m up | shortcut, skips the stair | Reaver's dash jump, Champion, Elementalist |
| Falls | **The leap**: off the end of the 22 m arch, 14 m out and 5 m down onto a 10 m landing | **required** | anyone with a run: the arch is the runway |
| Falls | **The plunge**: from the big landing to the pool, 30 m and 9 m down, over the falls | shortcut, skips six stones | Reaver, Dual mage's wings, Champion |
| Slalom | **The span**: from a 20 m runway beside the start (left), 25 m level to an island, then 25 m and 3 m down to the nest | shortcut, skips the slalom and the cave | Reaver, Champion, Dual mage |
| Slalom | **Out of the cave**: off a 10 m runway out of the cave mouth, 13 m and 3 m down to the nest | **required** | anyone with a run |
| Fork | **The high road**: a wall 8 m up a metre out from the hub, two tops, then a drop of 18 m and 8 m down to where the roads meet | optional branch (the low road is the other) | Elementalist, Blood mage's Grasp; the drop for the Dual mage |
| Fork | **The committed leap**: from where the roads meet, 15 m and 4 m down onto a 10 m landing | **required** | anyone |
| Gulf | 18 m level off a 20 m runway; 14 m and 3 m up; 22 m and 4 m down to the nest | **required** | Reaver, Champion, Elementalist |
| Gulf | **The high line** (right): 18 m and 10 m up from the first island, then 19 m and 11 m down to the nest | shortcut, skips the climb | Elementalist, Champion |

The required big jumps in the hard courses are long *with a drop*: round one's
search put every class past 15 m at four metres down. The tall ones are the
shortcuts, because three of the five classes have no answer to a 9 m rise.

## 1b · The Reach: a proving ground

`--arena reach`, last in the `N` cycle (tier *proving ground*). One hub, 20 m
deep and 64 m long, and everything leaves from it. **A fall always stands you
back on the hub**, here and on no other course (`course::step`). At each
takeoff edge, a row of yellow blocks marks the distance: **one block per five
metres**. The panel lists the distances too.

- **Gap lanes**, ahead, level, left to right: **10, 15, 20, 25, 30, 40 and
  50 m**, each onto a 6 m square target. Grass for 10–20 m, sand for 25–30 m,
  blue-grey stone for 40–50 m. Neighbouring targets are a short hop apart, and
  two stones lead from the 10 m target back to the hub's corner, so you can
  walk back as well as fall back.
- **Ledges**, behind, half a metre out from the hub's back edge: **5, 10, 15,
  20, 30 and 45 m** up. Step off toward the hub to come down.
- **Long and up**, off the right end: **15 m out and 5 m up**, and **25 m out
  and 10 m up**.
- **Grasp faces**, off the left end: tall walls with a top, **6 m out and 3 m
  up**, and **8 m out and 6 m up**.

The 50 m lane is the course's finish, so the clock times a run from the hub
to it.

## 1c · The Waterfall, on the road and alone (2026-10-10)

The owner, 2026-10-10: "I want a tall waterfall jump section on the main path
of the valley and as a standalone jump map", and then: "irregular basalt large
hexagons as the terrain theme". So it is **one table** of boxes
(`sim::arena::waterfall`) put down twice: at the head of the Pinewood, where
the road ends at its pool and goes on from its top
([valley.md](valley.md) §3), and as this course, hanging in the courses' air.

```
cargo run -p game -- --arena waterfall --p1 bulwark   # the course
cargo run -p game -- --arena pinewood --open          # walk east to it in the valley
cargo test -p sim --test waterfall
```

| Beat | Hops |
| --- | --- |
| Out of the pool | Three mossy column tops: 1.5 m and +1.0, 2.1 m and +1.5, 2.2 m and +1.7, then the first shelf (a cairn) |
| Up the right | A ledge 3 m along and +2.0; a column standing out of the pool, 2.6 m and +2.0; a ledge over the shelf, 2.4 m and +2.0 |
| **Behind the falls** | Three level leaps of 3.5 m (+0.5 each) along ledges between the cliff and the water, under the lip, to the second shelf (a cairn) |
| Up the left | Three hops of 3 m, 3 m and 2.9 m at +2.1, between face ledges and a column standing out; then 2.5 m and +2.2 onto the top |

- **Every hop is inside the Bulwark's plain running jump** (apex 2.7 m; 3.9 m
  out at +2.2), because the road is for everybody and the Bulwark sets the
  valley's gaps (valley.md §6). For the Bulwark it is close to the edge; for
  the Dual mage it is easy. `tests/waterfall.rs` reads every hop off the
  table and checks each class against `sim::envelope`, and then **jumps
  every hop with every class**, in the valley and in the course, by a
  stand-in player that tries a few run-ups.
- **It is hard by its height and its water, not its gaps.** The cliff is 22 m:
  a slip from the left side costs what the fall rule says, and in the valley
  that is a death and the last cairn. **The falling water pushes down** at
  9 m/s (`Waterfall, pushes down at` in the Oven): nobody jumps up through the
  curtain, and a leap behind it that drifts out into it drops. A guess.
- **It cannot be walked round** (`the_cliff_cannot_be_walked_round`): the land
  either side of the cliff is mountain.
- **Drawn as basalt** (`game::forms`, `Form::Basalt`): every box a cluster of
  irregular hexagonal columns, about two metres across, clipped to the box so
  the edge you see is the edge you stand on. The water is `game::falls`: the
  river over the top, a curtain bowed out from the lip, white streaks running
  down it on the simulation's clock, and foam at the foot.
- **Tier hard is a guess**, like every other here. Unplayed.

## 2 · The corner clip, fixed

A jump clipping a ledge's near top corner on the way up was lifted onto the top
by the resolve, **counted as standing**, and a still-held jump fired a second
takeoff stacked on the first's speed -- about 27–30 m/s up. Fixed in
`arena::resolve_among`: a body lifted onto a top while still rising is put on
the top and keeps rising; it is grounded only once it is not rising. Pinned by
`tests/corner.rs`. One other test moved with it and is not about it:
`tests/pair.rs`'s `the_two_never_land_within_the_gap_except_the_twin_pounce`
fails on its seeds after the fix, and already fails on the base commit over
seeds 1–24 -- a latent fault in the Pair's coordination, surfaced, not caused.
Left failing and reported.

## 3 · Tools (not design inputs)

Rounds two and three measured movement by search; round four does not use them
to design anything. They stay because they are cheap to keep and may answer a
narrow question later:

- `cargo run --release -p sim --bin envelope` -- a hill-climb search
  (`sim::search`) for each class's widest gap at each rise, in the lab arena
  (`--arena lab`). Lower bounds; its claimed lines are replayed by
  `tests/search.rs` from `tests/fixtures/envelope.txt`. Its best lines are one
  or two frames wide, which is exactly why it is a poor stand-in for a player.
- `cargo run --release -p sim --bin courses` (and `-- --bench`) -- every hop of
  every course per class, with its **timing window** (`sim::coursecheck`), and
  a bench arena for a kind of hop. The courses' per-class route fixtures were
  removed in round four.
- `sim::envelope` -- the plain envelope (run, jump, one airdodge), the only
  thing round four looked at, as a sanity check against absurd gaps.

## 4 · Props a shared world could give every class

**Vines and updraft vents are built** (2026-10-08), for the valley rather than
the courses: a vine is a face anybody climbs at 2.5 m/s holding jump, and a
vent lifts a body in the air at 8 m/s to its top. Both are tables in a
valley place and knobs in the Oven; see [valley.md](valley.md) §3. A course
could take them by the same tables; none does yet. Still not built:
**drifting rocks** on fixed paths (a timing gate the same for every class;
the sim has no moving solids yet).

## 5 · Not done, and next

- **Play them.** Every tier above is unplayed.
- The Dual mage, the Reaver's shadow and the Blood mage's pools (§0).
- **The Pair's stagger gap** (`tests/pair.rs`, §2).
