---
status: proposed; the initial kit is built
decided: 2026-09-10
revised: 2026-09-13
formerly: Statera
sources: docs/archive/combat-design/statera-skills.md, docs/archive/combat-design/class-builds.md
depends: ../dual-mage.md
---

# Dual mage — kit

**Identity.** Melee mage containing two forces. Power comes from riding as close to an edge
as you can while still able to pull back.

Read [../dual-mage.md](../dual-mage.md) first — this kit implements that meter and is
meaningless without it.

## Mechanic — the two-pole meter

- **Left click moves you darker, right click moves you lighter** — every input, not just
  autos. Stronger abilities push harder. Scroll click and both-click have no side, so they
  push further along your current path.
- **Power scales continuously with depth.** The same cast is weak at centre and large at
  the edge. Centre is where both forms are available and both are weak.
- **Coming back:** cast toward the far side (slow, weak — casting against the grain) or land
  a far-side auto (fast, but melee range only).
- Past a depth threshold you take a burn that stops the moment you come back inside.

**Ascension has no input.** It triggers when you max the bar by casting. See
[../dual-mage.md](../dual-mage.md).

Full input map in [../controls.md](../controls.md).

## What is bound today — built 2026-09-13

Five moves, and the two on the bare clicks are the class.

| Input | Move | What it is |
| --- | --- | --- |
| `L` | **Dark auto** | A punch with the **left** arm. Steers dark, on contact |
| `R` | **Light auto** | The same punch with the **right** arm. Steers light, on contact |
| `shift` + `L` | **Lance** | The committed line skillshot. Steers dark, on the press |
| `Q` | **Judgement** | The Light finisher. Gated past the deep threshold |
| `E` | **Sweep** | A wide cut across the whole front. No side, so it pushes you further along the path you are on |

`shift` + `R` throws the light auto unmodified: the kit wants a light *form* of the committed
cast there and there is not one built, so the modifier is ignored rather than made to mean
something it does not.

`cargo run -p sim --bin frametable` prints the live numbers.

### Why `E` carries an ability

