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

### At the mechanic

Wherever the class mechanic is standing. One move: the Reaver's Guillotine
lotus, whose blades erupt at the shadow.

The player *did* aim it with the crosshair — when they placed the shadow, which
is a grounded cast. Throwing the move only cashes that in. Re-aiming it at the
throw would quietly delete the reason shadow placement is a decision, which is
most of the class.

The volume follows the mechanic **live**, because the Reaver can recall the
shadow while the blades are out.

## Which move uses which

Declared per move in the move table, not inferred, so the question has an answer
for every slot and `cargo run -p sim --bin frametable` prints it in the `aimed`
column. It used to be worked out from what a move left behind, which answered
for the two abilities that plant something and quietly called everything else a
swing.

| Line of effect | Moves |
| --- | --- |
| **Grounded** | Fissure, Fire pillar, Black spike, Judgement |
| **Skillshot** | Bolt, Bloodletter, Grasp, Lance |
| **Swing** | every melee attack: Bash, Slam, Grapple, Sweep, Drive, Uppercut, Slash, Executioner, Rend, Step strike |
| **At the mechanic** | Guillotine lotus |

The mechanic inputs are aimed too, through the same two functions: Raise and the
shadow are grounded casts, and the Bulwark's thrown shield is a skillshot.

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
  1.55× reach and a grapple at arm's length plausibly want different answers,
  and a swing thrown while falling fast plausibly wants a different one again.
  Nobody has played it yet; it is one knob until somebody has.
