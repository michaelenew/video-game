---
status: M1–M3 built 2026-09-23; C1–C4 open — waiting on a person
opened: 2026-09-23
implements: ../dual-mage-v2.md
---

> **Progress, 2026-09-23.** The instrument (`goad`) and milestones M1, M2 and M3 are built,
> with every measured criterion a test in `crates/sim/tests/dual_mage.rs` or
> `crates/view/tests/wings.rs`, except two that the numbers could not both satisfy — the
> time-to-tier criteria in M1 and M2 are met at two and a quarter exchanges for the idle fall
> and four for the climb rather than one and one, and the feel log for that date says why.
> C1 was played the same day and came back overtuned; the first pass of M4 restated the class
> as eight benchmarks in `../dual-mage.md` and re-tuned to them, with each benchmark a test.
> C1 to C3 are open again against the new numbers, with their play scripts written; the rest
> of M4 and M5 wait on them. Two deviations from the brief, both argued in [`../dual-mage.md`](../dual-mage.md):
> "both full" is the lower bar at 95, and the blink has no invulnerability knob of its own.

# Dual mage v2 — action plan

This is a complete brief for one implementer. Read [`../dual-mage-v2.md`](../dual-mage-v2.md)
first: it is the specification, and this document is how to get there and how to know when
you have. Then read [`../dual-mage.md`](../dual-mage.md) for the mechanic you are replacing
and the reasoning that survives it, and [`../kits/dual-mage.md`](../kits/dual-mage.md) for
the moves, which do not change. Read [`../README.md`](../README.md) and
[`../../../CLAUDE.md`](../../../CLAUDE.md) before touching code.

## The objective, in one paragraph

Replace the single signed bar with **two bars**, Dark and Light, and a **runaway** between
them: inside a band nothing moves, outside it the higher bar rises and the lower falls and
she burns, faster the wider the gap. The **lower bar** gates what her body can do — a blink
at half, a second jump and a slow fall at three quarters, wings and unlimited jumps when both
are full — so goading both beings while holding them level is the class. The class is done
when a person playing it says they can *feel the hill*: that a one-sided cast starts
something they have to catch with the other hand, and that holding both high is hard, worth
it, and visible on her back.

## Ground rules

The repository's rules, restated because breaking them costs the most time.

- **`crates/sim` has no floats, no allocation, no I/O, no dependencies.** Fixed point is
  `Fx`. `World` stays under 4 KiB. `no_floats.rs`, `determinism.rs` and `budget.rs` enforce
  it. The struggle runs every frame for one player; it is a dozen integer operations and must
  stay that.
- **Every feel number is an Oven knob**, added in `oven.rs`, exposed in `tuning.rs`, and
  baked with `cargo run -p sim --bin bake_tuning`. **The Oven's move store is packed in class
  order with the Dual mage last**, and the Blood mage thread is appending slots and knobs
  ahead of yours at the same time. Expect every merge from `main` to shift your indices in
  `tuned.rs`; re-bake after each one and let `oven.rs` in the tests tell you when you forgot.
  Never renumber existing slots.
- **Aiming lives in `aim.rs` and nowhere else.** The blink asks `aim::clear_between` whether
  the line to where the dodge would end is open. No other aiming changes. If you find
  yourself writing an eye position or a look direction outside `aim.rs` and `math.rs`, stop.
- **Range and frame data never scale with a bar.** This is the one rule the old design got
  exactly right and it holds harder here. Power and size scale with the carried bar, the
  autos' size is exempt, and the thing that scales is drawn on her back.
- **The overlay draws what the hit test uses**, and by extension the wings are drawn at the
  bars' lengths — with a test in `view` that says so.
- **Design docs are the specification.** As each milestone lands, update `dual-mage.md` and
  `kits/dual-mage.md` so that they describe what is in the game, and write a feel-log entry.
  When the code and the proposal disagree and the code is right, change the proposal and say
  why.
- **Do not touch the Blood mage, or any other class**, beyond what a shared function forces.
  Keep the diff class-scoped so the merges are boring.
- **Do not add a button, a move, or a third bar.** The kit's six moves are unchanged. If a
  beat cannot be hit with two bars, the tiers and the existing moves, report it rather than
  growing the kit.
- Before every push: `cargo fmt --all`, `cargo clippy --workspace --all-targets`,
  `cargo test --workspace`. Nothing runs them for you.

## How you will know — the two kinds of evidence

You cannot feel the game. You can measure it, script it, step it, screenshot it, and reason
about it; a person can play it. Both are required and you must be honest about which you
have.

**Measured** evidence is tests, the frame table, and the instrument below. It proves that a
relationship holds. It does not prove that the hill is felt.

**Felt** evidence comes from the human checkpoints. At each one you stop, push, write a
feel-log entry whose verdict is *open*, and hand the person a play script: what to do, what
to look for, and the one or two questions only they can answer. Then iterate on what they
report — one knob at a time, baked, with the entry updated.

