---
status: proposed — v1 kit, nothing built
decided: 2026-09-23
supersedes: the mechanic and ability sections of kits/blood-mage.md, once built
sources: docs/archive/combat-design/blood-mage-skills.md, kits/blood-mage.md, this design thread
---

# Blood mage — v1 kit

**Identity.** Her blood goes out, and theirs comes back. Every cast is a cut she makes in
herself; every hit spills the other fighter onto the floor; and the only way to close her own
wounds is to put an ability through the blood she spilled. A war scythe, a reaper's reach that
grows with how open she has left herself, and a floor she wants to fight on.

This replaces the built kit's economy — *cost on the press, leech on the hit, a field that
drains* — which turned out to be four abilities with a red colour scheme and a percentage.
The percentage was the problem: a number that pays out wherever the hit lands gives the player
nothing to *go to*, and a class whose sentence is "sustain through aggression" shipped with
three tools that reward standing back. See [Why this shape](#why-this-shape) for the argument.

Input map in [controls.md](controls.md). The built kit, and what it got right, is still
[kits/blood-mage.md](kits/blood-mage.md).

## The three things a mechanic has to pay

The classes that feel right pay the player three ways from one object. The Champion's spear
is damage, and planted in the floor it is a vault. The Elementalist's stone is cover and a
combo piece, and raised under her feet it is a jump. The Reaver's shadow is a second threat
and the dash to it is her mobility. Nothing was added to get the third benefit: it fell out of
the thing already on the field.

The Blood mage's object is **essence** — the other fighter's blood, pooled on the floor where
she cut them. One kind of thing, and it pays all three:

| | From a pool |
| --- | --- |
| **Damage** | Black spike erupts *out* of one, at the pool's size. A Reap over one is the biggest hit she has |
| **Utility** | Grasp hauls a body onto the pool she is standing in. The spike's eruption launches and slows |
| **Movement** | Dodge with the crosshair on a pool and she is *there* — the Reaver's dash to her shadow, with the shadow replaced by a puddle she made |

And the fourth thing, which is hers alone: **it is the only heal in the class.** Nothing
returns health on the hit any more. A pool is essence sitting on the floor, and she gets it
back by putting an ability through it. Every pool is therefore a place she wants the fight to
be, and every pool is *also* a door. "A pool is a heal or a door" is the decision the class
asks, over and over, and it is a decision about where two bodies are standing.

## Mechanic — grey health and essence

### Grey health

Health she loses — spent on a cast, or taken from a hit — does not vanish. It turns **grey**:
a segment of the bar that is no longer hers but can still be reclaimed. Only healing from a
pool turns grey back to red. Grey **fades** on its own, slowly, at a fixed rate rather than a
fixed time, so a large wound stays open longer than a small one and there is always a clock.

**The more grey she carries, the stronger she is.** Damage scales with it, and the scythe's
**reach** scales with it — the blade is her blood, and the more of it she has let out the
further it extends. That is a line-of-effect change, which [aiming.md](aiming.md) and the Dual
mage's depth curve both refuse for good reason: a reach that changes with a bar only one
player can see is unlearnable. It is allowed here on one condition, the same one
[CLAUDE.md](../../CLAUDE.md) puts on the overlay: **the blade is drawn at the length it hits
at.** A longer scythe is longer on screen, for both players, and the grey on her bar is
visible to both as well. What is being read is a weapon, not a number.

So grey is risk and power in one segment:

- **A fresh cast opens a wound and lengthens the blade.** Casting at full health is how she
  gets going, and it costs her nothing she cannot get back — if she then makes a pool and
  reaches it.
- **Being hit does the same.** A Blood mage who has just taken a hit is *more* dangerous, not
  less, which is the fantasy's whole inversion: the correct answer to being wounded is to go
  in.
- **Healing gives the power back.** Drinking a pool converts grey to red and shortens the
  blade. So a skilled player *banks* grey — rides with a long blade and an open bar — and
  cashes it in when the fade is about to eat it or when the next hit would be the last. That
  is her edge to ride, and it is a different edge from the Dual mage's: *how much of my bar am
  I willing to leave open.*

**A cut can never kill her.** Self-damage clamps at one, the rule the built kit already has.
Enemy damage lands on red first and turns it grey; there is no ordering trick where a hit
erases grey — grey only fades, and only on its clock.

**Two knobs govern all of it:** how fast grey fades, and how much power and reach a full bar
of grey buys. Both in the Oven under *Blood mage*. Healing is never above the red line she
started the round with; grey is the ceiling. Which is what keeps the kernel's stall worry out
of this class: she cannot heal past where she stood a moment ago, only back to it, and only by
going somewhere.

### Essence pools

**Every hit she lands spills the target.** Blood pools on the floor under the point of
contact — under the fighter on a direct hit, under an arm's contact point on a Grasp, under
the creature's struck part projected to the ground in a hunt. The pool's **size is the damage
dealt**, so a scythe poke leaves a smear and a Reap leaves a floor.

- **Pools drain at a fixed rate**, litres a second rather than seconds a pool, so a big pool
  outlives a small one. That is the counterplay knob: a mobile opponent who never stands still
  leaves small pools far apart, and by the time she has forced anybody onto one it has gone.
  High mobility beats her; a stand-and-trade opponent feeds her. Both are what the class
  wants to be true.
- **They merge.** A hit onto an existing pool adds to it rather than stacking a second disc on
  the same floor, so a fight that stays in one place makes one large pool rather than clutter.
- **Cap of four**, oldest merged into the newest when a fifth arrives — the Elementalist's
  cap, for the Elementalist's reason: readability in third person matters more than the combo
  ceiling.
- **They are effects, not structures.** A slab on the floor, a fixed height, drawn at exactly
  the disc it is tested at. Nothing collides with them. You can stand in one, and standing in
  one is the point.
- **The creature bleeds too.** A Ridgeback on its side makes the largest pool in the game
  where it fell, and a pool under a toppled creature is a door onto its back. That is her
  climb route, and it needed no rule of its own.

**Her own blood never pools.** A cost is paid into the ability — the scythe blade, the Grasp
arms, the spike — and does not land on the floor as essence. If it did, every cast would leave
a free heal at her own feet and the loop would close without the other fighter in it.

### Double duty

An ability that **lands on a pool** does its ordinary job and *also* drinks: a share of the
pool's volume comes back as red health, converted out of grey, and the pool shrinks by what
was taken. Which abilities read a pool, and what for, is the kit below — but the rule is one
rule: **the heal is where the blood is, and she has to put something through it.**

That is what makes the skill of the class *forcing the opponent toward essence*: a Grasp that
hauls somebody onto the pool she is standing in has set up a Reap that hits them **and**
heals her in one swing. A Black spike cast on a pool erupts the whole pool at once. A player
who cuts, reads where the blood fell, and bends the fight back onto it gets two abilities'
worth out of every press. One who does not gets one, and bleeds.

### Naturally deals increased damage to disabled enemies

Kept exactly as built: ×1.4 to anything staggered, held, or a creature on its side. It is
what makes a Grasp a setup rather than a tax, and it multiplies the pool a Reap leaves.

## What is bound

| Input | Move | What it is |
| --- | --- | --- |
| `L` | **Reaping sweep** | The auto. A war scythe drawn across the front, low to high. Reach grows with grey |
| `R` | **Reap** | The committed heavy. The scythe brought over and down, unblockable. Over a pool it is the biggest hit and the biggest drink she has |
| `M` | **Bloodletter** | A blade thrown a fixed distance and back, cutting on both passes. The ranged bleed |
| `Q` | **Grasp** | Hold to choose a depth; four arms converge. All four catch, bind, and haul the victim onto her feet |
| `E` | **Black spike** | A spike out of the floor after a delay. On a pool, the pool erupts |
| `shift` + direction, crosshair on a pool | **Blink** | She is at the pool. Consumes it |

Five abilities, an auto, and the mechanic's own movement on the dodge — the Reaver's precedent,
where the class's mobility *is* the universal verb pointed at the class's object. Right click
is free on this class (no shield) and takes the committed melee, which is the slot that has
been stranded since shift stopped modifying clicks. Middle click is the third click and takes
the throw.

## Abilities

Fields per [ability-spec.md](ability-spec.md). Numbers are first guesses for the Oven and are
listed so the shape is unambiguous, not because any of them has been played.

### Reaping sweep — auto, `L`
**Startup** fast · **Recovery** short · **Range** melee, growing with grey · **Mechanic**
small cost; spills what it cuts; drinks a pool it passes over · **Repeat lockout** 100%

A war scythe drawn across her front from low on one side to high on the other — a wide, thin
arc. It is the Dual mage's wing reasoning with the shape turned into a weapon: a blade on a
long haft covers width, not depth, and a body that has closed inside the haft is inside the
sweep. The one piece of execution is the **tip**, which hits harder and is the part that
reaches. The reach it reaches *with* is grey health.

Passing the blade over a pool drinks a small share of it. The auto is the trickle heal, thrown
constantly; it is never the reason to make a pool, but it is why standing in one while she
trades is the right place for her and the wrong place for them.

Every hit spills the target for the damage dealt. The auto is therefore how most pools start.

### Reap — committed, `R`
**Startup** medium, telegraphed · **Recovery** long · **Range** melee, growing with grey ·
**Mechanic** medium cost; a large spill; the largest drink in the kit when it lands over a
pool · **Repeat lockout** 100%

The scythe raised and brought over and down in front of her — an overhead, so it goes over a
crouch, and it is **unblockable**, which is the class's guard breaker on the special's
neighbour rather than on the special itself (the pattern [ability-spec.md](ability-spec.md)
already records for two classes). Slow enough to read and punish. The reward is the biggest
number in the kit, the biggest pool a single hit can leave, and — landed on somebody standing
in essence — a drink proportional to the pool.

This is the *double duty* ability: a Reap on bare floor is a heavy; a Reap on a pool is a
heavy and a heal. Landing it there is the thing the rest of the kit exists to arrange.

> Replaces Rend the claw, which was the committed melee with no input, and was a slower poke
> with a health cost. The archive's Rend — a projectile that grows into a healing field — is
> **cut**: it is a second heal mechanism and the class now has exactly one.

### Bloodletter — throw, `M`
**Startup** fast · **Recovery** short · **Range** medium, skillshot · **Mechanic** small cost;
spills on both passes; drinks any pool the return crosses · **Repeat lockout** 100%

A blade thrown along the crosshair to a fixed distance and back to her, cutting on the way
out and on the way home. Kept from the built kit, moved off the auto: a returning blade is the
class's ranged way to *make* a pool under somebody who will not come to her, and its return
leg crossing pools on the floor is a small drink on the way back. No hitbox of its own; the
blade in the air is the whole threat.

What it lost is the leech on the catch. The blade brings back a cut, not health; the health is
on the floor where the cut happened.

### Grasp — special, `Q`
**Startup** medium, channelled · **Recovery** long · **Range** chosen by the hold, melee to
long · **Mechanic** high cost; each arm spills at its contact point; the catch hauls the victim
onto her feet · **Repeat lockout** 100%

Kept almost as built, because the built version is the best-argued ability in the class: hold
to choose a depth on the crosshair's line, four arms converge there, each arm damages and
spills, and **only all four catch**. The catch binds for a sixth of a second and then hauls at
speed to her feet, feet handed back the frame the haul ends.

One change, and it is what makes the ability the class's control tool rather than a snare: the
haul ends **at her feet**, and her feet are where she chose to stand. Standing in a pool when
the arms close means the victim arrives held, disabled, at ×1.4, *on the essence*, in reach of
the Reap that is already winding up. That is the whole "force them toward the essence" skill
in one press. Unblockable, and the class's other answer to a turtle.

Against something that cannot be hauled — the creature — the catch **hauls her to it**
instead. A Grasp that lands all four on a Ridgeback's flank pulls her across the gap onto its
side. That gives her a second route up, and it is the same ability doing the same thing with
the heavier body winning.

### Black spike — mechanic, `E`
**Startup** slow, telegraphed · **Recovery** medium · **Range** long, grounded · **Mechanic**
high cost; on bare floor a spike that spills; on a pool, the pool erupts · **Repeat lockout**
100%

A spike out of the floor where the crosshair is, after the same long delay as today: the cast
is the telegraph and a placement the other player cannot see coming is a trap rather than a
decision. What erupts depends on the floor:

- **Bare ground:** a spike. Damage, a launch, a short slow. It spills what it hits, so a spike
  is how she *seeds* a pool at range where no scythe reaches.
- **A pool:** the whole pool erupts as a field of spikes the size of the pool, launching and
  slowing everything standing in it, and **drinking the entire pool** in one go. The bigger
  the pool, the bigger the eruption and the bigger the heal. This is the payoff placement —
  cast it on the floor the fight has been bleeding onto and the floor comes up.

**The drain is gone.** There is no lingering field and nothing ticks. The spike either seeds
or cashes in, and both are a single event the other player can watch happen.

### Blink — the mechanic's movement, `shift` + a direction with the crosshair on a pool
**Startup** instant · **Recovery** the dodge's own · **Range** wherever a pool is ·
**Mechanic** consumes the pool · **Repeat lockout** the dodge's

A dodge thrown with the reticle on a pool puts her in it, instantly, and the pool is spent.
This is `aim::pointing_at` — the same question the Reaver's forward dodge asks about her
shadow — pointed at an effect instead of a body, and it is the class's mobility falling out of
its object rather than a move added to fill the slot.

It is deliberately expensive in the currency that matters: a blink eats a heal. "A pool is a
heal or a door" is the decision, and a player who blinks to every pool never heals, while one
who drinks every pool never moves. Pools draining on their own is the other limit: a door that
is closing is a door you have to decide about now.

Whether the blink is invulnerable, and for how many frames, is a knob. The dodge's own frames
are the default and the intent is that it be a *reposition* rather than an escape — the
Reaver's dash is the precedent, and hers is invulnerable for the crossing, which here would
be zero frames of crossing.

## Playing it

Cast to open a wound and lengthen the blade. Sweep to spill them. Read where the blood fell and
bend the fight back onto it: Grasp them onto the pool you are standing in, or Bloodletter them
where they stand to start one there, or blink to the pool they left behind and Reap them as
they come back for you. Cash grey in when it is about to fade or when you are about to die,
and not before — every point of grey you are carrying is reach.

In a hunt the same sentence reads: hit the creature until the floor under it is red, ride the
topple for the ×1.4, and when it is on its side the pool under it is a door onto its back.

**Counterplay.** Do not stand in blood. Keep moving, so the pools she makes are small and far
apart and gone before she can use them. Punish her while she is grey: a Blood mage with a
long blade is a Blood mage with a short bar, and she cannot heal without going somewhere you
can see.

## Why this shape

Three arguments, and the first is the one that decided it.

**One object, three payoffs.** The built kit had four abilities that shared a percentage.
This kit has five abilities and a dodge that share a *thing on the floor*, and the thing pays
damage (the eruption, the Reap), utility (the haul onto it, the launch out of it) and movement
(the blink) without a new move for any of them. That is the property the Champion, the
Elementalist and the Reaver have and the two mages did not, and it is the reason to rebuild
rather than tune.

**The heal has to be somewhere.** Leech pays out wherever the hit happens, which means the
player never has to *go* anywhere to be paid — and a class whose mechanic is sustain-through-
aggression then has nothing to be aggressive *toward*. A pool is a place. Making the heal a
place is what turns "aggression" from an adjective in the identity line into a direction on
the floor.

**Grey is the edge to ride.** The Dual mage rides a bar between two forces. The Blood mage
rides how much of her own bar she has left open — and because the open part is also her
reach, leaving it open is not a penalty she is waiting out but a weapon she is choosing to
keep. The moment she heals she is shorter. That is a decision every time, which is what a
mechanic is for.

## What it costs to build

Nothing in the aiming model changes. Every line of effect here is one the four kinds already
cover: the sweep and the Reap are swings, the Bloodletter and the Grasp are skillshots, the
spike is grounded, and the blink is `aim::pointing_at` against an effect's position — the
Reaver already asks that question, and a second caller is what the function is there for.

- **`Player` gains a grey segment** beside `health`. Damage and costs move red into it; the
  fade moves grey out of it; a drink moves grey back. Three arithmetic paths, all in the
  snapshot, and the 4 KiB cap has room for one more integer.
- **`EffectKind` gains a pool** — position, volume, a fixed height. It sits in the existing
  fixed array of eight, so a cap of four pools leaves four slots for a Grasp in flight and a
  spike, and nothing allocates. Merge-on-overlap and drain-per-frame are two lines in
  `step_effects`.
- **`Move` loses `leech`** and keeps `cost`. The double-duty rule is one function — *did this
  hit land over a pool, and how much does this move drink* — with five callers, the same
  shape `disabled_damage_mul` already has.
- **The scythe is a swing whose reach reads grey**, and the blade mesh is scaled to the same
  number, so the overlay rule holds by construction.
- **Two new clips**: a horizontal sweep and an overhead. The Grasp's, the spike's and the
  throw's clips are the built ones.

**Tests that change.** `feel.rs` has *a Blood mage ability thrown perfectly returns more than
it cost*; it becomes *a Blood mage ability landed over a pool returns more than it cost, and
one landed on bare floor returns nothing*. `every_class_can_beat_a_turtle` is satisfied twice
(Reap and Grasp). A new relationship worth pinning for the whole roster, from the argument
above: **every class has a move that carries the body.** The Champion's step, the Reaver's
dash, the Elementalist's stone lift and Landfall pass it today, and both mages fail it —
which is the fact this document and the Dual mage thread are both responding to.

## Open questions

- **How fast does grey fade?** Too fast and the reach never gets long enough to matter; too
  slow and it is simply "missing health makes you stronger", which every berserk mechanic has
  already been and which loses the *reclaimable* half. The fade is a rate, so the starting
  guess should be one that lets a full committed cast's worth of grey survive one exchange.
- **How much reach does a full bar of grey buy?** Enough to be read across the arena, since
  reading it is the whole justification for a scaling reach. A blade half again as long at
  full grey is the first number to try.
- **Is a blink that consumes the pool too weak, or a blink that does not too strong?** It is
  written as consuming. If the class turns out to have too few pools to ever spend one on
  moving, the middle answer is a blink that takes a *share*, the way the sweep does.
- **Should the blink cost health?** It is the one thing in the kit that does not. A cut on the
  blink would make it a *wound you take to arrive*, which is in character and gives the grey
  bar a mobility tax; it is also one more cost on an already expensive class.
- **Does the Grasp-onto-her-feet make the channel worth its stand-still?** The worry from the
  built kit stands: half a second of holding still while a marker slides is a long time.
  Hauling the victim onto the essence she is standing in is a much larger reward than the old
  catch, and may be what makes the price fair. If not, the bind is the first knob.
- **Pools on slopes and platforms.** A pool is a disc on the floor it was spilled onto. A
  fighter hit in the air spills where they are standing when they land, or not at all — the
  first is friendlier and the second is simpler, and neither has been decided.
- **Do the other classes interact with pools?** Today nothing anybody else throws touches a
  stone, and this document does not propose that anybody else touches a pool either. The
  Elementalist's fire over a blood pool is the obvious first interaction and is not in v1.
- **Costs.** Higher than today's, since they are recoverable: the auto a trickle, the Reap and
  the Grasp real, the spike the most. What fraction of a bar a cast should represent is the
  same TTK question it always was, and it needs the prototype.
- **The name of the essence.** "Pool" and "essence" are used interchangeably above and one of
  them should win.
