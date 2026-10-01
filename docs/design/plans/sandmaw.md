---
status: action plan — built
opened: 2026-10-01
implements: ../creatures/sandmaw.md
---

# Sandmaw — action plan

The specification is [`../creatures/sandmaw.md`](../creatures/sandmaw.md); the
parity bar is the Ridgeback ([`../monsters.md`](../monsters.md)); the recipes are
[`../species.md`](../species.md), [`../arenas.md`](../arenas.md) and
[`../hazards.md`](../hazards.md). The Mireback ([`mireback.md`](mireback.md)) is
the closest built creature: its swallow (`state::put_inside`, `Part::hollow`)
is what the Sandmaw's hold is built on. This is how to get there, milestone by
milestone, so a successor can pick it up from the checkboxes.

## State

Started and built 2026-10-01 on `claude/creature-sandmaw`. Every milestone
below is done; where it landed, numbers and all, is the creature doc's §13,
and the tuning passes are the feel log's entry of the same day. **Open**: the
threatening band (49 % perceived against 35) and walk-up (7 % against 20) --
the stand counts as threatening because the worm can act at once, which is a
reading of §9 for a person to settle before anybody tunes toward it -- and
beaches once in two minutes rather than once a minute.

## Decisions taken before building (the design reasons are in the creature doc)

- **`Monster::pos` is the head end while buried and the hole while it
  stands.** The swim clip carries the root five and a half metres behind the
  origin (`hips.x`), so the body's collision with rock (`FightDecl::collides`,
  `BodyRadius` 1.2) is the head's, and the tail follows the path the head
  took. The standing clips stand the column on the origin.
- **Buried is a mask, not a static flag**: `FightDecl::presence(m, rig)`
  names the parts with no body this frame -- no hurtbox, not solid, not
  mountable -- and the parts that are solid but no back (a worm that is not
  beached is never ridden); `shown` keeps buried parts from being drawn. The Sandmaw's answer is geometric: a part whose box is wholly
  under the sand is buried. So the 5 m of body still under the floor while it
  stands is buried too, as §1 says.
- **Its posture picks its clip**: `FightDecl::clip` -- buried it plays the swim
  whatever its speed, standing the stand, so a worm circling slowly does not
  pop up out of the sand.
- **Hearing has a multiplier** (`FightDecl::hearing`): hunger (×1.25) and the
  deafness after a breach.
- **The spit and the lash test their own hit** (`MoveDecl::own_hit`): a cone
  and a half-annulus are not cylinders. Each is a union of discs from one
  function that the hit test and the drawing (`marks`) both read, so the
  overlay rule holds. The spit also needs a clear line from the mouth (a
  solid, a stone or a planted shield stops it), which only the world can ask.
- **The swallow's escape reads the held fighter's press**
  (`FightDecl::from_inside`): the species is handed the input of a fighter
  inside one of its hollow parts and returns what they may still do -- here,
  nothing, and the `Q` is judged against the gulp.
- **The swallow's numbers**: 60 on the bite, 20 a gulp at 30, 60, 90, and
  spat out at 120 for 80 more -- the only reading of §2 that makes "80 on the
  first gulp, 220 missed throughout" both true.

## M0 · Seams in the shared code (Ridgeback bit-identical)

- [x] `FightDecl::presence`, `clip`, `hearing`, `from_inside`, `radius`, `steepest`; `MoveDecl::own_hit`; `noise::nth`.
- [x] `MarkLook::{Sand, Fin, Heard, Feel}` and their drawing in `game/src/ground.rs`.
- [x] Pins unchanged: `ridgeback_pin.rs`, `hunt/tests/pin.rs`.

## M1 · The Pan and the table

- [x] Arena `arena/sandmaw.rs`: 36 × 36 sand, 1.5 m rock rim, three rock
      islands 5 m across and 0.5 m high in a 14 m triangle, a boulder on each.
- [x] `species/sandmaw/`: bones (a 14-bone worm: root, five neck, head, two lips, five tail), parts (segments, vents,
      throat, head, tooth ring, gullet), moves, clips, own knobs; bootstrap
      `tuned.rs`/`baked.rs`; register; first-guess knobs with `bake_tuning --set`.
- [x] `nothing_passes_under_rock`.

## M2 · Noise, feel, and what it attends

- [x] Perception: felt (radius, height, on sand), hears; the attended record
      in the lore (kind, where, who, when); heard noises marked for drawing.
- [x] Prowl: circle the last noise, search spiral after `silence_patience`,
      circle an island.
- [x] The three perception tests; `determinism.rs`.

## M3 · Buried, and the rise-bite

- [x] The spine (head trail in the lore), wake, fin, feel disc, heard rings as marks.
- [x] Rise-bite: aim at the noise, kept 1.2 m off rock; travels under during
      the tell; stands for the recovery; shield thrown clear; stone beaches.
- [x] Marker tests; `SHOT_MOVE=rise_bite` screenshot.

## M4 · The rest of the moves

- [x] Breach (trail lead, lane, arc hurtbox, beach on rock / interrupt),
      undertow (sinkhole hazard, rise at its centre), spit (cone, cover, slow),
      lash (half-annulus, crouch ducks), swallow (grab, hold, gulps, escape,
      rescue, gag), sound (vents clamp, riders thrown, ring), dive from a beach.
- [x] Each move's test; frametable.

## M5 · The beach and the ride

- [x] Three routes; the beached body mountable at 2.2 m; vents; the writhe
      as a buck; the tooth ring and its consequence.
- [x] `beastcheck --species sandmaw`; the beach and tooth-ring tests.

## M6 · Animation

- [x] `anim/src/beast/sandmaw/`: swim, stand, every move, flinch, beached,
      dead; bake; contact sheets looked at.

## M7 · The hunter and the report

- [x] `hunt/src/plans/sandmaw.rs`: §9's plan; the tally lines; the new
      threatening/unperceived bands; the creature's own unanswerable rule.
- [x] Twelve+ hunts per class, tune to tier 2, feel-log entries.

## M8 · Finished

- [x] Look and dressing in `game`; screenshots of every telegraph.
- [x] Trophy (the tooth ring), tempers.
- [x] Docs: creature doc built + "Where it landed"; README map; manual.
- [x] Web smoke with `?hunt=sandmaw`.
- [x] Merge `origin/main`, fmt/clippy/test, push.