**Honesty rules.** A passing test is never reported as "it feels right". A milestone whose
measured criteria pass and whose felt criteria are unanswered is *built, unverified*, and is
described that way. If a criterion cannot be met without loosening a relationship in
`feel.rs`, that is a design finding: stop, write it down, ask. Record what you tried and
reverted.

## Build the instrument first

Before any mechanic change, add `cargo run -p sim --bin goad` beside `frametable`. It runs
named input scripts against an empty arena and against the training dummy, and prints both
bars, the gap, the burn taken and the tier held, frame by frame or summarised. The scripts
it must carry:

| Script | What it is |
| --- | --- |
| `alternate` | dark auto, dark Lance, light auto, light Sweep, repeated |
| `one-sided` | dark auto, then dark casts only |
| `finisher` | balanced to the band's edge, then one Judgement |
| `idle` | goad both to the second tier, then nothing |
| `ascend` | the fastest sequence you can find that reaches both bars full |

For each it reports: time to each tier, time to leave the band, time for the lower bar to hit
zero once outside it, total burn, and whether ascension was reached. Every criterion below is
a number this bin prints. It is how you self-evaluate in seconds, and the first thing a
reviewer will run.

## Milestones

In order. Each is playable, lands with tests, a re-bake, a doc update and a feel-log entry,
and the next does not start until the current one's measured criteria pass.

### M1 — Two bars and the hill, no tiers

**Build.**
- `Mechanic::Meter { value, colour, ascending }` becomes `{ dark, light, colour, ascending }`.
  Each bar runs from zero to `meter_max`. The carried force (`colour`) and the three tiers of
  push are unchanged: an auto goads its own bar, a cast goads the carried bar, the finisher
  goads it harder.
- The struggle in `step_mechanic`: `gap = |dark − light|`; if `gap ≤ band` nothing; else
  `excess = gap − band`, the higher bar rises and the lower falls by
  `min(drift_gain × excess, drift_cap)` per second, and health falls by
  `min(burn_gain × excess, burn_cap)` per second, never below one.
- Calm: both bars fall by `calm` per second, always, floored at zero.
- `state::depth` reads the **carried bar** for a cast, and the auto's own bar for an auto.
  Floor and ceiling knobs unchanged.
- The HUD draws two bars. `dual_mage.rs` in the tests is rewritten against them.
- Delete `meter_deep`, `meter_burn` as one-sided burn, and the centre.

**Measured criteria.**
- Inside the band no bar moves except by calm, and no health is lost.
- Outside the band the higher bar strictly rises and the lower strictly falls, each frame,
  until the lower is zero or the gap is back inside the band.
- The drift alone can never fill a bar: from any state with no input, `dark + light` is
  non-increasing.
- `finisher` from the band's edge leaves the band; `alternate` never leaves it; one cast from
  level stays inside it and two do not. That is the intended cadence and the band's first
  value follows from it.
