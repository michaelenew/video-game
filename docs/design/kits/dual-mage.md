---
status: built; the six moves are unchanged under the two bars of 2026-09-23
decided: 2026-09-10
revised: 2026-09-23
formerly: Statera
sources: docs/archive/combat-design/statera-skills.md, docs/archive/combat-design/class-builds.md
depends: ../dual-mage.md
---

# Dual mage — kit

**Identity.** Melee mage containing two forces, one in each arm, and a bar for each. Power is
the bar she is carrying; what her body can do is the lower of the two; how fast she is losing
it is the gap between them.

Read [../dual-mage.md](../dual-mage.md) first — this kit implements that mechanic and is
meaningless without it.

## Mechanic — two bars, and the hill between them — rebuilt 2026-09-23

**The six moves below did not change.** What changed is what they steer.

- **Left click goads Dark and right click goads Light — and only the autos have a side.** Every
  other input feeds whichever force she is already carrying. Middle click and the keys have no
  side, which is why Lance lives on middle click.
- **Three tiers of push:** an auto goads a bar 5, a cast 9, and the finisher 20. The last is
  what makes casting Judgement from high a question about what the other bar does next.
- **The hill.** Inside a band of 16 nothing moves on its own. Outside it the higher bar rises,
  the lower falls and she burns, faster the wider the gap. Both bars calm toward empty, always.
- **Power is the bar she is carrying**, continuously: a cast reads the carried bar, an auto its
  own. Empty is thin and full is the most she can hold.
- **The tiers read the lower bar.** At half the dodge is a blink; at three quarters she has a
  second jump and a slower fall; both full is wings. So the climb is alternating hands — dark
  auto, dark Lance, light auto, light Sweep — which uses this kit in both forms.
- **Coming back:** the far-side auto, and it is urgent now, because the low bar is falling
  while she waits. An auto alone only holds a runaway; the far-side *cast* wins it.

**Ascension has no input.** It triggers when both bars are goaded to the top together, and the
drift never delivers it. See [../dual-mage.md](../dual-mage.md).

### How the tiers read

| The lower bar | Reads as | On her back |
| --- | --- | --- |
| Below a third | The dodge is a dodge; space in the air does nothing | Nothing on that side |
| A third | — | The first wing of that side is there |
| Half | The dodge blinks: she is where it would have ended, on its first frame, and stands through the tail | One wing a side |
| Two thirds | — | The second wing |
| Three quarters | Space in the air jumps once more, and she falls slower | Two wings a side |
| Both full | Every press of space is a wing beat; no dodge | All six |

The wings are the two bars, three a side, dark on the left and light on the right, and each
one is there or is not — a bar is read as a count, not a length. Lopsided wings are a mage
about to burn; three and three is a mage about to fly. `cargo run -p sim --bin frametable`
prints the tiers under the class, and `cargo run -p sim --bin goad` runs the climb.

Full input map in [../controls.md](../controls.md).

## What is bound today — rebuilt 2026-09-16

Six moves on five inputs, and the two on the bare clicks are still the class.

| Input | Move | What it is |
| --- | --- | --- |
| `L` | **Dark auto** | A punch with the **left** arm that **pulls**. Steers dark by 5, and she is now dark |
| `R` | **Light auto** | The same punch with the **right** arm, and it **pushes**. Steers light by 5 |
| `M` | **Lance** | The committed line skillshot, in **two forms**: light bursts at the far end, dark tethers what it hits. Steers 12, along whichever way she is already going |
| `Q` | **Judgement** | The finisher. No depth gate; it earns its status through power, and it throws the bar by 26 |
| `E` | **Sweep** | Both arms round past both shoulders. Light throws them off their feet, dark slows and heals. Steers 12 |

**Shift is only a dodge now**, on every class — see [../controls.md](../controls.md). That
took Lance's old home away, and middle click is a better one than shift ever was: it has no
side, so by the class's own rule it pushes her further along whichever way she is already
going, while the light-or-dark form of the cast comes from the force she is carrying. The two
rules stopped fighting.

## Depth is the whole class, and it is in the numbers now — 2026-09-16, re-pointed 2026-09-23

