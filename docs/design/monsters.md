---
status: proposed
proposed: 2026-09-11
---

# Monsters — the Ridgeback

The first monster, end to end: a body you can stand on, six moves, a control
algorithm that decides between them, and a way to measure whether the fight is
any good.

Everything here is one creature. The plural in the filename is a promise, not a
claim — a second monster should reuse the body, ride and control machinery and
supply only its own parts and moves.

---

## 1 · What the fight is

**You cannot reach the thing that kills it from the ground.**

The Ridgeback is a quadruped roughly nine metres nose to tail. Its head, flanks
and tail are armoured; attacks there are chip damage. The soft spot is the
**ridge**, a strip of unarmoured back between the shoulders and the haunch, two
and a half metres up. There are two ways to touch it: a well-timed jump, which
buys one hit and leaves you falling in front of a monster, or **standing on the
creature**, which buys as many as you can take before it throws you off.

So the loop is:

1. **Ground phase.** Bait a move, dodge it, punish the recovery. Chip damage,
   and it builds toward a leg break.
2. **Mount.** Get up. The tail sits low after it sweeps; the arena's platforms
   are level with the flank; a foreleg drops when it rears.
3. **Ride.** Walk up the back to the ridge and hit it. Your controls are
   relative to the surface under your feet — the creature turning does not turn
   your movement.
4. **Buck.** It notices, and the moves change. Some you can brace against.
   Some throw you regardless, and those are the ones you leave for.
5. **Topple.** Enough ridge damage breaks its poise and it falls over. That is
   the big window, and it is the reward the climb is for.

Each of its moves has a *different* answer. That is the whole test of a
monster's move set, and it is the one the fight report measures directly.

| Move | What it is | The answer |
| --- | --- | --- |
| Bite | Head lunges forward | Sidestep, or dodge through it |
| Stomp | Foreleg comes down in front | Do not stand in front; back off |
| Tail sweep | Wide arc behind and to the sides | Jump it, or be at the head |
| Charge | Commits to a straight line | Step laterally late — it cannot turn |
| Rear and slam | Both forelegs up, then down | Long telegraph; get out from under |
| Shake | No damage; pure buck | Brace, or get off |

## 2 · The body is a set of boxes in the creature's own frame

Parts are axis-aligned boxes in **body space**: `+x` forward toward the head,
`+y` up, `+z` to the creature's right. The body's placement in the world is a
position, a **yaw** and a **pitch** — no roll, because nothing the creature does
rolls it and a third angle would cost a full rotation matrix for a motion that
never happens.

Local to world, with `θ` yaw and `φ` pitch (nose up positive):

```text
pitch first, in body space:   x' = x·cosφ − y·sinφ
                              y' = x·sinφ + y·cosφ
                              z' = z

then yaw, into the world:     X = px + x'·cosθ − z'·sinθ
                              Y = py + y'
                              Z = pz + x'·sinθ + z'·cosθ
```

Both rotations go through the existing `sin_turns` / `cos_turns` table, so the
creature costs the simulation no new determinism risk.

Because the parts are axis-aligned *in body space*, colliding a player with the
creature is the same routine as colliding them with the arena: transform the
player into body space, resolve against boxes by least penetration, transform
back. One collision rule, two frames of reference. Walking into a leg stops
you; landing on the back puts you on it.

### Parts, and what each is for

| Part | Mountable | Armour | Why it exists |
| --- | --- | --- | --- |
| Head | no | heavy | The bite's hitbox |
| Neck | no | heavy | Joins the two, and blocks the jump-in to the ridge from the front |
| Flank, left and right | **yes** | medium | Level with the arena platforms — the shortcut up |
| **Ridge** | **yes** | **none** | The weak point. What the climb is for |
| Haunch | **yes** | medium | The step between the tail and the ridge |
| Tail base | **yes** | medium | Low enough to jump onto. Sweeping throws you off it |
| Tail tip | no | heavy | The sweep's hitbox |
| Foreleg, left and right | no | light, **breakable** | Break one and it turns worse toward that side |
| Hindleg, left and right | no | light | Holds the back end up |

Armour is a damage multiplier per part. "Heavy" is not immunity — it is the
difference between a two-minute fight and a twenty-second one, which is the
difference the design document already asks for.

### Articulation is five numbers

A move moves the body, and the body carries whoever is standing on it. Rather
than a skeleton, the pose is five scalars, each a pure function of the action
and how many frames into it the creature is:

