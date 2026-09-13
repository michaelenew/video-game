---
status: partly unsettled
decided: 2026-09-10
revised: 2026-09-13
---

# Controls

Mouse and keyboard first. A controller scheme is more standard and easier and can follow.

The reference points are platform fighters, which get their feel by **removing a dimension**
so the controls stay small. This game keeps 3D movement, so the compensation is that
direction is *discretised* — four inputs, not a stick — and modifiers do the rest.

> **Revised 2026-09-11.** Sentence 4 below used to read *"space means you move more than you
> otherwise would — alone it jumps, with a direction it dodges"*, and there was a sixth rule,
> *"shift beats WASD when both are held"*. Neither survived the sandbox. Dodge has moved to
> shift and space is now only ever a jump; the knock-on effects are listed under
> [Open](#open-since-the-dodge-moved) and are **not settled**.

## The grammar

Six sentences, and everything else follows:

1. **Click means attack.**
2. **Shift means use an ability** when a click comes with it, and **dodge** when only a
   direction does.
3. **WASD means move.**
4. **Space means jump.** A vertical takeoff, every time, whatever your feet are doing.
5. **`Q` is the class special and `E` is the class mechanic.** The two things only that
   class does, each on its own key. `E` is usually an instant change of state — throw the
   shield, cycle the form, raise a structure — but it does not have to be: on two classes it
   is a fourth ability with a wind-up and a recovery like any other, either because the
   mechanic has no state to change or because changing it is itself a move. See
   [Where `E` is an ability](#where-e-is-an-ability).
6. **The mouse means *where*.** You look with it, you are pointed where you look, and
   your attacks go where you are pointed — **including up and down.** The crosshair is a
   line in space, and an area ability lands on the first thing that line meets.

### Middle click is an attack button again — settled 2026-09-12

Not a reversal of the section below. The special and the mechanic stayed on `Q` and `E`;
what changed is that **middle click is now the third click**, and the option table at the
bottom of this document always counted it as one.

Only the Champion uses it: its three mouse buttons are three weapons — sword, hammer, spear
— so the button that was "the least reachable one" is now a weapon you swing constantly.
That inverts the small problem below. A button nobody presses is found by aim and therefore
badly; a button you press every second is found by use. `U` stands in for it on a hand or a
trackpad that cannot press a scroll wheel at all, the same way `J` and `K` stand in for the
other two.

`E` no longer doubles as middle click in the keyboard bindings, because middle click means
something of its own now.

### Why the special and the mechanic left the mouse

They used to be middle click and shift + middle click. Two problems, and the second is the
real one.

The small problem: middle click is a scroll wheel on most hands, and pressing it reliably in
a fight is not a thing people can do.

The **large** problem: those two inputs are the class. The fire pillar, the uppercut, the
grapple, the shield throw — everything that makes a Bulwark not an Elementalist lives there.
Putting the class identity behind a modifier on the least reachable button said "this is the
optional one", and it played that way: people finished a match without ever pressing it.
`Q` and `E` are two of the three keys a left hand already rests next to.

This costs nothing elsewhere. Shift + click stays the committed attack, and no other input
moved.

### Why space stopped being clever

Space used to mean "you move more than you otherwise would": alone it jumped, with a direction
it dodged. That is a tidy sentence and it is wrong in the hand. You are holding a direction
almost all the time, so the jump button mostly did not jump — you pressed it expecting to
leave the ground and dodged sideways instead. A button whose meaning depends on whether you
happen to be walking is a button you cannot trust.

Shift was already the "stronger version of this" modifier, so dodge went there.

### What disambiguates shift

**A click.** Shift with a click is the stronger version of that attack. Shift with only a
direction is a dodge. The click is checked first, so a committed move thrown while walking
never comes out as a dodge.

### In the air

- **Airdodge**: shift plus a direction, **once per airtime**. It wipes vertical speed rather
  than adding to it, so it is a sideways commitment and never a second jump. A second one
  would turn a jump into flight.
- Space while airborne does nothing, **with one exception**: while a Champion's uppercut has
  hold of somebody, it takes the pair of you higher, once. See
  [kits/champion.md](kits/champion.md#uppercut--middle-click).
- **Airborne attacks are their own moves on the class that has them.** Settled 2026-09-12
  for the Champion and still open for everyone else — see below.

## Open since the dodge moved

Moving dodge onto shift retired **"shift beats WASD when both are held"**, which was the rule
that guaranteed a move-while-casting option always existed. These are consequences, and none
of them is settled:

- **Move + heavy attack is currently impossible.** This is the sharp edge. Shift + direction is
  a dodge and shift + click is the heavy version of an attack, so *holding a direction and
  throwing a heavy* has no input — the dodge takes it. Known and accepted for now; it is the
  first thing the attack grammar has to solve once movement is settled.
- **Differentiating move + attack.** Directional attacks (`w`/`a`/`d`/`s` + click) still work,
  but the modifier space is tighter than it was and the option table below was written under
  the old rule.
- **Aerials as variants — settled for the Champion, 2026-09-12.** The intended direction was
  that an airborne attack is a *variant of its grounded counterpart* — the same button,
  different move — rather than a separate move list, and that is exactly how the Champion is
  built: three buttons, and a row of the move grid per stance. It generalises, and nobody
  else has been given the treatment yet.
- **Neutral shift.** Shift with no direction and no click does nothing. A spot dodge in place
  is the obvious candidate.
- **Double jump.** Space while airborne does nothing. The airdodge is the only air commitment
  at present, which may be too few or exactly right.

## Everything is relative to the camera

`W` is away from the camera, not along some world axis. `D` is to the camera's right.
Aim is the camera direction. The player turns by turning the camera; there is no separate
turn control and no auto-facing.

**The camera sits directly behind the fighter.** Not a preference: the camera points at the
aim point, so an eye slid to one shoulder would turn the whole view and `W` would stop walking
up the screen.

### The camera is prescribed, zone by zone — settled 2026-09-12

**The camera is always on the surface of a sphere, looking inward past a tilt, and the mouse
walks it around that sphere at its own rate.** What the zones change is the sphere: where it is
centred, how big it is, and how far the view is tilted off the line to its centre. Nothing is
solved and nothing can saturate — the eye's place on the sphere is a subtraction.

The trick that makes it work is that **the sphere is centred on whatever is being framed**. The
line from the eye to that centre is the radius the eye is standing on, whichever way round it
has walked, so turning the view up off that line by a fixed angle puts the centre at a fixed
place on the screen — always, for free. A tilt *is* a screen position, written as an angle.

| Zone | Sphere centre | Radius | Tilt |
| --- | --- | --- | --- |
| **−90 to −85** | Not allowed — at the pole the fighter's vertical plane stops being defined | | |
| **−85 to −45** | The feet | Large | From the feet on the crosshair at the bottom, up to 5% |
| **−45 to −10** | The feet | Large | Fixed: the feet 5% up the screen |
| **−10 to 0** | Slides from the feet to the **head** | Large | To the head riding 5% under the crosshair |
| **0 to +10** | The head | Contracts to nearly nothing | To nothing: looking straight down the sight line |
| **+10 to +85** | The head | Small, fixed | None — the eye is the fighter's own |
| **+85 to +90** | Not allowed, as below | | |

Because the eye is placed by the same two angles the aim is made of, **screen centre is the look
direction** and the reticle sits exactly in the middle of the screen by construction rather than
by correction. One degree of mouse is one degree around the sphere, in every zone.

**The crosshair is the aim.** The ray that decides where an ability goes starts at the eye and
runs through the middle of the screen, and it stops at the first of three things: the floor, an
object that is not the floor, or the edge of that ability's own range. A grounded ability lands
exactly there. Anything not grounded targets the middle of a fighter *standing* there when the
ray met the floor, and the point itself when it met anything else. Then the ability is sent along
the line from where it is cast to that point — so what you pointed at is what you get, and the
travel is the fighter's business rather than the camera's.

**The fighter's own body gets out of the way, for two separate reasons.** It goes translucent as
it comes up on the crosshair, because a body the player is aiming past is worse than no body at
all. And it goes fully away when the eye is simply *close* to it — measured as a distance, not
as a zone, so that an arm pulled in by a wall behind the fighter takes the body away exactly the
same as walking into the head on purpose does.

**Zones hand over rather than swap.** A boundary left alone is continuous in position and not
in speed — the eye arrives at it moving one way and leaves moving another, which is not seen so
much as felt, and reads as the camera changing its mind. So each zone's ramp is **eased at both
ends**: it leaves and arrives at a standstill, and a neighbour that is already holding still has
nothing to hand over against. The size of that easing is a percentage of the zone's own span, so
one number means the same thing in a band four degrees wide and one seventy-five degrees wide.

Outside the easing the ramp is untouched, so **the waypoints stay exactly true**; at the maximum
the ramp is eased all the way through and its middle still lands on the waypoint, because the two
ends give back what each other took.

**The shape is the design; the numbers are knobs.** Every boundary angle, both radii, every
percentage and every easing is in the Oven under **Camera**.

**A note on the percentages.** They are written against the framing's own field of view, not the
player's. Set the two to the same number and the fractions are literally what you see; leave them
apart and the fighter sits a little nearer the middle than the knob reads — a one-degree
difference is worth about seven tenths of a percent of screen height.

Two consequences are worth stating because they are design, not implementation:

- **Facing locks the instant a move starts.** During startup, active and recovery frames
  the mouse moves the camera but not the fighter. Otherwise you could drag a live hitbox
  around during its active frames and rescue a whiff by turning after the fact, and whiff
  punishment is most of the game. You commit to a direction when you commit to the move.
- **Guard turns slowly.** Guard covers an arc rather than a bubble ([defense.md](defense.md)),
  and an arc you can flip instantly *is* a bubble. The camera still goes wherever the mouse
  goes; it is the character who cannot reorient that fast.

### Hindrance is proportional to commitment

A move takes your feet away in proportion to how much it commits you.

| | Speed |
| --- | --- |
| Free | 7.0 |
| Throwing a poke | 4.2 |
| Crouching | 3.0 |
| Guarding | 2.0 |
| Committed move | 0 — rooted |

Rooting is what commitment *means*, and it is right for the heavy moves: spacing only matters
if choosing to swing costs you the ability to reposition. It is wrong for a fast poke. The
poke is the neutral tool, thrown constantly, and stopping dead for every one makes neutral
sticky and reads as the game taking the controls away. Slowing you keeps the cost — you
cannot close or escape at full speed while swinging — without the lurch.

Even where rooting is correct, arriving at rooted takes about four frames rather than one.
The snap from a full walk to nothing was the jarring part, not the rooting. The distance slid
while bleeding off is about twenty centimetres: nothing for spacing, everything for how it
reads.

`cargo run -p sim --bin frametable` prints these alongside the frame data, and marks which
moves root you.

### Abilities land where the crosshair is

**Settled 2026-09-12.** An area ability used to appear a fixed distance straight ahead, so
the only way to place one anywhere was to walk there. It now lands where you are pointing.

**The full model is [aiming.md](aiming.md)**, which is the specification and the only place
the rule is written down — everything here is the short version.

One raycast, **from the camera through the crosshair**, ignoring anything behind the
character model. It meets terrain, structures, and the ability's own max-range sphere, and the
first thing it reaches is what you are pointing at. **It goes straight through people** — and
through the creature, which is the reason: up close a big animal fills the screen, so the
reticle sits on its chest well above the thing you meant to hit. The ray picks the *place*; who
is standing in the way of the shot is worked out along the shot's own line afterwards. Then:

- **Aim at a spot inside your reach and a grounded ability goes there.** Exactly there — this
  is the whole point, and it is what makes an area ability a placement decision rather than a
  step-forward decision.
- **Aim at the ground with something that flies** and it goes to that spot, at the height it
  left your hand: level over the place you are pointing rather than into the dirt.
- **Aim at a wall, a person, or the creature with something that flies** and it goes exactly
  there.
- **Aim past your reach and it goes as far along that line as it can.** Range means
  something again.
- **Aim at the sky with something that comes out of the ground** — a pillar of flame, a stone
  — and it arrives at full reach flat ahead. It has to come out of *somewhere*.

**Corrected 2026-09-12, twice.** The rule above used to read "follow the line the player is
looking along, out from the point abilities come out of". That is a ray from the *chest*
along the *look angle*, which is parallel to the crosshair's ray and never converges with it:
the reticle sits on one spot and the ability goes to another, by more the further away it is.
It survived because grounded abilities were separately settled onto the floor, which hid it —
until an ability that flies was built on the same sentence.

**Why a reach sphere and not just the terrain.** Trace to the terrain alone and the target
lurches: aim a hair over the lip of a platform and the hit jumps from two metres away to the
far wall, so a fraction of a degree of mouse movement swings the ability across the arena.
Stopping at the reach bounds that jump to the ability's own range, which is the most it could
ever have meant.

**The target locks when the move starts**, exactly as facing does. A target you could drag
during the startup would let an area be slid onto someone during the wind-up, and the
telegraph is most of what the Elementalist is.

**Pitch is on the wire.** It used to be renderer-local on the grounds that it moved the
camera and nothing else. A crosshair is a line and a line needs two angles, so it is gameplay
now and crosses the network beside the yaw.

### The crosshair never moves

It sits at the exact centre of the screen, always. The **camera** turns to keep the aim point
under it.

That ordering is the design. The alternative — leave the camera pointed along the raw look
axis and slide the reticle to wherever the aim really lands — is equally honest and feels
terrible: a reticle that wanders reads as the aim slipping out of your hands, and the reticle
is the one thing on screen a player is deliberately holding still. The parallax goes into the
view instead, where it is a few degrees of pitch nobody has to fight.

It dims while you are committed to something and the button will not answer.

### Settings

Three numbers are adjustable mid-match. They show above the control legend and are written
to `~/.config/arena/settings.conf` immediately. `ARENA_SETTINGS` overrides the path, which is
how two people on one machine keep separate settings without a profile system.

| Keys | Setting | Step |
| --- | --- | --- |
| `-` / `=` | mouse sensitivity | multiplicative |
| `F3` / `F4` | vertical field of view, degrees | 2° |
| `F5` / `F6` | camera distance, metres | 0.4 m |

Sensitivity steps by a **ratio** and the other two by a fixed amount, because that is how
each is perceived: a given ratio of sensitivity feels like the same change at any value,
whereas two degrees of view is two degrees of view whether you are at 45 or at 90.

Field of view and camera distance are settings rather than constants for a plain reason —
they are the first numbers anyone reaches for when a camera feels wrong, and a value you have
to rebuild to try is a value that gets tried once.

Aim is quantised to 1/65536 of a turn and sent over the wire alongside the buttons, because
where you look decides where you move and what you hit — which makes it gameplay, and
gameplay has to match on both machines exactly. See [architecture.md](architecture.md).

**Shift beats WASD when both are held.** So holding a direction while pressing shift+click
still gives you the shift ability, and you keep moving during it (where the ability allows).
This guarantees a move-while-casting option always exists, and it costs nothing, since
directional abilities and shift abilities were never going to be used simultaneously anyway.

## The option space

| Modifier | × | Button |
| --- | --- | --- |
| none · `w` · `a`/`d` · `s` · `shift` | | `L` · `R` · `M` (scroll click) · `LR` (both) |

Five by four is **20 distinct offensive inputs at any moment**, before airborne variants.
`a` and `d` mirror each other, which is why they count once.

> **This table predates the dodge move** and assumed "shift beats WASD". The counting still
> holds — shift plus a click is still an ability — but the reasoning behind it should be
> re-derived rather than trusted. See [Open](#open-since-the-dodge-moved).

That is a large space. It is not a target.

### Do not fill all twenty

Ship each class with autos, the six-ability kit, and a handful of directional basics. Leave
slots empty. Smash has roughly eighteen moves per character and they are only legible
because they group into families — tilts, smashes, aerials, specials. Twenty times two for
airborne, filled in at prototype stage, produces a class nobody can read.

Empty slots are headroom for the later specialisation layer, not a gap.

## What each region means

This is the part that keeps twenty inputs learnable — each region has one job, and it is the
same job on every class.

| Region | Meaning |
| --- | --- |
| **Unmodified `L`/`R`** | Autos. The neutral vocabulary, roughly shared across classes. |
| **`Q` and `E`** | The class special and the class mechanic. This is where identity lives, and it is different on every class. |
| **Direction + click** | Basic moves. A shared vocabulary — roughly the same shapes on every class. The two casters are the exception. |
| **Shift + click** | The six-ability kit. |
| **Shift + direction** | Dodge — **or the class's own mobility mechanic, where it has one.** Airborne, the once-per-jump airdodge. |

The prototype binds the first three of these: left click pokes, shift + left click is the
committed attack, `Q` is the special and `E` is the mechanic. `J`, `K` and `U` stand in for
the three clicks on keyboards where that is easier.

The Champion does not read that table at all. Its three clicks are three weapons and `E` is
Rush; shift + click and `Q` are unused on it. That is a deliberate exception rather than a
drift — the shared grammar is what lets one control scheme drive six kits, and a class whose
*identity* is which weapon is in its hands has to spend its clicks on the weapons.

That last row does real work. The Reaver's dash to its shadow *is* its dodge rather than an
extra input — thrown forward with the crosshair on the shadow, the roll becomes the crossing. **The Champion's Rush went to `E` instead** — it is the class mechanic in every sense
that matters (one charge, cancels recoveries, changes what the attack buttons do), and
putting it on shift + direction would have made the class's central decision share an input
with the universal defensive one. For the Reaver this is what makes movement and shadow
placement the same action, which is the fix that keeps the class from being denied its
mobility.

### Where `E` is an ability

**Settled 2026-09-12.** The mechanic key is described above as the thing only that class
does, and for most of the roster that is a state change with no frames to it. The Blood mage
is the exception, and the reason is worth stating because it will come up again: **her
mechanic is health**, which is spent by casting rather than by pressing a key, so `E` had
nothing to do and sat unused for the whole of a match.

It casts Black spike now — a real move with a startup you can be punished during, a reach the
crosshair aims, and a cost. Nothing about the grammar changed: `E` still means "the thing
only this class does".

What changed is the move table, which grew a **fourth slot** for it. The other three —
`LMB`, `Shift+LMB`, `Q` — still mean the same thing on every class, which is the property
that lets one control scheme drive six kits. The fourth means whatever that class's mechanic
means, and most classes leave it empty. `cargo run -p sim --bin frametable` prints the key
beside every move.

**The Dual mage is the second, added 2026-09-13**, and it is the same argument from the other
end: her mechanic is a meter, and the meter is steered by *which button attacks* rather than by
a key, so `E` had nothing to toggle either. It casts Sweep.

**The Shadow Reaver is the third, on 2026-09-14**, and she is the one that breaks the rule the
other two were about to establish. That rule was going to be *`E` is an ability exactly when
the class's mechanic has no state to change* — and her mechanic is emphatically a state.

Her mechanic is also not an **instant**: the shadow travels, and calling it back is an attack
that cuts and slows everything on the way home. An instant cannot have a startup somebody
reads, a damage number and a slow, and that one needs all three. So it became a move.

Then it went on **right click** rather than on `E`, because it is aimed and the mouse is where
aiming lives — see [Shadow Reaver](#shadow-reaver) — and `E` picked up the melee it displaced.
Which leaves this class as the one place where `E` carries something that is not the mechanic
at all.

Two rules survive that, and they are worth keeping apart:

> **A mechanic is an ability when pressing it is not free.** Where there is nothing to change
> (health, a meter) or where changing it takes frames and does damage (a second body crossing
> the arena), it needs a slot in the move table like anything else.
>
> **Which *button* it lands on is a separate question, and the crosshair answers it.** An
> aimed mechanic wants the mouse. An unaimed one can have the key.

## Movement

Movement is the core of how the game feels, so it gets pinned down before the attack and
ability grammar is settled around it.

### The jump is variable and floaty

Space is a vertical takeoff. **Hold it to go higher** — while the button is down and you are
still rising, gravity is reduced, up to a cap. **Letting go cuts what is left of the climb**,
once: a second press cannot resurrect a jump you already cut short, or the height stops being
something you chose.

Both mechanisms are needed, and it took a wrong turn to see why. The sustain alone was chosen
first, on the grounds that a cut makes a short hop feel like the jump was taken away from you
while a sustain makes the tall one feel earned. That is true of the *feel* and useless for the
*range*: with only a sustain, the short hop is exactly the sustain multiplier of the full one,
so a multiplier gentle enough to feel good leaves the floor at about two thirds of the ceiling.
That is not a second option, it is the same jump slightly lower.

The release cut lowers the floor without touching the ceiling — a full hop never releases while
rising, so it is untouched — and the short hop scales with the *square* of the cut, since apex
goes as velocity squared. Short hops now land at about a quarter of a full hop.

A short hop is deliberately too low to cross over another fighter; the full hop is for that.
What makes it worth having is that an aerial fits inside it.

Floaty on purpose, and **high** on purpose. Verticality is part of the positioning game, and
Smash characters routinely jump four or more times their own height. A full hop here reaches
**2.3 to 4.8 body heights** depending on class.

There is deliberately **no ceiling on airtime**. What a jump has to be is four separate things,
and each is worth stating on its own rather than collapsing into one frame count:

- **High enough to clear another fighter**, on the *short* hop, or verticality belongs only to
  the committed option.
- **Fast enough not to be a sitting duck** — every class gets above a standing opponent's head
  within 12 frames. Vulnerability while jumping is about the time spent at head height where
  you can be hit, not about total airtime.
- **Steerable enough to dodge** — strafing across a jump carries you at least two body widths
  from where you took off, so a hitbox aimed where you started can be left behind.
- **Slow enough to be punished** — airtime exceeds reaction time plus the fastest poke, so a
  jump can be seen, answered and hit.

`cargo run -p sim --bin frametable` prints the height and airtime for each class.

### Classes differ in the air first

| | Jump | Gravity | Fall cap | Steering |
| --- | --- | --- | --- | --- |
| Bulwark | ×0.9 | ×1.2 | ×1.1 | 0.9 |
| Champion | ×1.0 | ×1.0 | ×1.0 | 1.2 |
| Shadow Reaver | ×1.1 | ×0.9 | ×1.0 | 1.7 |
| Elementalist | ×1.1 | ×0.9 | ×0.9 | 1.0 |
| Blood mage | ×1.0 | ×1.0 | ×1.0 | 1.3 |
| Dual mage | ×1.1 | ×0.8 | ×0.9 | 1.4 |

Weight is the most legible difference a character can have. You can read it across the arena
in the first second of a match, before you know a single one of their moves — so it carries
identity for free, and every class sharing one jump arc would waste the channel. The Bulwark
gets 45 frames of airtime and the Dual mage 74.

`cargo run -p sim --bin frametable` prints these.

### Air control is Quake's, not a second walk

In the air, input **accelerates** rather than assigns, and the acceleration is granted against
*the component of your motion you have not already spent*:

```text
head_room = air_speed − (velocity · wish_direction)
```

Point where you are already going and that projection is large, so there is nothing left to
add — **holding forward in the air does essentially nothing**. Point across your motion and
the projection is near zero, so you get the full budget, which **turns** your velocity without
spending it.

That is the whole reason air movement has a skill ceiling. A single strafe spends a fixed
budget and stops; turning the camera while holding it keeps redefining which direction counts
as perpendicular, so the budget refills against the new heading. A player who does not turn
gets one nudge. A player who does can carve.

**One deliberate divergence from Source:** horizontal air speed is capped at 1.5× the walk. In
Source the gain is unbounded and that unboundedness became the genre; in a fighter built on
spacing, a player who can reach any part of the arena from any other has removed spacing from
the game. The cap is set high enough that good strafing is still rewarded, and whether it is
set right is open.

Momentum carries. Letting go of the stick mid-jump does not stop you — the difference between
a jump being a commitment and a jump being a hover.

### Aerials hang

An attack thrown in the air holds gravity off for a few frames, and **slows** whatever vertical
speed you had rather than deleting it. It is a **per-move property**,
because the hang *is* a move's air identity: a rising strike that holds you up for a beat plays
completely differently from one that drops you through it, and both are worth having. Gravity
is skipped outright rather than reduced, so the hang is a flat number of frames a player can
learn rather than a curve they have to feel.

Deleting the rise was the first attempt and it read as the game snatching the jump out from
under you. The extra control over jump height that attacking gives is worth keeping — it has to
arrive as a slowing rather than a stop, which is also what makes the hang read as float instead
of a pause. Gravity staying off through the window is what keeps it punchy: you hang, you do
not sag.

**A poke also shoves you the way you are holding.** The basic attack is the one you throw
constantly, so this is what makes attacking *part of* air movement rather than a pause in it:
the hang supplies the float, the shove supplies the punch. It needs a direction held, it is
clamped by the air speed cap like any other air movement, and the committed moves get none —
they are already a commitment, and one that also repositioned you would be strictly better than
a poke.

Every aerial hangs a little by default — an attack that drops you straight through gives the
air nothing to offer. Moves that want more say so; the Bulwark's Slam hangs twice as long as a
poke.

### Airborne attacks — ⚠️ open

Grounded and airborne should differ, as they do in every platform fighter. The intended shape,
**not yet settled and not yet implemented**: an aerial is a *variant of its grounded
counterpart* — the same move and the same identity, with different frame data — rather than a
second move list per class. That keeps the vocabulary a player has learned on the ground worth
something in the air, and keeps six classes from turning into twelve movesets.

What exists today: **space is a vertical takeoff**, and **shift plus a direction is an
airdodge, once per airtime**, which wipes vertical speed so it can never be a second jump.
Airborne attacks are still the grounded ones.

---

## The per-class schemes

> **These six sections predate the 2026-09-11 grammar** and are sketches of a twelve-ability
> kit rather than a record of what is in the game. They put the mechanic on `M` or on `R` and
> treat `q` and `e` as spare keys, which is exactly the arrangement the grammar above
> replaced. Read them as intent for the *shape* of each class's full kit; read the kit
> documents in [`kits/`](kits/) and `cargo run -p sim --bin frametable` for what is bound
> today.
>
> Two rows have been brought up to date because the abilities behind them were finished and a
> stale table beside a working implementation is worse than no table: the Elementalist's auto,
> which is a beam now, and the whole of the Blood mage's. The rest have not.

## Dual mage — built 2026-09-13

The mechanic is on the primary buttons, and it is not optional.

**`L` always moves you darker. `R` always moves you lighter.** Every input, not just autos.

What is bound today:

| Input | Result |
| --- | --- |
| `L` | **Dark auto** — a punch with the left arm. Steers dark **on contact**; a whiff steers nothing |
| `R` | **Light auto** — the same punch with the right arm. Steers light on contact |
| `shift` + `L` | **Lance**. Steers dark on the press |
| `shift` + `R` | The light auto again. The light *form* of the committed cast is not built |
| `Q` | **Judgement**, the finisher. Gated on depth |
| `E` | **Sweep**. No side, so it pushes you further along your current path |

**Right click is an attack on this class**, which is the one place the shared grammar bends.
Everywhere else `R` is guard, and guard is shield-gated — this class has no shield, so the
button was doing nothing while half of the mechanic had no input. It is not a special case in
the code either: which move a click asks for is one function
(`state::clicked_move`), and the Champion's three weapons already needed it.

The intended full kit, unbuilt:

| Input | Not built |
| --- | --- |
| direction + `L`/`R` | Basic moves, in dark or light form |
| `shift` + `L`/`R` | Abilities, in dark or light form |
| `M` / `LR` | Gated finishers — Judgement and Eclipse |
| `shift` + `M` / `shift` + `LR` | Ordinary abilities. Push further along your current path |

`M`, `LR`, `Q` and `E` are neither left nor right, so they cannot pick a direction. They push
you further down whichever path you are already on, and at dead centre they do nothing at all.
The grammar stays consistent: **direction comes from side-ness, and only left and right have
it.**

**The autos come out of the two arms** — dark from the left, light from the right — and the
hit volume leaves from that shoulder. It is the class's readout: which arm just landed is
which way the bar moved. See [kits/dual-mage.md](kits/dual-mage.md).

**Autos have a slight range boost** — the beings inside extend your reach. This matters
mechanically, not just as flavour: steering requires landing hits, so the class needs the
reach to steer under pressure.

**Steering is not optional.** You cannot cast without moving the bar, and you cannot move the
bar without committing to a side. This replaces the earlier tap-versus-hold proposal, which
put the direction choice in a modifier the player could ignore.

> **One reservation, for the prototype to settle.** `M` and `LR` are the slowest and least
> reliable inputs on most mice, and they are carrying the finishers — the class's payoff.
> The argument for keeping them there is Smash's: a heavy, deliberate input suits a heavy,
> deliberate move, and the finishers are depth-gated so the input is dead most of the time.
> The argument against is that a dropped payoff in a 60-second match feels terrible. If it
> proves bad, swap the finishers onto `shift`+`L`/`R` and move the ordinary abilities out.

## Champion — built 2026-09-12

**`L` / `M` / `R` are sword / hammer / spear**, and there is no form to switch to: pressing
the button *is* having that weapon in your hands. What the button means never changes. What
changes is which of that weapon's moves comes out, and that is decided by **where your feet
are**.

| | `L` | `M` | `R` |
| --- | --- | --- | --- |
| On foot | Sword — arc across | Hammer — arc down | Spear — a line ahead |
| Airborne | Air sword | Air hammer — spikes | Air spear — a fan around the aim |
| Rushing | Rush slash | Uppercut | Rush stab, or Pole vault aimed at the floor |

| Input | Result |
| --- | --- |
| `E` | **Rush.** One charge. A dash, and it cancels any recovery |
| `space`, airborne, holding somebody | The uppercut's second leap. Both of you go higher |

Ten moves, three buttons, one modifier key, no chords. The full kit is in
[kits/champion.md](kits/champion.md).

**The mid-animation swap still needs no new input**, and it is now easier to say what it
means: press a different weapon's button during active frames and the move ends in that
weapon. It is not built.

The directional variants below are not built on this class and may not be wanted — the
stance rows already do the work that `w`/`a`/`d`/`s` plus a click was meant to do, and
three rows of three is as much as one class should ask a player to hold.

| Input | Not built |
| --- | --- |
| `w` / `a`/`d` / `s` + click | Charge, lateral, low — directional knockback as the combo backbone |
| `shift` + click | The six-ability kit: Drive, Sweep, Throw, Brace |

## Bulwark

| Input | Result |
| --- | --- |
| `L` | Off-hand melee auto |
| `R` (hold) | **Guard.** Opening frames are the parry |
| `M` | **Throw** when held, **Recall** when planted. Reactivate mid-flight to leap to it |
| `shift` + `L` | Bash |
| `shift` + `R` | Slam |
| `shift` + `M` | Grapple |
| direction + `L` | Basic moves |
| `LR` | Reserved — the candidate slot for a dedicated ally-cover stance |

Shield position lives on `M`, so the whole three-state mechanic is one button with context.
Guard on the right button matches every game where alt-fire is the defensive option.

## Shadow Reaver

Current as of 2026-09-14, and the four rows marked **bound** are what the game does.

A half-caster. Click abilities should feel like real melee — strong individually rather than
combo-dependent — and the shadow abilities should reward being close and fast.

| Input | Result |
| --- | --- |
| `L` | **Slash** — the melee auto. The shadow throws it too, a beat later, for a quarter. **Bound** |
| `R` | **Send shadow** — out to the crosshair; pressed again it dashes home through anybody in the way, and drags an open lotus with it. **Bound** |
| `Q` | **Guillotine lotus** — six blades out of the shadow, held open, then chasing it home. **Bound** |
| `E`, or `shift` + `L` | **Executioner** — the committed melee, and an overhead. **Bound** |
| `shift` + forward, crosshair on the shadow | **The dash to it.** Invulnerable across the gap, and it collects the shadow |
| `shift` + forward, anywhere else | The ordinary dodge |
| — | Unplaced: Deadly mistake, which has no button left. Right click ignores `shift` and `shift` + `E` is Executioner |

**The mechanic is on the mouse and the melee is on the key.** That is the whole of what is
unusual here, and it reads as an exception to the grammar when it is the opposite — it is
sentence six of it, applied:

> **The mouse means *where*.**

Sending the shadow is the only thing in this kit the crosshair aims. It is a grounded cast:
you point at a patch of floor and the second body goes and stands there, and where you put it
is the decision the whole class is made of. That wants the hand already doing the pointing.

Executioner does not care where anything is — it is a swing off the body, yaw from the facing
and pitch from the camera — so it can live on a key, and it does. Four of the six classes get
the ordinary arrangement for free by having a mechanic with nothing to aim; this one had to be
told.

It was the other way round for a day, with Executioner on right click and the shadow on `E`,
and the argument against that is not on paper: placing something with the hand that is not
holding the mouse means committing to a spot you are about to stop looking at.

**The dodge is the mobility.** Shift plus a direction is the universal defensive option on
every class, and on this one, pointed at the shadow and thrown forward, it is also the way
to the shadow. One input, and the mouse chooses which. That is what keeps the class from
being denied either half.

## Elementalist and Blood mage

The two full casters. **Every open slot is a unique spell, including the directional
basics** — these classes have no generic basic moves at all, which is itself the
differentiation.

### Elementalist

| Input | Result |
| --- | --- |
| `L` | Ranged auto — a beam along the crosshair, and whatever it meets first |
| `R` | **Raise.** Spawn a structure — the mechanic on a primary button |
| right click | **Cataclysm.** A slow, long-range heavy along the same beam: destroys a structure in a cone blast, turns a fire pillar into a travelling tornado, or lands a real hit on a fighter |
| direction + click | Fissure, Quake, Ice blast |
| `shift` + click | Fire pillar, Flame spitter, and the heavier elemental work |

### Blood mage

Current as of 2026-09-12, and the four rows marked **bound** are what the game does.

| Input | Result |
| --- | --- |
| `L` | **Bloodletter** — the auto. A blade out to a fixed distance and back, cutting on both passes and paying out on the catch. **Bound** |
| `shift` + `L` | **Rend** — the committed melee rake. **Bound** |
| `Q` | **Grasp** — four arms out in a cone that arc inward to converge; all four roots. **Bound** |
| `E` | **Black spike** — a spike in a draining, slowing field, placed at long range. **Bound**, and the one place in the game where the mechanic key is an ability rather than a state change |
| direction + click | Cripple, Affliction, and the reactivating projectile Rend was meant to be |
| the seals | Unplaced. Seal of the unforgiven wants a button of its own and there is not an obvious one |

Every bound ability costs health on the press and returns a share of its damage on the hit.
That is the class mechanic, and it is the only class with a cost in the move table.

## Open questions

- ~~**`q` and `e`.** Currently unused.~~ **Answered 2026-09-11**: they are the class special
  and the class mechanic, and the reasoning is in
  [Why the special and the mechanic left the mouse](#why-the-special-and-the-mechanic-left-the-mouse).
  `q` is free again on the Champion specifically, whose three clicks are its three weapons.
- **Does `s` + click mean "low attack" or "defensive option"?** It should mean one thing
  across all classes. Low attack is the platform-fighter convention.
- **Camera-relative or character-relative direction?** Determines whether `a`/`d` really do
  mirror. Camera-relative is standard in third person and probably correct.
- **Held versus tapped clicks.** Several mechanics already want hold (Guard, charge attacks,
  Rush). Whether hold is a universal modifier or per-ability is unresolved.
