---
status: action plan — for one implementation thread
opened: 2026-09-23
implements: ../blood-mage.md
---

# Blood mage v1 — action plan

> **Status, 2026-09-23.** M1–M4 are built and their measured criteria pass; the instrument is
> `cargo run -p sim --bin essence`. **C1–C4 are unanswered** — no person has played it — so
> the class is *built, unverified*, and the feel-log entries for the four milestones are
> open. Each checkpoint's play script below is what to hand the person next.

This is a complete brief for one implementer. Read [`../blood-mage.md`](../blood-mage.md)
first: it is the specification, and this document is how to get there and how to know when
you have. Read [`../README.md`](../README.md) and [`../../../CLAUDE.md`](../../../CLAUDE.md)
before touching code. Then read [`../kits/blood-mage.md`](../kits/blood-mage.md) for what is
in the game today and why each piece of it is shaped the way it is — most of the Grasp and
the spike's telegraph survive unchanged.

## The objective, in one paragraph

Make the Blood mage a class whose one object — **essence pools** where she cut somebody —
pays her damage, control, movement and the only heal she has, so that a player finds
themselves steering the opponent toward the blood without being told to. Her lost health is
**grey** and reclaimable only from a pool; grey lengthens her scythe, so an open bar is reach
she chooses to keep. The class is done when a person playing it says the sentence *cut,
spill, bend them back onto it, cash in* in their own words.

## Ground rules

These are the repository's rules, restated because breaking them costs the most time.

- **`crates/sim` has no floats, no allocation, no I/O, no dependencies.** Fixed point is
  `Fx`. `World` stays under 4 KiB. `no_floats.rs`, `determinism.rs` and `budget.rs` enforce
  it; run them often.
- **Every feel number is an Oven knob**, added in `oven.rs`, exposed in `tuning.rs`, and
  baked with `cargo run -p sim --bin bake_tuning`. `knobs.rs` and `oven.rs` in the tests
  enforce it. **The Oven's move store is packed in class order with the Dual mage last**;
  appending a slot to the Blood mage shifts every Dual mage index in `tuned.rs`, and
  another thread is working on her at the same time. Append, re-bake, and expect to re-bake
  again after every merge from `main`. Never renumber existing slots.
- **Aiming lives in `aim.rs`, ray-versus-shape in `math.rs`, and nowhere else.** Every line
  of effect in this kit is one of the four existing kinds. The blink uses `aim::pointing_at`
  and `aim::clear_between`, the two functions the Reaver's dodge already uses. If you find
  yourself writing a look direction or an eye position outside those two files, stop.
- **The overlay draws what the hit test uses.** The scythe's drawn length *is* its reach, and
  a pool is drawn at exactly the disc it is tested at. This is not decoration: the reach
  scales with grey, and the only argument for letting a reach scale is that both players can
  see it.
- **Design docs are the specification.** As each milestone lands, update
  `kits/blood-mage.md` so that it describes what is in the game, and write a feel-log entry.
  When the code and the proposal disagree and the code is right, change the proposal and say
  why.
- **Do not touch the Dual mage, or any other class**, beyond what a shared function forces.
  Two threads are running; keep the diff class-scoped so the merges are boring.
- **Do not add a second heal, a second resource, or a new button.** The kit is five abilities,
  an auto and the dodge. If a beat cannot be hit inside that, report it rather than growing
  the kit.
- Before every push: `cargo fmt --all`, `cargo clippy --workspace --all-targets`,
  `cargo test --workspace`. Nothing runs them for you.

## How you will know — the two kinds of evidence

You cannot feel the game. You can measure it, script it, step it frame by frame, screenshot
it, and reason about it; and a person can play it. Both kinds of evidence are required, and
you must be honest about which kind you have.

**Measured** evidence is tests, the frame table, the hunt report, and the instrument below.
It can prove a relationship holds. It cannot prove the class feels right.

**Felt** evidence comes from the human checkpoints below. At each one you stop, push, write a
feel-log entry whose verdict is *open*, and hand the person a play script: what to do, what to
look for, and the one or two questions only they can answer. You then iterate on what they
report — one knob at a time, baked, with the entry updated to say what moved and why.

**Honesty rules.** A passing test is never reported as "it feels right". A milestone whose
measured criteria pass and whose felt criteria are unanswered is *built, unverified*, and is
described that way. If a criterion cannot be met without loosening a relationship in
`feel.rs`, that is a design finding: stop, write it down, and ask — do not weaken the test.
Record what you tried and reverted; reverted experiments are the most valuable entries in the
feel log.

## Build the instrument first

Before any ability changes, add `cargo run -p sim --bin essence` (beside `frametable` and
`beastcheck`). It runs a fixed scripted exchange against the training dummy and the creature
and prints, for each move, the pool it leaves and how long that pool lives; for each move
landed over a pool, how much it drinks; the grey bar over time — opened by a cast, opened by a
hit, faded, reclaimed; and the scythe's reach at each grey level. Every number it prints is
one that the success criteria below name. The bin is how you self-evaluate every change in
seconds, and it is the first thing a reviewer will run.