```text
pose = (yaw_extra, pitch, bob, tail_swing, head_reach)
```

`yaw_extra` and `pitch` rotate the whole creature; `bob` lifts or drops it;
`tail_swing` is an extra yaw applied to the tail parts alone, about the tail's
own pivot; `head_reach` slides the head and neck forward along `+x`.

Five is enough to make every move read from across the arena and — the part
that matters — enough to generate real shear under a rider. It is deliberately
not a skeleton: a skeleton would have to be snapshotted, and this is recomputed
from `(action, frame)` every tick, so a rollback reproduces it exactly.

## 3 · The ride

**While mounted, the player's position in the creature's body frame is the
authoritative one and the world position is derived from it.** Unmounted, it is
the other way round. That is the whole rule, and it is why the creature can spin
under you without sliding you off.

Three consequences, each load-bearing:

**Movement is relative to the surface.** The creature's yaw change each tick is
added to the rider's own look angle, as a `carry_yaw` that lives in the
snapshot. Movement is camera-relative already, so carrying the look angle makes
movement body-relative for free, and the camera turns with the creature rather
than the creature turning away underneath the camera. One mechanism, both
effects. The raw aim on the wire is never rewritten — `carry_yaw` is simulation
state, recomputed from the snapshot, so rollback reproduces it.

**Mounting is landing, not a button.** Fall onto a mountable part and you are on
it, the same way falling onto a platform puts you on the platform. Walk off the
edge of every mountable part and you are airborne again, carrying the creature's
velocity with you.

**Jumping leaves.** A jump converts the local position back to a world one,
adds the surface's world velocity, and hands you to ordinary air movement. That
is what makes leaving a real option rather than a special case.

### The buck is acceleration, not a flag

A rider comes off when **the surface under their feet accelerates harder than
they can hold on through**.

That is the honest rule, and it is worth the small cost of computing it, because
the alternative — tagging each move "this one throws riders" — authors the same
fact twice and lets the tag drift from the animation. With acceleration as the
rule, a move that whips the tail throws whoever is on the tail, and the same
move does nothing to someone standing on the shoulder, without anyone writing
that down.

Concretely: the world position of a rider's mount point is known this frame and
was known last frame, so its velocity is a subtraction and its acceleration is
one more. The acceleration is taken **in body space**, where the surface normal
is `+y`, and the throwing component is everything except acceleration pressing
the rider *into* the surface:

```text
throw = |(a_x, a_z, max(a_y, 0))|
```

Above `grip`, the rider is launched with the surface's own velocity plus a kick
along the normal, and spends a few frames unable to act.

**Crouch is brace.** Crouching multiplies grip — the answer to a shake that is
neither "dodge" nor "leave", and a real decision because bracing costs you the
attack you were about to throw. Some moves exceed even a braced grip. Those are
not meant to be survived on the creature's back; they are meant to be *left*,
which is what makes reading the creature's startup worth doing while you are on
top of it.

## 4 · The control algorithm

Three properties decide whether a monster is hard in a way anyone enjoys:

1. It must **commit**, so that whiff punishment exists.
2. It must **telegraph**, so that the decision the player is being asked for can
   actually be made.
3. It must not **read your inputs**, so that a good read stays good.

### It glances, rather than watching

The creature does not see the player continuously. Every `glance` frames it
takes one sample — **position and velocity** — and acts on that sample until the
next one. In between, it is working from stale information.

Aiming at where you were is how a predator misses, so the sample is
extrapolated: it aims at `seen_pos + seen_vel · lead`. Two knobs, and between
them they are the whole difficulty model:

- **`glance`** — how fresh its information is.
- **`lead`** — how well it extrapolates from it.

A short glance with a long lead is frightening. A long glance with no lead is an
animal you can walk around. And the skill the pair rewards is specific and
learnable: **change direction between its glances**. That is a thing a player
can notice, get better at, and explain to someone else, which is the test of
whether a difficulty knob is a difficulty knob or just a number.

It is also why the creature never reads a button. It cannot: it is working from
a position and a velocity that are already several frames old.

### It scores its moves and then does not always pick the best one

At a decision point — free, and the think timer expired — every move is scored:

```text
U(m) =  range(m)        how well the current distance suits the move
      + arc(m)          whether the target is in the move's cone
      + rider(m)        enormous for the bucking moves when someone is aboard
      + variety(m)      a decaying penalty on the move it just used
      + hurt(m)         wounded animals commit harder
```

