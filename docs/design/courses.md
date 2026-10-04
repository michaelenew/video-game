---
status: built; hand-authored for play; difficulty is the author's guess, unplayed
proposed: 2026-10-03
built: 2026-10-03; rebuilt 2026-10-04 (round four, hand-authored)
---

# Jump courses

[world.md](world.md) §2 describes **trails**: "short traversal places with no
creature, where the challenge is the movement system itself". These are the
first ones. Since round four they are **built to be played**, by a level
designer's judgement rather than by measurement: you play them and decide what
is good, bad and possible.

```
cargo run -p game -- --arena falls --p1 reaver    # desktop; N steps to the next course, Backspace restarts
?arena=falls&p1=reaver                            # the browser build, the same
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

Islands of rock hang **sixty metres and more** over a floor drawn nearly black,
with far floating peaks and, far below, spires of rock crowned with trees for
the eye to measure the drop by. The path is read by material: **grass** is a
path island, **sand** a stepping stone, **snow** a checkpoint (with a cairn on
it), **wood** the nest at the end, and grey **rock** is scenery, never the
route. Fall below the pit (six metres under the lowest island) and you are
stood on your last checkpoint; the clock runs from leaving the start to the
nest. `N` cycles all eight, in this order.

**Every tier is a guess. None of these has been played.**

| Course | `--arena` | Tier (guess) | What it is |
| --- | --- | --- | --- |
| The Stair | `stair` | easy | Six islands, each up to 2.5 m higher, gaps 2.5–4 m. |
| The Causeway | `causeway` | easy | Level gaps, two stepping stones, a 20 m bridge. |
| The Spiral | `spiral` | hard | Twenty ledges climbing round a 10 m pillar: a turn of short grass hops (+0.75 m each), then a turn of sand stones with longer gaps (+0.9 m), a checkpoint every four, the nest on the crown. |
| The Falls | `falls` | hard | Up a stone stair and a 22 m arch, then a committed 6 m leap down onto a wide landing, a zig-zag of six small stones stepping down 1.3 m each, a pool to rest on, the nest. |
| The Slalom | `slalom` | hard | Eight 2.2 m stones zig-zagging across a line of tall grey pillars, then three islands under a cave mouth -- the one roof in the set, two metres over a head. |
| The Fork | `fork` | hard | From a hub, a high road of stacked ledges (+1.5 m each) and tops, or a low road of seven stones stepping down; they rejoin, then a 7 m leap down and a runway to the nest. |
| The Spire | `spire` | barely possible (Elementalist) | The summit 32 m above the launch, half a metre out. |
| The Gulf | `gulf` | barely possible | Open-air long jumps: 8 m level onto a 3 m island, 7 m up 1.5 m onto 2 m, two 6 m hops onto 1.2 m stones, 9 m down to the nest. |

Each start has a screenshot in the [gallery](gallery/README.md#the-jump-courses).

Every course has room for every class's tools: open air round the islands for
the Reaver's shadow and the Champion's takeoffs, tops wide enough for the
Elementalist's stones and the Blood mage's pools, and walls (the Spiral's
pillar, the Slalom's columns) to dash or vault against.

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

Not built: **hanging vines** climbable by anyone (the vertical answer the Blood
mage and Champion lack); **updraft vents** (the Elementalist's Updraft for
everyone); **drifting rocks** on fixed paths (a timing gate the same for every
class; the sim has no moving solids yet).

## 5 · Not done, and next

- **Play them.** Every tier above is unplayed.
- The Dual mage, the Reaver's shadow and the Blood mage's pools (§0).
- **The Pair's stagger gap** (`tests/pair.rs`, §2).