**Power scales continuously with the bar of the force a move is made of, and it scales
everything.** One curve, `state::depth`: a straight line from `tuning::depth_floor` at empty to
`tuning::depth_ceiling` at full, with every point on it reachable. No thresholds, no snapping
between versions. A cast reads the bar she is carrying; an auto reads its own, so the far-side
auto is thrown from strength when the far side is the high one.

With a bar empty everything she throws is thin and slightly disappointing. Full, it is the
most she can hold.

**What it moves, and what it deliberately does not.**

| | On the curve | Never |
| --- | --- | --- |
| Damage | ✓ | |
| Knockback and pull | ✓ | |
| Launch, leech, what a field drains | ✓ | |
| How **big** what arrives is — `radius`, a field's radius | ✓ | |
| How **far** it is thrown — `reach`, a skillshot's range | | ✓ |
| Startup, active, recovery, hitstun, blockstun | | ✓ |

The two exclusions are one argument made twice: **spacing and timing are what two players
read each other with**, and a class whose range or frame data changed continuously with a bar
only one of them can see would be unlearnable from either side. So the bar changes how much it
hurts and how big the thing that arrives is; where you can put it and how fast it comes out are
fixed. A deep Judgement is a far bigger Judgement thrown exactly as far as a feeble one.

**A cast is worth where you were standing when you pressed the button**, not where its own push
has since taken you (`state::Player::thrown_at`). Throwing anything moves a bar on the press
and the finisher moves it a long way, so a cast read live would be worth its own push — and the
one move in the kit that is supposed to be embarrassing from empty would be the least
embarrassing thing there. It would also mean the number on the HUD never matched what the
player got.

**The two autos are exempt from the size half of it**, and only that half. Their reach is
pinned to the punch that throws them — `view/tests/kinematics.rs` checks that the blade's near
edge starts where the fist stops — and they are the one move in the kit thrown every second, so
a volume drifting away from the animation would make the steering wheel unreadable. What depth
does to an auto is what it *does*.

`cargo run -p sim --bin frametable` prints the live numbers, and the HUD draws the two bars
under her health in one track: Dark filling leftwards from the middle and Light rightwards, the
two tiers ticked on each side, and its border in the colour of the force she is carrying. The
wings on her back are the same two numbers, readable from across the arena.

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

### One pulls and one pushes — 2026-09-16

**The dark auto drags whoever it catches a short way toward her and returns a trickle of
health. The light auto shoves.** Same frames, same shape, mirrored arms — and opposite answers
to the question of where the two of you end up standing.

That is what makes which arm you punch with a **spacing decision as well as a meter decision**,
which is the whole point of the mechanic living on the buttons you press constantly. A fragile
melee mage stays attached to somebody with the dark hand and buys herself room with the light
one, and every time she does either she has also committed to a side.

A pull is a **negative knockback** (`moves::Move::knockback` is signed now) and it is measured
along the line between the two bodies rather than down her facing — the point of dragging
somebody is that they arrive at *you*, and the wing wraps around her, so a body caught out at
the side would otherwise be sent backwards past her instead of in.

**The tip is where the real force is, on both arms.** `tuning::wing_tip_shove` multiplies the
shove the same way `wing_tipper` already multiplies the damage, so landing the tip pulls them
that much further in or sends them that much further out. The tip was already the one piece of
execution in a move thrown constantly; now what it buys is spacing rather than only damage.

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

The intended shape: four abilities, each with a light and a dark form. **Nothing here is
built as two forms** — Lance and Sweep are one move apiece, and the form split is unbuilt
across the whole class. Their inputs today are `shift` + `L` for Lance and `E` for Sweep;
Step and Divide have no input yet.

### Step
**Startup** fast · **Recovery** short · **Range** short

| Form | Behaviour |
| --- | --- |
| **Light** | Forward blink. Damages on arrival; blinds at depth |
| **Dark** | Drain-dash. Steals health from everything passed through |

### Lance — **built, on `M`, and the first two-form ability in the game**
**Startup** fast (light) / slow (dark) · **Recovery** short · **Range** medium

