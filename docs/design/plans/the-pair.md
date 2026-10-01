---
status: action plan — built
opened: 2026-10-01
implements: ../creatures/the-pair.md
---

# The Pair — action plan

The specification is [`../creatures/the-pair.md`](../creatures/the-pair.md);
the parity bar is the Ridgeback ([`../monsters.md`](../monsters.md)); the
recipes are [`../species.md`](../species.md), [`../arenas.md`](../arenas.md)
and [`../hazards.md`](../hazards.md). The Sandmaw and the Mireback added the
hooks a creature that changes its own fight plugs into; the Pair reuses them
and adds only what two bodies need. This is how to get there, milestone by
milestone, so a successor can pick it up from the checkboxes.

## State

Started and built 2026-10-01 on `claude/creature-the-pair`. Every milestone
below is done; what is open is in the creature doc's §13 ("What is off") and
"Open" at the end of this one.

## Decisions taken before building (the design reasons are in the creature doc)

- **The pair brain lives in the lore, and runs in the species' `frame` hook.**
  `World` gets no `pair` field: the hunt's lore is already "whatever state the
  fight's species keeps", and a frame hook with the whole `World` is where two
  bodies can be looked at together. Each cat's own state (role, scar, the
  rake's hold, enraged) is in `Monster::own`; the shared state (the swap
  clock, the twin pounce, the bond, the first death) in the lore's own cells.
  What a cat's brain hooks need of its partner (where it is, its role, when
  its next hit lands) is written into the lore by the frame hook, so the
  brain's window stays `Quarry` and `Mind`.
- **Two bodies are a species fact**: `FightDecl::bodies` (one for everybody
  else), so `World::hunt_of(.., PAIR)` -- the picker, `?hunt=pair`, the
  harness -- places both. Nothing about one-creature fights moves.
- **A leap is the species' motion.** The pounce, the twin, the perch, the dive
  and the drop are flown by the frame hook along a line from where the cat
  left to its aim point (`Brain::aim`), so the body lands where the marker
  says. Their volumes are `lobbed`: drawn and struck at the aim, and they
  slide along the facing at the move's `Travel` for the skid -- the one
  change to the shared hit volume, zero for every lobbed move there was.
- **Nobody stands on a cat**: a part flag, `sheds`, read by `Rig::resolve` --
  a top face that is a slope, not a floor, as §10 asks.
- **A perch needs a height**: `FightDecl::keeps_height`, so the walk leaves
  the cat's height to its hook and the fence reads it.
- **The enrage and coop numbers are per cat**: `FightDecl::pace` (speed and
  turn multiplier) and `FightDecl::glance` (frames between glances) read
  from the cat's own state.
- **The fight report reads every creature**: per-slot commit state, the
  smaller of the two `frames_until_free`, moves summed across the slots. A
  one-creature fight prints exactly what it did.

## M0 · Seams in the shared code (Ridgeback bit-identical)

- [x] `FightDecl::bodies`, `keeps_height`, `pace`, `glance`; `Part::sheds`;
      a lobbed volume slides at `Travel`.
- [x] The report reads every creature (`hunt/src/report.rs`).
- [x] Pins unchanged: `ridgeback_pin.rs`, `hunt/tests/pin.rs`, and the
      twelve-seed Ridgeback report byte-for-byte.

## M1 · The cat as data, and the Den

- [x] `species/pair/`: the Ridgeback's eighteen bones re-proportioned, parts
      (`sheds`), legs, the move table, clips, own knobs; bootstrap
      `tuned.rs`/`baked.rs`; register; first guesses with `bake_tuning --set`.
- [x] Arena `arena/pair.rs`, the Den: 30 × 30, 1.5 m walls 1.5 m thick, the
      two platforms, three standing stones.
- [x] `nobody_stands_on_a_cat`; `beastcheck --species pair`.

## M2 · Animation

- [x] `anim/src/beast/pair/`: idle, walk, bound, every move, flinch,
      tripped/dazed, crash, dead; bake; contact sheets looked at.

## M3 · One cat

- [x] Pounce (coil, flick at 10, leap, skid), rake (hold drawn at the
      second commit), swat, tail trip (mirrored), perch/dive/drop.
- [x] The solo brain; the per-move answer tests.

## M4 · The pair

- [x] Roles, the swap, the feint, the ambush, `stagger_gap`, the twin pounce
      and its crash; cats against cats.
- [x] The contract tests; `SHOT_MOVE=twin`.

## M5 · What changes it

- [x] Sight (P5) through `aim::sight_clear`, the scar and its habit, the bond
      and the interpose, the howl and the enrage.
- [x] Their tests.

## M6 · Measured

- [x] `hunt/src/plans/pair.rs`: the view (a camera yaw turned at a person's
      rate), §9's plan, the report lines, the marker-on-screen rule.
- [x] Twelve+ hunts per class; tune toward tier 3; feel-log entries.

## M7 · Finished

- [x] Look (dun and dark), the windup glint, screenshots of every telegraph.
- [x] Trophy, tempers.
- [x] Docs: creature doc built + "Where it landed"; species.md; README map;
      manual.
- [x] Web smoke with `?hunt=pair`.
- [x] Merge `origin/main`, fmt/clippy/test, push.

## Open

- **Threatening 78–81 %** against §9's 45: with two animals one is nearly
  always free. Either the measure wants to be per cat for a pair, or the
  cats want longer beats between moves; a person's call.
- **The torn-ear trophy** is not drawn (the hunt's trophy is recorded as
  every creature's is). No audio, so no snarl or roar; the glint stands in.
- **The Blood mage and the Dual mage** lose every hunt to the harness, which
  does not play their pools or bars.
- **Moves landing under one in ten** against the harness's perfect reactions:
  rake, second rake, tail trip, twin pounce, interpose. A person playing it is
  the test.

