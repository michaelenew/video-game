---
status: action plan — carried out 2026-09-23; measured criteria met, felt criteria unanswered
opened: 2026-09-23
implements: ../bulwark-v2.md
---

# Bulwark v2 — action plan

This is a complete brief for one implementer. Read [`../bulwark-v2.md`](../bulwark-v2.md)
first: it is the specification, and this document is how to get there and how to know when
you have. Then read [`../kits/bulwark.md`](../kits/bulwark.md) for what is built,
[`../bulwark.md`](../bulwark.md) for why the class exists, and [`../defense.md`](../defense.md)
for the block layer it stands on. Read [`../README.md`](../README.md) and
[`../../../CLAUDE.md`](../../../CLAUDE.md) before touching code.

## The objective, in one paragraph

Make blocking produce something. Every hit taken on the shield is stored as **weight**;
Slam and Throw spend it, and a planted shield is a real **structure** sized by it. Slam gets
the free middle click. Health goes highest on the roster. The class is done when a person
playing it alone, in versus and in a hunt, says that blocking felt like loading rather than
waiting — and that the opponent could see it.

## Ground rules

- **`crates/sim` has no floats, no allocation, no I/O, no dependencies.** Weight is one
  integer carried on every `Shield` variant.
- **Every feel number is an Oven knob**, baked with `cargo run -p sim --bin bake_tuning`.
  Other threads are appending knobs at the same time; re-bake after every merge from `main`.
  Never renumber slots.
- **Aiming lives in `aim.rs`.** Nothing here aims anything new: the throw is the skillshot it
  is, the slam is the swing it is. If you find yourself writing a ray outside `aim.rs` or
  `math.rs`, stop.
- **The overlay draws what the hit test uses.** The planted wall is drawn at the size it is
  tested at; the slam's shake is drawn at its radius; the shield's drawn weight is the number
  in the snapshot.
- **The planted shield joins the structure system rather than duplicating it.** The solid
  query that walks stones walks the planted shield too. One function, two sources. Do not
  write a second collision path.
- **The health table is shared** with the Reaver thread. If `Health` is in the Oven when
  you merge, use it; otherwise add it and the other thread rebases.
- **Do not touch any other class** beyond what a shared function forces. No new button; Slam
  goes on middle click, which is free.
- **Design docs are the specification.** Update `kits/bulwark.md` as each milestone lands,
  and write a feel-log entry.
- Before every push: `cargo fmt --all`, `cargo clippy --workspace --all-targets`,
  `cargo test --workspace`.

## How you will know — the two kinds of evidence

You cannot feel the game. You can measure, script, step, screenshot and reason; a person can
play. Both are required and you must be honest about which you have.

**Measured** evidence is tests, the frame table, the hunt report, and the instrument below.
**Felt** evidence comes from the human checkpoints: stop, push, write a feel-log entry whose
verdict is *open*, hand over a play script with the questions only a person can answer, and
iterate one knob at a time on what they say.

**Honesty rules.** A passing test is never "it feels right". Built with unanswered felt
criteria is *built, unverified*. A criterion that needs a `feel.rs` relationship loosened is
a design finding: stop, write it down, ask. Record what you tried and reverted.

## Build the instrument first

Add `cargo run -p sim --bin weight` beside `frametable`. It runs scripted exchanges and
prints: weight after each blocked and parried hit from each class's opener and from each of
the creature's moves; weight over time with nothing landing; slam damage and shake radius at
five weights; throw speed, damage and whether it knocked down at the same five; planted wall
size at the same five; pushback taken by a guard at the same five. Scripts:

| Script | What it is |
| --- | --- |
| `load` | block five hits from a Champion sword string, then parry one |
| `decay` | load to the cap, then nothing |
| `slam` | load, leap to a thrown shield, slam |
| `wall` | load, throw, then walk a dummy and fire a Bolt at the planted shield |
| `stomp` | block one of each creature move, in a hunt |

Every criterion below is a number this bin prints.

## Milestones

### M1 — Weight, and the health table

**Build.**
- `Shield` gains `weight: i32` on every variant. In `apply_hit`, where `blocked` and `parried`
  are already decided, a blocked hit adds its damage and a parried hit adds `parry_load ×`
  its damage, capped at `weight_cap`. `step_mechanic` decays it by `weight_decay` per second.
- A heavy guard is pushed back less: blocked knockback multiplied by a curve from one at
  empty to `heavy_pushback` at the cap.
- The `Health` family, shared with the Reaver thread; the Bulwark's multiplier above one.
- The shield mesh scales and darkens with weight; the HUD shows it.

**Measured criteria.**
- `load` prints weight equal to the sum of the blocked damage, plus the parry multiple, and
  never past the cap.
- `decay` reaches zero on the knob's clock.
- `stomp` shows the creature's blows loading the shield, through the existing guard test in
  the creature trade.
- Pushback at the cap is measurably less than at empty, and
  `blocking_costs_no_health_but_does_cost_a_vulnerable_window` still passes.
