---
status: action plan — for one implementation thread
opened: 2026-09-23
implements: ../shadow-reaver-v2.md
---

# Shadow Reaver v2 — action plan

This is a complete brief for one implementer. Read
[`../shadow-reaver-v2.md`](../shadow-reaver-v2.md) first: it is the specification, and this
document is how to get there and how to know when you have. Then read
[`../kits/shadow-reaver.md`](../kits/shadow-reaver.md), which describes the shadow, the dash,
the lotus and the recall as built — none of which change — and the two feel-log entries on
the second body and the carry. Read [`../README.md`](../README.md) and
[`../../../CLAUDE.md`](../../../CLAUDE.md) before touching code.

## The objective, in one paragraph

Give the Reaver the moment her pattern is missing. The shadow out on the field turns to
face whoever is in reach, so its copies land; every hit it lands from the field **marks** the
target; crossing to the shadow by dash opens a short window in which her first landed swing
**spends** the marks for a burst, and at a full tally also staggers. Her health drops to
roughly three quarters of the Champion's, and the attending copy's share halves. The class is
done when a person playing it says, in their own words, *send, mark, cross, cash, leave* —
and says that staying in melee after the cash-in felt wrong.

## Ground rules

The repository's rules, restated because breaking them costs the most time.

- **`crates/sim` has no floats, no allocation, no I/O, no dependencies.** `Fx` is fixed
  point. `World` stays under 4 KiB. Marks are two integers on `Player`; the window is one.
- **Every feel number is an Oven knob**, baked with `cargo run -p sim --bin bake_tuning`.
  Two other threads may be appending knobs at the same time; re-bake after every merge from
  `main` and let the `oven.rs` test tell you when you forgot. Never renumber slots.
- **Aiming lives in `aim.rs`.** The shadow turning to face a target is a decision about
  where a thing goes, so it is a function there and `shadow.rs` calls it. If you write a
  yaw-to-nearest-body anywhere else, `one_aim.rs` is right to fail you.
- **The overlay draws what the hit test uses.** Marks on a body are drawn where the hit
  test reads them. The copy's volume is already `state::hitbox` on the echo body; keep it so.
- **The health table is shared.** The Bulwark thread wants the same per-class table. If
  `Health` is already in the Oven when you merge, use it; if not, add it with one multiplier
  per class and the other thread rebases.
- **Do not touch any other class** beyond what a shared function forces. Do not add a
  button, a move, or bind Deadly mistake; it stays open.
- **Design docs are the specification.** As each milestone lands, update
  `kits/shadow-reaver.md` to describe the game as it is and write a feel-log entry.
- Before every push: `cargo fmt --all`, `cargo clippy --workspace --all-targets`,
  `cargo test --workspace`.

## How you will know — the two kinds of evidence

You cannot feel the game. You can measure it, script it, step it, screenshot it and reason
about it; a person can play it. Both are required and you must be honest about which you
have.

**Measured** evidence is tests, the frame table, and the instrument below. It proves a
relationship holds. **Felt** evidence comes from the human checkpoints: you stop, push, write
a feel-log entry whose verdict is *open*, and hand the person a play script with the one or
two questions only they can answer. Then iterate on what they report, one knob at a time.

**Honesty rules.** A passing test is never reported as "it feels right". A milestone whose
measured criteria pass and whose felt criteria are unanswered is *built, unverified*. A
criterion that cannot be met without loosening a `feel.rs` relationship is a design finding:
stop, write it down, ask. Record what you tried and reverted.

## Build the instrument first

Add `cargo run -p sim --bin tally` beside `frametable`. It runs scripted exchanges and
prints, per script: the shadow's copy hit rate against a dummy walking a fixed pattern at a
fixed range, with and without the self-aim; marks on the dummy over time, including the
fade; the damage of a cash-in at each mark count and whether it staggered; and the frames
from the send to the cash-in for the fastest scripted pattern. Scripts it must carry:

| Script | What it is |
| --- | --- |
| `range` | shadow sent to six metres, dummy strafing across it, Reaver swinging on a rhythm |
| `stall` | marks built to the cap, then nothing, until they are gone |
| `pattern` | send, swing until the cap, dash, slash |
| `greedy` | the same, cashing with Executioner |
| `stick` | shadow attending, Reaver in melee, swinging for the length of `pattern` |

Every criterion below is a number this bin prints.

## Milestones

In order. Each is playable, lands with tests, a re-bake, a kit-doc update and a feel-log
entry, and the next does not start until the current one's measured criteria pass.

### M1 — The shadow aims itself, and the health table

**Build.**
- `aim::shadow_faces(shadow_pos, reach, scene) -> Option<V3>`: the flat direction from the
  shadow to the nearest fighter or the creature's nearest part within `reach` plus a slack
  knob, or `None`. `shadow::echo_body` uses it for the facing when the shadow is `Waiting`;
  attending, the eased facing stands.
- A `Health` family in the Oven: one multiplier per class, the Reaver's first value three
  quarters, the Bulwark's above one, the rest one. `max_health` for a player reads it.

