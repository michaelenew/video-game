---
status: proposed
decided: 2026-09-09
depends: ../bulwark.md, ../defense.md
---

# Bulwark — kit

**Identity.** The wall. Wins by denying space and funnelling the opponent into where you are
already aimed. Committed, not slow.

Greenfield — no archive material. Read [../bulwark.md](../bulwark.md) for the class rationale
and [../defense.md](../defense.md) for the block system it is built on.

## Mechanic — the shield has its own position

Three states, and every ability reads differently in each:

| State | You | Shield |
| --- | --- | --- |
| **Held** | Protected in a facing arc, slow to turn | With you |
| **Planted** | Mobile, exposed, faster | Holding space where you left it |
| **Returning** | Can leap to it | In flight, damaging along its path |

**Traits:** reduced block stunlock, and a shield wide enough to cover an ally. Carrying it
costs movement speed — which you get back when it is planted.

**Two systems share the object.** As a *world volume* it stops projectiles by collision,
with no timing or state involved — which is why a planted shield keeps blocking while you
are elsewhere. As a *guard state* it handles melee, with stunlock and parry timing. Only the
second requires you to be holding it.

## Auto attack

Short melee with the off hand. Unremarkable on purpose — the class does not win by autoing.

## Abilities

### Guard — mechanic input
**Startup** instant · **Recovery** short · **Range** facing arc · **Mechanic** requires the
shield held

Hold to block. Blocked hits knock you back and stunlock you briefly; the Bulwark's stunlock
is short. **The opening frames are a parry** — timed correctly it staggers the attacker
instead. One input, two outcomes.

### Bash
**Startup** fast · **Recovery** short · **Range** melee · **Mechanic** requires the shield
held

A short forward shield strike. The safe poke, and the natural follow-up to a parry stagger.

### Slam
**Startup** slow · **Recovery** committed · **Range** short area · **Mechanic** requires the
shield held; scales with downward velocity

Drive the shield into the ground. Shakes the ground and staggers everything close. Damage
and stagger strength scale with how fast you were falling — so it rewards using it out of a
jump or a leap recall.

### Throw
**Startup** medium · **Recovery** medium · **Range** medium · **Mechanic** moves the shield
to planted

Hurl the shield. It damages along its path and plants where it lands, continuing to block
projectiles as a volume. You are now faster, exposed, and unable to block.

### Recall
**Startup** fast · **Recovery** short · **Range** to the shield · **Mechanic** requires the
shield planted; returns it to held

The shield flies back, damaging everything along the return path. **Reactivate mid-flight to
leap to it instead**, arriving at its position with the shield in hand.

Recall is the class's mobility and its approach tool — throw to commit, leap to follow. It
also sets up Slam, since arriving from a leap carries downward velocity.

### Grapple
**Startup** slow · **Recovery** committed · **Range** melee · **Mechanic** requires the
shield held

A command grab. Beats blocking outright, loses badly to dodge. Long startup and a punishing
tail — this is the payoff for a read on a turtling opponent, not something to throw out.

## Playing it

Hold space with Guard and Bash, aimed so that the only open approach is the one you want
them to take. Throw and leap to close, land Slam with the leap's velocity, Grapple when they
start blocking instead of moving.

In coop, plant the shield in front of the party as cover, or hold it wide to body-block for
someone who cannot take the hit.

## Open questions

- **Can the planted shield be destroyed, or only bypassed?** Bypassed is simpler and makes
  positioning the only counterplay.
- Does the shield block allied projectiles? A real coordination mechanic in coop and
  potentially maddening.
- Does holding the shield wide for an ally need its own input, or is it just the Guard arc
  being large? Probably the latter — fewer inputs is better.
- Is six enough here? This class has the most state and may need a seventh for a dedicated
  ally-cover stance.
