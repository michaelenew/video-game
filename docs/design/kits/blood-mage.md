---
status: proposed
decided: 2026-09-09
revised: 2026-09-12
sources: docs/archive/combat-design/blood-mage-skills.md, docs/archive/combat-design/class-builds.md, docs/archive/combat-design/new-system.md
---

# Blood mage — kit

> ⚠️ **A v1 rebuild is proposed, 2026-09-23.** [../blood-mage.md](../blood-mage.md) replaces
> the economy below — leech on the hit and a draining field — with **grey health** and
> **essence pools** on the floor: her blood is the ability, theirs pools where she cut them,
> and the only heal is putting an ability through a pool. The weapon becomes a war scythe, the
> auto a sweep, Reap takes right click, and a dodge with the crosshair on a pool is a blink.
> Nothing of it is built. This document is still what is in the game.

**Identity.** Sustain through aggression. Everything costs health and the good outcomes give
it back, so the class is always spending itself forward.

> **Revised 2026-09-12.** The implemented kit changed shape. Black spike moved off shift +
> click onto `E`; the auto became the archive's returning blade; Reaper's debt was cut and
> replaced by Grasp; Rend took the committed slot the spike vacated; and **every ability now
> has a health cost in the move table**, which is the first time the class's own mechanic has
> existed in the simulation rather than only in this document. See
> [Implemented](#implemented) for exactly what is in the game.
>
> **Revised 2026-09-13.** Grasp became the game's first *channelled* move: hold `Q` to choose
> how far out the arms converge, watching a marker travel in front of you, and let go to throw
> it. Its catch stopped being a teleport-plus-root and became a bind and then a haul you can
> watch, with the victim's feet handed straight back at the end. `Player::rooted` went with it
> — nothing applied a root any more, and a primitive with no ability behind it is a promise
> the code has not earned.

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

**Disabled means staggered, held, or a creature on its side.** Not hitstun, and that
exclusion is the whole of the definition — hitstun happens on every hit anybody lands, so
counting it would turn the trait into "increased damage from the second hit onward", which is
a flat damage bonus in a costume. What is on the list is what
[../ability-spec.md](../ability-spec.md) calls a hard stop, and the design only allows those
behind a hard condition: a parry for the stagger, a grab that landed or every arm of a Grasp
for the hold, a broken poise bar for the topple. Each one was *earned*, which is what a payoff
should be waiting on. Blockstun is not on the list: they blocked, which was correct, and
paying the attacker for it would make guarding worse than standing still.

It applies to every way she deals damage — a swing, a blade in the air, an arm of a Grasp, a
field ticking, and all of the same against the creature. A trait that applied to three of a
class's four abilities would not be a trait, it would be a bug somebody finds in a match.

**This is what makes Grasp a setup.** Four arms is expensive and the catch is short — too
short to react to on purpose, see below; without something already happening on the other side
of it, landing all four would be a small reward for a hard read. With it, the catch is a window
you spend — and because leech is a percentage of damage dealt, the bonus compounds into the
health you get back.

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
| ~~`Shift+LMB`~~ | **Rend** | The committed melee. A raking claw at chest range. **No input since 2026-09-16** — see below |
| `Q` | **Grasp** | Hold to choose a depth, then four arms converge there. All four catch |
| `E` | **Black spike** | A spike in a draining, slowing field, placed at long range |

### Why Black spike is on `E`

`E` is the class mechanic, and on three of the six classes that is an instant change of state
— throw the shield, Rush, raise a structure. The Blood mage's mechanic is *health*, which is
not a thing you press a button to change, so her `E` did nothing at all. Putting the spike
there costs nothing and buys the class a fourth ability; shift + click, which meant "the
committed version of your attack" on every other class, went back to meaning that here. It
means nothing anywhere now — see
[../controls.md](../controls.md#shift-is-one-verb-now-2026-09-16) — which is why Rend is
stranded and Black spike is not.

This was the first ability in the game on the mechanic slot, so the move table grew a fourth
column — see `moves::on_e`. It is no longer the only one: the Dual mage's `E` carries Sweep
for the same reason (a meter has no state to toggle either), the Reaver's carries Executioner
for a different one (her mechanic went to the mouse because it is aimed), and the
Elementalist's is an instant standing up and Landfall off the floor.

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

### Rend — committed, and currently unbound

> **Stranded, 2026-09-16.** Shift plus a click is not an attack input any more, on any class —
> see [../controls.md](../controls.md#shift-is-one-verb-now-2026-09-16). This move is still in
> the table, still tuned and still printed by the frame table, and there is no button that
> throws it. Finding it a home is its own job: the right answer is different per class, and
> guessing three of them at once is how a grammar gets worse.

**Startup** medium · **Recovery** medium · **Range** melee · **Mechanic** medium cost; large
return on hit

A raking claw at chest-to-chest range. It slows you to a crawl, it hurts, and connecting
returns half of it.

> **Changed 2026-09-12.** Rend was the auto until the auto became the Bloodletter, and it
> moved down into the committed slot rather than being cut. The archive's version is a
> reactivating projectile that slows and grows; that is still the intended Rend and is still
> not implemented. What is in the game is the claw, given committed-slot weight.

### Grasp — special, `Q`
**Startup** medium · **Recovery** long · **Range** you choose it · **Mechanic** high cost;
large return if most of it lands

Four arms — top left, bottom left, top right, bottom right. They leave in a cone, bow
outward, and arc back inward to converge at one point. Each arm damages on its
own. **Anything caught by all four is seized**: bound where it stood, then hauled back to the
caster. A held enemy takes 1.4× from everything this class has.

That is one ability doing the archive's two things at once — the four converging arms, and the
tendrils that only pay out if you stay close enough to collect. It closes the range for you.

#### Holding `Q` chooses the depth

The only move in the game you aim with *time*. Press and hold and a small marker leaves the
caster's chest and travels outward over half a second, from melee range out to ten metres; let go
and the arms converge on wherever it had got to. Hold past the end and it throws itself, so
there is no storing a paid-for ability and waiting.

**The marker is not a projectile.** Nothing collides with it, nothing is hit by it, and the
simulation does not know it is drawn. It is the far end of `Player::aim_path`, which is solved
every frame of the wind-up by the same `sim::aim` call the finished move uses — so it cannot
promise a depth the arms do not deliver. That is the whole reason it is built this way: two
pieces of arithmetic agreeing with each other is how the crosshair and the ability came to
disagree three times before (see [../aiming.md](../aiming.md)).

**The line is solved at the move's full reach every frame, and the hold only picks a point
along it.** That split is the whole readability of the move, and it took two wrong versions to
find.

Solving the *aim* at the wound-up range — the first version — means the raycast's answer
changes as the range grows: the far end walks off the floor and onto a wall and back, so
holding the mouse perfectly still you watched the marker jump about while trying to choose a
depth. Making the move a **swing** instead — a ray off the body, dead-zoned to stay level while
standing — held the line still, but a level line out of a platform passes clean over anybody on
the floor below, and the only way to land one was to aim well under the target on screen.

Solving the line once, at the full reach, has neither problem. It is the crosshair's line, so
it converges on whatever you are looking at from any height; it does not move while the mouse
does not; and the hold slides a point along it.

**The line is read for its direction and nothing else.** How far the arms go is the hold, on
its own — point at a wall two metres away, wind to full, and it is still a ten-metre Grasp that
converges eight metres behind the wall. That is deliberate: a wall is a thing to punch an
ability through, and a wind-up that quietly shortened itself against the scenery would stop
meaning one thing.

Aiming stays live through the wind-up — the body keeps turning with the mouse — and locks on
the frame the button comes up, which is where every other move locks it too. The health is
paid on the *press*, because there is no cancelling out of a channel: an ability you started
is an ability you bought.

Half a second is a long time to stand still, and that is the cost. The whole wind-up is a
telegraph: at max depth you have spent half a second of neutral before anything has left your
hands, in plain view of somebody who can simply walk out of the cone.

**Aiming it from above works because the crosshair converges.** Standing on a platform and
putting the reticle on somebody below, the ray goes from the eye through the reticle and meets
the floor at their feet; raised to the middle of a fighter standing there, that is a hit. No
adjustment, no aiming short. This is the general rule in [../aiming.md](../aiming.md) rather
than anything the Grasp does for itself — the same fix landed on Bolt, Bloodletter and Lance at
the same time.

#### The catch: bound, then hauled

Two phases, and the first is why it is not a teleport:

1. **Bound** — about a sixth of a second where nothing moves. You are held exactly where the
   arms closed on you.
2. **Hauled** — dragged back to the caster at forty metres a second, across real ground, in
   frames you can watch. Nine metres takes about a quarter of a second.

Then the hands let go and **you have your feet back the same frame**. No root on the end of
it, no extra recovery. A victim who cannot tell where the ability stopped is a victim being
stunlocked, which is the thing [../bulwark.md](../bulwark.md) warns about.

**The whole catch is shorter than her most expensive cast takes to come out.** That is
deliberate and it is the design of the move: you cannot see the arms close and *then* decide
what to do about it. Whatever is supposed to meet them at the end of the trip — a Black spike
already in the ground, a Bloodletter already in the air — has to have been committed to before
the Grasp was known to have landed. High risk, high skill, and it backfires when the read was
wrong: you have paid forty-five health, spent your recovery, and put nobody anywhere.
`preying_on_the_disabled_is_worth_feeling_and_is_not_an_execution` in `feel.rs` pins it.

The volume the arms sweep is a lens rather than a line, so standing anywhere near it gets you
clipped by one or two — damage, and nothing else. All four is a much smaller place to be:
about a metre wide where the cone closes, against up to ten metres of reach. That is what
makes the catch a read rather than a tax, per the rule in
[../ability-spec.md](../ability-spec.md) about hard stops needing hard conditions.

**The grab has to wait for all four, and not only for flavour.** It hauls its victim toward the
caster, so one applied by the first arm to land would pull them out from under the other three
— the bottom pair connect a frame before the top pair — and the catch would cancel itself.

**The hold has to outlast the haul.** The haul covers real distance at a real speed, so if the
hold runs out first a Grasp landed at full range drops its victim halfway home — standing in
mid-air, mid-drag, suddenly able to walk. Invisible up close and total at long range, which is
the worst way for a number to be wrong, so
`a_grasp_always_finishes_hauling_before_it_lets_go` holds the two together.

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
  ability you build a fight around and one you survive a timer to collect on.
- **At full health you will not see it.** Not a bug, and worth writing down because it has
  been reported as one twice. Health cannot go over the bar, and the eruption's own leech
  arrives first: cast at 1000 out of 1000, the sixty it cost comes back the instant the spike
  lands, and every tick of the field after that is clamped away. Measured, casting on a target
  standing in the field:

  | Cast at | Eruption returns | Field returns |
  | --- | --- | --- |
  | 1000 / 1000 | +59 | **+1** |
  | 700 / 1000 | +59 | +90 |

  Which is the class working: she heals when she is hurt and gains nothing when she is whole,
  so the spike is close to free at full health and a large swing when she needs it. It does
  mean the ability reads as broken in exactly the situation you test it in — the first cast of
  a fresh round. **If the drain's return should be felt at the top of the bar**, the eruption
  is what is eating the headroom, and the fix is to stop the eruption leeching: the written
  design says the spike *returns a share of everything it drains*, and the arrival damage is
  not a drain. That would need a second leech number, since one move has one today, and it
  would cut what the ability returns overall — so it is a decision rather than a correction. This replaces
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

Put Black spike in the ground **first**, then use Grasp to drag somebody into it. That is the
order, and the catch being too short to react to is what forces it: the spike takes half a
second to erupt and the whole hold is shorter than that, so a mage who waits to see the arms
close has already missed the window. Hold `Q` to whatever depth they are standing at, let go,
and if all four land they arrive in the field disabled and taking 1.4×. Pay yourself back with
Bloodletter in between. You are always a little below full health on purpose.

The other read is Rend: fourteen frames of wind-up against a hold that runs a little longer
than that, thrown as the haul starts rather than when it finishes.

In a hunt the same sentence reads differently and means the same thing: the climb breaks its
poise, the topple is the disable, and everything you do to a Ridgeback on its side is worth
half again as much.

## Open questions

- Health cost as a flat amount or a percentage? Implemented as flat. Percentage is
  self-balancing but makes the class stronger the healthier it is, which inverts the comeback
  fantasy.
- Are the costs anywhere near right? They are a first pass: 8 for the auto, 30 for Rend, 45
  for Grasp and 60 for the spike, against a thousand-point bar. Nothing has been played
  against them.
- **Is half a second the right channel?** It went to a full second while the marker was still
  jumping about, on the theory that it was too fast to read; once the line stopped moving,
  half was enough again. It is two decisions at once: how long the wind-up is, and how far the
  slider travels in it. Melee to ten metres over thirty frames means the marker moves about
  twenty-eight centimetres a frame, which is quick — fast enough that placing a specific depth
  is a real skill rather than a wait. Both ends are move-table knobs (`Channel, longest hold`
  and `Channel, reach at no hold`), so the answer stays a play question.
- **Should the Grasp be cancellable?** It is not: the health is paid on the press and the only
  way out is to throw it. A channel you could back out of would be a free look at what the
  other player does with half a second of your commitment, which is the opposite of the
  ability's design.
- **Is 1.4× the right bonus against a disabled enemy?** Guessed. The feel tests bound it
  between 1.2 and 2 — below the floor nobody notices it and Grasp goes back to being a catch
  with no payoff; above the ceiling one read ends the round, which is the opposite of a game
  built on whiff punishment. Where it sits inside that range is a play question.
- **Should the trait be the whole answer to Grasp's cost?** Grasp is the most expensive thing
  in the kit and the catch is short *on purpose*. If pre-committing turns out to be too hard
  rather than merely demanding, the bind is the first knob — it is the part of the hold the
  caster gets to spend — and the bonus is the second.
- Only one seal is specified. The archive has three; the other two are power-level knobs and
  can wait.
