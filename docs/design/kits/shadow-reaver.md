---
status: proposed
decided: 2026-09-09
sources: docs/archive/combat-design/shadow-reaver-skills.md, docs/archive/combat-design/class-builds.md
---

# Shadow Reaver — kit

**Identity.** Two bodies. Every option is a function of the line between you and your
shadow. Tactical positioning in a second dimension, cashed out in burst.

## Mechanic — the shadow

A second body with a maximum leash. If you leave the leash it dashes to you, lightly
damaging and slowing anything on the way.

**Resolved: mobility is never locked out.** The old design gated every mobility skill on
having a shadow already placed, which loses hard to an opponent who denies the setup.
Fixed by making the baseline dash *create* the shadow — so movement and setup are the same
action, and the class can never be left standing still.

Input map in [../controls.md](../controls.md).

## Auto attack

Melee. The first auto after reclaiming your shadow deals bonus damage scaled by dexterity.

## Abilities

### Shadow dash — mechanic input
**Startup** fast · **Recovery** short · **Range** short · **Mechanic** leaves a shadow at
your origin; replaces an existing shadow

Dash in any direction. Hitting an enemy cancels the remainder. The shadow you leave behind
is the setup for everything else.

### Shadow swap
**Startup** instant · **Recovery** short · **Range** leash · **Mechanic** requires shadow;
keeps it (you and it trade places)

Trade places with your shadow. The core repositioning payoff and the reason placement
matters.

### Reascend
**Startup** medium · **Recovery** medium · **Range** leash · **Mechanic** requires shadow;
consumes it

Slide to your shadow, passing through and damaging enemies on the way, then dash slightly
onward. Can be cancelled into Shadow swap mid-slide.

### Guillotine lotus
**Startup** fast · **Recovery** short · **Range** at the shadow · **Mechanic** requires
shadow; keeps it

Blades erupt from the shadow. They return after a delay, or immediately if you move the
shadow, dealing damage scaled to the target's missing health. The execute tool.

> **Implemented, at the shadow.** It is the one move in the game aimed *at the mechanic*
> rather than at the crosshair or along the body — the player aimed when they placed the
> shadow, and throwing this only cashes that in. See [../aiming.md](../aiming.md).
>
> It briefly erupted **on the Reaver's own body**: the move was declared a swing with a reach
> of zero, so its volume came out where she was standing. Nothing about the ability worked,
> and the overlay showed exactly that. The volume follows the shadow live, so recalling it
> while the blades are out takes them with it.
>
> The delay, the return, and the scaling with missing health are still unbuilt.

### Executioner
**Startup** slow · **Recovery** committed · **Range** short · **Mechanic** stronger with
shadow; consumes it

Blink upward, then slash down in a long arc. With the shadow out, it slashes down too and
dashes forward, adding damage and bleed. The big commitment.

### Deadly mistake
**Startup** fast · **Recovery** long on whiff · **Range** self · **Mechanic** requires
shadow for the teleport

A brief counter stance. Passive: enemies that attack your shadow bleed. If struck during
the stance while your shadow is out, you appear behind the attacker and leave your shadow
where you were. Without a shadow it is only a dodge.

## Playing it

Dash to place, act off the line, cash out with Executioner or Guillotine lotus, reclaim
and re-place. The skill is keeping the shadow somewhere that gives you an escape *and* a
threat at once, which are usually different places.

## Open questions

- Does the shadow have collision, or is it purely a marker? Collision makes it
  denial-able, which cuts both ways.
- Should Shadow swap have any cost at all, or is the leash the only constraint?