- The frame table prints the Bulwark's health beside the jump line.

**Human checkpoint C1.** Play script: *Dummy set to attack. Block five hits and watch the
shield. Parry one. Walk away for ten seconds.* Questions: Can you see the shield loading?
Does the parry feel like a bigger deposit? Does the decay feel like a clock or like a leak?

### M2 — Slam on middle click, spending weight

**Build.**
- `clicked_move` binds middle click to Slam for the Bulwark. Slam's damage and its shake
  radius scale with weight and spend it; the fall-speed scaling sketched in the kit is built
  alongside. At the cap the shake is an area stagger of `slam_stagger` frames.

**Measured criteria.**
- `slam` prints damage and radius monotone in weight, weight zero afterwards, and a stagger
  only at the cap.
- Slam is an overhead a crouch does not duck; `every_attack_is_punishable_on_block` and
  `committed_moves_are_more_punishable_than_pokes` pass at every weight (frames do not
  change with weight; only damage and radius do).
- `hindrance_is_proportional_to_commitment` passes with weight read as commitment.

**Human checkpoint C2.** Play script: *Block three, slam. Block none, slam. Throw, leap,
slam.* Questions: Does the loaded slam feel like giving back what you took? Is the empty
slam still worth throwing? Does the leap-into-slam read as the class's big turn?

### M3 — The throw, and the wall

**Build.**
- Throw reads weight: flight speed down, damage up, and at any weight above a threshold the
  impact applies a knockdown — a launch with no height and a floor stagger. Weight is spent
  on impact.
- **The planted shield is a structure.** The solid query that walks every Elementalist's
  stones also walks any Bulwark's planted shield, sized by the weight it landed with. It
  stops bodies and shots. Recall is the ordinary recall; the planted wall is gone when it
  lifts. Drawn at the size it is tested at.

**Measured criteria.**
- `wall` shows a walking dummy stopped by the planted shield and a Bolt stopped by it, at a
  size that follows weight, and neither stopped once it is recalled.
- A loaded throw knocks down; an empty one does not.
- The Reaver's `a_stone_across_the_line_leaves_her_with_an_ordinary_dodge` has a sibling: a
  planted shield across her line does the same.
- `budget.rs` passes in `sim` and `view` with the wall drawn.

**Human checkpoint C3.** Play script: *Load the shield, throw it between you and the dummy,
walk the dummy into it. Fire an Elementalist's Bolt at it from the other seat. Recall it.*
Questions: Does the wall feel like the class's promise kept? Is a loaded throw a decision or
the obvious button? Does the wall's size read as what you took?

### M4 — Alone in both modes, and tuning

**Build.** Nothing new. The knob pass: cap, decay, parry load, slam radius, knockdown
threshold, wall size per weight, heavy pushback. `fight --class bulwark` runs and the hunter
blocks a stomp and slams a leg; add weight to the report if the hunter can reach it.

**Measured criteria.** Everything above passes after every bake. The hunt report shows a
blocked creature blow followed by a slam that breaks a leg's poise.

**Human checkpoint C4.** Play script: *One versus round against a Champion. One as the
Champion against a Bulwark. One hunt.* Questions, as the Bulwark: did blocking feel like
loading? Did you ever want to stand and take a hit on purpose? As the Champion: could you
read the shield's weight, and did it change whether you attacked? In the hunt: did the
shield do something?

### M5 — Done

- Every measured criterion passes and `weight` agrees with the tests.
- `kits/bulwark.md` describes weight, the wall and Slam's input as built; `bulwark.md`'s
  "shield as volume" section says it is built; `defense.md`'s pushback-resistance note says
  it is built; the README's roster row and open questions are current.
- One feel-log entry per milestone with a verdict, including everything reverted.
- `cargo test --workspace` green, tuning baked, the web build runs the class.

## The iteration loop

1. Run `weight` and the tests. Write down what the numbers say before forming an opinion.
2. Compare to the criteria and to what the person said. Change the one knob most likely to
   move the felt result. Bake.
3. Write or update the feel-log entry with the `weight` summary after the change.
4. Push and hand over the next play script.

Stop conditions: a `feel.rs` relationship that would have to be loosened; a merge from
another thread that changes a shared function's meaning; a beat that cannot be hit without
a new button. And one more, specific to this class: **if after C4 the person says blocking
alone still does not feel like a reason, stop and report.** The proposal names the fallback
— a grappler on a chain or a hook — and that is a new document, not a knob.

## Knobs you should expect to add

Under *Bulwark*: weight cap; weight decay; parry load; heavy pushback; slam radius per
weight; slam damage per weight; slam stagger at the cap; throw speed per weight; throw
damage per weight; knockdown threshold; wall size per weight. Under *Health*: one
multiplier per class, if the Reaver thread has not already added it.

## Handoff

The pull request describes the class in the proposal's terms, lists every knob and its first
value, links the feel-log entries, and says which criteria are measured and which are felt
and answered. It does not claim the class feels right. It says what the person said.
