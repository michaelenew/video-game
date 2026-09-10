---
status: proposed — ascension still open
decided: 2026-09-10
formerly: Statera
sources: docs/combat-design/statera-skills.md, docs/combat-design/class-builds.md
depends: ../dual-mage.md
---

# Dual mage — kit

**Identity.** Melee mage containing two forces. Power comes from riding as close to an edge
as you can while still able to pull back.

Read [../dual-mage.md](../dual-mage.md) first — this kit implements that meter and is
meaningless without it.

## Mechanic — the two-pole meter

- **Every ability has a Light form and a Dark form.** Tap for one, hold for the other.
  Casting a form pushes the meter toward that side; stronger abilities push harder.
- **Power scales continuously with depth.** The same cast is weak at centre and large at
  the edge. Centre is where both forms are available and both are weak.
- **Coming back:** cast the far-side form (slow, weak — you are casting against the grain)
  or auto attack (fast, but melee range only).
- Past a depth threshold you take a burn that stops the moment you come back inside.

**Ascension is not in this kit.** It is an open proposal in
[../dual-mage.md](../dual-mage.md) and slots in as a seventh input once resolved.

## Auto attack

Melee, moderate. Pulls the meter toward centre — the fast return, and the reason the class
has to close distance exactly when it is strongest.

**Tempest** (passive): abilities mark enemies on hit. Autoing a marked enemy consumes the
mark for bonus damage and a short burst of movement speed. This is what makes closing to
centre attractive rather than a chore.

## Core abilities

Four slots. Each has both forms; both are always castable.

### Step
**Startup** fast · **Recovery** short · **Range** short

| Form | Behaviour |
| --- | --- |
| **Light** | Forward blink. Damages on arrival; blinds at depth |
| **Dark** | Drain-dash. Steals health from everything passed through |

### Lance
**Startup** fast · **Recovery** short · **Range** medium

| Form | Behaviour |
| --- | --- |
| **Light** | Line skillshot that detonates at maximum range for area burst |
| **Dark** | Line skillshot that tethers the first target hit, draining while it holds |

### Sweep
**Startup** medium · **Recovery** medium · **Range** short cone

| Form | Behaviour |
| --- | --- |
| **Light** | Pushes back, and staggers anything carrying a Tempest mark |
| **Dark** | Slows, and heals you per target caught |

### Divide
**Startup** medium · **Recovery** short · **Range** medium

A dome that rotates into existence and blocks enemy projectiles for two to three seconds.
Hitting it with any other ability dashes you to the impact point, keeping momentum.

| Form | Behaviour |
| --- | --- |
| **Light** | Damages on the way up; larger with depth |
| **Dark** | Drains everything caught inside it |

## Finishers — gated by depth

These have only one form each, and are available only past a depth threshold on their own
side. They are the reason to leave centre.

### Judgement — Light
**Startup** slow, delayed · **Recovery** committed · **Range** medium · **Mechanic** pushes
hard toward Light

A delayed area strike: massive instantaneous damage at the centre, then a wide low-damage
field. While the field lasts, moving through it grants you speed, bonus damage, and
lifesteal. Applies only one Tempest mark regardless of how many it hits.

### Eclipse — Dark
**Startup** medium · **Recovery** committed · **Range** long, channelled · **Mechanic**
pushes hard toward Dark

A beam that fires for a fixed duration, damaging everything in front of you and applying a
stacking slow. Heals a fraction of the damage dealt.

## Playing it

Commit to one side with repeated casts of that form, land the finisher from the deepest
position you can survive, then close to melee and auto back toward centre — or cast the far
side to bleed back slowly if you cannot close.

Switching sides means crossing the whole bar, so which edge you commit to is a real
strategic choice rather than a moment-to-moment one. Oscillating at centre is always
available and always weak.

## Open questions

- **Ascension.** The one blocking item. See [../dual-mage.md](../dual-mage.md).
- Should the finishers be visibly greyed out from the wrong side, or hidden entirely?
  Greyed is friendlier and teaches the mechanic; hidden is cleaner to read.
- Does Divide's dash-to-impact work with either form of the ability that hits it, or only
  matching forms? Matching-only would be a strong combo constraint worth testing.
- Naming the two forces.
