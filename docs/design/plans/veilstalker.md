---
status: action plan — built
opened: 2026-10-01
implements: ../creatures/veilstalker.md
---

# The Veilstalker — action plan

The specification is [`../creatures/veilstalker.md`](../creatures/veilstalker.md);
the parity bar is the Ridgeback ([`../monsters.md`](../monsters.md)); the
recipes are [`../species.md`](../species.md), [`../arenas.md`](../arenas.md)
and [`../hazards.md`](../hazards.md). The Pair are the nearest relative: the
same eighteen-bone topology, a frame hook that flies leaps and keeps state in
the lore, and a scripted hunter with a camera. This file is how to get there,
milestone by milestone, so a successor can pick it up from the checkboxes.

## State

Started 2026-10-01 on `claude/creature-veilstalker`; built the same day.
Every milestone below is done; where it landed is the creature document's
§13, and the passes are in the feel log. `origin/main` (the Broodmother)
merged in. What is open is at the end.

**Tuning method** (for a successor): the baked file is the truth. A
scratch script reads every `// id = value` comment out of
`crates/sim/src/species/veilstalker/tuned.rs`, resets it to the empty
bootstrap, bakes once to learn the ids, and bakes again with `--set` for
every id plus any override file of `id=value` lines -- so adding an own
knob (which shifts the move rows' indices) keeps every value. Rewrite it
from that description if it is gone.

## Decisions taken before building

- **Eighteen bones, not twelve.** The doc's "lighter rig" was a snapshot
  estimate; bones cost nothing in the snapshot (`Monster` is the same size
  whatever the species) and the Pair's eighteen-bone cat topology comes with
  its animation builders. Re-proportioned to a 6.5 m lizard-cat, three metres
  of it tail.
- **The look is the facing.** The sim keeps no input in the `World`; a
  fighter's `facing` is the look's yaw whenever they can act
  (`state::step_player`), and the doc's "the character's head and shoulders
  turning" is exactly that. The frame hook glances it every `Glance` frames
  into the lore, with the position, and the brain's view gate reads it there.
  The eye is built from it by a new `aim::in_view_from` (level pitch), so no
  eye is built outside `aim.rs`.
- **Decloak = the move's startup**, less what comes after the reveal (the
  lunge's gather, the pounce's flight): `fight::decloak(kind)`. One knob,
  `DecloakFloor` (range bottom 18), and `feel.rs` checks every attack's
  decloak against it.
- **Shown is derived, not stored**: from the move in progress (the ramp over
  the startup's first `DecloakRamp` frames), the re-cloak fade timer, speed
  (shimmer), paint, mottle, and hazards under each part (outline). One
  function, `fight::shown`, behind `World::shown`.
- **The mimic's ghost is drawn from the lore**: where it stands, which way,
  which rear it plays, and its frame. `fight::apparition` is what is drawn of
  any decloak, real or not -- the only thing the hunter reads to see one.
- **Paint at the point of contact**: the `struck` hook only has a part, so the
  frame hook paints the point of that part's box nearest the striking
  fighter's chest, in the part's own frame (so it moves with the body).
- **Report windows**: a `Tally::until_free` default method (identity) so the
  Veilstalker's tally can add the decloak floor and the stalk to
  `frames_until_free`; nothing else in the report moves.

## Lore layout (24 cells available)

| Cells | What |
| --- | --- |
| 6 | hazards: coals ×4, smoke, flash |
| 8 | footfall ring, 32 × 4 bytes |
| 1 | paint ring, 4 × 4 bytes |
| 5 | veil state, glances, quills, mimic, braziers, report counters |

## Milestones

### M1 · The body and the veil
- [x] `species/veilstalker/`: bones, parts (four hide regions), legs, the
      move table, clips, own knobs; bootstrap `tuned.rs`/`baked.rs`; register;
      first guesses baked.
- [x] `fight.rs`: lore words, `shown`, the re-cloak fade, shimmer.
- [x] `aim::in_view_from`.
- [x] Test: `a_cloaked_body_is_hit_exactly_like_a_visible_one`.

### M2 · The Ashwood and the trail
- [x] `arena/veilstalker.rs`: 36 × 36 snow, ash pits, stream, six trunks,
      four braziers (sites), the 3 m cordwood wall with a ledge.
- [x] Footfall ring stamped by the gait and by every real decloak.
- [x] Test: `every_footfall_is_in_the_ring_and_a_rollback_lays_the_same_trail`.

### M3 · The strikes
- [x] Lunge, spear (lane locks at 12), rake + second swipe, pounce from a
      perch (climb), quills (own hit, five in a fan, stopped by solids).
- [x] Tests: decloak floor (sim + `feel.rs`).

### M4 · Paint, mottle, fire
- [x] Paint ring; regions and mottle; smoke (P4, grows, burns off);
      braziers (tip once, coals 8 s); outlines; panic and its lockout.
- [x] Tests: paint, mottle, panic, smoke.

### M5 · The brain
- [x] Glance of the look; view gate; edge; bait; exposure; leave; stalk;
      bound; mimic and its quiet; retreat and the recoil break; coop.
- [x] Tests: view gate, mimic, retreat; determinism green.

### M6 · Animation
- [x] `anim/src/beast/veilstalker/`: idle, slink, bound, every move, flinch,
      stumble, panic roll, dead; bake; contact sheets looked at.

### M7 · The hunter and the report
- [x] `hunt/src/plans/veilstalker.rs`: the view, §9's plan, the lines.
- [x] Twenty-four seeds per class; tier 4 (22 of 144 won); feel-log entries.

### M8 · Drawn
- [x] Look; veil rendering (shimmer, prints, paint, breath, ghost, quills);
      the Ashwood's dressing; screenshots of every telegraph.

### M9 · Finished
- [x] Trophy, tempers; docs (creature doc built + "Where it landed",
      species.md, README map, manual); web smoke `?hunt=veilstalker`;
      merge `origin/main`, fmt/clippy/test, push.

## Open

- **The way-in window** reads 6-8 % against ~20: the measure files the long,
  visible recoveries as walk-up (§13).
- **The Elementalist's fire and the Blood and Dual mages** are not played by
  the harness; their hunts say nothing about the fire window or the pools.
- **Not built**: Veilstep, a foreign cloud outlining it, prints across water,
  the mottled-pelt trophy, a refracting shimmer.
