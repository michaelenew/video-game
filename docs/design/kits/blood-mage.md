---
status: proposed
decided: 2026-09-09
sources: docs/archive/combat-design/blood-mage-skills.md, docs/archive/combat-design/class-builds.md
---

# Blood mage — kit

**Identity.** Sustain through aggression. Everything costs health and the good outcomes give
it back, so the class is always spending itself forward.

## Mechanic — health as resource

Abilities cost health. Landing them returns it. Missing is the punishment; there is no
separate resource to run dry.

**This class is downstream of TTK.** At ~60 second versus TTK, self-damage classes are
either broken or unplayable depending on how much of your health bar a cast represents. The
numbers cannot be set from the design; they come out of the prototype. Flagged rather than
guessed.

Naturally deals increased damage to disabled enemies — carried forward from the archive as
the class's damage identity.

Input map in [../controls.md](../controls.md).

## Auto attack

Short-range melee with small lifesteal. The safe way to stay topped up between commitments.

## Abilities

### Rend — mechanic input
**Startup** fast · **Recovery** short · **Range** medium · **Mechanic** low cost; heals if
you are in the area on collapse

Fire a small fast projectile that passes through enemies. Reactivate and it slows, grows
rapidly, slowing and damaging in its area, then shrinks back after a few seconds. If you
are inside when it collapses, you are healed.

The reactivation is the class's signature decision — where you put yourself relative to your
own projectile.

### Black spike
**Startup** slow · **Recovery** medium · **Range** medium · **Mechanic** medium cost;
returns health scaled to damage dealt

A spike erupts at the target area after a delay, damaging and slowing. Enemies nearby are
tethered to it, draining health per second for several seconds. Tethers can be broken by
leaving. Returns health and a burst of movement speed when it expires or all tethers break.

> **Implemented** (shift + click). The field drains on a tick and slows anyone inside it. The
> slow is the part that matters: damage alone makes a puddle you step out of, and the slow is
> what makes leaving cost time — which is what turns it into something you put *between*
> yourself and someone else. The tether-break payout is not in yet.

### Cripple
**Startup** fast · **Recovery** short · **Range** short · **Mechanic** low cost; no return

A heavy slow on everything close, decaying over a couple of seconds. Deals no damage, but
grants you movement speed scaled by how many it caught. The disengage and the setup.

### Affliction
**Startup** medium · **Recovery** medium · **Range** medium · **Mechanic** high cost; no
direct return

A slow projectile, reactivate to detonate or it detonates at maximum range. Detonation
marks everything in the area. Marked enemies take escalating damage over four seconds, then
the mark detonates for a flat amount plus a fraction of everything they took while marked.
Enemies that die while marked spread it.

The coop payoff ability, and deliberately weak in a duel.

### Reaper's debt
**Startup** charged · **Recovery** committed · **Range** short cone · **Mechanic** cost
scales with channel; large return on hit

Channel in a fixed direction; the longer you hold, the wider the cone. Releases for high
damage to everything inside. You cannot turn while channelling — the commitment is
directional as well as temporal.

### Seal of the unforgiven
**Startup** instant · **Recovery** medium · **Range** self, then short · **Mechanic** no
health cost; locks the other seals

Channel, immobile, up to three seconds. Take 60% reduced damage and store everything
blocked. On release, discharge all of it to everything nearby.

The defensive option that is also the payoff — the more you eat, the more you deal. Using
any seal locks the others for a window, which is the archive's universal seal cooldown
re-expressed as a shared resource lockout.

## Playing it

Open with Cripple or Rend to create the safe window, commit to Black spike or Reaper's debt
for the health swing, and use the Seal to turn incoming pressure into damage. You are
always a little below full health on purpose.

## Open questions

- Health cost as a flat amount or a percentage? Percentage is self-balancing but makes the
  class stronger the healthier it is, which inverts the comeback fantasy.
- Does the class have any guard breaker of its own, or does it rely on special attacks?
  Reaper's debt is the natural candidate.
- Only one seal is specified. The archive has three; the other two are power-level knobs and
  can wait.
