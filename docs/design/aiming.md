---
status: decided
decided: 2026-09-12
revised: 2026-09-14
---

# Aiming

**The player's only frame of reference is the crosshair.** Everything below
follows from one requirement: *the thing the reticle is pointing at must be the
thing the attack hits.* Not approximately, and not in a way that drifts with
distance.

This document exists because aiming was got wrong three separate times, and each
time the mistake was the same shape — someone needed to know where an ability
should go, and worked it out next to the ability instead of asking. It is the
specification; [`crates/sim/src/aim.rs`](../../crates/sim/src/aim.rs) is the
implementation, and `crates/sim/tests/one_aim.rs` fails the build if anything
else in the simulation decides for itself.

## The raycast

> **Every skillshot starts with one raycast, from the camera through the
> crosshair, ignoring anything behind the character model. The first thing it
> meets is what the player is pointing at.**

It meets, in one list:

- **terrain** — the floor, the arena's walls and platforms
- **structures** — the Elementalist's stones
- **the max-range sphere**, centred on the casting player, of the ability's own
  range

Whatever it reaches first wins. A future element may add a fourth kind of thing;
nothing today is an exception.

**Ignoring anything behind the character model** matters because the camera sits
behind the shoulder. A wall the camera happens to be looking through is scenery,
not a target, and aiming through your own cover is not a mechanic anybody asked
for.

### Bodies are not on that list

Neither other fighters nor the creature. Changed 2026-09-13, and it is the one
part of the model a player would not guess, so it is worth saying exactly what
the ray is *for*.

**The ray answers "which place in the world is under the crosshair".** A body is
not a place; it is a thing standing in one. So the ray goes through it, stops on
the geometry behind, and the ability crosses the ground that body is standing on
— which hits them, and is the same shot, only aimed at a point that does not
move when they do.

The creature is what proved it. It is large. Up close it fills the screen, so
the crosshair lands on its chest or its head — several metres up and, because
you are right next to it, barely a metre away. Measured, standing five metres
from its centre: **its head was the first thing the ray met at every look angle
in a sweep from 40° below the horizon to 40° above**, including aiming squarely
at the dirt. So every skillshot came out as a stub about a metre long pointed
three metres into the sky, at exactly the range where you cannot miss.

Fighters have the same problem in miniature. Stand nose to nose, and a camera
that sits above the shoulder puts the reticle on the top of somebody's head.

Nothing is lost by the change, because *what a shot runs into* was never this
question — see "What the path runs into" below. That test walks the ability's
own path, not the camera's ray, and the two were never the same line: the camera
is behind and above, so a body it could not see was always still a body the shot
went through.

**Fire is also not on the list**, for its own reason: you can see through flame,
so a fire pillar never steals the crosshair — but a shot that *travels through*
one still notices it.

### Ground, and everything else

The one distinction the raycast draws is whether the surface it hit **faces
upward**: the floor, the top of a platform, the top of a stone. That is "the
ground". The side of a platform, the side of a stone and the range sphere are
not. Bodies do not come up: they are not on the ray.

## The four lines of effect

Two are **skillshots**: they start with the raycast above and go where it lands.
Two are not: they are pointed by something the player already decided — which
way their body is facing, or where they put the mechanic — and consult nothing.

Every move declares which, in the move table. There is no fifth.

## The two kinds of skillshot

### Grounded

Things that come out of the floor, whatever the caster was doing when they cast
them: a structure, a fire pillar.

- **Hit the ground** — cast it *exactly* there. Not a pixel different.
- **Hit the max-range sphere** — cast it at max range on the ground, in the
  direction the mouse is facing. Settling the sphere's own point instead would
  make an upward aim land short, which reads as the ability refusing to go where
  it was pointed.
- **Hit anything else** — a body, a wall — it drops to whatever is underneath,
  because the thing being placed can only exist on the floor.
- **If it travels**, it travels from the character model to that point.

### Not grounded

