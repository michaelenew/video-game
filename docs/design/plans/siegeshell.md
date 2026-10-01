---
status: action plan — one implementation thread
opened: 2026-10-01
implements: ../creatures/siegeshell.md
---

# Siegeshell — action plan

> **State, 2026-10-01.** M1-M5 built and tested (`crates/sim/tests/siegeshell.rs`,
> 24 tests); the plan and the report lines exist (M7 first pass). Numbers so
> far: coop 2 Champions 4/8 at ~374 s mean (short of 8-20 min), solo 0/8;
> 3-4 unanswerable hits across runs not yet investigated. Next: budget
> scenario, M6 (A3), tuning, docs, web. Branch `claude/creature-siegeshell`,
> worktree `/home/user/wt/siegeshell`.

The specification is [`../creatures/siegeshell.md`](../creatures/siegeshell.md);
the recipes are [`../species.md`](../species.md), [`../critters.md`](../critters.md)
§6, [`../arenas.md`](../arenas.md) and [`../hazards.md`](../hazards.md) §7. Parity
bar: [`../monsters.md`](../monsters.md), contract [`../bestiary.md`](../bestiary.md) §1.

## Decisions taken up front

- **One `Monster`, two channels.** The shared `Doing` is the **body** channel
  (Shed, Plough, Shrug, Shiver, the Siege beam) and is chosen by the shared
  brain with the species' `appetite` hook. The **legs** channel (Stamp, Drag)
  lives in `Monster::own` (the pose has to read it: a stamping foot is
  collision geometry) and is run by the frame hook. The **Footfall** is the
  gait: the stride accumulator crossing a tripod's landing phase is its
  active frame. Leg moves and the footfall are `own_hit` rows in the move
  table (`never_chosen`, `unanimated`), so their numbers are knobs and the
  report can name them.
- **The walk is the species'.** The frame hook holds the body (`rooted`, as
  the Pair and the Veilstalker do) and integrates its own walk along the
  valley at its own pace: `Walk` times the limp per broken ankle, the phase's
  hurry and any slow; halted by a stumble, a kneel or the siege line. The
  stride advances with ground covered, so the beat is a distance (180 frames
  at 0.8 m/s, 144 hurried, longer limping) and a halted body has no beat.
  Steering is the shared yaw controller toward a point ahead on the centre
  line (`prowl_to`), at a turn rate that is barely there.
- **Legs are procedural** (`repose`): every foot is placed from the stride --
  planted where it landed, swung to where it will land -- by a two-link solve
  in the leg's plane, the ankle levelled so the pad is flat. Clips animate the
  shell, crown, neck and head only. The stamp and the drag move one foot; a
  broken leg drags; a stumbling side's broken legs **splay** into the stair;
  a kneel splays all six.
- **Health is the anchors.** It has 30,000 health that no blow reduces
  (`struck` refunds it) except an anchor breaking, which takes a third; the
  third kills it. So `missing()` -- the strain thresholds' desperation -- falls
  with anchors broken, as §4 asks, and the HUD's bar reads in thirds.
- **Ankles and anchors are breakable parts**, their health in
  `Monster::breaks`; `struck` decides what a hit does (break, buckle, open).
  Broken and buckled ankles, anchors broken, the stumble's side, the open
  anchor and the leg channel are bits in `Monster::own`.
- **Stumble = shared `Doing::Stumble`** (side in `own`), **kneel = shared
  `Doing::Toppled`**, so the report's openings and topples count them and no
  body move runs inside one. `StumbleFrames`/`ToppleFrames` ranges widened
  for everyone (values unchanged).
- **The Siege beam is a body move** with a 1200-frame startup: the
  `MonsterField` frame ranges are widened to 1200 for every species (no value
  moves).
- **Parasites are the Gnawers' gnawer**, as the Broodmother's brood: the
  gnawer's five moves at the head of the table, the gnawer's mind wrapped by
  a roost mind (perched on the shell until the belly, a straggler or -- once
  roused -- a rider calls them down).
- **Vents are P4 hazards on parts** (two kinds: a grate that is drawn and a
  blast that scalds and lifts; the frame hook swaps the kind on the cycle).
- **The wall is a P7 objective** at the arena's site; a beam strikes it for
  half its health, so the first breach is the gate and the second the end.
- **The call** between the two scripted hunters is the crown hunter's
  position (standing at an anchor), which the legs hunter reads off the
  world like a person reads their partner -- not simulation state, and no
  shared memory between two plans.
- **A3 lands on its own** (M6), with the Ridgeback's pins and twelve-seed
  report measured before and after.

## Milestones

- [x] **M1 · The body walks.** Species table (30 bones, ~46 parts, 6 legs,
      13 moves, clips, knobs), bootstrap files, registry lines, the Last
      Valley, the walk and the procedural legs, `beastcheck --species
      siegeshell` heights, geometry tests, a screenshot.
- [x] **M2 · The beat.** Footfall rings and pads from the gait, the marks,
      the beat tests, a hunt where jumping the beat takes nothing.
- [x] **M3 · The legs.** Ankles, breaks, limps, buckles, the stumble and its
      stair; Stamp and Drag. Stumble tests; the legs plan breaking a side.
- [x] **M4 · The shell.** Vents, gnawers roosting and dropping, Shrug and
      Shiver, falls. Vent and fall tests; the crown plan reaching an anchor.
- [x] **M5 · The crown and the clock.** Anchors, halts, phases, the Opening,
      Shed, Plough, the Siege beam, the wall. A full solo hunt runs to an end.
- [ ] **M6 · The aim rule (A3).** Top face of a mountable part is a place;
      Ridgeback pins and report before/after; own commit.
- [ ] **M7 · Two hunters.** Coop and solo plans, report lines, per-region
      windows, tuning passes, feel log.
- [ ] **M8 · Animation, look, reading it, world, docs, browser.** Clips,
      contact sheets, look and dressing, telegraph screenshots, trophy,
      camera, budget, docs, manual, web smoke, merge main, checks, push.
