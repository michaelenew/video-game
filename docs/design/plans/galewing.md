---
status: action plan — one implementation thread
opened: 2026-10-01
implements: ../creatures/galewing.md
---

# Galewing — action plan

> **State, 2026-10-01 (started).** Branch `claude/creature-galewing`. Nothing
> built yet beyond this plan. A successor resumes from the first unchecked
> box; the decisions below are binding unless a milestone's note says one
> was revisited.

The specification is [`../creatures/galewing.md`](../creatures/galewing.md);
the recipes are [`../species.md`](../species.md), [`../arenas.md`](../arenas.md)
§4 and [`../hazards.md`](../hazards.md) §7. Parity bar:
[`../monsters.md`](../monsters.md), contract [`../bestiary.md`](../bestiary.md) §1.

## Decisions taken up front

- **Flight lives in the species' frame hook**, not in `monster.rs`. The
  shared step still runs (glance, tick, brain, steer, walk), and the hook
  then overwrites the body's position and yaw from its own flight state in
  the lore -- so the shared walk's drift is discarded every frame while the
  bird is aloft. On the ground for good it lets the shared walker drive. The
  bird's bank and pitch are the root bone's roll and pitch, put on by
  `FightDecl::repose`, so the rig, the parts, the riders and the grip test
  all see the banked body with no shared change. A rider's look gets the
  hook's own turn added to `carry_yaw` by the hook (the shared spin only
  sees the turn made inside the step).
- **Moves.** The Stoop is `lobbed` (its circle is the shared telegraph, its
  hit the shared volume). The buffet is a shared cylinder that
  `mirrors_to_target_side`. The talon pass, the Downwash, the volley and
  the Screech are `own_hit`: tested in the frame hook against the same
  shapes its `signs` draw (lane strip with a front, ring with an eye and
  lees, rake lane, the cone as discs -- the Sandmaw's way).
- **The ride cycle, the carry, the perch, the crash** are frame-hook state
  (lore), with `never_chosen` moves where the creature plays a clip for
  them (carry, lift, the roll, the swoop, the perch).
- **Wings**: each wing's break bar and the wing poise live in
  `Monster::own` (the `struck` hook only has the body). Root parts carry a
  x2 hide, so a root hit fills its bar double by construction.
- **The Cliffs** are a 12 m plateau *solid* over a shelf at the floor
  (y = 0): the floor of the game is at zero, so the drop must be a solid
  standing on it. The spawn places bodies on the ground under their marks
  (`ground_under`), which is zero everywhere a mark has ever stood.
- **Falls** use the built rule (free to 9 m, 25 a metre, halved slow). The
  shelf is 12 m down so the edge costs 75, not nothing. galewing.md's
  numbers are rewritten to the rule.
- **Aim (A4)** is one change in `aim.rs`: `swing_path` measures its dead
  zone against the up of the surface underfoot; `+y` on the floor leaves
  everything bit-identical. **`top_under`** gets a ceiling: the camera asks
  for the highest top below the fighter's feet.

## Lore layout

(Filled in as built; `tests/lore.rs` checks it fits.)

## Milestones

- [ ] **M1 · The body.** Species table (bones, parts, moves, clips, own
      knobs), bootstrap files, registry lines, first guesses baked.
- [ ] **M2 · The Cliffs.** Arena table: plateau, shelf, stairs, tower with
      eight ledges, four stones, the rock face. Registry lines. Spawn on the
      ground under a mark. `beastcheck` / arena tests.
- [ ] **M3 · Flight.** Flight controller (3D, bank-limited turn, climb and
      sink limits, mass), circling, perching, the approach of each air move,
      landing and taking off, the crash fall. Rider yaw carry.
- [ ] **M4 · The moves.** Stoop, talon pass and carry, Downwash and lee,
      volley, Screech, buffet, the grounded hop. Signs. Brain: line-up arc,
      wind, place, follow-ups, hurt, lockouts. Move tests.
- [ ] **M5 · Wings.** Bars, poise, crash and ramps, one wing / two wings.
      Wing tests.
- [ ] **M6 · The sky ride.** Take-off with riders, laps, wingbeat heave,
      roll, swoop, riding a broken wing down. Ride tests.
- [ ] **M7 · Aim and camera.** `swing_path` against the surface's up;
      `top_under` below the feet; guard tests; pins unchanged.
- [ ] **M8 · Animation.** Factory clips, bake, contact sheets.
- [ ] **M9 · Drawn.** Look, Cliffs dressing, sun overhead, SHOT_MOVE
      screenshots.
- [ ] **M10 · The hunt.** Plans A and B, report lines, tuning passes,
      feel log.
- [ ] **M11 · Finished.** Trophy, tempers, docs (built + where it landed),
      README map, manual, web smoke, merge main, checks, push.

## Open

(Filled in as found.)
