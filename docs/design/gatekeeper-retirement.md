---
status: decided
decided: 2026-09-09
retires: docs/combat-design/gatekeeper-skills.md
---

# Gatekeeper — retired

The Gatekeeper is cut from the roster and replaced by the [Bulwark](bulwark.md).

## Why

**It has no identity.** It is a collection of individually interesting ideas — gates, a
bow, arcana arrows — that do not add up to a fantasy or a coherent way to play.

**The control scheme is fatal, not just awkward.** Gates require position *and*
orientation, set remotely, then maintained while simultaneously aiming a bow and dodging.
That is three concurrent precision tasks in a third-person camera. No control scheme
rescues it.

**The bow's fantasy and its balance requirement are in direct opposition.** A realistic
bow cannot have an imposed range, but reasonable damage at any range is unbalanceable
against melee classes. The proposed fix — arrow lifespan creating an artificial range —
produces a weapon that arbitrarily stops working, which feels broken to the player.

## What is salvaged, and where it goes

- **The imploding arrow** — fire, reactivate, it pulls everything toward it *including
  the caster*. Mobility and control in one input with real self-risk. Worth rehoming;
  candidates are the Elementalist air kit or the Bulwark.
- **The reactivate-your-projectile pattern** — fire, then make a second decision about the
  thing in flight. This is a genuine mechanical test and already appears in the Blood
  mage's Rend. Promote it to a **design pattern used across classes**, not a class. It is
  already load-bearing for the Bulwark's shield recall.
- **The teleport gate's auto-exit-above behaviour** — an enemy who walks into the entry
  gets spat out overhead. Good trap. Candidate for a single Elementalist structure
  interaction.

Not salvaged: acceleration, reflection, multiplication. Generic, and they duplicate the
Elementalist's shoot-through-your-own-object combo.

## The ranged hole

The Gatekeeper existed to be the ranged physical damage dealer. **The roster does not
backfill that role**, and the hole is not treated as one.

In a closed arena fighter, "nobody has a long-range poke" is a legitimate design choice
rather than a gap — most Smash characters have no real projectile. The Elementalist already
covers mid-range control, so no meaningful role is lost.

A bow as a universal secondary was proposed as a fill and is **parked**, leaning against.
See [parked.md](parked.md) for the reasoning. The secondary slot itself is not parked — the
shield occupies it, and shield-or-nothing is already a real choice.
