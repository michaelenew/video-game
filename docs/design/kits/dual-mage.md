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
- **Coming back:** throw a far-side auto. That is the only way — casts follow the force she is
  *carrying*, and an auto is the only thing that changes which force that is. Casting against
  the grain is not an input any more.
- Past a depth threshold you take a burn that stops the moment you come back inside.

**Ascension has no input.** It triggers when you max the bar by casting. See
[../dual-mage.md](../dual-mage.md).

Full input map in [../controls.md](../controls.md).

## What is bound today — built 2026-09-13

Five moves, and the two on the bare clicks are the class.

| Input | Move | What it is |
| --- | --- | --- |
| `L` | **Dark auto** | A punch with the **left** arm. Steers dark by 5, and she is now dark |
| `R` | **Light auto** | The same punch with the **right** arm. Steers light by 5 |
| `shift` + `L` | **Lance** | The committed line skillshot. Steers 12, in whichever force she carries |
| `Q` | **Judgement** | The finisher. **No longer gated** — see below |
| `E` | **Sweep** | A wide cut across the whole front. Steers 12, in whichever force she carries |

`shift` + `R` throws the light auto unmodified: the kit wants a light *form* of the committed
cast there and there is not one built, so the modifier is ignored rather than made to mean
something it does not.

`cargo run -p sim --bin frametable` prints the live numbers, and the HUD draws the bar under
her health: a two-poled track, filled out from the centre toward whichever end she is on, with
the deep thresholds marked and its border in the colour of the force she is carrying.

### Why `E` carries an ability

