---
status: mostly decided; ascension is an open proposal
decided: 2026-09-10
formerly: Statera
supersedes: docs/archive/combat-design/statera-skills.md (resource system), docs/archive/combat-design/class-builds.md (Statera section)
---

# Dual mage

Formerly **Statera**. Renamed to pair with the Blood mage — two casters named for what they
run on — and because "statera" (scales, balance) described the meter rather than the person.

## Fantasy

The Dual mage contains **two immense forces**, each of which would individually kill the
human trying to hold it. They sit in the balanced tension between them. Lose control and
you lose your life.

The play fantasy is to be **just on the line** — to wield as much power as you can by
executing right at the edge of where you can still barely pull yourself back.

This is a containment story, not a channelling story. The human is not a conduit; they are
a vessel under load.

## The meter

A single bar with a centre and two ends.

```
  Dark  <——————————————[ centre ]——————————————>  Light
   (extreme)                                        (extreme)
```

- **Power scales continuously with distance from centre**, symmetrically. Centre is the
  weakest place to stand. The ends are the strongest and the most lethal.
- There are no discrete zones and no threshold effects. It is a gradient, which is what
  removes the dead-zone problem in the old three-state design.

The original design put human, balanced, and divine on a single axis of *how much damage
and CDR you get*, which made the middle a strictly worse version of the ends. That could
not be fixed by tuning, because all three states were the same quantity.

## The last auto is the force you are carrying — revised 2026-09-13

**The autos have sides. Nothing else does.** Left click is dark and right click is light,
and landing one sets which of the two forces the mage is *carrying*. Every other input —
the committed cast, the key abilities — is made of that force and pushes the bar that way.

This replaces "every input picks a side by which button threw it", which could not survive
the kit growing keys: `Q` and `E` have no side, and a rule that says "left click is dark"
has nothing to say about a key. The revision is smaller than it looks, because the thing it
protects is the same: **you cannot cast without moving the bar, and you cannot move the bar
without committing to a side.** What changed is that the commitment is made with the button
you press constantly rather than restated by every other button.

It also gives the colour something to *be*. Which force she is carrying is what decides which
form her abilities take — the light/dark split every ability is written with — so it has to be
a thing the player sets deliberately and can read off her own animation.

| Input | Which way it pushes | How far |
| --- | --- | --- |
| Dark auto (`L`) | Dark, and she is now dark | `tuning::meter_auto_push` — 5 |
| Light auto (`R`) | Light, and she is now light | 5 |
| Anything else | Whichever force she is carrying | `tuning::meter_cast_push` — 12 |

Two tiers rather than a number per ability. "Stronger abilities push harder" was a formula
over damage, which meant a knob nobody could find and a finisher that pushed about as hard as
a poke; two numbers, both in the Oven, are legible and are what a tuning pass can actually
move. **The finisher's own tier — "a deep finisher nearly throws you over the edge" — is not
built yet**; today it pushes like any other cast.

Before the first auto lands she is carrying neither force, and a cast pushes her further along
whichever way she was already going — which at dead centre is nowhere. That is the same rule
this document already states for every input that has no side.

## Every ability has two forms, and the button picks

**Left click always moves you darker. Right click always moves you lighter.** Every input —
autos, directional basics, shift abilities — not just some of them.

Power scales with meter depth rather than snapping between states:

| | At centre | Deep on that side |
| --- | --- | --- |
| Step (Light) | Short blink, minor damage | Long blink, damage and blind |
| Step (Dark) | Short dash, minor drain | Long drain-dash, large steal |

**Centre is not a third form.** It is the position where both forms are available and both
are weak — most options, least power. That property falls out of the mechanic instead of
being asserted.

### Autos are the steering wheel

The autos are dark (left) and light (right), and **they change your mode on contact**. A
whiff steers nothing, so steering requires landing hits — and since 2026-09-13 they are the
*only* thing that picks a side at all. See the section above.

**Autos have a slight range boost**, powered by the beings inside. That is mechanical, not
decorative: if steering depends on connecting, the class needs the reach to steer while
under pressure.

**Steering is not optional.** You cannot cast without moving the bar, and you cannot move the
bar without committing to a side. An earlier draft put the direction choice on a tap-versus-
hold modifier, which made it something the player could ignore; direction belongs in the
input the player uses constantly.

Scroll click and both-click are neither left nor right, so they cannot pick a direction —
they push you **further along whichever path you are already on**. Direction comes from
side-ness, and only left and right have it.

### Why not a neutral form at centre

An earlier draft gave each ability three states — a neutral behaviour at centre, plus Light
and Dark at depth. That breaks at zero: a neutral form has no side, so at centre nothing
votes and the meter cannot leave the middle. Two incompatible ideas had been merged —
*abilities have sides* and *abilities transform with position*. They work at depth and fail
at the origin.

### Oscillating is possible, and correctly weak

Alternate left and right autos and sit at centre indefinitely — and never threaten anything.
Power requires repeated commitment to one side.

See [controls.md](controls.md) for the full input map.

## Coming back

Two return paths at different prices:

| Path | Cost | Speed |
| --- | --- | --- |
| Cast toward the far side | Weak effect — you are casting against the grain | Slow |
| **Auto attack the far side** | Requires melee range, and requires landing it | Fast |

This preserves the melee-mage identity — closing distance is how you recover *quickly*, which
forces the class into melee exactly when it is most powerful and most fragile — while meaning
you are never helpless if you cannot get there.

## The burn

Past a depth threshold on either side, the Dual mage **takes damage over time**, scaling
with distance from centre. This is the force burning them from the inside.