**Measured criteria.**
- `range` shows the copy hit rate rising from near zero to most swings once the self-aim is
  on, at a dummy that is inside reach and off her line.
- `a_shadow_out_on_the_field_swings_from_out_there` still passes; a new test shows the copy
  from an attending shadow does *not* turn.
- `one_aim.rs` passes with no new exemptions.
- `time_to_kill_is_in_the_right_neighbourhood` passes with the Reaver at her multiplier, and
  the frame table prints health beside the jump line.

**Human checkpoint C1.** Play script: *Send the shadow to mid range beside the dummy. Stand
off and swing at the air. Then move the dummy around the shadow.* Questions: Do the copies
now land on something? Does the shadow turning read as it fighting, or as it twitching?

### M2 — The tally

**Build.**
- `Player` gains `marks: u8` and `mark_clock: u16`. Every shadow damage site — the echo
  strike, each lotus pass, the recall's cut — adds one mark per victim per event, capped. The
  clock removes one mark every `mark_fade` frames and resets on a new mark.
- Marks are drawn on the marked body in the HUD and the overlay.
- The attending shadow's copy deals `shadow_damage_attending` (halved) and marks nothing.

**Measured criteria.**
- `pattern` reaches the cap; `stall` decays to zero on the clock; `stick` never marks.
- A lotus out-and-home marks at most twice per victim.
- The recall's cut marks once.
- `budget.rs` passes; `World` is under 4 KiB.

**Human checkpoint C2.** Play script: *Send, then watch the dummy as the copies land. Walk
away and watch the marks fade. Then fight from the shadow's shoulder for the same time.*
Questions: Can you read the tally on the body without the HUD? Does the fade feel like a
clock you are racing? Does fighting with the shadow at your heel feel like the wrong way to
play now?

### M3 — The cash-in

**Build.**
- Arriving by dash starts `cash_window` frames (first value: the carry's length; second: twice
  it). Her hit sites read it: a landed swing inside the window on a marked victim multiplies
  damage by one plus `marks × mark_worth`, clears the marks, and at the cap applies
  `cash_stagger`. Recall and the leash do not open the window.
- The frame table prints the cash-in multiple at the cap beside Slash and Executioner.

**Measured criteria.**
- `pattern` and `greedy` print a cash-in larger than the plain swing by the intended
  multiple, with the stagger at the cap and not below it.
- A recall home followed by a swing multiplies nothing.
- A swing one frame after the window closes multiplies nothing.
- The new roster relationship in `feel.rs` — *a class's largest hit is gated behind
  something the opponent could see coming* — passes for the Reaver.
- The cash-in at the cap with Executioner is the largest single hit in the frame table and
  is still under the ceiling `preying_on_the_disabled_is_worth_feeling_and_is_not_an_execution`
  uses; add a sibling test that says so.

**Human checkpoint C3.** Play script: *Run the pattern: send, let it mark, dash, Slash. Then
again with Executioner. Then dash and wait a beat before swinging.* Questions: Is the window
findable, and is missing it your fault? Does the greedy cash feel worth its risk? Does the
stagger at a full tally feel earned?

### M4 — Against a person, the creature, and tuning

**Build.** Marks on the creature, with its own cap if needed. The knob pass: cap, fade,
window, worth, the attending share, the health multiplier. One at a time, each with a
feel-log entry and the `tally` numbers re-recorded.

**Measured criteria.** Everything above still passes after every bake. `fight --class reaver`
completes and the hunter can be seen crossing to cash on a leg.

**Human checkpoint C4.** Play script: *One versus round against a Champion. One as the
Champion against a Reaver. One hunt.* Questions, as the Reaver: did the round have the
rhythm — send, mark, cross, cash, leave — or did you stick? As the Champion: could you see
the marks climbing and do something about it? Did she feel like glass when you caught her?

### M5 — Done

- Every measured criterion passes and `tally` agrees with the tests.
- `kits/shadow-reaver.md` describes the tally, the window and the health as built, with the
  proposal folded in; the README's roster row and open questions are current.
- One feel-log entry per milestone with a verdict, including everything reverted.
- `cargo test --workspace` green, tuning baked, the web build runs the class.

## The iteration loop

1. Run `tally` and the tests. Write down what the numbers say before forming an opinion.
2. Compare to the criteria and to what the person said. Change the one knob most likely to
   move the felt result. Bake.
3. Write or update the feel-log entry with the `tally` summary after the change.
4. Push and hand over the next play script.

Stop conditions: a `feel.rs` relationship that would have to be loosened; a merge from
another thread that changes a shared function's meaning; a beat that cannot be hit without
a new button. Each is a report, not a decision to make alone.

## Knobs you should expect to add

Under *Shadow Reaver*: self-aim slack; mark cap; mark fade; cash window; mark worth; cash
stagger; attending share. Under *Health*: one multiplier per class. Each with a comment in
`tuning.rs` saying where its first value came from.

## Handoff

The pull request describes the class in the proposal's terms, lists every knob and its first
value, links the feel-log entries, and says which criteria are measured and which are felt
and answered. It does not claim the class feels right. It says what the person said.