Print it in the frame table's style: one row per event, headers, and a closing note that
says what the numbers mean.

## Milestones

Work them in this order. Each one is playable on its own and lands with tests, a re-bake, a
kit-doc update and a feel-log entry. Do not start the next before the current one's measured
criteria pass.

### M1 — Grey health and the scythe

**Build.**
- `Player` gains `grey: i32`. Costs and enemy damage move red into grey. Grey fades at a
  fixed rate per frame (knob). Self-cost clamps red at one, as today.
- The auto becomes **Reaping sweep**: a swing, wide and thin, low-to-high across the front,
  with a tip that hits harder. Its `reach` is multiplied by a curve on grey — base at zero,
  `reach at full grey` (knob) at a full bar of grey. Damage scales on the same curve with its
  own knob.
- **Reap** on right click: overhead, unblockable, slow, long recovery, the biggest hit in the
  kit, reach on the same curve.
- The scythe is drawn at the reach it hits at. Add the check to `view/tests/kinematics.rs`.
- Two clips in `crates/anim/src/clips/blood.rs`: the sweep and the overhead. The clip
  contract in `view::clips` refuses to bake with one missing, so both must exist before the
  game runs.
- Remove `leech` from `Move` and from the frame table. Nothing returns health in M1.

**Measured criteria.**
- `red + grey + gone == max` on every frame of a scripted exchange, with `gone` only ever
  growing by the fade.
- A cast at full health leaves red down by its cost and grey up by the same.
- A hit of `d` from the dummy leaves red down by `d` and grey up by `d`.
- Grey at a full committed cast's worth survives one full exchange (the time from a Reap's
  startup to the end of its recovery plus one dodge) before the fade takes half of it. That
  is the starting fade rate; it is a knob.
- Reach at full grey is at least half again the base reach, and the drawn blade matches the
  hit volume at three grey levels.
- `every_class_can_beat_a_turtle` passes on Reap. `every_attack_is_punishable_on_block` and
  `committed_moves_are_more_punishable_than_pokes` pass with the new frames.
- The frame table prints both moves with a `reach at full grey` note.

**Human checkpoint C1.** Play script: *Pick the Blood mage, dummy on. Cast Reap into the air
twice and watch the blade. Let the dummy hit you twice and watch it again. Then sweep at the
dummy from a range that was out of reach before.* Questions: Does the blade visibly grow?
Do you feel more dangerous after being hit, or just lower? Is the sweep's arc readable as a
scythe?

### M2 — Essence pools and drinking

**Build.**
- `EffectKind::Pool { volume }` in the fixed effect array, height fixed, radius a function of
  volume (knob). Spawned under the point of contact on every hit she lands, volume equal to
  the damage dealt. Drains at a fixed volume per frame (knob). A pool spawned overlapping an
  existing one merges into it. Cap of four; a fifth merges into the newest. Her own costs
  never pool. The creature pools under the struck part's floor projection.
- **Double duty**: one function, `drink(move, pool) -> health`, called wherever a move's hit
  test succeeds while the volume or the target overlaps a pool. It converts grey to red, never
  above max, and shrinks the pool by what it took. The sweep drinks a small share, the Reap a
  large one, the Bloodletter's return leg a small one as it crosses.
- Pools are drawn as the disc they are tested at, in the arena and in the overlay.
- Bloodletter moves to middle click, unchanged otherwise except that it no longer leeches.

**Measured criteria.**
- A pool's radius and lifetime are monotone in the damage that made it; a Reap's pool outlives
  a sweep's by at least the ratio of their damage.
- Five hits on one spot produce one pool, not five.
- `the_blood_mage_pays_for_everything_and_nobody_else_pays_for_anything` still passes.
- The feel test that said *an ability thrown perfectly returns more than it cost* is
  rewritten: *an ability landed over a pool of its own making returns more than it cost, and
  one landed on bare floor returns nothing.* Sweep, Reap and Bloodletter pass it.
- No effect is allocated: `budget.rs` passes, `World` is under 4 KiB.
- `essence` prints the pool table and the drink table and they agree with the tests.

**Human checkpoint C2.** Play script: *Sweep the dummy three times and look at the floor.
Step back, then Reap the dummy while it stands in the pool and watch your bar. Then do the
same with the dummy pulled off the pool.* Questions: Did you know where to stand? Did the
heal feel earned or automatic? Is the pool readable at a glance in third person, and does
four of them clutter the arena?

### M3 — The blink, the haul onto her feet, and the spike on a pool

**Build.**
- **Blink.** In the dodge branch of `state.rs`, beside the Reaver's, a Blood mage whose
  crosshair is on a pool (`aim::pointing_at`, with a clear line by `aim::clear_between`) is
  placed in that pool on the dodge's first frame, invulnerable for the dodge's window, and
  the pool is consumed. On the ground and in the air; the airborne one spends the airdodge.
