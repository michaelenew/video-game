---
status: action plan — in progress
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

Started 2026-10-01. Surveyed the foundation: the hazard list, the lore region,
`Terrain` (slag as solids), fire from `World::fires`, the `frame` hook and the
sentinel are all there, so the doc's M1/M2 are mostly *using* P4 rather than
building it. What the foundation does not have, and the Mireback needs, is a
handful of seams in the shared creature code (below, M0). Each is a hook that
is `None` for every other species, so the Ridgeback stays bit-identical.

## M0 · Seams in the shared code (Ridgeback bit-identical)

- [ ] `Monster::step_in` takes the hunt's `Lore`; `FightDecl` gains brain hooks:
      `appetite` (the species' own scoring terms), `prowl_to` (where it walks
      when it is not walking at you), `commit` (what it decides as a move starts).
- [ ] `MoveDecl::lobbed` and `Brain::aim` (a point in centimetres): a move whose
      volume lands at a point chosen at commit (the Spew, the flop), and a move
      whose travel stops at the mark (the tongue, at the first solid).
- [ ] Body hooks: `FightDecl::hide` (a multiplier on a part from the creature's
      own state: the coat) and `struck` (the species takes the frame after a hit:
      sac poise, the stomach, the wallow broken); `Monster::own`, four words on
      the body, hashed only when not zero.
- [ ] `FightDecl::landed`: a creature's move landing on a fighter, with the
      whole `World` (the tongue's grab, the spat-out swallow, the Backwash's tar).
- [ ] `Part::hollow`: a part you are *inside*, mounted only by being put there,
      walls that keep you in, no jump, no buck (the stomach).
- [ ] Fire spreads *after* `Spread` frames, not on the first one; a strand that
      burns lights what touches its line (the coals' lane).
- [ ] `FightDecl::marks` and `props`: what a species draws on the floor besides
      its hazards and telegraph (the flop's ring, the belch's pools) and the
      objects it owns (braziers), read by the renderer and the overlay.
- [ ] Startup/active/recovery ranges to 240 (the Wallow's 150 active).
- [ ] A species tally in the fight report (its own lines).
- [ ] Pins: `ridgeback_pin.rs`, `hunt/tests/pin.rs`, fight report unchanged.

## M1 · The species and its floor

- [ ] `species/mireback/`: bones (a toad), parts (rim at 5.2, crown 7.0, four
      warts, throat sac, hollow stomach), legs, moves, clips, own knobs.
- [ ] Hazard kinds: tar, burning tar, slag, coals. Frame hook: tar's 20-frame
      tail, tarred, pools merging (areas add, cap 4.5 m), list full merges into
      the nearest, slag cap 6.
- [ ] Bootstrap `tuned.rs`/`baked.rs`; register; `bake_tuning --set` first guesses.
- [ ] Arena `arena/mireback.rs`: 36 × 36 m peat, 1.5 m banks, four plinths with
      braziers; registered; dressing in `game/src/arenas/mireback.rs`.
- [ ] Tests: `tar_slows_the_walk_and_halves_the_dodge_but_keeps_its_invulnerability`,
      `the_hazard_list_never_holds_more_than_sixteen`, `what_is_drawn_on_the_floor_is_the_hazard_list`.

## M2 · Fire, slag, braziers, the coat

- [ ] Braziers: tipped by any hit (fighter or creature), coals lane 5 × 2 m,
      relight after `brazier_relight`.
- [ ] Self-burn by footprint (up to three pools), coat loss while burning.
- [ ] Slag: burnt pool → mound; tar on slag + fire → second layer; flop shatters.
- [ ] Tests: `fire_runs_along_tar_that_touches_and_stops_where_it_does_not`,
      `a_burnt_pool_leaves_one_slag_mound_at_its_centre`,
      `slag_burnt_again_is_a_step_the_bulwark_can_climb_from`,
      `the_mireback_burns_in_its_own_tar`.

## M3 · The body and the moves

- [ ] `beastcheck --species mireback` routes table beside §1.
- [ ] Spew (lobbed glob, pool; two in the tide), Backwash (tarred), Belly flop
      (leap to the mark, crash, ring of pools, shatters slag), Tongue (line that
      stops at a solid) and Swallowed (hollow stomach, acid, x2.5, retch at 900
      or 120 frames, spat 8 m), Inflate (shove, sac poise → winded),
      Flint belch (marks pools, ignites them, cone; backfire), Wallow (coat back,
      interrupt → stagger, lit → gutted).
- [ ] Warts: four breakable, x2, each burst takes a quarter of its tar.
- [ ] The brain: no gallop, walks to its tar's centroid; floor, kindle, coat,
      crowd, flee terms; the tide under three tenths.
- [ ] Tests: the rest of §10's list.

## M4 · Animation

- [ ] `anim/src/beast/mireback/`: idle, walk, every move, flinch, winded
      (stumble), gutted (topple), dead; bake; contact sheets looked at.

## M5 · Reading it

- [ ] Look (`game/src/species/mireback.rs`): hide, coat sheen, warts, sac.
- [ ] Floor marks: the flop's ring, the belch's pools, braziers and coals.
- [ ] `SHOT_MOVE=<move>` screenshots of every telegraph, looked at.

## M6 · Measured

- [ ] `hunt/src/plans/mireback.rs` (plan + `swallow_greed`), report lines (§9).
- [ ] Fight across classes and twelve seeds; tune to tier 2; feel-log entries.
- [ ] Trophy on a win; tempers.

## M7 · Finished

- [ ] Docs: creature doc built + "Where it landed"; README map; manual tables.
- [ ] Web smoke with `?hunt=mireback`, screenshot looked at.
- [ ] Merge `origin/main`, re-run fmt/clippy/test, push.