The same reason the Blood mage's does, arrived at from the other direction. `E` is the class
mechanic key, and this class's mechanic is a **meter steered by which button attacks** — there
is no state for a key to toggle. So the key is free, and an ability with a startup and a
recovery is a better use of it than nothing. See [../controls.md](../controls.md#where-e-is-an-ability).

## Auto attack

Two autos: **left is dark, right is light**, and they **change your mode on contact** — a
whiff steers nothing. They carry a slight range boost, powered by the beings inside, which
matters because steering depends on connecting.

The autos are the steering wheel. Landing the far-side auto is the fast way back toward
centre, and the reason the class has to close distance exactly when it is strongest.

### They come out of the arms, and that is load-bearing

**The dark auto is thrown with the left arm and the light auto with the right**, and the hit
volume leaves from that shoulder rather than from the middle of the chest. It is not
decoration: which of the two just landed is the entire information the player has about which
way the meter moved, and an attack that came out of the sternum both times would make the
mechanic unreadable. One move field says which arm (`moves::hand`), the volume starts there
(`aim::hand_origin`), the animation is one punch mirrored (`Pose::other_arm`), and
`view/tests/kinematics.rs` fails if the simulation and the renderer ever pick different arms.

### The shape: a punch, and a wing behind it

Each auto **reads as a punch** — a short step into a straight arm, five frames of startup, the
other hand thrown back behind. What it *does* is much larger than the arm that threw it:

- the volume starts **a little behind the fist**, so stepping inside the punch is not the
  answer to it;
- it **sweeps outward** — away from the body on whichever side the arm is — while it
  **grows**, so the tip travels a spiral rather than an arc;
- it ends **two to three arm lengths** past the hand.

What that carves out over the active frames is a wing. It is its own hit shape
(`moves::Shape::Wing`) rather than a swing with unusual numbers, because a swing's head stays
at a fixed reach and this one does not, and because a swing has a haft where this has a wing
root. Both autos share one `arc`; the arm supplies its direction, so the mirror cannot drift.

**Tempest** (passive, not built): abilities mark enemies on hit. Autoing a marked enemy
consumes the mark for bonus damage and a short burst of movement speed. This is what makes
closing to centre attractive rather than a chore.

## Core abilities

Four slots on `shift` + click. Left click casts the dark form, right the light form.

### Step
**Startup** fast · **Recovery** short · **Range** short

| Form | Behaviour |
| --- | --- |
| **Light** | Forward blink. Damages on arrival; blinds at depth |
| **Dark** | Drain-dash. Steals health from everything passed through |

### Lance
**Startup** fast · **Recovery** short · **Range** medium

| Form | Behaviour |
| --- | --- |
| **Light** | Line skillshot that detonates at maximum range for area burst |
| **Dark** | Line skillshot that tethers the first target hit, draining while it holds |

### Sweep — **built, on `E`**
**Startup** medium · **Recovery** medium · **Range** short cone

Both arms thrown across the whole front at once, driven from the hips, left to right. It is
the answer to somebody already inside the punches: they are long and thin and lose to anyone
who has closed, and this is the thing that moves that person.

Built as **one move rather than two forms**, which is true of every ability on this class so
far — the form split is unbuilt everywhere, not skipped here. Being on a key rather than on a
click, it has no side, so it pushes you further along the path you are already on.

| Form | Not built |
| --- | --- |
| **Light** | Pushes back, and staggers anything carrying a Tempest mark |
| **Dark** | Slows, and heals you per target caught |

### Divide
**Startup** medium · **Recovery** short · **Range** medium

A dome that rotates into existence and blocks enemy projectiles for two to three seconds.
Hitting it with any other ability dashes you to the impact point, keeping momentum.

| Form | Behaviour |
| --- | --- |
| **Light** | Damages on the way up; larger with depth |
| **Dark** | Drains everything caught inside it |

## Finishers — gated by depth

One form each, available only past a depth threshold. They are the reason to leave centre.

**Judgement is on `Q` today**, not on `M`: `Q` is the class special on every class, the
finisher is what this class's special *is*, and the reservation below about `M` and `LR`
carrying the payoff moves has not been answered. Eclipse is unbuilt, and `M` and `LR` are
still where the design intends the pair of them to live if the reservation resolves the other
way. The gate is on **depth** rather than on depth *on Judgement's own side*, which is a
simplification the implementation should lose when Eclipse arrives and the two need telling
apart.

### Judgement — Light
**Startup** slow, delayed · **Recovery** committed · **Range** medium · **Mechanic** pushes
hard toward Light

A delayed area strike: massive instantaneous damage at the centre, then a wide low-damage
field. While the field lasts, moving through it grants you speed, bonus damage, and
lifesteal. Applies only one Tempest mark regardless of how many it hits.

### Eclipse — Dark
**Startup** medium · **Recovery** committed · **Range** long, channelled · **Mechanic**
pushes hard toward Dark

A beam that fires for a fixed duration, damaging everything in front of you and applying a
stacking slow. Heals a fraction of the damage dealt.

## Playing it

Commit to one side with repeated casts of that form, land the finisher from the deepest
position you can survive, then close to melee and auto back toward centre — or cast the far
side to bleed back slowly if you cannot close.

Switching sides means crossing the whole bar, so which edge you commit to is a real
strategic choice rather than a moment-to-moment one. Oscillating at centre is always
available and always weak.

In the hand it comes out as **which arm you are punching with**, which is the point: the
mechanic is not a bar you manage on the side, it is the left and right buttons you are already
pressing, and you can read your own commitment off your own animation.

## Open questions

- **Are `M` and `LR` reliable enough to carry the finishers?** They are the slowest inputs on
  most mice and these are the payoff moves. See [../controls.md](../controls.md). Judgement
  sits on `Q` until this is answered.
- **What goes on `shift` + `R`?** The kit wants the light form of the committed cast; nothing
  is built, so it throws the light auto. The first class to build a two-form ability answers
  this for the whole kit.
- **Does the wing want a dead zone below the horizon of its own?** It is a swing, so it takes
  the shared one (`tuning::swing_level_to`) — a number chosen for a hammer and a spear. A
  volume that opens sideways as much as forwards is a different question and nobody has
  played it.
- Should the finishers be visibly greyed out from the wrong side, or hidden entirely?
  Greyed is friendlier and teaches the mechanic; hidden is cleaner to read.
- Does Divide's dash-to-impact work with either form of the ability that hits it, or only
  matching forms? Matching-only would be a strong combo constraint worth testing.
- Naming the two forces.
