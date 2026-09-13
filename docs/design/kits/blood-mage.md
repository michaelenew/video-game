---
status: proposed
decided: 2026-09-09
revised: 2026-09-12
sources: docs/archive/combat-design/blood-mage-skills.md, docs/archive/combat-design/class-builds.md, docs/archive/combat-design/new-system.md
---

# Blood mage — kit

**Identity.** Sustain through aggression. Everything costs health and the good outcomes give
it back, so the class is always spending itself forward.

> **Revised 2026-09-12.** The implemented kit changed shape. Black spike moved off shift +
> click onto `E`; the auto became the archive's returning blade; Reaper's debt was cut and
> replaced by Grasp; Rend took the committed slot the spike vacated; and **every ability now
> has a health cost in the move table**, which is the first time the class's own mechanic has
> existed in the simulation rather than only in this document. See
> [Implemented](#implemented) for exactly what is in the game.

## Mechanic — health as resource

Abilities cost health. Landing them returns it. Missing is the punishment; there is no
separate resource to run dry.

Two numbers per ability say it, and both live in the move table beside the damage:

- **Cost** — health spent the moment the button goes down, before anything has happened. It
  can never kill you: self-damage clamps at one, the same rule the Dual mage's meter burn
  already follows.
- **Leech** — a percentage of the damage dealt that comes back, wherever the damage happens.
  A direct hit pays immediately; a field pays on every tick; a thrown blade pays when it is
  caught.

**Cost is paid on the press and the return is earned on the hit**, which is the whole design
in one sentence. A Blood mage who whiffs is worse off than when she started.

**The auto is the only one that reliably profits.** Every committed ability is close to break
even at its best and a straight loss at its worst, so the shape of a healthy Blood mage's
match is: spend on a commitment, then earn it back with autos. That is the same role the
melee auto had in the first draft of this document; the ability changed and the job did not.

**This class is downstream of TTK.** At ~60 seconds versus TTK, self-damage classes are
either broken or unplayable depending on how much of your health bar a cast represents. The
numbers are a first pass out of the prototype rather than a derivation, and
`cargo run -p sim --bin frametable` prints what they currently are.

### Naturally deals increased damage to disabled enemies

The class's damage identity, carried forward from the archive, and **implemented**: everything
a Blood mage does is multiplied by 1.4 against something that cannot move.

**Disabled means rooted, staggered, held, or a creature on its side.** Not hitstun, and that
exclusion is the whole of the definition — hitstun happens on every hit anybody lands, so
counting it would turn the trait into "increased damage from the second hit onward", which is
a flat damage bonus in a costume. What is on the list is what
[../ability-spec.md](../ability-spec.md) calls a hard stop, and the design only allows those
behind a hard condition: a parry for the stagger, a grab for the hold, every arm of a Grasp
for the root, a broken poise bar for the topple. Each one was *earned*, which is what a payoff
should be waiting on. Blockstun is not on the list: they blocked, which was correct, and
paying the attacker for it would make guarding worse than standing still.

It applies to every way she deals damage — a swing, a blade in the air, an arm of a Grasp, a
field ticking, and all of the same against the creature. A trait that applied to three of a
class's four abilities would not be a trait, it would be a bug somebody finds in a match.

**This is what makes Grasp a setup.** Four arms is expensive and the root is short; without
something waiting on the other side of it, landing all four would be a small reward for a hard
read. With it, the root is a window you spend — and because leech is a percentage of damage
dealt, the bonus compounds into the health you get back.

The archive also makes lifesteal the class's *weapon* identity — "anything a blood mage
reforges becomes a reaping ___ and has lifesteal" — which is the same loop one layer down and
is a reason to be confident the leech percentages belong in the move table rather than being a
property of a handful of abilities.

Input map in [../controls.md](../controls.md).

## Implemented

Four abilities, and they are the four buttons the control scheme gives a class:

| Key | Ability | What it does |
| --- | --- | --- |
| `LMB` | **Bloodletter** | The auto. A blade out and back, cutting on both passes |
| `Shift+LMB` | **Rend** | The committed melee. A raking claw at chest range |
| `Q` | **Grasp** | Four arms out in a cone that converge. All four roots |
| `E` | **Black spike** | A spike in a draining, slowing field, placed at long range |

### Why Black spike is on `E`

`E` is the class mechanic, and on five of the six classes that is an instant change of state
— throw the shield, cycle the form, place the shadow, raise a structure. The Blood mage's
mechanic is *health*, which is not a thing you press a button to change, so her `E` did
nothing at all. Putting the spike there costs nothing and buys the class a fourth ability;
shift + click, which means "the committed version of your attack" on every other class, goes
back to meaning that here.

This is the first ability in the game on the mechanic slot, so the move table grew a fourth
column. Most classes leave it empty — see `moves::NAMES`.

## Abilities

### Bloodletter — auto, `LMB`
**Startup** fast · **Recovery** short · **Range** medium · **Mechanic** small cost; returns a
share of everything it cuts, on the catch

Throw a blade a fixed distance and it comes back. It damages on both passes, and the blood it
takes is banked and paid to you **when it returns**, not when it lands.

That delay is the whole ability. The cut happens immediately and the payment has to survive
the flight home, so an auto attack is a small commitment rather than a free poke, and the
simplest possible statement of the class: give something away, get it back if things go well.

The move has no hitbox of its own. The blade in the air is the entire threat, and the caster
standing where they threw it from is harmless.

### Rend — committed, `Shift+LMB`
**Startup** medium · **Recovery** medium · **Range** melee · **Mechanic** medium cost; large
return on hit

A raking claw at chest-to-chest range. It roots you, it hurts, and connecting returns half of
it.

> **Changed 2026-09-12.** Rend was the auto until the auto became the Bloodletter, and it
> moved down into the committed slot rather than being cut. The archive's version is a
> reactivating projectile that slows and grows; that is still the intended Rend and is still
> not implemented. What is in the game is the claw, given committed-slot weight.

### Grasp — special, `Q`
**Startup** medium · **Recovery** long · **Range** short · **Mechanic** high cost; large
return if most of it lands

A short-range skillshot that fires four arms — top left, bottom left, top right, bottom
right. They leave in a cone, bow outward, and arc back inward to converge at the far end.
Each arm damages on its own. **Anything caught by all four is rooted** for about two thirds
of a second — and a rooted enemy takes 1.4× from everything this class has, which is what the
root is for.

The volume they sweep is a lens rather than a line, so standing anywhere near it gets you
clipped by one or two arms. All four is a much smaller place to be, which is what makes the
root a read rather than a tax — see the rule in [../ability-spec.md](../ability-spec.md)
about hard stops needing hard conditions.

Rooted means your feet do not carry you and you cannot dodge or jump. It is not a stun: you
can still turn, guard and swing at whoever put the arms round your legs. The root is
deliberately longer than the hitstun of the arms that deliver it, or it would expire before
the victim could notice it — and longer than her fastest move's startup, or there would be
nothing she could land inside it.

Unblockable, and the class's answer to a turtle now that Reaper's debt is gone.

### Black spike — mechanic, `E`
**Startup** slow · **Recovery** medium · **Range** long · **Mechanic** high cost; returns most
of everything it drains, continuously

A spike erupts at the target area after a long delay, damaging on arrival. It then stands in
a field that drains and slows anything inside it for several seconds, and a large share of
what it drains goes straight back to the caster.

**The eruption is one hit.** It spent a day as five, because `hits again every` — the move
table's re-hit interval, in frames — was set to 1 while chasing a bug that turned out to be
somewhere else entirely. That knob governs the *move's own hitbox* during its active frames;
it has nothing to do with the field, which has its own clock in
`effects.damage tick interval`. The spike's active window is four frames, so a re-hit of one
made the eruption land four or five times.

Three things about it are deliberate:

- **The slow is the part that matters.** Damage alone makes a puddle you step out of. The
  slow is what makes leaving cost time, which is what turns it into something you put
  *between* yourself and someone else.
- **The return is continuous**, not a lump sum when the field expires. A Blood mage standing
  in a fight is being paid the whole time it is up, which is the difference between an
  ability you build a fight around and one you survive a timer to collect on. This replaces
  the archive's tether-break payout, which needed tethers nobody has built.
- **The cast is the telegraph.** Thirty frames — twice reaction time, the longest wind-up in
  the class — because the ability is a placement, and a placement the other player cannot see
  coming is a trap rather than a decision.

You can jump over it. The field is a slab as tall as the spike standing in it, not an
infinitely high cylinder, and it is drawn at exactly the size it is tested at.

> **Was** on shift + click, at two and a half metres of reach, with an eighteen-frame cast,
> no return of any kind, and no spike in the model — only a stain on the floor. In a hunt it
> did nothing whatsoever, because effects were applied to fighters and the creature was not
> one.
>
> **And then, once it did:** the field took health off the creature and gave the caster none
> of it, for a fortnight, because the two field effects asked the creature-damage path what it
> had dealt and threw the answer away. Only in a hunt — in versus a field never meets the
> creature at all — and only in the *field*, since the eruption on the same cast paid out
> correctly a few frames earlier, which is exactly enough to make a broken field look like a
> working one. See the feel log for 2026-09-13.

## Not implemented

Still in the design, still not in the game. The archive's versions are in
[`blood-mage-skills.md`](../../archive/combat-design/blood-mage-skills.md).

### Rend, the projectile
Fire a small fast projectile that passes through enemies. Reactivate and it slows, grows
rapidly, slowing and damaging in its area, then shrinks back after a few seconds. If you are
inside when it collapses, you are healed. The reactivation is the class's signature decision —
where you put yourself relative to your own projectile.

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

### Seal of the unforgiven
**Startup** instant · **Recovery** medium · **Range** self, then short · **Mechanic** no
health cost; locks the other seals

Channel, immobile, up to three seconds. Take 60% reduced damage and store everything
blocked. On release, discharge all of it to everything nearby.

The defensive option that is also the payoff — the more you eat, the more you deal. Using
any seal locks the others for a window, which is the archive's universal seal cooldown
re-expressed as a shared resource lockout.

### Further out

Not sized, not slotted, and listed because they are the rest of what the archive has for this
class — the six-ability kit above is a prototype target and the eventual twelve has to come
from somewhere.

- **Murmur of the forgotten** — an area, ten seconds, that heals allies for a share of the
  damage you do *to yourself*. The coop face of the mechanic, and the only ability in the
  class that makes the self-damage somebody else's gain.
- **Demon hunger** — a chain of three detonations in a line, each bigger than the last. If
  the first three hit nobody they start walking back toward you, and the sixth goes off on
  the caster. A whiff that costs you rather than one that costs you nothing.
- **Seal of the voracious** — more damage, health draining per second.
- **Seal of the forsaken** — three seconds of taking no damage at all, self-damage included,
  for 99% of current health.
- **A health-stealing dash** — invulnerable, medium speed, to a target location, stealing
  health from whatever it passes through. The class has no mobility ability at present, and
  `ability-spec.md` asks every kit to cover mobility.
- **Five radial tendrils** — a cone of tendrils that stop at the first enemy each hits, slow,
  and steal health over three seconds, with a base area **the caster must stay inside** to
  collect. A close relative of Grasp, and the one that makes the positioning the cost.
- **A hand out of the ground** — delayed, smashes a medium area. The archive's other
  telegraphed placement, and a sibling to Black spike.

## Cut

### Reaper's debt
A cone charged by holding, the longer the hold the wider the cone, no turning while
channelling. Replaced by Grasp on `Q`.

It was the class's guard breaker and its only charged move, and losing the charge is a real
loss. What it was not was *this class*: a directional cone that gets bigger the longer you
hold it is a fine ability and would sit as comfortably on the Champion. Grasp reaches out,
closes, and holds somebody still, which is a thing only a Blood mage does. The animation
survived the swap intact — the arms were already opening wide and snapping shut in front of
the sternum, which turned out to be a picture of four arms leaving in a cone and arriving
together.

## Playing it

Open with Black spike to make a place the other player does not want to be, use Grasp when
they have to cross it, commit to Rend while the root holds them — that is where the 1.4×
lives, and Rend's fourteen frames of wind-up fit comfortably inside forty frames of root — and
pay yourself back with Bloodletter in between. You are always a little below full health on
purpose.

In a hunt the same sentence reads differently and means the same thing: the climb breaks its
poise, the topple is the disable, and everything you do to a Ridgeback on its side is worth
half again as much.

## Open questions

- Health cost as a flat amount or a percentage? Implemented as flat. Percentage is
  self-balancing but makes the class stronger the healthier it is, which inverts the comeback
  fantasy.
- Are the costs anywhere near right? They are a first pass: 15 for the auto up to 120 for the
  spike, against a thousand-point bar. Nothing has been played against them.
- Does the root want to stop you attacking as well? It does not, on the grounds that the
  design's default is a root you can still act in. If the class turns out to need a real
  opening rather than a slow one, this is the knob.
- **Is 1.4× the right bonus against a disabled enemy?** Guessed. The feel tests bound it
  between 1.2 and 2 — below the floor nobody notices it and Grasp goes back to being a root
  with no payoff; above the ceiling one read ends the round, which is the opposite of a game
  built on whiff punishment. Where it sits inside that range is a play question.
- **Should the trait be the whole answer to Grasp's cost?** Grasp is the most expensive thing
  in the kit and the root is short. If the follow-up window turns out to be too tight to use,
  the root's length is the first knob and the bonus is the second.
- Only one seal is specified. The archive has three; the other two are power-level knobs and
  can wait.
