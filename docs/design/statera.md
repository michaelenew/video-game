---
status: mostly decided; ascension is an open proposal
decided: 2026-09-09
supersedes: docs/combat-design/statera-skills.md (resource system), docs/combat-design/class-builds.md (Statera section)
---

# Statera

## Fantasy

The Statera contains **two immense forces**, each of which would individually kill the
human trying to hold it. They sit in the balanced tension between them. Lose control and
you lose your life.

The play fantasy is to be **just on the line** — to wield as much power as you can by
executing right at the edge of where you can still barely pull yourself back.

This is a containment story, not a channelling story. The human is not a conduit; they
are a vessel under load.

## The meter

A single bar with a centre and two ends.

```
  Dark  <——————————————[ centre ]——————————————>  Light
   (extreme)                                        (extreme)
```

- Casting an ability **pushes the meter toward that ability's side**.
- **Power scales with distance from centre**, symmetrically. Centre is the weakest place
  to stand. The ends are the strongest and the most lethal.
- There are no discrete zones. It is a continuous gradient, which is what removes the
  dead-zone problem in the old three-state design.

The old design put human, balanced, and divine on a single axis of *how much damage and
CDR you get*, which made the middle a strictly worse version of the ends. That is why it
read as a dead zone, and it could not be fixed by tuning, because all three states were
the same quantity.

## Lever 1 — the core kit transforms

Four to six core abilities never leave the bar. Each slot holds one ability whose
**behaviour** changes with meter position — not its numbers, its function.

Example, one slot:

| Meter position | Behaviour |
| --- | --- |
| Centre | Neutral short dash |
| Deep Light | Forward blink that damages and blinds on arrival |
| Deep Dark | Drain-dash that steals health from anything passed through |

Why this and not a stat scale:

- **Nothing is ever removed.** Players are loss averse; they hate a debuff and do not
  mind a different toolkit. The bar always looks the same and the buttons always do
  something.
- **Muscle memory is preserved** across the whole meter.
- **It is art-efficient.** Variations on one animation rather than three separate moves.
- The class plays *differently* on each side rather than *harder*, which is the point of
  a light/dark identity.

## Lever 2 — finishers are gated by depth

Two or three large abilities per side that only become available past a depth threshold
on that side. These are the payoff for riding out, and the reason to leave centre at all.

Combined with Lever 1: continuity of kit, plus something earned by going deep. Nothing is
taken from the player; something is unlocked.

## The return valve — auto attacks pull toward centre

**Auto attacks move the meter toward centre**, from either side. Always available, never
gated, requiring melee range.

This is the piece that makes the system work.

- It forces the melee mage into melee **exactly when they are most powerful and most
  fragile**, which is the right risk shape.
- Thematically, physical contact is the human reasserting itself. You ground yourself by
  hitting something with your hands.
- It kills the dead-zone problem outright. Transiting the centre is fast and active, not
  a toll. Centre is where the whole kit is available in neutral form and where mobility
  lives. It is a gear change, not a wasteland.

Note the idea already exists in the source notes as "Auto attacks remove resource,
gaining extra range and scaling with magic damage."

## The burn

Past a depth threshold on either side, the Statera **takes damage over time**, scaling
with distance from centre. This is the force burning them from the inside.

Deliberately **not** a heal-on-exit. Relief comes from *stopping*, not from a reward —
get back inside the line and the burn ends. Same mechanical function as a heal (it bounds
how long the edge can be ridden) with the correct emotional beat for a containment story.

It also puts a real-time clock on the edge, so "just on the line" is tense moment to
moment rather than only in the abstract.

## The execution test — vote weight scales with power

**Stronger abilities push the meter harder.** Neutral pokes barely move it. A deep
finisher nearly throws you over the edge.

This is what turns the fantasy into a skill. There is a precise meter position from which
each finisher can be cast survivably, and it differs per ability. Knowing where that line
sits, under pressure, while also managing physical position, is the class's mastery curve.

The player is solving a two-dimensional positioning problem: where they stand, and where
they sit on the meter.

## Ascension — OPEN PROPOSAL