`range` is a tent function peaking at the move's ideal distance and reaching
zero at the edges of its window — a tent rather than a bell because two
subtractions is the whole of it and nothing here needs a curve. `arc` is the
dot product against the move's own cone, which is what makes standing at the
creature's flank *mean* something: the bite cannot score there, the tail sweep
can.

Selection is **not** the maximum. Everything scoring at least `decisiveness × best`
goes into a weighted draw, and one is pulled with a generator seeded from the
snapshot. `decisiveness` at one is always the best move and therefore a script
you can memorise; at zero it is noise. In between, the *distribution* is
learnable while the next move is not, which is the only definition of "hard but
fair" that survives contact with a player who has fought the thing fifty times.

### Turning is the fight

A creature this size cannot snap around, and that limit is not flavour — it is
where the player's advantage lives. Yaw is driven by a proportional controller
behind two limits:

```text
error          = wrap(desired_yaw − yaw)          into ±half a turn
desired_rate   = clamp(error · turn_gain, ±turn_rate_max)
yaw_rate      += clamp(desired_rate − yaw_rate, ±turn_accel · dt)
yaw           += yaw_rate · dt
```

The rate limit is how fast it can turn. The **acceleration** limit is what gives
it mass: it cannot reverse a turn instantly, so it overshoots slightly when it
has been swinging hard, and a player who cuts back across its nose gets a window
that a rate limit alone would not have given them. Committed moves lock the yaw
entirely, for exactly the reason the players' moves do.

A broken foreleg drops `turn_rate_max` toward that side. The player can see the
consequence of a thing they did, which is the only reason to have breakable legs
at all.

### Poise, and the topple

Ridge damage fills a poise pool. Full, the creature **topples**: a long,
threat-free recovery, during which the ridge takes extra damage and everything
else is irrelevant. Poise regenerates while it is standing, so the window has to
be earned again rather than accumulated over a fight.

The topple is what the climb is *for*. Without it, riding is a damage-per-second
choice and the fight is arithmetic; with it, riding is a wager on reaching the
ridge before the next buck, and that is a decision.

## 5 · Judging whether the fight is any good

"Feels good" is not a test. What can be tested is whether the fight has the
properties a good fight needs, and `cargo run -p sim --bin fight` plays a
scripted hunter against the creature and reports them.

| Measure | Why it says something | Where it should land |
| --- | --- | --- |
| **Reactable share** | Fraction of the creature's attacks with startup at or above human reaction — about 15 frames. All of them, and the fight is a metronome; none, and it is memorisation | a majority, not all |
| **Openings per minute** and their **mean length** | A fight is a rhythm of threat and answer. An opening shorter than the player's fastest meaningful attack is not an opening | openings long enough to punish |
| **Idle share** | Frames the creature is doing nothing at all | low, but not zero |
| **Move entropy** and **longest repeat** | Shannon entropy over the move distribution. A one-note monster scores near zero however hard it hits | high enough that no move dominates |
| **Move coverage** | Moves it never once used are design fiction | every move used |
| **Ride share** and **mean ride length** | Long enough to reach the ridge and hit it; short enough that the back is not a safe room | bounded, and not trivial |
| **Time to kill** | The design document asks for one to twenty minutes coop | inside the band |
| **Unanswerable hits** | Hits taken from an attack whose startup was below reaction *and* which the player had no positional warning of | as close to none as the design allows |

The last one is the important one and the easiest to get wrong. A monster can
score well on every other line and still feel cheap, and it will be because of
damage the player had no way to avoid. It is measured rather than argued about.

`crates/sim/tests/fight.rs` pins the ones that are design decisions rather than
observations, in the style of the rest of the feel harness: relationships, not
values.

## 6 · Deliberately not yet

- **A second monster.** The machinery is built to be shared, but a second one
  is what proves it, and it should be built when there is something to learn
  from it.
- **Baked animation.** The pose is procedural. The animation factory in
  `crates/anim` bakes clips from spring recipes and the creature should
  eventually go through it; nothing about the pose function's shape changes when
  it does.
- **Coop tuning.** Both players can fight it, and both can be on it at once, but
  the numbers are set for one. A monster tuned for two is a different monster.
- **Roll.** The creature yaws and pitches. A body roll would be a third angle
  and a full rotation matrix, for a motion nothing in the move set makes.
