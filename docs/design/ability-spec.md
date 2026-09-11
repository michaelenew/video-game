---
status: proposed
decided: 2026-09-09
---

# Ability spec

The shared format every class kit is written in. Exists so six kits written from the same
archive do not diverge into six different vocabularies.

## The unifying idea — the class mechanic is the resource

Under no cooldowns, an ability needs a cost. There are two, and neither is a mana bar:

1. **Frames.** The primary cost, universal. What you pay is the time you are committed and
   vulnerable. This is the Smash-shaped cost and it does most of the balancing work.
2. **The class mechanic.** The secondary cost, different for every class.

| Class | What abilities spend |
| --- | --- |
| Shadow Reaver | Shadow position |
| Elementalist | Structure slots on the field |
| Blood mage | Health |
| Dual mage | Meter position |
| Champion | Rush charge, and which form you are left in |
| Bulwark | Shield position |

**No universal resource bar.** Each class's economy *is* its identity, which means every
ability cost is also a characterisation choice. A Reaver ability that consumes the shadow
is expensive in exactly the way that makes the Reaver the Reaver.

## Fields

Every ability is specified as:

- **Startup** — `instant` · `fast` · `medium` · `slow` · `charged` (hold to extend)
- **Recovery** — `short` · `medium` · `long` · `committed` (no cancel, animation runs out)
- **Range** — `melee` · `short` · `medium` · `long`
- **Mechanic** — what it requires of the class mechanic, and what it does to it. The most
  important field; it is where the class lives.
- **Effect** — what happens.

Deliberately absent: cooldown, mana cost, damage numbers. Frame counts and damage need a
prototype and cannot be reasoned to.

## Kit size

**Six abilities per class**, plus an auto attack and the mechanic input. That is a minimal
playable kit — enough for a real match, few enough to balance and to keep readable in
third person.

The archive's twelve-slot layout is the eventual target, not the prototype target. Slots
seven through twelve are where intra-class specialisation lives later.

Each kit should cover: **mobility**, a **safe poke**, a **committed payoff**, a **control
tool**, and something that is **only that class**.

## Rules that apply to every kit

- **Reactivation is a design pattern, not a class feature.** Fire something, then make a
  second decision about it in flight. Used by the Blood mage's Rend, the Bulwark's shield
  recall, and the Reaver's shadow. Reach for it freely.
- **No hard stops without a hard condition.** Slows, roots that still allow attacking,
  pushes and pulls are the default. True staggers are gated behind reads — a parry, a
  telegraphed commitment. See [defense.md](defense.md).
- **Every class needs an answer to a blocking opponent.** Special attacks are the universal
  guard breaker; a class with none of its own is relying on that.
- **Recovery is the balance knob.** When an ability is too strong, lengthen the tail before
  touching the effect. It preserves the fantasy and moves the fight to the right place.
