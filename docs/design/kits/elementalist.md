---
status: proposed
decided: 2026-09-09
sources: docs/archive/combat-design/elementalist-skills.md, docs/archive/combat-design/class-builds.md
---

# Elementalist — kit

**Identity.** Terrain author. You build the battlefield, then combo through what you built.
Ranged control that creates its own targets.

## Mechanic — structures

Physical objects you spawn. Every ability has a second behaviour when it hits one.

- **Cap of three on the field.** Spawning a fourth collapses the oldest. That cap is the
  resource — structures are spent by being consumed in combos, and hoarding them costs you
  new ones.

> **Implemented** (`E`). Structures are objects standing in the arena with a lifetime, not a
> list of coordinates. The cap holds, and the clock is a second cost: hoarding them loses them
> anyway.
- **Contested, not owned.** Enemies can use them as cover, destroy them, and displace them
  short distances with attacks, but cannot combo through them nearly as well. See
  [../elementalist.md](../elementalist.md).

The two overlapping kit versions in the archive are reconciled here; where they disagreed
this kit takes the `elementalist-skills.md` version, which is the later document.

## Element loadout

**Earth is always equipped** — it is what generates structures. A second element slots
alongside it. This kit specifies **earth plus fire**; ice, air, and lightning are the
specialisation axis for later.

Input map in [../controls.md](../controls.md).

## Auto attack

Ranged bolt, low damage, no structure interaction. A poke, not a win condition.

## Abilities

### Raise — mechanic input
**Startup** fast · **Recovery** short · **Range** short · **Mechanic** spawns a structure

Spawn a structure at the cursor. Cast beneath yourself to launch into the air. Cast at an
existing uncaptured structure to kick it forward through the ground for low damage.

### Fissure
**Startup** medium · **Recovery** medium · **Range** long · **Mechanic** spawns a structure
at the point of impact

A skillshot that races forward through the ground and stops at the first enemy hit,
staggering them. Leaves a slowing field along its path for several seconds.

### Quake
**Startup** slow, telegraphed · **Recovery** medium · **Range** medium · **Mechanic** spawns
a structure at the centre

A small area shakes immediately, staggering anything moving through it, then erupts after a
delay for moderate damage. The telegraph is the point — it is an area denial tool that
punishes movement, not a damage spell.

### Fire pillar
**Startup** medium · **Recovery** medium · **Range** medium · **Mechanic** on a structure,
detonates it for a much wider blast

A focused pillar of flame with a small staggering core and a moderate surrounding area.
The bread-and-butter structure detonation.

> **Implemented** (`Q`). The pillar is two volumes rather than one: it starts narrow and
> short, then the **base spreads out** while the **column reaches up** and widens only
> slightly. The base is what catches someone walking past it; the column is what stops them
> jumping over. It burns everyone but the Elementalist, on a tick rather than every frame, and
> it stands long after her recovery frames are over.

### Flame spitter
**Startup** fast · **Recovery** medium · **Range** medium, channelled · **Mechanic** on a
structure, melts it into a lasting magma field that damages and slows

Channelled flame toward the cursor. Damage increases further from the caster, so the
spacing is inverted from most channels — you want them at the tip.

### Ice blast
**Startup** fast · **Recovery** short · **Range** short cone · **Mechanic** launches
structures it hits as projectiles

A quick cone. Structures caught in it are knocked forward, dealing extra damage and
staggering whatever they hit. This is the class's answer to a blocking opponent — the
structure is the guard breaker.

## Playing it

Raise or Fissure to seed the field, then read the opponent's position and detonate the
structure that catches them. Structures are simultaneously your damage, your cover, and
their cover — the skill is placing them where they serve you more than the opponent.

## Open questions

- Do structures block your own projectiles? Almost certainly yes, and that self-obstruction
  is a real cost worth keeping.
- Structure durability and displacement force are the tuning knobs, per
  [../elementalist.md](../elementalist.md). Both need a prototype.
- Three may be the wrong cap. It is the number that keeps the arena readable in third
  person, which matters more than the combo ceiling.
