---
status: action plan — built, waiting on play
opened: 2026-10-01
implements: ../creatures/mireback.md
---

# Mireback — action plan

The specification is [`../creatures/mireback.md`](../creatures/mireback.md); the
parity bar is the Ridgeback ([`../monsters.md`](../monsters.md)); the recipes are
[`../species.md`](../species.md), [`../arenas.md`](../arenas.md) and
[`../hazards.md`](../hazards.md). This is how to get there, milestone by
milestone, so a successor can pick it up from the checkboxes.

## State

Started and built 2026-10-01. Every milestone below is done but the coat
sheen (M5); where it landed, with the numbers, is
[`../creatures/mireback.md`](../creatures/mireback.md) §13, and the passes
are in [`../feel-log.md`](../feel-log.md). What is open is a person's: the
questions in §12 and §13 of the creature document.

To pick it up: `cargo run -p hunt --bin fight -- --species mireback --class
<c> --repeats 24` is the measure (`--gamble` for `swallow_greed`);
`cargo run -p sim --bin beastcheck -- --species mireback` the body;
`SHOT_MOVE=<move> GAME_ARGS="--hunt mireback" ./scripts/screenshot.sh` a
telegraph. Every Mireback seam in the shared code is `None` or off for every
other species, and both pins are unchanged.

## M0 · Seams in the shared code (Ridgeback bit-identical)

- [x] `Monster::step_in` takes the hunt's `Lore`; `FightDecl` gains brain hooks:
      `appetite` (the species' own scoring terms), `prowl_to` (where it walks
      when it is not walking at you), `commit` (what it decides as a move starts).
- [x] `MoveDecl::lobbed` and `Brain::aim` (a point in centimetres): a move whose
      volume lands at a point chosen at commit (the Spew, the flop), and a move
      whose travel stops at the mark (the tongue, at the first solid).
- [x] Body hooks: `FightDecl::hide` (a multiplier on a part from the creature's
      own state: the coat) and `struck` (the species takes the frame after a hit:
      sac poise, the stomach, the wallow broken); `Monster::own`, four words on
      the body, hashed only when not zero.
- [x] `FightDecl::landed`: a creature's move landing on a fighter, with the
      whole `World` (the tongue's grab, the spat-out swallow, the Backwash's tar).
- [x] `Part::hollow`: a part you are *inside*, mounted only by being put there,
      walls that keep you in, no jump, no buck (the stomach).
- [x] Fire spreads *after* `Spread` frames, not on the first one; a strand that
      burns lights what touches its line (the coals' lane).
- [x] `FightDecl::marks`: what a species draws on the floor besides its
      hazards and telegraph (the flop's ring, the belch's pools) and the
      objects it owns (braziers, as standing marks -- no separate `props`),
      read by the renderer from the snapshot.
- [x] Startup/active/recovery ranges to 240 (the Wallow's 150 active).
- [x] A species tally in the fight report (its own lines).
- [x] Pins: `ridgeback_pin.rs`, `hunt/tests/pin.rs`, fight report unchanged.

## M1 · The species and its floor

- [x] `species/mireback/`: bones (a toad), parts (rim at 5.2, crown 7.0, four
      warts, throat sac, hollow stomach), legs, moves, clips, own knobs.
- [x] Hazard kinds: tar, burning tar, slag, coals. Frame hook: tar's 20-frame
      tail, tarred, pools merging (areas add, cap 4.5 m), list full merges into
      the nearest, slag cap 6.
- [x] Bootstrap `tuned.rs`/`baked.rs`; register; `bake_tuning --set` first guesses.
- [x] Arena `arena/mireback.rs`: 36 × 36 m peat, 1.5 m banks, four plinths with
      braziers; registered; dressing in `game/src/arenas/mireback.rs`.
- [x] Tests: `tar_slows_the_walk_and_halves_the_dodge_but_keeps_its_invulnerability`,
      `the_hazard_list_never_holds_more_than_sixteen`, `what_is_drawn_on_the_floor_is_the_hazard_list`.

## M2 · Fire, slag, braziers, the coat

- [x] Braziers: tipped by any hit (fighter or creature), coals lane 5 × 2 m,
      relight after `brazier_relight`.
- [x] Self-burn by footprint (up to three pools), coat loss while burning.
- [x] Slag: burnt pool → mound; tar on slag + fire → second layer; flop shatters.
- [x] Tests: `fire_runs_along_tar_that_touches_and_stops_where_it_does_not`,
      `a_burnt_pool_leaves_one_slag_mound_at_its_centre`,
      `slag_burnt_again_is_a_step_the_bulwark_can_climb_from`,
      `the_mireback_burns_in_its_own_tar`.

## M3 · The body and the moves

- [x] `beastcheck --species mireback` routes table beside §1.
- [x] Spew (lobbed glob, pool; two in the tide), Backwash (tarred), Belly flop
      (leap to the mark, crash, ring of pools, shatters slag), Tongue (line that
      stops at a solid) and Swallowed (hollow stomach, acid, x2.5, retch at 900
      or 120 frames, spat 8 m), Inflate (shove, sac poise → winded),
      Flint belch (marks pools, ignites them, cone; backfire), Wallow (coat back,
      interrupt → stagger, lit → gutted).
- [x] Warts: four breakable, x2, each burst takes a quarter of its tar.
- [x] The brain: no gallop, walks to its tar's centroid; floor, kindle, coat,
      crowd, flee terms; the tide under three tenths.
- [x] Tests: the rest of §10's list.

## M4 · Animation

- [x] `anim/src/beast/mireback/`: idle, walk, every move, flinch, winded
      (stumble), gutted (topple), dead; bake; contact sheets looked at.

## M5 · Reading it

- [x] Look (`game/src/species/mireback.rs`): hide, warts lit, burst warts dark.
- [ ] Coat sheen and a swollen sac: the renderer has one material per
      paint, not one per creature state; not built.
- [x] Floor marks: the flop's ring, the belch's pools, braziers and coals.
- [x] `SHOT_MOVE=<move>` screenshots of every telegraph, looked at.

## M6 · Measured

- [x] `hunt/src/plans/mireback.rs` (plan + `swallow_greed`), report lines (§9).
- [x] Fight across classes and twelve seeds; tune to tier 2; feel-log entries.
- [x] Trophy on a win; tempers.

## M7 · Finished

- [x] Docs: creature doc built + "Where it landed"; README map; manual tables.
- [x] Web smoke with `?hunt=mireback`, screenshot looked at.
- [x] Merge `origin/main` (the Gnawers, one `Tally` for both), re-run
      fmt/clippy/test, push.
