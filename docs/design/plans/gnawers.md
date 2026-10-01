---
status: action plan — one implementation thread
opened: 2026-10-01
implements: ../creatures/gnawers.md
---

# Gnawers — action plan

> **State, 2026-10-01 (evening).** M1–M6 built and committed: species,
> mind and rules (`crates/sim/src/species/gnawers/`), the Commons arena, the
> look, generic critter telegraphs, the hunter plan and the report's pack
> measures and Gnawers lines, 28 rule tests in `crates/sim/tests/gnawers.rs`.
> Knobs are set by `bake_tuning --set` from a list kept in the session
> scratchpad (every knob, since inserting an own knob shifts indices).
> Harness (12 seeds): Champion, Reaver, Elementalist, Blood mage, Bulwark win
> 11–12 of 12 in 25–55 s; the Dual mage 1/12 (the scripted hunter does not
> play her bars -- it loses the Ridgeback too). Next: M7 tuning toward the
> 1–2 minute band, M8 poses and screenshots, M9 docs and the browser.

The specification is [`../creatures/gnawers.md`](../creatures/gnawers.md); the
recipe is [`../critters.md`](../critters.md) §6. Parity bar:
[`../monsters.md`](../monsters.md), contract [`../bestiary.md`](../bestiary.md) §1.

## Milestones

- [x] **M1 · Species, arena, ring.** `species/gnawers/` (moves, kinds, pack,
      mind), bootstrapped and baked `tuned.rs`, four registry lines, the
      Commons arena with a den in the bank, a look. A hunt shows six gnawers
      ringing at 5 m, rear first; size/alloc/determinism tests pass.
- [x] **M2 · Aim check.** gnawers.md §1a against what the foundation built
      (`aim::stands_at`, `aim::stoop`, `first_along`); `critcheck --species
      gnawers` for the gnawer and the Big One; every auto touches at 2 m.
- [x] **M3 · Bites and tokens.** Dart-bite (crouch tracks, locks at
      `dart_lock`), hamstring (rear third only, slow, latch shed by a dodge),
      tokens, a generic `PackMind::landed` hook, generic critter floor
      telegraphs in the renderer (drawn over the character).
- [x] **M4 · The pack's behaviours.** Pile-on (sequenced leaps, three landed
      is a knockdown), scatter (Big One only 3 m), morale/den/regroup, treed,
      the arc at three.
- [x] **M5 · The Big One.** Maul (stops at 1.0 m), howl (rally, lockout,
      cancelled by any hit, desperation), hang-back, strain stumble window.
- [x] **M6 · Safe spots, hunter, report.** Scramble, gnaw; the scripted
      hunter's §9 plan; the §9 report lines (swings over, behind you,
      hamstrings, pile-ons, crouches interrupted, howls, scatter windows,
      leader dead at, hidden commits), the pack's four windows.
- [ ] **M7 · Tuning by harness.** Champion plus every class §7 singles out,
      many seeds, until §9's targets hold; relationships pinned in
      `tests/gnawers.rs`; feel-log entries.
- [ ] **M8 · Reading it.** Per-move critter poses (crouch, scuttle, leap, rear
      to howl, low maul, dig), screenshots of each telegraph (`SHOT_MOVE`).
- [ ] **M9 · World, docs, browser.** Trophy and tempers; gnawers.md marked
      built with "Where it landed"; README map, manual tables; web smoke with
      `?hunt=gnawers`; merge main, checks, push.
