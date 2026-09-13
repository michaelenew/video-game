---
status: decided
decided: 2026-09-13
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

It buys one more thing. Four to eight frames of a fighter's input being ignored is long
enough to lose a button in, so **whatever was pressed during a freeze is handed back on the
first frame that is not frozen**. The Champion's "we are settling this in the air" leap is
pressed inside exactly that window, and before the buffer existed it was being swallowed.

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

### It is a duel mechanic

**Neither swell applies while the creature is on the field.** In a hunt, a blow does exactly
what the move table says.

Not a balance decision — the mechanic's premise fails in coop. The swell is a fighter's share
of *their own bar*, and it is only fair because the other fighter has a bar the same size and
is running along it at the same rate. Take turns being hurt and you take turns being
launchable, which is why it reads as an arc rather than as a punishment for falling behind.

The creature carries ten times the health. It barely moves along its own bar while the hunter
runs the length of theirs, so the mechanic becomes a one-way ratchet: every hit you take makes
the next one hold you longer and throw you further, against something that is never held longer
in return. Measured, ungated it took the scripted hunt from four wins in six to none — the
hunter spent the fight being thrown off the animal and walking back to it.

The creature's presence is the condition rather than a separate flag, the same way
`effects_reach` already decides whether two fighters can hurt each other at all.

## The arc this produces

The gap between the two growth rates is the entire combo design. Knockback swells far faster
than hitstun does, so the distance a victim covers outruns the window their attacker has to
follow them.

Measured against the built kits, with the attacker chasing:

| Point in the round | Champion | Dual mage |
| --- | --- | --- |
| **Full health** | nothing links | nothing links |
| **Half** | nothing links | **all four Dark/Light alternations link** |
| **A quarter** | Sword and Spear link into themselves | closed again |
| **A tenth** | and into each other | closed |

The Dual mage is the clean case: the window opens in the middle of a round on exactly the
alternation the kit is built around — the two autos are how the meter is steered, so the combo
and the mechanic are the same action — and it has closed by a quarter bar, because the autos
then throw people too far to follow.

The Champion gets no raw-move links until a quarter bar, where they become kill confirms. Its
real combo is the designed one: hammer, cancel the recovery with Rush, uppercut into the air,
leap, spike. That works at any health, because **Rush buys the frames rather than hitstun
doing it** — which is the more interesting way for a class to combo, and the reason the class
has a mechanic at all.

`combos_open_up_mid_round_and_close_again` and
`nothing_links_into_itself_while_the_victim_could_live_through_it` in
`crates/sim/tests/combat.rs` play this out against the real simulation rather than asserting it
from arithmetic, because it depends on walking speed, hitbox reach, recovery frames and the
decay curve all at once, and any closed form of it would be a lie in at least one of those
places.

### No move may loop into itself

The one hard rule underneath the arc. Every class has a fast move, and the frame maths says a
fast move's swelled stun eventually outgrows its own startup — so **distance is the only thing
that ends a self-loop**, which is what the knockback decay is for. Two moves needed their own
numbers changed to satisfy it rather than the system bending around them: the Blood mage's
Bloodletter, whose stun sat inside its own throw cycle with eight frames to spare, and both
field ticks against their twelve-frame cadence.

Late in a round the same loop is a **kill confirm**, and that is a different thing. The bound
the test holds is on damage, not on hits: a chain that only lasts because the victim dies
partway through it is the system working.

## Steering out of it — the skill in it

**A launched fighter keeps their air control.** That is the whole mechanic, and it is
deliberately not a second one.

Smash calls this directional influence and gives it a dedicated pass. The first version here
did too — the direction held on the last frozen frame bent the launch by about twenty degrees.
It was deleted, because the game already has the thing it was imitating. [Quake air
strafing](architecture.md) is *already* a rule that turns your velocity without adding to it:
point where you are already going and the projection leaves nothing to add, point across your
motion and you get the whole budget. That is DI. Writing it twice would have meant two sets of
numbers to keep agreeing with each other.

So what a hit takes away is everything that makes you dangerous — the move, the jump, the
dodge, the guard — and what it leaves is the one tool that only repositions. **Being launched
is a trade rather than a sentence:** you are held longer, and you get your steering back to use
while you are held.

Two rules keep it honest.

**It may never slow you down.** The air speed cap is a rule about how fast a fighter may make
themselves go, and knockback routinely exceeds it. Left alone, holding a direction during a
launch would have halved the hit you just took. Pressing a direction must never be worth less
knockback than pressing nothing, so the speed is restored along the new heading afterwards and
below the cap the whole thing is inert.

**It is an air privilege.** A grounded stun is the plain tax it has always been. Skidding along
the floor is not a moment anybody is flying through, and letting it be steered would hand every
grounded hit an escape.

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
  being the same button four times. Worth having; it belongs to damage rather than to stun, and
  it would be a second answer to the self-loop problem the knockback decay currently handles
  alone.
- **Smash directional influence.** Shifting position during the freeze itself, on top of
  steering after it. It is a mashing mechanic, and mashing is the part of Smash's stun system
  least worth copying.
- **Tumble, knockdown and teching.** At some knockback threshold a victim should hit the ground
  rather than land on their feet, with a timed input to recover. This is the obvious next piece
  now that hits genuinely throw people, and it wants the arena settled first.
- **A guard meter and guard breaks.** Specified in [defense.md](defense.md), still unbuilt.
  Guard break is a stagger, not a stun.
- **Hitlag on the creature's own flinch.** The animal freezes when it is cut and when it
  connects, which is what keeps the exchange fair, but its flinch and poise are a separate
  system from hitstun and do not swell with anything. That is the reason the swell has to be
  switched off in a hunt rather than balanced there.

## Open numbers

None of these can be reasoned out, and all of them are in the Oven under **Stun**.

- **Is the combo window in the right place?** It opens around half health on the Dual mage and
  has closed by a quarter. Whether that reads as the fight heating up or as the rules changing
  halfway through is a question for a person with a controller.
- **Does the freeze read as impact or as a hitch?** Three to eight frames by damage. The
  heaviest move in the game freezes for less than reaction time, which is the bound the feel
  test holds, but the floor may be too low to feel at all on a poke.
- **Knockback decay moved from 0.86 to 0.93 for this work**, which roughly doubles the distance
  a shove carries and is what lets the swell close a self-loop at all. It is also the most
  invasive number here: every knockback in the game is now worth about two and a half times the
  distance its author intended. The two combo classes and the creature were re-checked and the
  creature's numbers scaled to match; **nothing else was**, and blocked pushback in particular
  now cedes noticeably more ground than it did. That is the direction
  [defense.md](defense.md) wanted, and it was not measured against anyone's hands.
- **Is the buffer the right length?** It currently holds for exactly the freeze. A fighting
  game normally buffers through recovery as well, which is a larger decision about how
  forgiving the inputs are.
