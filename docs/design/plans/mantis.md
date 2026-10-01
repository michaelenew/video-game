---
status: action plan — one implementation thread
opened: 2026-10-01
implements: ../creatures/mantis.md
---

# Mantis — action plan

> **State, 2026-10-01 (started).** Reading done; M1 in progress. Branch
> `claude/creature-mantis`, worktree `/home/user/wt/mantis`.

The specification is [`../creatures/mantis.md`](../creatures/mantis.md); the
recipes are [`../species.md`](../species.md), [`../arenas.md`](../arenas.md),
[`../hazards.md`](../hazards.md) §7. Parity bar: [`../monsters.md`](../monsters.md),
contract [`../bestiary.md`](../bestiary.md) §1. Also read
[`../defense.md`](../defense.md) and [`../sparring.md`](../sparring.md).

## Decisions taken up front

- **The creature guard is a seam, asked at every place a blow reaches a
  creature.** `monster::Blow` (where it comes from, unblockable, who threw
  what) and `FightDecl::guard`, called by `Monster::take_blow` before
  `take_hit` at every hit site (swing, echo, recall, effects, beam, bolts,
  gusts, debris, fire bolt). `None` everywhere else, so every other creature
  and both pins are bit-identical. The cone test is shared with the fighters'
  `guard_against` (`state::in_guard_arc`), the elevation is the creature's
  one extra comparison.
- **Its eyes are a delay line in the hunt's lore**, written by its frame hook
  (positions every 4 frames, a short history of what each fighter began and
  when), and read by a new seam `FightDecl::sight` that replaces the glance's
  present sample with the delayed one. The brain's window is still positions,
  velocities and -- new, and only for this species -- the *deed* a fighter
  began at least D frames ago, which is snapshot state (an action and its
  start), never an input.
- **Guard, Ready, Prayer and the staggers are moves** (zero damage), so the
  report, the windows, the telegraph and the clip machinery read them as
  they read everything. The guard is up during `Active{GUARD}`; its first
  `parry_window` frames parry; the frame hook lowers it (to its recovery)
  after the minimum hold. The counter is a never-chosen move the guard seam
  starts on a parry.
- **Branching is the frame hook's**: the first slash `then`s into the fast
  second slash and the hook swaps the held variant in at commit; the Leap
  `then`s into the Dive.
- **The habit ring is written only from the guard seam** (a blow meeting the
  guard), through a two-word mailbox on the body (`Monster::own`) the frame
  hook drains into the lore ring. Nothing reads an input.
- **Blades are the breakable parts** (the scythe segment of each raptorial
  arm); breaking one is the shared stumble (its length 90), and the guard seam
  reads `broken` to make the guard one-sided.

## Milestones

- [ ] **M1 · Body, Shrine, guard.** Species table, bootstrap files, registry
      lines, the Shrine; `FightDecl::guard` and the hit sites; guard tests;
      `beastcheck --species mantis` with the cone's top against every hop.
- [ ] **M2 · Its eyes and the coil.** Sight ring, D's wander, deeds; coil and
      lunge with the lane and the stop at a solid. Sight and coil tests.
- [ ] **M3 · The rest of the set.** Pair, counter, Leap/Dive, flare, pivot,
      prayer and haste; per-move tests.
- [ ] **M4 · Blades and desperation.** One-sided guard, hurt and desperate
      stages, coop numbers.
- [ ] **M5 · The habit.** Ring, Ready, the notches (marks), the flag and the
      geometric guess; the three habit tests.
- [ ] **M6 · Animation.** Clips in `anim/src/beast/mantis/`, baked, sheets
      reviewed.
- [ ] **M7 · Measured.** The duellist's plan and three ablations, report lines
      and the guarded band, tuning passes, feel log.
- [ ] **M8 · Reading it, world, docs, browser.** Look, Shrine dressing,
      telegraph screenshots, trophy, docs, manual, web smoke, merge main,
      checks, push.