Things that fly: the Elementalist's auto, the Bulwark's thrown shield, and the
Blood mage's Bloodletter and Grasp — the last two throw something that then
travels on its own, so what the crosshair gives them is the *line* rather than a
landing spot. The Grasp then picks a point along that line with a channel — see
[Aiming with time](#aiming-with-time).

- **Hit the ground** — raise that spot to the **middle of a fighter standing on
  it** (`aim::standing_middle`). The floor is never really the target: bodies
  are not on the ray, so aiming at somebody puts the crosshair through them and
  onto the ground behind, and a ground hit means *there*. Sent to the dirt at
  *there*, the shot would pass under whoever is standing on it.

  **Half a body up from the ground the ray met**, not up to the caster's own
  cast height, which is what this used to be. The two are the same number on
  flat ground and nothing like it off it: from the top of a platform the
  caster's cast height is metres above the arena floor, so every skillshot aimed
  at somebody below flew out level and over their head, and the only way to land
  one was to aim at a patch of floor well short of them. Measuring from the
  ground the ray met makes the rule true from any height.
- **Hit terrain that is not ground, a structure, or the range sphere** — the
  point of intersection, exactly.
- Either way the ability follows a **straight line from the caster to that
  point**, and that line is its whole reach. There is no separate range number:
  the sphere is part of the raycast.

## The two that are not skillshots

### Swing

A melee attack. A body moving, so it does not raycast and nothing can stop it
short — its reach is simply the move's reach, off the body.

But it is **not level**. Melee happens in the air and on slopes, and a swing
pinned to the horizontal misses things that are plainly in front of you. The
yaw is the body's facing, which already follows the mouse at the turn rate. The
pitch comes from the camera, with a **dead zone below the horizon**:

```text
   above the horizon      the swing follows the camera exactly
   the first 45° below    the swing stays level — the standard arc
   further down           the swing follows what is left over
```

So at −45° the swing is the same as at 0°, at −46° it is that swing tilted one
degree down, and so on. The dead zone is the whole trick: the camera sits above
the shoulder, so looking *at* somebody standing at your own height means looking
slightly **down** at them. Without it, every swing thrown at an opponent would
tilt into the floor. At the dead zone's edge the tilt is still zero and moves a
degree per degree from there, so there is no step to feel.

45° is [`tuning::swing_level_to`](../../crates/sim/src/tuning.rs).

**The dead zone is a standing rule.** Off the floor the pitch is followed
exactly, all the way down. Two reasons, and the first is the one that matters:

- The dead zone corrects for *the camera sitting above the shoulder of somebody
  standing on the same floor as their target*. That is a fact about two
  fighters on one floor. In the air, the thing you are looking down at really
  is below you, and charging the first 45 degrees of that is just a swing that
  misses.
- It also makes the air game arithmetically impossible. The look-down limit is
  85 degrees, so a 45-degree dead zone caps the tilt a falling fighter can
  reach at 40 -- and the Champion's aerial spike only connects at 45 or more.
  Measured rather than guessed: on `main` the spike lands for look angles
  between 45 and 85 degrees down.

`aim::swing_path` takes `grounded` for exactly that, which is the same split
the Champion's own swing shapes already make.

**Which arm it comes out of** — added 2026-09-13. A swing also takes a `hand`,
and it moves *where the swing starts* and nothing else: a one-armed move leaves
from that shoulder rather than from the middle of the chest, at the same height
and along the same line. Almost every move in the game is `Hand::Centre` and is
unaffected.

It exists because one class is built on the distinction. The Dual mage holds two
forces apart, one in each arm, and her two autos are the same punch thrown left
and right; which of them just landed is the whole of how her meter is steered, so
two volumes a player could not tell apart would make the mechanic unreadable.

The side is declared in the move table next to the shape (`moves::hand`), and it
is turned into geometry in exactly two places, both here: `aim::across` for
*where a one-armed swing starts*, and `Hand::outward` for *which way round a
shape that sweeps goes* — which is what makes the Dual mage's two autos one ring
read from either side, sharing a single tuned arc that cannot drift a sign
apart.

The sides are the **skeleton's**. The body is authored with `+Z` along the facing
and its left arm at `-X`, which is a left-handed frame in a right-handed world,
so "the left arm" is the side a quarter turn *toward* the strafe-right axis. What
an animation and a hitbox have to agree about is which arm the player can see
swinging, and `view/tests/kinematics.rs` fails if they ever part company.

### At the mechanic

Wherever the class mechanic is standing. One move: the Reaver's Guillotine
lotus, whose blades erupt at the shadow.

The player *did* aim it with the crosshair — when they placed the shadow, which
is a grounded cast. Throwing the move only cashes that in. Re-aiming it at the
throw would quietly delete the reason shadow placement is a decision, which is
most of the class.

The volume follows the mechanic **live**, because the Reaver can recall the
shadow while the blades are out — and doing exactly that is what the ability is
for. The twelve blades take their centre from the shadow's position every frame,
so a recall drags them the length of the arena. An effect that had been pinned to
the patch of floor it was cast on would have made the class's biggest turn
impossible to express.

It is also the reason the shadow is **never absent** (2026-09-13): a move aimed
at the mechanic needs the mechanic to be somewhere. `mechanic_path` still falls
back to the caster's own feet, and on this class the fallback is now
unreachable.

## Which move uses which

Declared per move in the move table, not inferred, so the question has an answer
for every slot and `cargo run -p sim --bin frametable` prints it in the `aimed`
column. It used to be worked out from what a move left behind, which answered
for the two abilities that plant something and quietly called everything else a
swing.

| Line of effect | Moves |
| --- | --- |
| **Grounded** | Fissure, Fire pillar, Black spike, Judgement, Send shadow |
| **Skillshot** | Bolt, Cataclysm, Air bolt, Gale, Bloodletter, Grasp, Lance |
| **Swing** | every melee attack: Bash, Slam, Grapple, Slash, Executioner, Rend, Landfall, the Dual mage's Sweep and both of her autos, and all nineteen of the Champion's |
| **At the mechanic** | Guillotine lotus |

The table is a convenience and the move table is the authority; where they
disagree, the `aimed` column of the frame table is right and this is stale.

The mechanic inputs are aimed too, through the same two functions: Raise is a
grounded cast and the Bulwark's thrown shield is a skillshot. The Reaver's is no
longer a mechanic *input* at all — Send shadow is a move in the table like any
other, and it is a grounded cast, so a shadow lands on the floor exactly where
the crosshair is.

That is also why it sits on **right click** rather than on `E`: it is the one
thing in her kit the crosshair aims, and the mouse is where aiming lives. The
swing it displaced went to the key, which does not read the crosshair as a
place. See [kits/shadow-reaver.md](kits/shadow-reaver.md).

### Three things that are not lines of effect

**Is the crosshair on the shadow?** `aim::pointing_at` answers it, and the
Reaver's forward dodge reads the answer to decide whether it is a dodge or the
dash to her second body. It is not a fifth kind of aiming — it points nothing
anywhere — but it lives in `aim.rs` for the same reason everything else here
does. The obvious alternative is an angle between the look direction and the
line to the shadow, worked out beside the dodge, and that is the parallel-ray
mistake in its usual disguise: it agrees with the crosshair at long range and is
out by a whole body at short.

**Is there a way through to it?** `aim::clear_between` answers that one, and the
same dodge asks it second — added 2026-09-14, when the dash learned to go up.
Nothing occludes `pointing_at`: a shadow is a shadow and you can point at one
through a wall. Whether she can *get* there is a question about the world rather
than about the camera, and the rule is deliberately blunt:

> **The dash goes to wherever the shadow is, along the straight line between the
> two bodies. Only a total obstruction stops it — meaning no line between them is
> clear at all.**

Not "a ledge is in the way of her feet", which is every dash onto anything. Both
bodies are upright columns standing over a fixed spot, so **every line between
them has the same horizontal footprint** and they differ only in how they rise.
That makes the four corner lines — soles to soles, soles to crown, crown to
soles, crown to crown — the extremes of the whole family, and a solid that
crosses all four crosses everything in between. One getting through is enough.

**Where does a thing go that nobody aimed?** `aim::planted_ahead` answers that
one — added 2026-09-14, for the Elementalist's Landfall. The move's own line of
effect is a **swing**: she is a body arriving, and its volume is a disc on the
floor at her own feet. The slab of rock the arrival levers out of the ground is a
second thing, and it goes a fixed distance in front of her rather than anywhere
the crosshair chose, because *she is landing, not aiming*. The facing is used
flat: a plunge that put its slab nearer because she happened to be looking down
would be aiming after all.

It is here rather than beside the move for the reason the two above are. Written
there it would be a facing, a distance and a floor query sitting next to an
ability — which is the exact shape of the mistake this document exists to
prevent, three metres of it at a time.

It is what lets her dash *up*. At the foot of a platform with the shadow on the
deck, the line from her soles goes into the wall of it and the line from her
crown goes over the lip: there is a way, so she takes it. It is also why the
blockout refuses her nothing — the platforms and the walls are one and a half
metres and a fighter is one point eight, so she can always see over. A
**structure** is exactly a fighter's height, so an Elementalist's stone raised
squarely on the line leaves no way through, and cutting the Reaver's line
becomes something another class can do on purpose.

The four lines are measured from just above the soles and just below the crown,
by the collision skin. A body standing on a surface is standing *exactly* on it,
so a line taken from the soles themselves grazes the thing it is standing on and
reads as a wall. Trimming the body rather than the world is the choice that
matters: shaving the solids instead opens a hairline between two stacked ones
that a ray can thread, and two stones on top of each other have to be one
obstruction.

**A swing still commits to a plane, and the crosshair is where the plane comes
from.** Added 2026-09-12 with the Champion's rebuild. The yaw of a swing is the
facing, which is locked when the move starts; the *pitch* is the rest of the
same look, and `aim::swing_path` is the one place that turns the two into a
line. No raycast: a swing stops where the weapon stops rather than where the
crosshair lands, so there is nothing for it to hit-test against.

The distinction is worth keeping straight, because the two halves of the
sentence pull opposite ways:

- **Where the volume sits** is the body's business. A disc at arm's length is
  placed along the flattened `facing` and always has been — aiming at the floor
  does not move it, which is the rule above.
- **Which plane a shaped weapon sweeps through** is the crosshair's. The
  Champion's hammer comes down in the plane you are looking along, its aerials
  are thrown at the floor or the sky on purpose, and a spear levelled at
  somebody below you is most of why pitch is on the wire at all.

A class whose swings are discs never reads the pitch, so this changes nothing
for five of the six.

## What the path runs into

Separate from the aiming ray, and separate on purpose. The camera's ray says
*where the player is pointing*; this says *what is in the way of the thing they
threw*. The two lines are not the same line — the camera is behind and above —
so a body the camera could not see is still a body the shot passes through.

Each ability states which kinds of thing its path can meet. The Elementalist's
beam meets bodies, stones and **fire**; the fire bolt that a pillar lights meets
bodies and stones but not fire, or it could not leave the pillar that lit it.

An ability whose *effect* travels — the Blood mage's thrown blade — takes the
path's direction and flies its own distance along it, rather than stopping where
the crosshair's ray stopped. A blade thrown at something four metres away still
flies its full distance; the crosshair picked the line.

**What comes back can follow a person.** The blade's return leg is drawn to
wherever its caster is standing this frame rather than to the spot it left, so a
mage who walks while it is in the air has it curve after her and land in her
hand. The outward leg never moves: the throw was aimed, and re-aiming an ability
that is already out is the thing this document is about. See `Effect::home`.

## Aiming with time

One move is aimed with the *length of a button press*: hold the Blood mage's
Grasp and the reach it is solved at walks from melee out to its own `reach` over
half a second. A small marker shows where that has got to — it leaves the caster's
chest and travels outward, and where it stops is where the arms will converge.

**The marker is not a second answer.** It is the far end of `Player::aim_path`,
which the wind-up re-solves every frame through the move's own aiming with the
reach the hold has bought so far. The renderer reads its `to` and draws a ball
there. Nothing else is computed anywhere, which is the point: the failure mode
this whole document exists to prevent is two pieces of arithmetic that agree
today.

**The line is solved at the move's full reach every frame, and the hold only
picks a point along it.** That split is the whole of what makes a marker
readable, and getting it wrong is what the first two attempts did.

Solving the *aim* at the wound-up range means the raycast's own answer changes
as the range grows: the far end walks off the floor and onto a wall and back, so
a player holding the mouse perfectly still watches the marker jump about while
choosing a depth. Fixing that by making the move a **swing** — a ray off the
body, dead-zoned to stay level while standing — holds the line still, but a
level line out of a platform passes clean over anybody on the floor below, and
the only way to land one was to aim well under the target on screen.

Solving the line once, at the full reach, has neither problem. The line is the
crosshair's, so it converges on what the player is looking at from any height;
it does not move while the mouse does not; and the hold slides a point along it.

**The line is read for its direction and nothing else.** How far out the marker
is comes from the hold alone, and it does not ask what the ray stopped on: point
at a wall two metres away, wind to full range, and it is still a ten-metre
Grasp — the arms converge eight metres *behind* the wall. A wall is a thing to
punch an ability through, not a shorter version of the ability. Cutting the
marker back to the wall was tried on the way here and is wrong for the same
reason a slider that snapped to whatever was in front of it would be: the
wind-up would stop meaning one thing.

The aim stays live for the whole wind-up — the body turns with the mouse — and
**locks on the frame the button comes up**, which is the frame the move starts.
That is where every other move locks it too; a channel does not move the rule,
it makes the frame later. What is stored across the gap is the solved path's
*length*, cut back to wherever the hold had got.

## What this rules out

**A ray from the chest along the look direction.** This is the mistake, and it
looks completely reasonable: the fighter is at the chest, the player is looking
that way, so send it from there along there. But that ray is *parallel* to the
crosshair's and parallel rays never converge. The reticle sits on one spot and
the ability goes to another, by metres, and the error grows with distance —
which is exactly the bug report that produced this document.

> **`aim::swing_path` is that ray, and is not that mistake**, which is worth
> being precise about because the two are one line apart. The mistake is using
> it to answer *where does this go* — a target, at a distance, which the
> crosshair is also pointing at and disagrees about. A swing asks nothing of
> the kind: it stops at arm's length, the crosshair is not promising anything
> out there, and what it takes from the look is the **angle it sweeps at**
> rather than a point it is trying to reach. Two lines a metre long that start
> at the same shoulder cannot be metres apart at the end of them.
>
> The test for whether a new use is the mistake: *is it pointing at something
> the player can see the reticle on?* If yes, it is a skillshot and belongs in
> the raycast. If it stops on the body's own scale, it is a swing.

**A hitbox at a fixed distance in front of the character.** The version before
that. Aiming up did nothing at all.

## Gravity

Nothing arcs today; every path is a straight line. When a projectile wants
gravity, the target point above stays the target point — what changes is the
*path* to it, and the change belongs in `aim.rs` so every ability that wants it
gets the same one.

## Open

- **Fissure** travels along the ground to its target, which the grounded path
  already provides as `from` → `to`. It is aimed correctly now; the travel and
  the structure it plants at the point of impact are still unbuilt, so today its
  volume simply appears at the target.
- **The dead zone is one number for every class and every move.** A spear at
  three and a half metres and a grapple at arm's length plausibly want different answers,
  and a swing thrown while falling fast plausibly wants a different one again.
  Nobody has played it yet; it is one knob until somebody has.
- **The Champion's weapons are shapes, not points.** They sweep through a plane
  rather than arriving somewhere, which is the strongest argument yet that a
  swing is its own line of effect and not a very short skillshot: a skillshot's
  answer is a point.