> **Status: unresolved.** The section below is a proposal for review, not a decision.
> The design constraints beneath it are settled; the mechanism is not.

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
reposition, do nothing. That is the entire neutral game. Remove the option to not act,
and you have genuine loss of control with every input still mattering completely.

The analogy is a car with the accelerator stuck to the floor. You still steer. You cannot
stop.

### Entry

Ascension is **voluntary**, triggered by an input that becomes available once the meter
is past a deep threshold on either side. The player chooses it, knowing the quota and the
clock.

If the meter reaches the hard cap **without** the player choosing, ascension triggers
anyway, on worse terms — reduced clock, or an increased quota. This keeps meter
management meaningful (being forced in is bad) while making the deliberate entry the
skill expression.

### While ascended

Duration is short — target ~3 seconds, tunable.

- **No dodge, no block, no cancel.** Every defensive option is gone.
- **Movement is ability-driven only.** Normal locomotion is suppressed; you move by
  casting, using the dashes, blinks, and teleports built into the kit. Offence and
  mobility become the same resource.
- **Abilities fire in their largest form.** The player does not choose power level; it is
  chosen for them.
- **A hard clock.** The state is collapsing from the moment it starts.

Every frame pushes the player forward. They cannot back off, wait, or defend. The only
available verb is *attack*. That is loss of control, felt precisely, with full agency
retained — which is what makes it satisfying in a game about control.

### The quota and the discharge

Ascension carries a **damage quota**.

- Meet it before the clock expires → **discharge**. A large detonation that expels the
  force and returns the Statera to centre. This is the big bang, and it is both the
  mechanical reward and the visual payoff.
- Miss it → the state collapses on its own. The Statera is dumped to centre at a sliver
  of health, with the burn lingering briefly. In a 60-second match this is usually
  losing, but not automatically fatal.

### Why this does not violate constraint 8

The objection to "abilities remove resource once ascended" is that it makes no sense to
do more of the thing that caused the problem. The resolution is that these are **two
different verbs**:

- Normal deep play: abilities **accumulate**. You are gathering power.
- Ascension: abilities **vent**. You are getting it out of you.

The discharge is expulsion, not accumulation. You are frantically dumping the force into
the world because that is the only way to survive holding it. Lever 1 makes this visible
— the abilities are already transforming with meter position, so their ascended forms can
read as venting rather than gathering.

### Separation from the normal loop

These are two distinct systems and should not be confused:

| | Push out | Pull back |
| --- | --- | --- |
| **Normal deep play** | Abilities | Auto attacks |
| **Ascension** | (n/a — no meter) | Discharge on quota |

You never auto your way down from ascension. That is the anticlimax, and it is
structurally excluded — during ascension the meter is not the operative resource, the
quota is.

Because the discharge returns you to centre, you can immediately begin building back out.
Ascension is repeatable within a match, satisfying constraint 4.

### Asymmetry between the sides

The two sides should ascend differently, which makes the choice of which edge to ride a
strategic one rather than cosmetic:

- **Light ascension** — burst and zone flavour. Quota met by large clean hits. Wants a
  committed opponent or a big target.
- **Dark ascension** — drain and lifesteal flavour. Quota met by sustained contact. Wants
  to be glued to something.

Light is a gamble on landing a read. Dark is a gamble on staying attached. Different
failure modes, same all-or-nothing shape.

### Known risk — quota scaling across modes

A flat damage quota is trivially easy to meet in a 20-minute coop fight against a large
stationary boss, and very hard to meet against one mobile player in versus. Left
unaddressed, ascension becomes a safe rotation button in coop and a desperation play in
versus.

Options, undecided:

- Scale the quota with encounter or target count.
- Express the quota as a fraction of a target's health rather than flat damage.
- Shorten the clock enough that even against a boss the big abilities must land cleanly.

## Open questions

- Naming for the two forces. The existing skill lists already carry a light/judgement
  vocabulary (Judgement, Eclipse, Dark pulse, Culling, Mark of the Merciful) that can be
  mined.
- Whether the low-tier abilities need a spam check beyond frame data, given no cooldowns
  and near-zero meter movement.
- Which of the ~35 loose skills in `statera-skills.md` map to core transforming slots
  versus depth-gated finishers.
