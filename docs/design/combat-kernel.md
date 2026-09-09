---
status: decided
decided: 2026-09-09
supersedes: partially supersedes docs/combat-design/*
---

# Combat kernel

The rules every class is designed against. Where an older document under
`docs/combat-design/` conflicts with this one, this one wins — those documents are
revisions written against the old system and are kept as source material, not as spec.

## Frame

- **Structure** — peer-to-peer, isolated battle arena. No open world at launch.
- **Modes** — coop against monsters, or versus.
- **Camera** — 3D, third person. Ability spectacle is an explicit design goal: abilities
  should be visually rewarding to use, and the camera exists to show them off.
- **Mechanical test** — Smash, not Tekken. Spatial and temporal commitment, whiff
  punishment, reads, and space control. Not input strings or execution chains.
- **Art thesis** — the closed arena means far less art, writing, and content than an
  open world. Design choices that add depth without adding assets are preferred, and
  this is a real tiebreaker, not a nicety.

## No cooldowns

Abilities have **no cooldowns**. They are balanced by:

1. **Frame data** — startup, active, and recovery windows. The cost of throwing out a
   move is the time you are committed to it and vulnerable during it.
2. **Resource cost** — where a class has a resource.
3. **Positional commitment** — where the move puts you, and what it gives up.

Rationale: cooldowns make the mechanical test *"did I have it available"*, which is an
RPG question. Frame data makes it *"was that the right thing to throw out right now, and
can I survive the recovery"*, which is a fighting-game question. Cooldowns also fight the
third-person camera — the player should be watching the fight, not a row of icons.

## Time to kill

| Mode | TTK |
| --- | --- |
| Versus | ~60 seconds |
| Coop | 1–20 minutes, by fight difficulty |

Consequences that flow from the 60-second versus number:

- A 2–3 second burst window is roughly 5% of a match. Burst states can be short and
  still be decisive.
- Health-as-resource classes (Blood mage, Statera) are tunable now that TTK is fixed.
  They were previously blocked on this number.
- Sustain and healing must be small. In a 60-second match, meaningful healing trivially
  becomes a stall strategy.

## What the no-cooldown decision breaks

The following exist in the current documents and are expressed in cooldown terms. Each
needs re-expression before it can be used.

- **The Affinity stat** — currently defined as "increases cdr". With no cooldowns it has
  no meaning. Proposed replacement: **Affinity reduces ability recovery frames**. That is
  the direct analogue — it was always "how soon can I act again" — and it keeps the stat
  meaningful without reintroducing cooldowns.
- **The entire Statera resource system** — human/balanced/divine are defined as 20% /
  40% / 95% CDR. Fully reworked in [statera.md](statera.md).
- **Weapon enchantments** — "Taking damage reduces all cooldowns based on the percentage
  of your current health lost" (Evolution 2) and "Ravenous ... grants cooldown reduction"
  (Evolution 3). Both need new effects.
- **Blood mage seals** — "using a seal incurs a universal cooldown on all seals" survives
  in spirit as a **shared lockout window** between the seals, which is a resource
  constraint rather than a per-ability cooldown. Keep the constraint, rename the concept.
- **Shadow Reaver** — "Shadow swap ... goes on ¼ of normal cooldown" and "Rifen ... long
  cooldown" need recosting as frame data and shadow-state constraints.

## Open kernel questions

These block class work and should be settled next.

### Block and parry — unspecified

There is no block or parry system defined anywhere in the notes. The Shifter has
"raise shield, scroll back to brace" and "parrying gives a stacking increased resistance
buff"; nothing else exists.

This is now a kernel decision, not a class detail, because **two** classes depend on it
(Shifter and Bulwark). Needs answers to:

- What does blocking cost? Is there chip damage?
- Is there a parry window with a reward, or only damage reduction?
- Can blocking be broken, and by what?
- Does blocking work against melee, projectiles, or both?

### Other open items

- **Arena size and shape.** Directly determines whether a committed, space-denying class
  can ever corner anyone.
- **Stocks or a single health pool** in versus.
- **The bow as a universal secondary.** See [gatekeeper-retirement.md](gatekeeper-retirement.md).
  Decided in principle, unspecified in detail.
- **Forced engagement in versus.** At 60-second TTK, two players who both decline to
  approach produce nothing. Whether the answer is a shrinking arena, an objective, or
  purely class-level pressure is undecided.
