---
status: proposed — ascension still open
decided: 2026-09-09
sources: docs/combat-design/statera-skills.md, docs/combat-design/class-builds.md
depends: ../statera.md
---

# Statera — kit

**Identity.** Melee mage containing two forces. Power comes from riding as close to an edge
as you can while still able to pull back.

Read [../statera.md](../statera.md) first — this kit implements that meter and is meaningless
without it.

## Mechanic — the two-pole meter

- Abilities push the meter toward their side; stronger abilities push harder.
- **Auto attacks pull toward centre.** Always available, melee range only.
- Power scales with distance from centre. Past a depth threshold you take a burn that stops
  the moment you come back inside.
- Four core abilities **transform** with meter position. Two finishers are **gated by depth**.

**Ascension is not in this kit.** It is an open proposal in
[../statera.md](../statera.md) and slots in as a seventh input once resolved.

## Auto attack

Melee, moderate. Pulls the meter toward centre — the return valve, and the reason the class
has to close distance exactly when it is strongest.

**Tempest** (passive): abilities mark enemies on hit. Autoing a marked enemy consumes the
mark for bonus damage and a short burst of movement speed. This is what makes the pull to
centre attractive rather than a chore.

## Core abilities — these transform

Four slots. Each does something different at centre, deep Light, and deep Dark.

### Step
**Startup** fast · **Recovery** short · **Range** short

| Position | Behaviour |
| --- | --- |
| Centre | Short dash in any direction |
| Light | Forward blink; damages and blinds on arrival |
| Dark | Drain-dash; steals health from everything passed through |

### Lance
**Startup** fast · **Recovery** short · **Range** medium

| Position | Behaviour |
| --- | --- |
| Centre | Small fast line skillshot, low damage |
| Light | Detonates at maximum range for burst damage in an area |
| Dark | Tethers the first target hit, draining while the tether holds |

### Sweep
**Startup** medium · **Recovery** medium · **Range** short cone

| Position | Behaviour |
| --- | --- |
| Centre | Pushes everything back |
| Light | Staggers anything carrying a Tempest mark |
| Dark | Slows, and heals you per target caught |

### Divide
**Startup** medium · **Recovery** short · **Range** medium

A dome that rotates into existence and blocks enemy projectiles for two to three seconds.
Hitting it with any other ability dashes you to the impact point, keeping momentum.

| Position | Behaviour |
| --- | --- |
| Centre | Small, brief |
| Light | Larger; damages on the way up |
| Dark | Drains everything caught inside it |

## Finishers — gated by depth

### Judgement — Light, deep only
**Startup** slow, delayed · **Recovery** committed · **Range** medium · **Mechanic** pushes
the meter hard toward Light

A delayed area strike: massive instantaneous damage at the centre, then a wide low-damage
field. While the field lasts, moving through it grants you speed, bonus damage, and
lifesteal. Applies only one Tempest mark regardless of how many it hits.

### Eclipse — Dark, deep only
**Startup** medium · **Recovery** committed · **Range** long, channelled · **Mechanic**
pushes the meter hard toward Dark

A beam that fires for a fixed duration, damaging everything in front of you and applying a
stacking slow. Heals a fraction of the damage dealt.

## Playing it

Push out with casts on one side, land the finisher from the deepest position you can
survive, then close to melee and auto back toward centre. Transiting centre is the
repositioning phase — everything is available in neutral form and nothing threatens much.

Switching sides means crossing the whole bar, so which edge you commit to is a real
strategic choice rather than a moment-to-moment one.

## Open questions

- **Ascension.** The one blocking item. See [../statera.md](../statera.md).
- Are the finishers castable at all from the wrong side, or fully hidden? Fully hidden is
  cleaner to read; partially available is friendlier.
- Do the low-tier abilities need a spam check beyond frame data, given they barely move the
  meter?
- Naming the two forces. The archive's vocabulary (Judgement, Eclipse, Culling, Mark of the
  Merciful, Dark pulse) is worth mining.