One input, two moves, and the arm she last punched with decides which. They are two rows in
the move table rather than one row with a flag, because **the thing that has to differ is the
wind-up**: a person standing opposite gets that and nothing else to choose between getting out
from under a burst and closing to break a tether, and the two answers are opposites.

So the two clips are authored against each other at every point. The light one rises off the
right shoulder, goes early and dives down its own line. The dark one sinks onto the rear leg,
drags the left hand down past the hip, and comes through low with the palm open — and its
contact pose leans *away* from the arm, because something on the end of it is about to start
pulling. Same input, opposite silhouettes.

| Form | Behaviour |
| --- | --- |
| **Light** | The line pokes on the way out and **detonates where it ran out**, so it is a thing you aim *past* somebody. The burst is most of the damage; landing it means picking a point behind them rather than on them |
| **Dark** | No volume of its own. It throws a line that **catches the first thing it crosses** and drains it every tick until the leash parts — and the leash is the only number on the class that depth does not touch, because staying next to what she caught is the cost rather than the reward |

**The leash is fixed at `tuning::tether_leash` whatever the bar says.** Depth buys a harder
drain and a longer hold, which is power; how far she may stray is a *rule*, and it is the whole
ability — a leash that grew with the bar would hand the deep version the one thing it is meant
to pay for, and at the edge it would reach most of the arena.

**Guard denies the hold.** The line still lands as an ordinary blow and then goes slack, which
is the counterplay a two-second drain has to have.

### Sweep — **built, on `E`, and it has weight now**
**Startup** medium · **Recovery** medium · **Range** short, and **round past both shoulders**

Both arms thrown across the whole front at once, driven from the hips, left to right. It is
the answer to somebody already inside the punches: they are long and thin and lose to anyone
who has closed, and this is the thing that moves that person.

**It is the one move of hers that reaches a little behind the shoulders**, and that is a
geometric claim rather than a mood. Somebody inside the punches is standing beside her or past
her shoulder, and a cut that only covered her front would miss exactly the person it exists
for. The arc runs most of a half turn; the clip winds the hands behind the left shoulder and
carries them through behind the right, because the volume does, and an animation that had
already thrown the arms across by the time the hitbox appeared would be drawing the move a
shoulder's width in front of where it hits.

Still **one move rather than two**, and now for a reason rather than for want of building it:
the shape is the same either way. Only what happens to whoever it caught changes, which is a
thing to branch on at the moment of contact rather than a second animation. Being on a key
rather than on a click it has no side, so it pushes you further along the path you are already
on.

| Form | Behaviour |
| --- | --- |
| **Light** | **Built.** Throws them back and off their feet — the move's own knockback plus a launch |
| **Dark** | **Built.** Slows them and heals her `tuning::sweep_heal` **per target caught**, which is what makes the answer to being swarmed the same move as the answer to being cornered |

The Tempest stagger the light form once carried waits on Tempest, which is unbuilt.

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

What was lost with the gate was the finisher's status as a payoff, and **that came back as
power on 2026-09-16** — see "Depth is the whole class" above. A Judgement thrown from the
centre is a weak one rather than a refused one, and one thrown from the edge is the largest
thing in the game and very nearly throws her over it.

Eclipse is unbuilt. `M` now carries Lance rather than a finisher, so the reservation below is
about `LR` alone.

### Judgement — Light, and **built on `Q`**
**Startup** slow, delayed · **Recovery** committed · **Range** medium · **Mechanic** pushes
hard toward Light — 26, its own tier, more than twice a cast

A delayed area strike where the crosshair is: instantaneous damage at the centre, and then a
wide low-damage **field** that burns anybody standing in it and makes *her* fast while she is
in it. So a Judgement thrown at somebody's feet is also a Judgement thrown at her own next few
seconds.

**Its status is power now, not availability.** At the centre it is a small radius, a small
number and a field that is over before anybody walks through it — an embarrassing version of
itself. At the edge it is the biggest thing in the game, and the 26 it throws the bar means
that a deep one lands her either well into the burn or over the edge into ascension. That is
the question the finisher is supposed to ask.

