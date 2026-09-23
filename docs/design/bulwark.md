---
status: proposed
decided: 2026-09-09
replaces: docs/archive/combat-design/gatekeeper-skills.md
---

# Bulwark

> ⚠️ **A v2 is proposed, 2026-09-23** — [bulwark-v2.md](bulwark-v2.md). It answers the question
> this document never asked: *why use a shield alone.* Blocking loads the shield; the shield's
> attacks spend it; the planted shield joins the structure system and becomes the wall this
> document promises. Nothing of it is built.

The defensive class. Replaces the retired Gatekeeper in the roster.

## Why the roster needs it

Across the existing six classes, nobody plays defensively as an identity. There is no
wall, no punisher, nothing that wins by making the opponent come to it. Of the archetypes
missing from the set, this is the one worth a class slot:

- It is the archetype that most directly teaches the game's core mechanical test —
  commitment, whiff punishment, and stagger.
- It has a real job in **both** modes. Coop: hold the line, be mobile cover. Versus: win
  neutral by being hard to approach. Most archetypes only work in one mode.
- The stagger system is the most fighting-game-ish mechanic in the notes and currently no
  class is *about* it. A class built on stagger, parry, and armour makes that system earn
  its complexity.
- It is the cheapest class to build. Animation and hitboxes, no projectiles, no spawned
  objects, no persistent field state. This directly serves the art thesis.

## Lore hook

The Champion comes from the old cities' duel tradition — the aristocrat's individual
champion. The new cities had no aristocracy and no duel tradition; they were villages that
banded together and organised collective defence.

**Champion = the hired duellist. Bulwark = the communal defender.** The contrast uses lore
that is already written.

## Committed, not slow

The single most important design rule for this class.

- **Slow** means low movement speed. It loses neutral, never touches anyone, and is the
  classic failure mode of this archetype — in a 3D arena with dodges and mobility
  everywhere, a slow short-range character simply never engages.
- **Committed** means normal movement speed with long active and recovery windows and
  enormous payoff.

The fighting-game heavy is slow-*acting*, not slow-*moving*. Once those are separated the
archetype stops being cursed. Do not give the Bulwark a movement speed penalty.

## The shield is a volume, not a stat

The shield is **a physical space nothing can enter** — not damage reduction. Projectiles
stop. Bodies stop. Elementalist structures already establish that the game has
world-objects that block; the shield is a mobile one the player aims.

This is what solves the approach problem. The Bulwark wins the way a zoner wins — by
denying space and funnelling the opponent into the gap left open — except the space is
*adjacent* to the character rather than far from it. The opponent does not approach the
wall; they get channelled past it into where the committed moves are already aimed.

It also gives an immediate coop job that is not damage: mobile cover for the team.

## The shield has a position independent of the character

This is the structural trick that makes the class deep enough to be a class, rather than
one blocking ability. It is the same mechanism that makes the Shadow Reaver work — two
bodies, and your options are a function of the line between them — but defensive rather
than offensive. That mechanism is already validated in another class.

Three shield states. **Every ability reads differently in each**, which is where the kit
depth comes from:

| State | Character | Shield |
| --- | --- | --- |
| **Held** | Protected, directional, slow to turn | Aimed where you face |
| **Planted / thrown** | Mobile, exposed | Holds space over there |
| **Returning** | Can leap to it | Recall path is a damage line |

## Kit direction

Confirmed elements, to be specified:

- **Downward shield slam** — ground shake, stagger. Committed, high payoff.
- **Thrown shield** — leaves you exposed, holds space remotely, and is the setup for the
  recall.
- **Boomerang recall with leap** — the primary mobility tool and the class's approach
  option. Fits the reactivate-your-projectile pattern used elsewhere in the game.
- **Grapple** — the payoff for a read, not a poke. Long startup, huge reward. In a 3D
  arena with dodges, a command grab is the correct counter to turtling, but only if it is
  a genuine commitment.

### Control lattice, not hard stops

Build a **lattice of soft controls** — slows, roots that still allow acting and attacking,
pushes, pulls. Gate true staggers behind hard conditions such as a successful parry.

Being rooted while still able to fight back is fine to play against. Being stunlocked is
not. The class should feel oppressive without ever taking the opponent's hands off the
controls.

## Forcing engagement

At 60-second versus TTK, if neither player approaches, nothing happens. The Bulwark must
not out-damage anyone at range, which means *it* has to approach — so it needs at least
one committed approach tool. The thrown shield plus leap is that tool.

## Dependencies

- **Block and parry** are proposed in [defense.md](defense.md), which was the blocker on
  specifying this class. It supplies facing-arc blocking, pushback resistance as the
  Bulwark's class trait, parry-gated staggers, and the distinction between the shield as a
  projectile-stopping volume and as a melee guard state.
- **Shared blocking implementation.** The shield blocking projectiles and Elementalist
  structures blocking projectiles should be one system, not two.

## Open questions

- Can the planted shield be destroyed, or only bypassed?
- Does the shield block allies' projectiles in coop? Friendly-fire blocking is a real
  coordination mechanic but can be maddening.
- Weapon pairing — is the Bulwark shield-and-hammer, shield-only, or shield-plus-variable?