- `idle` from the second tier falls below the first inside the time a real exchange takes
  (a Judgement's startup to the end of its recovery, twice). Calm's first value follows.
- Every button on the class moves a bar with nothing in range — the test the old design
  already carries survives.
- `time_to_kill_is_in_the_right_neighbourhood` passes; a burn from a full one-sided bar does
  not by itself kill inside the versus TTK.
- `goad` prints all five scripts and agrees with the tests.

**Human checkpoint C1.** Play script: *Dummy on. Alternate hands for ten seconds and watch
the bars. Then throw two Judgements from level and watch what happens. Catch it with the other
hand. Then do not catch it.* Questions: Do you feel the hill — that the one-sided cast started
something you have to answer? Is the band wide enough to fight in and narrow enough to fall
off? Does the burn arrive as a consequence or as a surprise?

### M2 — The tiers

**Build.**
- **Blink at half.** In the dodge branch of `state.rs`, beside the Reaver's dash: a Dual mage
  whose lower bar is at or above `tier_blink` is placed where the dodge would have ended, on
  its first frame, if `aim::clear_between` says the line is open; otherwise at the first
  obstacle. Invulnerable for the dodge's own window, then the dodge's tail. In the air it is
  the airdodge and spends it.
- **Second jump and slow fall at three quarters.** Space in the air, once per airtime, gives a
  jump impulse (knob, as a fraction of the ground jump). Reset on landing. Fall cap multiplied
  by `slow_fall` while the tier is held. The Champion's uppercut exception is the precedent
  for space-in-the-air; this is the second and it is per-class.
- A tier is held while the lower bar is at or above it, and lost the frame it drops below —
  including mid-air.
- The frame table prints the tiers under the class header, beside the jump line.

**Measured criteria.**
- Below the first tier the dodge is a dodge and space in the air does nothing. At the first
  tier the dodge is a blink; at the second, space in the air jumps once.
- A blink never passes through a stone or the arena's terrain; it ends at the obstacle.
- `a_dodge_has_a_vulnerable_tail` and `the_dodge_outruns_a_walk` pass with the blink.
- `nobody_hovers`, `classes_are_not_the_same_in_the_air` and
  `air_control_is_weaker_than_walking_for_everyone` pass at the second tier.
- **`every_class_has_a_move_that_carries_the_body`** — the roster-wide relationship the Blood
  mage thread is adding to `feel.rs` — passes for the Dual mage by the blink. Coordinate on
  the test's shape if both of you reach it in the same week; whoever lands second rebases.
- `goad` reports time-to-tier for `alternate`, and that time is inside one exchange for the
  first tier and inside two for the second. Those are first guesses for the goad amounts.

**Human checkpoint C2.** Play script: *Alternate to half. Dodge. Alternate to three quarters.
Jump twice and fall. Then throw a Judgement while at three quarters and watch what you lose.*
Questions: Did you notice the dodge became a blink before you were told? Is the second jump
and the float worth holding the bars for? When the low bar collapsed after the Judgement, did
losing the tier feel like a price you chose to pay?

### M3 — Ascension and the wings

**Build.**
- Ascension triggers only when both bars are at `meter_max`. While ascending: space in the air
  always jumps; the dodge is refused; casts read the top of the power curve; health drains
  per frame as today; landing any hit returns `ascension_refund` health (knob). It ends when
  its clock runs out, with both bars at zero and a stagger scaled by how much was landed
  (a floor and a ceiling, both knobs).
- Two wing meshes in `view`, dark on the left and light on the right, each spanned by its
  bar. Full span while ascending. A test in `view/tests/` that the drawn span is the bar's
  value, the way `kinematics.rs` checks the blade against the fist.

**Measured criteria.**
- `ascend` reaches ascension and no other script does; the drift never does.
- During ascension the dodge is refused every frame, the jump is granted every press, and the
  drain over the whole window is inside the "half to all of the bar" the original design asks
  for.
- It ends with `dark == light == 0` and a stagger inside `[floor, ceiling]`, shorter the more
  was landed.
- The wing test passes at five bar values including zero, full and lopsided.
- `budget.rs` in `view` passes with the wings drawn.

**Human checkpoint C3.** Play script: *Climb to both full — it should be hard. Fly. Cast.
Come down.* Questions: Was the ride worth the climb? Could you read your own bars off the
wings without looking at the HUD? When it ended, did the empty bars and the stagger feel like
the vent the design describes, or like a punishment for succeeding?

### M4 — Against a person, and tuning

**Build.** Nothing new; this is the knob pass. Expect to move the band, the drift gain, calm,
the goad amounts and the tier thresholds, one at a time, each with a feel-log entry.

**Measured criteria.** Everything above still passes after every bake. `goad`'s
`alternate`, `one-sided` and `idle` numbers are re-recorded in the feel-log entry after every
change so the history of what moved is legible.

**Human checkpoint C4.** Play script: *One versus round against a Champion, then one as the
Champion against a Dual mage.* Questions, as the mage: Did you spend the round managing the
hill or ignoring it? Did you ever ascend, and was it the right moment? As the Champion: Could
you read the wings — did a lopsided mage look like a mage about to burn, and did full wings
look like something to avoid or something to punish?

### M5 — Done

The class is done when C4's answers say the hill is felt from both seats, and:

- Every measured criterion passes and `goad` agrees with the tests.
- `dual-mage.md` is rewritten to describe two bars, with the single bar recorded under a
  "was" heading and the v2 proposal folded in; `kits/dual-mage.md` says the moves are
  unchanged and how the tiers read; the README's roster row, documents table and open
  questions are current.
- The feel log has one entry per milestone with a verdict, including everything reverted.
- `cargo test --workspace` is green, the tuning is baked, and the web build
  (`./crates/web/build-game.sh`) runs the class.

## The iteration loop

At every checkpoint and after every report from the person:

1. Run `goad` and the tests. Write down what the numbers say before forming an opinion.
2. Compare to the criteria and to what the person said. Name the single knob most likely to
   move the felt result. Change that one. Bake.
3. Write or update the feel-log entry: what moved, from what to what, why, the verdict if
   there is one, and the `goad` summary after the change.
4. Push and hand over the next play script.

Stop conditions: a criterion that needs a `feel.rs` relationship loosened; a beat that cannot
be hit with two bars and the existing six moves; a merge from the Blood mage thread that
changes a shared function's meaning. Each is a report, not a decision to make alone.

## Knobs you should expect to add

Under *Dual mage* in the Oven: band; drift gain and cap; burn gain and cap; calm; the two
tier thresholds; blink invulnerable frames; second jump impulse; slow fall multiplier;
ascension refund per hit; stagger floor and ceiling. The goad amounts and the depth curve's
floor and ceiling already exist. Each new one gets a comment in `tuning.rs` saying where its
first value came from, and `goad`'s scripts are where most of them come from.

## Handoff

The pull request describes the class in the proposal's terms, lists every knob and its first
value, links the feel-log entries, and states plainly which criteria are measured and which
are felt and answered. It does not claim the class feels right. It says what the person said.