Deliberately **not** a heal-on-exit. Relief comes from *stopping*, not from a reward — get
back inside the line and the burn ends. Same mechanical function as a heal (it bounds how
long the edge can be ridden) with the correct emotional beat for a containment story, and
it puts a real-time clock on the edge.

## The execution test — vote weight scales with power

**Stronger abilities push the meter harder.** Neutral pokes barely move it. A deep finisher
nearly throws you over the edge.

This is what turns the fantasy into a skill. There is a precise meter position from which
each finisher can be cast survivably, and it differs per ability. Knowing where that line
sits, under pressure, while also managing physical position, is the mastery curve.

The player is solving a two-dimensional positioning problem: where they stand, and where
they sit on the meter.

## Ascension

**Accepted in shape.** A timed nova: fast, short, the ride is the reward, and you fall off a
cliff if you do not execute. Numbers below need the prototype.

### Entry — there is no ascend button

Ascension happens **when you max the bar by casting**. It is not a separate input, and it is
not free. You drove there, one cast at a time, which is what keeps it a decision without
making it a button you mash on cooldown.

**Built 2026-09-13, as a clock and nothing else.** Reaching either end starts
`tuning::ascension_frames` — three seconds — during which `tuning::ascension_drain` a frame
comes off her health, nothing steers the bar, and when it runs out she is put back at the
centre and staggered for `tuning::ascension_stun`. The drain over the whole window is about
seventy per cent of a health bar, which is inside the "half to all of it" this document asks
for below.

Everything else here is still unbuilt: no refund on casting or hitting, no larger form of each
ability, no graduated stun. What was wrong before it was built is worth writing down, because
it is the failure mode any resource with an edge has: **there was no exit.** Riding to the end
of the bar burned her down to one health and then went on burning, with no clock, no stun, no
reset, and no signal that anything had happened. A cost with no end is not a cost, it is a
broken state you play around.

### While ascended — roughly three seconds

- **Your health drains rapidly.** Somewhere between half and all of your bar over the
  duration, depending on how it feels. **This drain is the clock** — no separate timer.
- **Casting pulls some health back. Landing a hit pulls back more.**
- **No dodge, no block, no cancel.** Every defensive option is gone.
- **Movement is ability-driven only.** You move by casting, using the dashes and blinks in the
  kit. Offence and mobility become the same resource.
- **Abilities fire in their largest form.** You do not choose power level.

Tying the refund to casting *and especially to hitting* is what makes this work moment to
moment. You are not filling a quota to be checked at the end — you are staying alive one
connection at a time, and every whiff is felt immediately.

### Loss of control, without loss of input

**Loss of control does not mean loss of input. It means losing the ability to decline.**

In a fighting game, control *is* the option to not commit — to wait, block, dodge, reposition,
do nothing. Remove that, and you have genuine loss of control with every input still
mattering completely. The analogy is a car with the accelerator stuck: you still steer, you
cannot stop.

### Exit — graduated, not binary

At the end, **you are stunned if you did not reach the damage threshold, and the closer you
got, the shorter the stun.**

A continuous landing is much better than a pass/fail one. A near-miss reads as a near-miss
rather than a disaster, which is what lets players learn the timing instead of fearing it. It
also softens the mode-scaling problem — the difference between a coop boss and a mobile duel
opponent becomes a matter of degree rather than success versus catastrophe.

### Counterplay

This is what keeps it from degenerating into a race to nova every match. Opponents beat it by:

- **Playing defensively and evasively** — deny the hits, and the health refund never comes.
- **CC** — a stagger or root during the window is devastating, because you cannot dodge out.
- **Strategic blocking** — eat the damage on a shield to deny the threshold.

Every class and every monster has at least one of these, so it is always answerable. Note the
class-level consequence: **the Bulwark is ascension's hard counter**, since blocking is
exactly the tool that denies the threshold.

You win with it by **timing it for when your opponent is already vulnerable** and comboing
them into oblivion — not by reaching it as fast as possible.

### Why this does not require doing more of what caused it

Normal deep play **accumulates** — you are gathering power. Ascension **vents** — you are
getting it out of you, and the refund on hit is that expulsion paying you back. Different
verbs, so casting your way out is coherent rather than circular.

### Separation from the normal loop

| | Push out | Come back |
| --- | --- | --- |
| **Normal play** | Cast toward a side | Far-side casts (slow), far-side autos (fast) |
| **Ascension** | (n/a — no meter) | Survive the drain; graduated stun on exit |

You never auto your way down from ascension — that is the anticlimax, and it is structurally
excluded, since during ascension the meter is not the operative resource.

### Asymmetry between the sides

- **Light ascension** — burst and zone flavour. Refunds come from large clean hits. A gamble
  on landing reads.
- **Dark ascension** — drain and lifesteal flavour. Refunds come from sustained contact. A
  gamble on staying attached.

Different failure modes, same shape, which makes which edge you ride strategic rather than
cosmetic.

### Open on ascension

- The drain percentage, the refund rates, the threshold, and the stun curve. All prototype
  questions.
- Whether the threshold should scale with target count, so coop and versus feel comparable.

## Open questions

- **Does the colour want to be visible on the character rather than only in the bar?** It
  decides what her abilities are made of, and it is currently readable from the HUD and from
  which arm she last punched with. A caster whose *hands* say it would not need either.
- **Should the finisher have a tier of its own?** See the table above.
- Naming for the two forces. The existing skill lists carry a light/judgement vocabulary
  (Judgement, Eclipse, Dark pulse, Culling, Mark of the Merciful) worth mining.
- Whether low-tier abilities need a spam check beyond frame data, given they barely move the
  meter.
- Whether the far-side cast at depth should be merely weak, or gated entirely past some
  distance. Weak is friendlier and keeps the escape hatch always open.
