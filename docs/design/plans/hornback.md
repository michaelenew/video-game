---
status: action plan — one implementation thread
opened: 2026-10-01
implements: ../creatures/hornback.md
---

# Hornback herd — action plan

> **State, 2026-10-01 (built).** Every milestone below is built; the
> numbers and the decisions to review are the spec's §13. Open: the Blood
> mage's plan (5/24: it does not use her pools), the Dual mage (1/24, the
> known harness gap), the two-hunter crossing (1/12), the Bulwark and the
> Reaver long (160–190 s), and the carter's key (trophies are one per
> creature and temper). `crates/sim/examples/hb_probe.rs` and
> `crates/hunt/examples/` are local scratch tools, uncommitted.

The specification is [`../creatures/hornback.md`](../creatures/hornback.md);
the recipes are [`../critters.md`](../critters.md) §6,
[`../species.md`](../species.md), [`../arenas.md`](../arenas.md) and
[`../hazards.md`](../hazards.md) §7. Parity bar:
[`../monsters.md`](../monsters.md), contract [`../bestiary.md`](../bestiary.md) §1.

## Decisions taken up front

- **Pack-only species** (`Species::pack_only`), two kinds: the cow and the
  bull. The bull is a critter with extra state in `Pack::memo` (§10).
- **Boulders are solid hazards** (`HazardDecl::solid`), raised into the
  `Terrain` like slag: a boulder kind, a cracked kind, and rubble that is not
  solid. Placed by the species' frame hook on the first frame, so the arena
  table stays static data.
- **The stampede is a cow move**: every cow winds it up with the Bellow and
  runs it as one long active move, steered by the species' rules (lane clamp,
  lees, the fallen) -- so the report's per-move counts, the hidden-commit
  check and the threat windows read it like any other hit.
- **Generic hooks added where the machinery was not generic yet** (each
  listed in critters.md when it lands): a guard on a critter, a side for a
  critter's move, species floor signs, and riding a critter.

## Milestones

- [x] **M1 · The meadow and the grazing herd.** Species table, kinds, knobs,
      arena (48 × 40, bank, ford), boulders as hazards, flocking, alarm by
      radius and hit, calm-down. Registry lines. Alarm test, determinism.
- [x] **M2 · The stampede.** Bellow, lane, clamp, lees, return home,
      knockdown, the fallen not trodden. Floor signs. Lane/lee tests, budget.
- [x] **M3 · Paw & charge.** Tracking, lock, the swept stop on solids, the
      stun, cracks, wary, the edge pull-up, stones and shields. Charge tests.
- [x] **M4 · The close game.** Hook, Trample, Shoulder, Guard and horns, cow
      kicks, strain. Their tests.
- [x] **M5 · Riding a cow.** Mountable critter, the buck, grip. Buck test,
      `beastcheck`-style print of a cow's back against every hop.
- [x] **M6 · The brain, the hunter, the report.** Scoring, interpose,
      desperation, the §9 plan and report lines; tuning passes.
- [x] **M7 · The crossing.** The cart (P7), its arena, the escort plan.
- [x] **M8 · Reading it.** Stock poses, horns, telegraph screenshots.
- [x] **M9 · World, docs, browser.** Trophy and tempers; doc "Where it
      landed"; README, manual; web smoke; merge main; checks; push.
