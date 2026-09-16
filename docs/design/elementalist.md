---
status: decided
decided: 2026-09-09
extends: docs/archive/combat-design/elementalist-skills.md
---

# Elementalist

The class is largely complete. This document records the one structural decision that was
outstanding, which only mattered in versus.

## Structure interaction in versus — decided

Structures are a **contested resource**, not private property.

| Enemy players can | Enemy players cannot |
| --- | --- |
| Use structures as cover | Combo through them as effectively as the Elementalist |
| Destroy them | |
| Displace them short distances with attacks | |

⚠️ **Only the first row is built.** A stone stops bodies and shots, the Elementalist's own
included, so cover works and works for everybody. Nothing any *other* class throws touches a
stone: destroying one is Cataclysm and displacing one is her beam, both hers. Until some other
class can move or break a structure, "contested, not owned" is a ruling with nothing behind
it, and the built behaviour is closer to "free cover for the opponent" — which is the
weakness this section argues for, arriving without the counterplay half.

## Why this is the right ruling

**The Elementalist's own power generation hands the opponent tools.** Every structure
created is also cover the enemy can use. That is a built-in weakness that costs nothing in
stats — far healthier than a damage nerf, because it scales naturally with how much the
Elementalist is doing.

It also makes the real tuning knobs **structure durability** and **displacement force**,
rather than ability damage. Those are much finer instruments.

## Consequences

- In coop this decision is inert; monsters need not interact with structures at all
  beyond collision.
- Structure count on the field becomes a versus-legibility concern. A cluttered arena is
  hard to read in third person.
- **Shared blocking implementation.** Structures blocking projectiles and the Bulwark's
  shield blocking projectiles should be one system. See [bulwark.md](bulwark.md).
  **Half built:** a stone stops bodies and shots; a shield stops nothing, thrown or planted.
  So the shared system exists and the shield has not been wired into it.

## Remaining work

Edge-level only. The class fantasy and core mechanics are sound.

- Recost all abilities for no cooldowns. See [combat-kernel.md](combat-kernel.md).
- Decide whether the salvaged Gatekeeper imploding arrow and teleport-gate behaviour land
  here. See [gatekeeper-retirement.md](gatekeeper-retirement.md).
- Resolve the two overlapping versions of the fire/ice/earth kits between
  `elementalist-skills.md` and the Elementalist section of `class-builds.md`.
