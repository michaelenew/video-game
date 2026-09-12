---
status: decided
decided: 2026-09-12
implements: the hit reaction the whole combat kernel assumed
---

# Stun — what happens to whoever got hit

Every point of damage in this game stuns, interrupts and shoves. That sentence is the
whole mechanic, and everything below is a consequence of it.

Modelled on Smash, deliberately and closely. The [combat kernel](combat-kernel.md) already
says the mechanical test is "Smash, not Tekken", and the hit reaction is where that choice
actually lives: it is the difference between a fight made of *exchanges you commit to* and
a fight made of input strings.

## Not stagger

These are two different mechanics and the names are easy to confuse.

| | **Stun** | **Stagger** |
| --- | --- | --- |
| **When** | Every time you take damage | Only when you are read — a parry today, a guard break later |
| **How long** | A few frames to half a second | Long enough to cash in the slowest punish |
| **Where it leaves you** | Wherever the knockback puts you | Where you were standing |
| **Why it happened** | Someone hit you | You made a mistake |

Stun is the *tax on being hit*. Stagger is the *prize for a correct read*. Stun is
constant and small; stagger is rare and decisive. See [defense.md](defense.md) for stagger.

## The four things a hit does

### 1 · Freeze — hitlag

Both fighters stop dead for a few frames, scaled by the damage. Nothing advances: not the
attacker's move, not the victim's stun, not gravity.

The frames are handed back to both sides, so **hitlag is not frame advantage for anyone**.
What it buys is that a hit reads as a collision rather than as a number going down. The
renderer gets this free — positions and poses are simulation state, so a frozen fighter is
a frozen picture without the renderer knowing the mechanic exists.

It also buys the victim something, which is the next-to-last section.

### 2 · Stun — the interrupt

Whatever the victim was doing ends. The move they were in, the hang an aerial was holding
them in, the jump they were still sustaining, the crouch they were ducking under something
with. They cannot act again until the stun runs out.

**This is what enables combos.** A hit that only subtracted health would leave both
fighters swinging past each other and the winner of an exchange would be whoever pressed
first, not whoever landed first. Taking the turn away is what makes a second hit something
you can *guarantee* rather than race for.

### 3 · Knockback — the shove

The victim is given a velocity along the attacker's facing, plus whatever upward launch the
move carries, and it decays while the stun runs.

Knockback is why a combo eventually stops working, and it is also the reason blocking costs
space rather than health — the same shove, at a smaller number, applied to a guard.

### 4 · The swell — damage makes the next hit bigger

This is Smash's percent, in a game whose bar counts down instead of up.

A move's `hitstun` and `knockback` in the move table are **what it does to someone
untouched**. Both are multiplied by a factor that grows with the fraction of health the
victim has already lost:

```
knockback = table value x (1 + hurt x (rise + blow x damage share)) / weight
hitstun   = table value x (1 + hurt x stun rise)
```

`hurt` is zero at full health and one at death. Two terms appear in the knockback growth
for the same reason they do in Smash: the first grows with what the victim has taken, and
the second with the damage of the blow that just landed, so a committed move scales harder
against a hurt opponent than a poke does. Without the second term, every move would swell
by the same factor and a jab would become a launcher purely by being thrown late.

Damage is applied **before** the swell is read, exactly as Smash does it. The practical
consequence is that the killing blow of a round lands with the largest swell there is, so
the body goes a long way and both players can see the round end.

## The arc this produces

The gap between the two growth rates is the entire combo design. Knockback swells far
faster than hitstun does, so the distance a victim covers outruns the window their attacker
has to follow them:

| Point in the round | What a hit does |
| --- | --- |
| **Full health** | Short stun, small shove. The exchange resets and neutral starts again |
| **Middle** | Stun has grown, knockback has not yet outrun it. **Moves link** |
| **Near death** | The victim is thrown clear before the attacker recovers. The link is a launch |

So combos are a *mid-round* phenomenon. They are not available immediately, they are not
available forever, and the window closing is what stops a fight ending in a stunlock.

`combos_open_up_mid_round_and_close_again` in `crates/sim/tests/combat.rs` plays this out
against the real simulation rather than asserting it from arithmetic, because it depends on
walking speed, hitbox reach, recovery frames and the decay curve all at once, and any
closed form of it would be a lie in at least one of those places.

## Directional influence — the skill in it

**The victim chooses where the knockback puts them. They never choose how much of it they
take.**

