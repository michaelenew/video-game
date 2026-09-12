---
status: decided
decided: 2026-09-12
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
- **other players**
- **monsters**
- **structures** — the Elementalist's stones
- **the max-range sphere**, centred on the casting player, of the ability's own
  range

Whatever it reaches first wins. A future element may add a fifth kind of thing;
nothing today is an exception.

**Ignoring anything behind the character model** matters because the camera sits
behind the shoulder. A wall the camera happens to be looking through is scenery,
not a target, and aiming through your own cover is not a mechanic anybody asked
for.

**Fire is deliberately not on the list.** You can see through flame, so a fire
pillar never steals the crosshair — but a shot that *travels through* one still
notices it. Where the ability goes and what is in the way of it are two
different questions; see "What the path runs into" below.

### Ground, and everything else

The one distinction the raycast draws is whether the surface it hit **faces
upward**: the floor, the top of a platform, the top of a stone. That is "the
ground". The side of a platform, the side of a stone, a body, the creature and
the range sphere are not.

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
travels on its own, so what the crosshair gives them is the *line* rather than
a landing spot.

- **Hit the ground** — draw a line straight up from that spot to the height of
  the character's ability origin. The shot flies level over the place the
  crosshair is on rather than diving into the dirt.
- **Hit terrain that is not ground, a character, a monster, or the range
  sphere** — the point of intersection, exactly.
- Either way the ability follows a **straight line from the caster to that
  point**, and that line is its whole reach. There is no separate range number:
  the sphere is part of the raycast.

### Not a skillshot at all

A melee swing. A sword is a body moving, and pointing the camera at the floor
must not put the blade there — a swing comes out along `facing`, at the move's
own reach. It is named here so that "which of the three is this move" has an
answer for every move rather than being a thing each caller decides.

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

An ability whose *effect* travels — the Blood mage's thrown blade, her Grasp —
takes the path's direction and flies its own distance along it, rather than
stopping where the crosshair's ray stopped. A blade thrown at something four
metres away still flies its full distance; the crosshair picked the line.

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

- **Fissure** is written as a grounded ability that races along the ground to
  its target. The path it needs already exists; the ability does not.
- **The melee swing** could in principle become a very short non-grounded
  skillshot, which would make the matrix two entries rather than three. Nobody
  has argued for it, and "pointing the camera down must not swing at the floor"
  is the reason not to. The Champion's rebuild made the case weaker rather than
  stronger: its weapons are *shapes* that sweep through a plane, and a
  skillshot's answer is a point.
