---
status: proposed
decided: 2026-09-10
formerly: Bellator, Shifter
sources: docs/archive/combat-design/shifter-skills.md
depends: ../champion.md
---

# Champion — kit

**Identity.** Range bands and flow. Three weapon forms, and the depth is in changing form
*inside* an animation so the move ends differently than it started.

## Mechanic — forms and the mid-animation swap

Three forms, each changing the auto attack and the tail of every ability:

| Form | Reach | Character |
| --- | --- | --- |
| **Hammer** | Melee, short | Heavy, staggers, slow to recover |
| **Sword** | Melee, medium | Balanced, fastest recovery |
| **Spear** | Melee, long | Thin hitbox, best spacing, weakest per hit |

**Swapping form during a move's active frames changes its ending.** Three forms in, three
forms out — nine endings per ability from two animations each. This is the class, and it is
where the art efficiency comes from.

**Rush** is a stored charge that both dashes and **cancels the recovery of any move**. One
charge. It is the only way out of a committed tail, so spending it is always a real decision.

Input map in [../controls.md](../controls.md).

## Auto attack

Form-dependent, per above. Hammer autos stagger when used out of a Rush. Spear autos aimed at
the ground while moving or rushing let you vault.

## Abilities

### Rush — mechanic input
**Startup** charged · **Recovery** short · **Range** short · **Mechanic** one charge; also
cancels any recovery

Hold to charge, release to dash. A full charge can be banked and spent later. All abilities
are castable while rushing; the dash itself does no damage.

### Drive
**Startup** medium · **Recovery** long · **Range** melee, forward

A committed forward attack. The illustrative case for the swap matrix — see below.

### Uppercut
**Startup** medium · **Recovery** long · **Range** melee

A rising attack that launches. In sword form it throws the weapon upward at the apex, where
it hangs briefly before falling.

> **Implemented** (`Q`). It leaps, and it **takes whoever it catches up with it** — both
> fighters leave the ground. Either half alone is a different move: without the lift it is a
> launcher you cannot follow up on, without the launch it is an escape. The smash-down
> counterpart waits on aerials being their own moves rather than the grounded ones.

### Sweep
**Startup** fast · **Recovery** medium · **Range** melee, wide arc

The safe poke. Horizontal, hits multiple targets, short enough to be thrown out in neutral.

### Throw
**Startup** medium · **Recovery** medium · **Range** medium · **Mechanic** leaves you
formless until it returns

Throw the current form. It travels, then returns, damaging along both legs. While it is out
your autos are unarmed — short, weak, fast. The class's only ranged option and a real cost.

### Brace
**Startup** instant · **Recovery** medium · **Range** self

A short parry stance using the shared defensive system. Successful parries stagger the
attacker and additionally give the Champion a stacking resistance buff, per
[../defense.md](../defense.md).

## The swap matrix — Drive

One ability, nine endings, two animations. Start in the left column, swap to the top row
during active frames.

| Start ↓ / End → | Hammer | Sword | Spear |
| --- | --- | --- | --- |
| **Hammer** | Slam, heavy stagger | Slam into a rising slash | Slam into a low extending thrust |
| **Sword** | Slash into downward slam | Three-hit advancing combo | Slash into a long lunge |
| **Spear** | Thrust into overhead slam | Thrust into a wide cut | Double thrust, maximum reach |

Same-form endings are the linear play and are tuned to be genuinely strong. Cross-form
endings trade reliability for reach, stagger, or speed — and whiff harder when read.

## Playing it

**Linear:** pick the form for the range, throw out the right move, hit confirm. Tuned to be
one of the strongest linear kits in the game.

**Nonlinear:** open in one form for its startup and end in another for its tail — start a
spear thrust for the reach, finish in hammer for the stagger. Bank Rush to escape the tail
when the read was wrong.

## Open questions

- **Is the swap free, or does it cost Rush?** Free is more expressive and leaning that way;
  the whiff is the risk. Costed makes it compete with the cancel, which may be more
  interesting.
- **Window or any time during active frames?** A window is a cleaner mechanical test.
- Three forms or four? Four gives sixteen endings and a steep readability cost.
- The archive's reforge idea — combining weapons at the cost of an accessory slot,
  "quicksilver ___" — sits in parked equipment territory for now.
