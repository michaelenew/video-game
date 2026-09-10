---
status: mostly decided; ascension is an open proposal
decided: 2026-09-10
formerly: Statera
supersedes: docs/combat-design/statera-skills.md (resource system), docs/combat-design/class-builds.md (Statera section)
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

## Every ability has two forms, and you choose at cast

**Each ability exists in a Light form and a Dark form. The player picks which one they are
casting, every time.** Casting a form pushes the meter toward that side.

Power scales with meter depth rather than snapping between states:

| | At centre | Deep on that side |
| --- | --- | --- |
| Step (Light) | Short blink, minor damage | Long blink, damage and blind |
| Step (Dark) | Short dash, minor drain | Long drain-dash, large steal |

**Centre is not a third form.** It is the position where both forms are available and both
are weak — most options, least power. That property now falls out of the mechanic instead
of being asserted.

### Why this, and not a neutral form at centre

An earlier draft gave each ability three states — a neutral behaviour at centre, plus Light
and Dark at depth. That breaks at zero: a neutral form has no side, so at centre nothing
votes and the meter cannot leave the middle. Two incompatible ideas had been merged —
*abilities have sides* and *abilities transform with position*. They work at depth and fail
at the origin.

Two forms chosen at cast fixes it and makes the class's stated identity literal: **which
abilities you use votes for which side you are moving toward.**

### Oscillating is possible, and correctly weak

Cast Light, cast Dark, sit at centre indefinitely — and never threaten anything. Power
requires repeated commitment to one side. That is the right shape, and because the player
is always choosing rather than being pushed, nothing is ever taken away from them.

### Input

**Tap the ability key for one side, hold for the other.** No extra keys, and hold already
exists in the game's vocabulary.

The alternative worth keeping in mind is **two hands** — left casts Dark, right casts
Light, which is thematically ideal and uses the existing separate left/right hand actions.
It becomes the better option if the class ever wants a hold-to-charge ability, since that
would collide with hold-to-invert.

## Coming back

Two return paths at different prices:

| Path | Cost | Speed |
| --- | --- | --- |
| Cast the far-side form | Weak effect — you are casting against the grain | Slow |
| **Auto attack** | Requires melee range | Fast |

Auto attacks pull toward centre. This preserves the melee-mage identity — closing distance
is how you recover *quickly*, which forces the class into melee exactly when it is most
powerful and most fragile — while meaning you are never helpless if you cannot get there.

Note the auto idea already exists in the source notes as "Auto attacks remove resource,
gaining extra range and scaling with magic damage."

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

## Ascension — OPEN PROPOSAL

> **Status: unresolved.** The section below is a proposal for review, not a decision. The
> design constraints beneath it are settled; the mechanism is not.

### Constraints this must satisfy

1. Ascension is a **strategy** — hard to live through, but sometimes the right call.
2. Immense damage, immense personal risk, all-or-nothing.
3. A second or two before everything collapses.
4. It must be **repeatable** across a match. A biggest-coolest-moment behind a threshold
   you can only cross once does not work.
5. The exit must be a **big bang**, not an anticlimactic scramble back down with autos.
6. It must *feel* like loss of control while remaining mechanically satisfying.
7. **No RNG.** Random output turns the tense last moments of a match into a dice roll.
8. Thematically coherent — it should not require the player to do *more of the thing that
   got them there* in order to escape it.

### The reframe

**Loss of control does not mean loss of input. It means losing the ability to decline.**

In a fighting game, control *is* the option to not commit — to wait, block, dodge,
reposition, do nothing. That is the entire neutral game. Remove the option to not act, and
you have genuine loss of control with every input still mattering completely.

The analogy is a car with the accelerator stuck to the floor. You still steer. You cannot
stop.

### Entry

Ascension is **voluntary**, triggered by an input that becomes available once the meter is
past a deep threshold on either side. The player chooses it, knowing the quota and the
clock.

If the meter reaches the hard cap **without** the player choosing, ascension triggers anyway
on worse terms — reduced clock, or an increased quota. This keeps meter management
meaningful while making the deliberate entry the skill expression.

### While ascended

Duration is short — target ~3 seconds, tunable.

- **No dodge, no block, no cancel.** Every defensive option is gone.
- **Movement is ability-driven only.** Normal locomotion is suppressed; you move by casting,
  using the dashes and blinks built into the kit. Offence and mobility become the same
  resource.
- **Abilities fire in their largest form.** The player does not choose power level.
- **A hard clock.** The state is collapsing from the moment it starts.

Every frame pushes the player forward. They cannot back off, wait, or defend. The only
available verb is *attack* — loss of control, felt precisely, with full agency retained.

### The quota and the discharge

Ascension carries a **damage quota**.

- Meet it before the clock expires → **discharge**. A large detonation that expels the force
  and returns the Dual mage to centre. The big bang, and both the mechanical reward and the
  visual payoff.
- Miss it → the state collapses. The Dual mage is dumped to centre at a sliver of health
  with the burn lingering briefly. In a 60-second match this is usually losing, but not
  automatically fatal.

### Why this does not violate constraint 8

The objection to "abilities remove resource once ascended" is that it makes no sense to do
more of the thing that caused the problem. The resolution is that these are **two different
verbs**:

- Normal deep play: abilities **accumulate**. You are gathering power.
- Ascension: abilities **vent**. You are getting it out of you.

The discharge is expulsion, not accumulation. You are frantically dumping the force into the
world because that is the only way to survive holding it.

### Separation from the normal loop

| | Push out | Pull back |
| --- | --- | --- |
| **Normal play** | Cast toward a side | Far-side casts (slow), autos (fast) |
| **Ascension** | (n/a — no meter) | Discharge on quota |

You never auto your way down from ascension — that is the anticlimax, and it is structurally
excluded, since during ascension the meter is not the operative resource. Because the
discharge returns you to centre, ascension is repeatable within a match.

### Asymmetry between the sides

- **Light ascension** — burst and zone flavour. Quota met by large clean hits. A gamble on
  landing a read.
- **Dark ascension** — drain and lifesteal flavour. Quota met by sustained contact. A gamble
  on staying attached.

Different failure modes, same all-or-nothing shape, which makes the choice of which edge to
ride strategic rather than cosmetic.

### Known risk — quota scaling across modes

A flat damage quota is trivially easy to meet in a 20-minute coop fight against a large
stationary boss, and very hard against one mobile player in versus. Left unaddressed,
ascension becomes a safe rotation button in coop and a desperation play in versus.

Options, undecided: scale the quota with encounter or target count; express it as a fraction
of a target's health; or shorten the clock enough that the big abilities must land cleanly
even against a boss.

## Open questions

- Naming for the two forces. The existing skill lists carry a light/judgement vocabulary
  (Judgement, Eclipse, Dark pulse, Culling, Mark of the Merciful) worth mining.
- Whether low-tier abilities need a spam check beyond frame data, given they barely move the
  meter.
- Whether the far-side cast at depth should be merely weak, or gated entirely past some
  distance. Weak is friendlier and keeps the escape hatch always open.