The kit's "bonus damage and lifesteal inside the field" is not built; the speed is. One
Tempest mark regardless of how many it hits waits on Tempest.

### Eclipse — Dark
**Startup** medium · **Recovery** committed · **Range** long, channelled · **Mechanic**
pushes hard toward Dark

A beam that fires for a fixed duration, damaging everything in front of you and applying a
stacking slow. Heals a fraction of the damage dealt.

## Playing it

Climb with both hands — dark auto, dark Lance, light auto, light Sweep — and the bars rise
together inside the band. At half the dodge is a blink; at three quarters you have a second
jump. Spend the height you have goaded on a finisher from the bar you are carrying, and the
other bar starts to collapse: catch it with the far-side hand before the tier goes, or let it
go and take the burn for the burst. Both full is wings, three seconds of them, and then empty.

**And which arm you punch with is now a spacing question at the same time.** The dark hand
drags them in, which is how a body this fragile stays attached to somebody long enough to
matter; the light hand shoves, which is how it gets out. So the two buttons that steer the bar
are also the two that decide the range you are fighting at, and you cannot ask for one without
answering the other. That is the mechanic doing its job: it is not a bar you manage on the
side, it is the thing your hands are already doing.

Being one-sided is easy, powerful and a runaway. Being two-sided is hard, mobile and the only
road to the wings. Alternating is how she climbs; what is weak is not goading at all.

In the hand it comes out as **which arm you are punching with**, which is the point: the
mechanic is not a bar you manage on the side, it is the left and right buttons you are already
pressing, and you can read your own commitment off your own animation.

## Open questions

- **Is `M` reliable enough to carry Lance?** It is the slowest input on most mice and this is
  now the committed cast rather than a finisher, which makes the question sharper rather than
  softer: it is a button she presses in every exchange. `U` stands in for it, and the answer
  may be that the keyboard stand-in has to be the real binding. See
  [../controls.md](../controls.md).
- **Is "the form is the arm you last punched with" findable?** It is the whole of the two-form
  idea and there is nothing on the HUD that teaches it beyond the bar's colour. The two Lance
  clips are authored to be opposites so the *opponent* can read it; whether the player can read
  their own is a different question and is not answered by the same thing.
- **Does the depth curve read as continuous in the hand, or only on paper?** `depth_floor` is
  0.5 and `depth_ceiling` is 2.0, so a full bar is four times an empty one. That is a big
  spread and it is deliberately a guess: the risk in one direction is that a low bar feels
  broken rather than weak, and in the other that nothing below the last quarter is worth
  casting from.
- **Is a Judgement from a full bar too much of a health bar?** It is the biggest number in the
  game by design, and now it is thrown from a position where the *other* bar is about to
  collapse rather than from one that is burning her. Whether that is a fair price is a play
  question.
- **The blink shares the dodge's window.** The plan listed a knob for the blink's own
  invulnerable frames; it has none, because `Action::invulnerable` reads one number for every
  dodge in the game and a second one would be a second dodge. If a blink wants a shorter or a
  longer window than the roll, that is the day it gets one.
- **Should the wing tilt at all?** Standing, its plane is the floor's and the camera's pitch
  does not touch it — which is the shape as specified, and which means an auto thrown at
  somebody on a ledge above or below misses them by geometry rather than by aim. Airborne it
  tilts with the look. Nobody has played either.
- Should the finishers be visibly greyed out from the wrong side, or hidden entirely?
  Greyed is friendlier and teaches the mechanic; hidden is cleaner to read.
- Does Divide's dash-to-impact work with either form of the ability that hits it, or only
  matching forms? Matching-only would be a strong combo constraint worth testing.
- Naming the two forces.
- **Should the tether pull?** It does not: the dark *auto* is what pulls, and giving the drain
  a pull as well would make one of the two redundant. But a leash that never tugs may read as a
  rope rather than a tether.
- **Should a tether hold the creature?** It catches a Ridgeback with its first pass and then
  parts, because nothing holds a ten-metre animal on a leash. In a hunt that makes the dark
  form strictly the worse of the two, which may or may not be the right answer.