- **Grasp** hauls to her feet rather than to her arm's length, and when all four arms land on
  the creature it hauls *her* to the contact point instead. Everything else about the channel
  and the catch is as built.
- **Black spike** on bare floor: a spike, damage, launch, short slow, spills what it hits.
  On a pool: erupts at the pool's radius, launches and slows everything in it, drinks the
  whole pool. The drain field is deleted, and with it the effect kind that carried it.

**Measured criteria.**
- The blink is refused with no pool under the crosshair, with the line blocked by a stone,
  and in the air after the airdodge is spent. It is taken in every other case and the pool
  is gone afterwards.
- `one_aim.rs` passes without new exemptions. No new caller of `camera::eye` or `look_dir`
  exists outside `aim.rs`.
- The new roster-wide relationship **`every_class_has_a_move_that_carries_the_body`** is added
  to `feel.rs`, and the Blood mage passes it by the blink. (The Champion's step, the Reaver's
  dash and the Elementalist's Landfall pass; the Dual mage fails today, and the other thread
  owns that. Write the test so it is per-class and her row can be marked pending rather than
  breaking the suite.)
- `a_grasp_always_finishes_hauling_before_it_lets_go` still holds with the new endpoint, and a
  victim hauled onto a pool stands in it on the frame the hold ends.
- A spike on a pool of volume `v` erupts at the pool's radius and returns the pool's whole
  drink; the same spike on bare floor returns nothing and leaves a pool where it hit.
- `preying_on_the_disabled_is_worth_feeling_and_is_not_an_execution` passes.

**Human checkpoint C3.** Play script: *Bloodletter the dummy at range to start a pool under
it. Dodge with the crosshair on the pool. Stand in a pool and Grasp the dummy onto it, then
Reap. Put a spike on a big pool and on bare floor.* Questions: Does the loop — cut, spill,
bend them back onto it, cash in — occur to you unprompted? Is choosing between blinking and
drinking a real decision or does one always win? Is the spike-on-pool eruption readable as
"the floor came up"?

### M4 — The creature, the hunt, and tuning

**Build.**
- Pools under a toppled Ridgeback; the blink onto its back; the Grasp haul onto its flank.
- `cargo run -p hunt --bin fight -- --class blood` runs to completion. Add pools made and
  pools drunk to the report if the hunter can reach them, and make the hunter drink when
  standing in one.
- Costs raised to the proposal's shape: the auto a trickle, Reap and Grasp real, the spike the
  most. All knobs.

**Measured criteria.**
- The hunt report shows the class reaching the topple and healing from the pool under it.
- `the_blood_mage_does_not_kill_in_two_buttons` and
  `time_to_kill_is_in_the_right_neighbourhood` pass at the new costs.
- `essence` against the creature shows the largest pool in the game under a topple.

**Human checkpoint C4.** Play script: *One hunt. Then one versus round against a Champion
who keeps moving, and one against a Champion who stands and trades.* Questions: Against the
mover, do the pools vanish before you can use them — and does that feel like correct
counterplay or like the class not working? Against the trader, do you out-heal them, and by
how much? Do you find yourself banking grey for the reach, or healing as soon as you can?

### M5 — Done

The class is done when C4's answers say the sentence holds against both opponents, and the
following are all true:

- Every measured criterion above passes and the `essence` bin agrees with the tests.
- `kits/blood-mage.md` describes the game as it is, with the proposal folded in and the old
  economy recorded under a "was" heading, and the README's roster row and open questions are
  current.
- The feel log has one entry per milestone with a verdict, including everything reverted.
- `cargo test --workspace` is green, the tuning is baked, and the web build
  (`./crates/web/build-game.sh`) runs the class.

## The iteration loop

At every checkpoint and after every report from the person:

1. Run `essence` and the tests. Write down what the numbers say *before* forming an opinion.
2. Compare to the criteria and to what the person said. Name the single knob most likely to
   move the felt result. Change that one. Bake.
3. Write or update the feel-log entry: what moved, from what to what, why, and the verdict
   if there is one. If you reverted, say what it felt like.
4. Push and hand over the next play script.

Stop conditions: a criterion that needs a `feel.rs` relationship loosened; a beat that cannot
be hit inside five abilities, an auto and the dodge; a merge from the Dual mage thread that
changes a shared function's meaning. Each of these is a report to the person, not a decision
to make alone.

## Knobs you should expect to add

Under *Blood mage* in the Oven: grey fade per frame; reach at full grey; damage at full grey;
pool radius per unit volume; pool drain per frame; pool cap; drink share per move (a move
column); blink invulnerable frames; spike eruption radius per unit volume. Each with a comment
in `tuning.rs` saying where its first value came from.

## Handoff

The pull request describes the class in the proposal's terms, lists every knob and its first
value, links the feel-log entries, and states plainly which criteria are measured and which
are felt and answered. It does not claim the class feels right. It says what the person said.