The direction they are holding on the last frozen frame bends the launch, by at most about
twenty degrees. The whole freeze is the window to decide in, which is what makes hitlag part
of the skill system rather than only part of the presentation.

The rule is one projection, and it is the same trick the Quake air control uses. Take the
direction the victim is holding and subtract the part of it pointing along the launch; what
is left is the part lying *across* it. Add that to the launch and renormalise. Hold the way
you are already being sent and there is nothing left over, so nothing happens. Hold square
to it and you get the full bend.

The speed is restored exactly afterwards. That asymmetry is the design: influence is a
positioning read — away from a follow-up, toward the middle of the arena, along a wall
instead of into it — and never an escape. A victim who could shorten their own knockback
would have a defensive option that costs nothing and is always correct.

Influence does not apply to a hit you blocked. Blocked pushback is a fixed price in ground
(see [defense.md](defense.md)) and letting it be aimed would quietly turn blocking into a
movement option.

## Weight

Each class divides incoming knockback by its own weight. The Bulwark is the heaviest thing
on the roster and the Dual mage the lightest.

Weight divides knockback and **not** hitstun, which is a deliberate departure from Smash.
Smash derives hitstun from knockback, so weight moves both; here frame data is authored per
move, and a class-dependent hitstun would mean the frame table's "on hit" column stopped
being a property of the move. The cost of that choice is the interesting part: a heavy takes
the same stun and travels less far out of it, so **heavies are combo food**. That is the
same answer a platform fighter gives, arrived at from the other direction.

Weight also delivers, for free, a line that was already written down in
[defense.md](defense.md): *"The Bulwark resists pushback."* Blocked knockback is divided by
weight like everything else, so the class trait falls out of the class being heavy rather
than being specified twice.

## Every point of damage, including the ones that are not attacks

A fire pillar tick and a drain field tick stun and shove like anything else. They go through
the same code path as a sword — one funnel, `state::strike`, which is what makes "damage
stuns" a rule of the game rather than something most damage happens to do.

A field that only drained health would be a number going down in the corner of the screen.
A field that stuns and shoves is *a piece of ground taken away from you*, which is what an
Elementalist and a Blood mage are supposed to be doing.

Two rules keep that from becoming a trap with no exit:

- **A fighter already frozen cannot be hit again until the freeze ends.** Nobody can exploit
  the window, because being frozen is precisely being unable to act.
- **No repeating source of damage may hold you for longer than its own interval.** Pinned by
  a feel test. A field that stunned for longer than its cadence would land its next tick on
  someone who never got to move, and the player who laid it would not have to be there for
  any of it.

**Damage you do to yourself is not a hit.** The Blood mage pays health for its kit and the
Dual mage burns at depth; neither stuns its owner. What self-damage *does* do is swell the
knockback they take from everyone else, which is a real cost and the right one — a class
that sustains through aggression should be easier to launch for it.

## What is not in yet

- **Stale moves.** Smash weakens a move that has just been used, which is what stops a combo
  being the same button four times. Worth having; it belongs to damage, not to stun, and it
  wants the move table to grow past three moves a class first.
- **Smash directional influence.** Shifting position during the freeze itself, on top of
  bending the launch after it. It is a mashing mechanic, and mashing is the part of Smash's
  stun system least worth copying.
- **Tumble, knockdown and teching.** At some knockback threshold a victim should hit the
  ground rather than land on their feet, with a timed input to recover. This is the obvious
  next piece, and it needs the arena and the aerial game settled first.
- **A guard meter and guard breaks.** Specified in [defense.md](defense.md), still unbuilt.
  Guard break is a stagger, not a stun.

## Open numbers

None of these can be reasoned out, and all of them are in the Oven under **Stun**.

- **Is the combo window in the right place?** It currently opens around half health and has
  closed by the last fifth. Whether that reads as "the fight is heating up" or as "the rules
  changed halfway through" is a question for a person with a controller.
- **Is twenty degrees of influence findable?** Too small and nobody learns it exists; too
  large and it is an escape.
- **Does the freeze read as impact or as a hitch?** Four to eight frames by damage. The
  heaviest move in the game freezes for less than reaction time, which is the bound the feel
  test holds, but the floor may be too low to feel at all on a poke.
- **Knockback decay moved from 0.86 to 0.93 for this work**, which roughly doubles the
  distance a shove carries. It is what gives the swell teeth — at 0.86 a hit at death moved
  a fighter about two metres, which is not a launch — but it changes how every existing
  knockback number reads, blocked pushback included. See [feel-log.md](feel-log.md).
