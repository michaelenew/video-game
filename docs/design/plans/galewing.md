---
status: action plan — one implementation thread
opened: 2026-10-01
implements: ../creatures/galewing.md
---

# Galewing — action plan

> **State, 2026-10-01 (in progress).** Branch `claude/creature-galewing`.
> The species, its flight, every move, the wings, the crash, the perch and
> the ride are built and pinned by `crates/sim/tests/galewing.rs`; its clips
> are authored and baked; its hunter and report lines run
> (`cargo run -p hunt --bin fight -- --species galewing`); it is drawn
> (look, the Cliffs, the sun overhead) and the aim and camera changes are
> in. Next: tuning across classes (M10), the docs and the finish (M11). A successor resumes from the
> first unchecked box; the decisions below are binding unless a milestone's
> note says one was revisited.
>
> **Tuning method**: the baked file is the truth. A scratch script reads
> every `// id = value` comment out of
> `crates/sim/src/species/galewing/tuned.rs`, resets it to the empty
> bootstrap, bakes once to learn the ids, and bakes again with `--set` for
> every id plus any override file of `id=value` lines -- so adding an own
> knob (which shifts the move rows' indices) keeps every value. Rewrite it
> from that description if it is gone (the Veilstalker's plan has the same
> note).

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

Twelve own cells (48 words; 38 used), no hazards, noises or objectives:
flight state (position, speed, climb, heading, turn rate), flags, wind,
rest, dwell, the carried fighter, the talon lane and its pivot, the
Downwash's point, the volley's lane and feathers, who the Screech and the
buffet reached, the ride's lap and phase, the clipped glide, what the frame
hook saw for the brain, report counters, the circle it is flying, and the
approach's intended move. `fight::word` lists them. On the body
(`Monster::own`): the two wing bars, the carry's leg damage (or a lobbed
move's aim height), the pitch, and the posture flags with the bank.

## Decided while building

- **The circle drifts over its target** (`flight::drift`): a fixed circle
  round the tower left a hunter inside it never in the line-up arc. Of the
  circles with the target on its rim, eight ways round, the one inside the
  arena and nearest the circle site; drifting at `CircleDrift`.
- **One decision per approach** (`fight::intend`): as the target comes into
  the arc it draws the air move this approach is for; the brain throws it
  when its range fits and its score clears `Patience`. Without it the
  long moves always pre-empted the mid ones, since the arc is entered far.
- **The ground dwell**: after a Stoop it stays on the floor `GroundDwell`
  frames (the buffet and the Screech its price) before it gathers itself.
- **Low** is measured from the plateau (the `circle` site's height), not
  the ground under it: passing over the tower is not low.
- **A bird coming down has no body to shove with**: every part is passable
  through a Stoop's dive and hit (`fight::presence`).
- **Terrain following**: round its circle it keeps `TowerOver` above
  whatever is under it or ahead of it, climbing hard to clear the rock face.

## Milestones

- [x] **M1 · The body.** Species table (bones, parts, moves, clips, own
      knobs), bootstrap files, registry lines, first guesses baked.
- [x] **M2 · The Cliffs.** Arena table: plateau, shelf, stairs, tower with
      eight ledges, four stones, the rock face. Registry lines. Spawn on the
      ground under a mark. `beastcheck` / arena tests.
- [x] **M3 · Flight.** Flight controller (3D, bank-limited turn, climb and
      sink limits, mass), circling, perching, the approach of each air move,
      landing and taking off, the crash fall. Rider yaw carry.
- [x] **M4 · The moves.** Stoop, talon pass and carry, Downwash and lee,
      volley, Screech, buffet, the grounded hop. Signs. Brain: line-up arc,
      wind, place, follow-ups, hurt, lockouts. Move tests.
- [x] **M5 · Wings.** Bars, poise, crash and ramps, one wing / two wings.
      Wing tests.
- [x] **M6 · The sky ride.** Take-off with riders, laps, wingbeat heave,
      roll, swoop, riding a broken wing down. Ride tests.
- [x] **M7 · Aim and camera.** `swing_path` against the surface's up
      (`aim::underfoot_up`); `top_under` below the feet; guard tests
      (`crates/view/tests/galewing.rs`, the banked swing and the lee in
      `crates/sim/tests/galewing.rs`); pins unchanged.
- [x] **M8 · Animation.** (first pass; review in the game) Factory clips, bake, contact sheets.
- [x] **M9 · Drawn.** Look (wing roots pale, cracked wings rust, broken
      dark: a `Tint`), Cliffs dressing, sun overhead (`arenas::sun`),
      SHOT_MOVE screenshots (`fight::ready_for`) looked at. Floor markers
      are drawn on `Terrain::floor_below` their anchor -- they were drawn at
      zero, under the plateau, for every shared telegraph.
- [ ] **M10 · The hunt.** Plans A and B, report lines, tuning passes,
      feel log.
- [ ] **M11 · Finished.** Trophy, tempers, docs (built + where it landed),
      README map, manual, web smoke, merge main, checks, push.

## Open

(Filled in as found.)