The same reason the Blood mage's does, arrived at from the other direction. `E` is the class
mechanic key, and this class's mechanic is a **meter steered by which button attacks** — there
is no state for a key to toggle. So the key is free, and an ability with a startup and a
recovery is a better use of it than nothing. See [../controls.md](../controls.md#where-e-is-an-ability).

## Auto attack

Two autos: **left is dark, right is light**, and they **change your mode** — on the press
since 2026-09-13, not on contact, because the contact rule left the whole mechanic immovable
with nothing in reach. See
[../dual-mage.md](../dual-mage.md#autos-are-the-steering-wheel), which also says what should
come back and in what shape.

They carry a slight range boost, powered by the beings inside. The autos are the steering
wheel, and throwing the far-side one is the way back toward centre.

### They come out of the arms, and that is load-bearing

**The dark auto is thrown with the left arm and the light auto with the right**, and the hit
volume leaves from that shoulder rather than from the middle of the chest. It is not
decoration: which of the two just landed is the entire information the player has about which
way the meter moved, and an attack that came out of the sternum both times would make the
mechanic unreadable. One move field says which arm (`moves::hand`), the volume starts there
(`aim::hand_origin`), the animation is one punch mirrored (`Pose::other_arm`), and
`view/tests/kinematics.rs` fails if the simulation and the renderer ever pick different arms.

### The shape: a punch, and a blade that comes round from behind

Each auto **reads as a punch** — a short step into a straight arm, five frames of startup, the
other hand thrown back behind. What it *does* is a **thin curved blade**, swept through the
air on a ring that she is not standing in the middle of:

```text
                  .-  -  -.
              . '           ' .        the band is only the outer quarter of
   starts   ;                    :     the ring: the near edge passes just
   behind    \                  /      outside where her fist finishes, and
   her        ' .    (x)    . '        the far edge is the move's reach
                  ' - , - '
      o                        ^
      |                        '-- finishes in front of that fist,
   the mage,                       a little off her centre line
   punching
   left-handed              (x) the ring's middle: pushed toward the
                                other arm, and forward
```

- the **band** runs from `tuning::wing_inner` of the reach out to the reach — three quarters
  of the way out and no further in. It is a blade travelling, not a slice of pie: the version
  that reached from her own elbow to full range was a filled disc with a pinhole in it, and
  caught anyone standing anywhere in the quadrant;
- the **ring's middle is not her**. It sits `tuning::wing_offside` toward the *other* arm and
  `tuning::wing_ahead` in front of her. A ring centred on a fighter is the same distance from
  them at every bearing, so a piece of one reads as a halo however short you make it. Pushed
  off her, the blade comes in close beside the punching fist and swings wide in front — a
  swipe passing by rather than a circle drawn around her;
- it **finishes in front of its own hand**, `tuning::wing_finish` off her centre line toward
  the punching side, rather than dead ahead. Which of the two just landed is the whole of how
  the meter is steered, and two autos that both ended on the sternum put the answer in the
  same place twice;
- it **opens** rather than travels: the section starts closed behind her, on the punching
  arm's side, and its leading edge comes round toward the front while its trailing edge stays
  where the punch threw it. So the blade is one that *grows* along the ring rather than one
  that slides along it — that is the wing, the beings inside her extending the movement past
  where an arm could take it.

The arc is a **shallow** one, and deliberately: a long arc on a short radius is a circle round
her feet whatever else is true of it. Wide radius, narrow angle, middle pushed off her — those
three together are what make it read as a cut through the air in front of her.

### The tip

**The last frame is the tip alone, and it hits `tuning::wing_tipper` times as hard.**

The tip is a **ball at the end of the blade** — `tuning::wing_tip_radius` across, out for one
frame, at the foremost point of the ring. The wing opens to most of its arc and stops
`tuning::wing_tip` of the span short; the tip arrives at the end of that gap on the final
active frame. So the only thing that ever reaches the point out in front of her at full
extension is the tip, and everything the wing already opened over has already been swept.

A ball rather than the last slice of the section, which is what it used to be. A slice of a
ring is metres of arc: the "tip" was the widest thing the move ever put in the world, caught
the whole front of her at once, and was the easiest part of the attack to land rather than the
hardest.

That makes it a **spacing decision** rather than a damage bonus attached to a frame number:
the body of the wing is what catches somebody who is already on top of you, and the tip is
what catches somebody who thought they were out of range. It is the one piece of execution in
a move that is otherwise thrown constantly, and the debug overlay draws it in its own colour
so it can be learned.

It is its own hit shape (`moves::Shape::Wing`) rather than a swing with unusual numbers, for
three reasons that are one reason. It is hung off a **ring** rather than off a shoulder, so it
curves rather than reaches. It has a **hole**, where a swing has a haft. And it **starts
behind her** rather than crossing her front, which is what makes it read as something thrown
off the arm rather than as the arm itself.

The volume is the section itself — `math::Sector`, the one thing in the game that is not a
capsule — and the hit test, the debug overlay and the arena's own drawing all read it. A
straight line through a curve either misses the inside of it or claims the outside.

Both autos share one `arc` and one reach and are mirrored by `Hand::outward`, so there is one
set of numbers rather than two that can drift — but they are two rows in the Oven, and
`crates/sim/tests/dual_mage.rs` is what fails if a tuning session moves one and not the other.
The two statements made against her body rather than in metres — the band starting where the
fist stops, and the tip landing two to three times as far out as the fist gets — are checked
against the baked animation in `view/tests/kinematics.rs`, because the simulation has no idea
where a fist is.

**The plane is the floor's while she is standing**, which is the one place this move ignores
the camera's pitch; off the ground there is no shared floor to be parallel to and the ring
tilts with the aim, the same split `moves::swing_base` already makes for a sweep thrown in the
air.

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

## Finishers

One form each. They are meant to be the reason to leave centre.

**Judgement is on `Q`, and the depth gate is gone** — removed 2026-09-13. `Q` is the class
special on every class and the finisher is what this class's special *is*, so gating it on
depth meant the key did **nothing at all** for the opening of every match: a player pressing
it had no way to tell an ability from an empty binding. A special you cannot press is not a
special.

What is lost with the gate is the finisher's status as a payoff, and that has to come back as
*power* rather than as availability — the class's own principle is that depth scales strength
continuously, so a Judgement thrown from the centre should be a weak one rather than a refused
one. Nothing scales with depth yet; that is the next thing this class needs.

Eclipse is unbuilt, and `M` and `LR` are still where the design intends the pair of them to
live if the reservation below resolves that way.

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
- **Nothing scales with depth yet.** Power scaling continuously with distance from centre is
  the class's founding idea and none of it is built: a cast at the edge is the same cast as one
  at the middle. With the finisher's gate gone this is the largest hole in the class.
- **Should the wing tilt at all?** Standing, its plane is the floor's and the camera's pitch
  does not touch it — which is the shape as specified, and which means an auto thrown at
  somebody on a ledge above or below misses them by geometry rather than by aim. Airborne it
  tilts with the look. Nobody has played either.
- Should the finishers be visibly greyed out from the wrong side, or hidden entirely?
  Greyed is friendlier and teaches the mechanic; hidden is cleaner to read.
- Does Divide's dash-to-impact work with either form of the ability that hits it, or only
  matching forms? Matching-only would be a strong combo constraint worth testing.
- Naming the two forces.
