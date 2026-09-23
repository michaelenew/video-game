---
status: partly unsettled
decided: 2026-09-10
revised: 2026-09-16
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
2. **Shift means dodge.** Only that, since 2026-09-16 — see
   [Shift is one verb now](#shift-is-one-verb-now-2026-09-16).
3. **WASD means move.**
4. **Space means jump.** A vertical takeoff, every time, whatever your feet are doing.
5. **`Q` is the class special and `E` is the class mechanic.** The two things only that
   class does, each on its own key. `E` is usually an instant change of state — throw the
   shield, Rush, raise a structure — but it does not have to be: on three classes it
   is an ability with a wind-up and a recovery like any other, either because the
   mechanic has no state to change or because changing it is itself a move, and on a fourth
   it is an instant standing up and an ability off the floor. See
   [Where `E` is an ability](#where-e-is-an-ability).
6. **The mouse means *where*.** You look with it, you are pointed where you look, and
   your attacks go where you are pointed — **including up and down.** The crosshair is a
   line in space, and an area ability lands on the first thing that line meets.

### Middle click is an attack button again — settled 2026-09-12

Not a reversal of the section below. The special and the mechanic stayed on `Q` and `E`;
what changed is that **middle click is now the third click**, and the option table at the
bottom of this document always counted it as one.

Two classes use it. The Champion's three mouse buttons are three weapons — sword, hammer,
spear — so the button that was "the least reachable one" is now a weapon you swing constantly.
That inverts the small problem below. A button nobody presses is found by aim and therefore
badly; a button you press every second is found by use. `U` stands in for it on a hand or a
trackpad that cannot press a scroll wheel at all, the same way `J` and `K` stand in for the
other two.

**And the Dual mage's Lance, since 2026-09-16**, for a reason that is about the button's
*shape* rather than its reachability: middle click has no side. Left is dark and right is light
on that class, so a cast on either of them would push her that way whatever force she was
holding; a cast on the button with no side pushes her further along the path she is already on,
which leaves the light-or-dark form free to come from the arm she last punched with. It is the
only place in the roster where one input throws two different moves.

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

This cost nothing elsewhere at the time: shift + click stayed the committed attack and no
other input moved. Shift stopped meaning that on 2026-09-16 — see
[Shift is one verb now](#shift-is-one-verb-now-2026-09-16) — which is a separate change with a
separate argument, and not a reversal of this one.

### Why space stopped being clever

Space used to mean "you move more than you otherwise would": alone it jumped, with a direction
it dodged. That is a tidy sentence and it is wrong in the hand. You are holding a direction
almost all the time, so the jump button mostly did not jump — you pressed it expecting to
leave the ground and dodged sideways instead. A button whose meaning depends on whether you
happen to be walking is a button you cannot trust.

Shift was already the "stronger version of this" modifier, so dodge went there.

### Shift is one verb now — 2026-09-16

**Shift plus a click is not an attack input any more, on any class.** Shift means dodge and
nothing else.

The section this replaces was called *What disambiguates shift*, and the fact that it needed
one is the argument. Shift had two meanings and the way to tell them apart was whether a click
happened to be held — so what the key did depended on what the rest of your hand was doing,
which is a modifier you cannot trust and cannot teach. Every other key in the game means one
thing.

What it costs is real and is being paid deliberately: **three classes have a committed move
with no input at all** — the Bulwark's Slam, the Elementalist's Fissure and the Blood mage's
Rend. They are still in the move table, still tuned, still printed by the frame table, and a
player cannot throw them. Finding each of them a home is a separate job, one kit at a time,
because the right answer is different per class and guessing three of them at once is how a
grammar gets worse. The Shadow Reaver loses nothing, because her Executioner was already on
`E`. **The Bulwark's is home**, since 2026-09-23: Slam is on middle click, because it is where
the shield's weight is spent and the third click was free — see
[bulwark-v2.md](bulwark-v2.md).

What it bought, immediately, is the Dual mage. Her committed cast moved to **middle click**,
which has no side — so by that class's own rule it pushes her further along whichever way she
is already going, and the light-or-dark *form* of the cast is then free to come from the force
she is carrying. Two rules that had been fighting on shift stopped. See
[kits/dual-mage.md](kits/dual-mage.md).

It also closes the **Move + heavy attack** gap below by removing one side of it: there is no
shift-plus-click to collide with shift-plus-direction any more. What is left open is where the
heavies go.

**A click still wins when both are held.** `w` + shift + click throws the attack rather than
dodging, which is the same precedence the old rule had; the difference is that the click no
longer *changes* which attack it is.

### In the air

- **Airdodge**: shift plus a direction, **once per airtime**. It wipes vertical speed rather
  than adding to it, so it is a sideways commitment and never a second jump. A second one
  would turn a jump into flight.
- Space while airborne does nothing, **with two exceptions**, both per class. While a
  Champion's uppercut has hold of somebody, it takes the pair of you higher, once — see
  [kits/champion.md](kits/champion.md#the-takeoffs--a-weapon-on-the-way-off-the-floor). And
  the Dual mage jumps once more while the lower of her two bars holds three quarters, and on
  every press while she is ascending — earned by the mechanic rather than given by the grammar,
  see [dual-mage.md](dual-mage.md#the-tiers--what-frenzy-does-to-her-body). Added 2026-09-23.
- **Space plus a weapon, on the ground, is a takeoff** — again on the Champion only, added
  2026-09-14. Three moves, one per weapon, thrown as the feet leave the floor. It is the
  first time `space` has modified anything, and the section below on that class is where
  the argument lives.
- **Airborne attacks are their own moves on the class that has them.** Settled 2026-09-12
  for the Champion and still open for everyone else — see below.

## Open since the dodge moved

Moving dodge onto shift retired **"shift beats WASD when both are held"**, which was the rule
that guaranteed a move-while-casting option always existed. These are consequences, and none
of them is settled:

- **Move + heavy attack — half-answered, 2026-09-16.** The collision is gone, because shift
  plus a click is not an input at all now: holding a direction and clicking throws the attack.
  What is not answered is **where the heavies live**, and until that is settled three classes
  have a committed move with no button. See
  [Shift is one verb now](#shift-is-one-verb-now-2026-09-16).
- **Differentiating move + attack.** Directional attacks (`w`/`a`/`d`/`s` + click) still work,
  but the modifier space is tighter than it was and the option table below was written under
  the old rule.
- **Aerials as variants — settled for the Champion, 2026-09-12.** The intended direction was
  that an airborne attack is a *variant of its grounded counterpart* — the same button,
  different move — rather than a separate move list, and that is exactly how the Champion is
  built: three buttons, and a row of the move grid per stance. It generalises, and nobody
  else has been given the treatment yet.
- **Neutral shift.** Shift with no direction does nothing. A spot dodge in place is the
  obvious candidate, and the case for it got stronger on 2026-09-16: shift means one verb now,
  so the neutral case is a gap in that verb rather than a third meaning.
- **Double jump.** Space while airborne does nothing. The airdodge is the only air commitment
  at present, which may be too few or exactly right. **What "airborne" means got stated on
  2026-09-17**, because the obvious reading deleted a technique: it means *nothing under you*.
  A surface under you is a different thing, even a surface that is moving — the Elementalist's
  erupting stone catches her feet mid-rise and she can jump off it again, and off a second one
  raised behind it. That is a jump off a surface, not a jump off the air, and it costs
  structure slots and a one-frame window to set up.

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
ray met the floor, and the point itself when it met anything else — so aiming down at somebody
from a platform lands on them rather than flying out level over their head. Then the ability is sent along
the line from where it is cast to that point — so what you pointed at is what you get, and the
travel is the fighter's business rather than the camera's.

**The fighter's own body gets out of the way, for two separate reasons, and only one of them can
finish the job.** It **dims** as it comes up on the crosshair, because a body the player is
aiming past is worse than no body at all — but it stays a dim, however squarely the head sits
under the reticle. And it goes **fully away** when the eye is simply *close* to it — measured as
a distance, not as a zone, so that an arm pulled in by a wall behind the fighter takes the body
away exactly the same as walking into the head on purpose does.

The asymmetry is deliberate and it is a spacing argument rather than a rendering one. Looking
down puts your own body high on the screen and under the reticle with the camera still a full
nine metres behind you; taking it away there leaves you with no idea where you are standing, and
where you are standing is what every judgement about range is made from. When the eye really is
inside you there was nothing to see anyway, so that is the one case that earns the whole body.
`Body dims to (%)` is the knob.

**Zones hand over rather than swap.** A boundary left alone is continuous in position and not
in speed — the eye arrives at it moving one way and leaves moving another, which is not seen so
much as felt, and reads as the camera changing its mind. So each zone's ramp is **eased at both
ends**: it leaves and arrives at a standstill, and a neighbour that is already holding still has
nothing to hand over against. The size of that easing is a percentage of the zone's own span, so
one number means the same thing in a band four degrees wide and one seventy-five degrees wide.

Outside the easing the ramp is untouched, so **the waypoints stay exactly true**; at the maximum
the ramp is eased all the way through and its middle still lands on the waypoint, because the two
ends give back what each other took.

### Off the ground the camera stops following you — added 2026-09-14

The zones above are a function of **pitch**, and they were authored for a fighter standing on
the floor. That is most of a match and none of the air.

The complaint that found it: *"in the air, aiming down, the bolts go seemingly random
directions nowhere near where the cursor was."* Two things were wrong and they compounded. The
[aiming](aiming.md) half is `aim::standing_middle`, written up there and arrived at from the
other end — the same mistake sent every shot over the head of anybody standing below a
platform. This is the camera half. The sphere is
anchored to the fighter, so a jump takes the eye up five metres with them while the crosshair
stays nailed to the middle of the screen. At a fixed downward pitch the patch of floor under
that crosshair is decided by how high the eye is, so it sweeps metres further out while the
player's hand never moves. There is nothing to learn there; the aim point is simply not yours.

So height is a second driver, and what it does is decline to follow:

> Off the ground, the sphere stays centred **below your feet, on the height you left**. You
> climb the screen toward the crosshair instead of the camera climbing beside you — until you
> are just under it, and from there it follows again.

A vertical impulse already moves you toward the reticle. This is only the camera getting out of
the way and letting it read that way.

**How far it may hold is not a knob, it is the geometry.** It is exactly the rise that takes you
from where the pitch zone has put you on screen up to *the head riding 5% under the crosshair* —
the turn zone's own top waypoint, which is already the sentence "the crosshair rides just above
the head" — and no further, because past that you would be climbing over the reticle. That makes
the awkward cases answer themselves: look steeply down and the floor zone has already carried
you to the crosshair with your feet on the ground, so there is no room left and the camera stays
glued to you, which is exactly what you want when the thing you are looking at is the patch of
floor you are standing over.

Two knobs shape the approach, and both are what keep it from being disorienting:

- **An S-curve in height**, so a hop barely moves it and a real jump takes it all the way over.
  At the current numbers a short hop fits inside the hold entirely: the camera simply does not
  move, and you bob up toward the reticle and back.
- **A rate limit in time, and an asymmetric one** — quick to take hold on the way up, slower to
  let go on the way down. Terminal velocity is eighteen metres a second, so the last few metres
  of a long fall arrive in a handful of frames, and a framing that tracked height exactly would
  snap the view through its whole travel in them. Set the two rates equal and the asymmetry is
  off.

**This makes the framing simulation state**, which is a real cost and worth naming. It has
memory — it may only move so far a frame — and the eye is where the aiming ray starts, so it
lives in the rollback snapshot (`state::Player::aloft`) and is folded into the desync checksum
along with the camera's knobs. A camera with memory that was not in the snapshot would come back
from a rollback framing the fight differently than the first pass did, which is the same bug the
knobs were pulled into the checksum for.

**The shape is the design; the numbers are knobs.** Every boundary angle, both radii, every
percentage and every easing is in the Oven under **Camera**, the airborne framing's six
included.

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
| Committed move | 1.4 — a crawl |

**Nothing roots you.** ⚠️ **Changed 2026-09-14.** The committed moves used to sit at zero, and
rooting was taken to be what commitment *meant*. It is not, and the giveaway is that the
complaint it drew was word for word the one the poke drew three days earlier: a character who
ignores the stick reads as the game taking the controls away. That is true of a heavy move
more than of a poke, not less — a forty-frame commitment is the longest the game ever holds
you, so it is the worst place to hold you *still*.

What commitment means is the other half of it, and that half is untouched: **for the whole of
a move you cannot jump, dodge, guard, or throw anything else.** You have the stick and nothing
else, and the stick is worth 1.4 metres a second. Over the longest heavy in the game that is
under a metre of ground — less than the move's own reach, and less than a single dodge — so
the spacing game the rooting was there to protect never depended on it. Slower than guarding,
which is the slowest thing you can otherwise choose to do, so a committed move is still the
most your feet ever cost you.

**Arriving at the hindered speed takes about four frames, not one.** This is the older half of
the same lesson and it applies at every rung of the table: the snap from a full walk to the
new speed is a separate complaint from the speed itself, and dropping straight to a crawl
would have put the lurch back with a different number underneath it. Direction follows the
stick immediately and only the magnitude bleeds, so you are steering from the first frame —
you are just not going anywhere fast yet. Let the stick go mid-move and the same ramp takes
you to a stop.

`cargo run -p sim --bin frametable` prints these alongside the frame data, and gives every
move its own walking speed in metres.

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
- **Aim at the ground with something that flies** and it goes to that spot raised to the
  **middle of a fighter standing on it** — half a body up from the ground the ray met, not up
  to the height it left your hand. The two are the same number on flat ground and nothing like
  it off it; measuring from the ground the ray met is what makes the rule true from a
  platform.
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

Three numbers are adjustable mid-match. They are written
to `~/.config/arena/settings.conf` immediately. `ARENA_SETTINGS` overrides the path, which is
how two people on one machine keep separate settings without a profile system.

| Keys | Setting | Step |
| --- | --- | --- |
| `-` / `=` | mouse sensitivity | multiplicative |
| `F3` / `F4` | vertical field of view, degrees | 2° |

Sensitivity steps by a **ratio** and the field of view by a fixed amount, because that is how
each is perceived: a given ratio of sensitivity feels like the same change at any value,
whereas two degrees of view is two degrees of view whether you are at 45 or at 90.

**Camera distance is no longer one of them.** It was, and it stopped being one when the eye
became the place the aiming ray starts: the distance is the framing sphere's radius, the
radius decides where the eye is, and the eye decides where your abilities land — so two
players with different distances would place the same fire pillar in different spots. It is
tuned in the Oven under **Camera** with the rest of the framing. Field of view survives as a
setting only because the framing is measured against a tuned field of view of its own, so the
player's choice changes what is projected and never where the eye is. See
[architecture.md](architecture.md).

A setting rather than a constant for a plain reason — the field of view is the first number
anyone reaches for when a camera feels wrong, and a value you have to rebuild to try is a
value that gets tried once.

Aim is quantised to 1/65536 of a turn and sent over the wire alongside the buttons, because
where you look decides where you move and what you hit — which makes it gameplay, and
gameplay has to match on both machines exactly. See [architecture.md](architecture.md).

**Shift means dodge, and nothing disambiguates it any more** — see
[Shift is one verb now](#shift-is-one-verb-now-2026-09-16). A click is checked first, so
holding a direction and clicking gives you the attack and you keep moving during it at
whatever the move's own hindrance allows; what changed on 2026-09-16 is that the click no
longer *changes* which attack it is.

⚠️ Twice over. This used to be the rule *"shift beats WASD when both are held"*, which
guaranteed a move-while-casting option always existed; **dodge moving onto shift retired it**.
Then it was *"a click is what disambiguates shift"*, which retired itself: a key whose meaning
depends on what the rest of your hand is doing is a key you cannot teach. What the first rule
guaranteed has still not been replaced: see [Open](#open-since-the-dodge-moved).

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
| **Unmodified `M`** | The third click. The Champion's hammer; the Dual mage's Lance, where "no side" is the point. |
| **Direction + click** | Basic moves. A shared vocabulary — roughly the same shapes on every class. The two casters are the exception. |
| ~~**Shift + click**~~ | ~~The six-ability kit.~~ **Retired 2026-09-16.** Shift is one verb — see [Shift is one verb now](#shift-is-one-verb-now-2026-09-16). Where the six-ability kit goes instead is open. |
| **Shift + direction** | Dodge — **or the class's own mobility mechanic, where it has one.** Airborne, the once-per-jump airdodge. |

The prototype binds the first four of these: left click pokes, middle click is a second attack
on the two classes that use it, `Q` is the special and `E` is the mechanic. `J`, `K` and `U`
stand in for the three clicks on keyboards where that is easier.

The Champion does not read that table at all. Its three clicks are three weapons and `E` is
Rush; `Q` is unused on it. That is a deliberate exception rather than a drift — the shared
grammar is what lets one control scheme drive six kits, and a class whose *identity* is which
weapon is in its hands has to spend its clicks on the weapons.

That last row does real work. The Reaver's dash to its shadow *is* its dodge rather than an
extra input — thrown forward with the crosshair on the shadow, the roll becomes the crossing,
**on the ground and in the air alike**. **The Champion's Rush went to `E` instead** — it is the class mechanic in every sense
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
`LMB`, the committed slot and `Q` — still mean the same thing on every class, which is the
property that lets one control scheme drive six kits. (The committed slot lost its input on
2026-09-16 and has not got a new one on three of them; see
[Shift is one verb now](#shift-is-one-verb-now-2026-09-16).) The fourth means whatever that class's mechanic
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

Floaty on purpose, and **no longer high on purpose — ⚠️ retuned 2026-09-17.** It reached 2.3
to 4.8 body heights, which is Smash's range, and Smash is a game where the jump *is* the
movement system. Here it is the floor of one: every class has a technique of its own, and a
jump that goes most of the way to the technique without it makes the technique a flourish. The
Champion's pole vault was worth only a quarter more height than pressing space.

So the takeoff came down a twelfth, which is a third off every apex — height goes as the
square of takeoff speed and airtime only as the speed, which is why the height was taken here
rather than out of gravity. A full hop now reaches **1.5 to 3.3 body heights**, still clears a
standing fighter and the arena's platforms on every class, and still lasts most of a second in
the middle of the roster. The sustain was stiffened with it: see the two mechanisms above, and
[feel-log.md](feel-log.md) for why the old pair read as an elevator.

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
gets 54 frames of airtime and the Dual mage 94.

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

**One deliberate divergence from Source:** horizontal air speed is capped at about 2× the
walk (`air speed cap (x walk)` in the Oven). In
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
poke and the Dual mage's autos hang three times as long, which is the longest in the game and
her air identity.

**And each hang in one airtime is worth less than the last — added 2026-09-17.** A hang costs
nothing but the move that carries it, and the repeat lockout only stops *one* move being thrown
twice — so a class with two interchangeable pokes can alternate them and simply never come
down. The Dual mage is that class by construction: her two autos are the same punch mirrored
and throwing them alternately is how she steers her meter.

Compounding rather than a cap, because a cap has an edge somebody finds and plays against and
this has none: the sum of every hang an airtime can contain is `first / (1 - falloff)`, bounded
however long you stay up. Reset on landing, like the airdodge. It applies to every class, so a
multi-aerial string is mildly worse than it was, which is correct for the same reason.

A hang is also always **shorter than the move that carries it** (`sim/tests/feel.rs`). A hang
longer than its move is a float with an attack attached rather than the other way round, and it
would let a whiffed aerial stay safe by simply remaining out of reach.

### Airborne attacks — ⚠️ open

Grounded and airborne should differ, as they do in every platform fighter. The intended shape,
**not yet settled and not yet implemented**: an aerial is a *variant of its grounded
counterpart* — the same move and the same identity, with different frame data — rather than a
second move list per class. That keeps the vocabulary a player has learned on the ground worth
something in the air, and keeps six classes from turning into twelve movesets.

What exists today: **space is a vertical takeoff**, and **shift plus a direction is an
airdodge, once per airtime**, which wipes vertical speed so it can never be a second jump.
Airborne attacks are still the grounded ones.

Two classes spend that airdodge on something else. The Reaver's forward airdodge, thrown with
the crosshair on her shadow, is the **dash to it** — added 2026-09-14, because mobility that
switched off the moment she jumped was mobility in the wrong place. It costs the airdodge
like any other air commitment; what it buys is the whole distance to the second body rather
than a sideways shove. The Dual mage's, while the lower of her bars holds half, is the
**blink** — added 2026-09-23: the same commitment with the travelling taken out, on the ground
and in the air alike, and airborne it costs the airdodge the same way.

---

## The per-class schemes

> **All six sections now carry the grammar above.** They used to predate it — sketches of a
> twelve-ability kit that put the mechanic on `M` or on `R` and treated `q` and `e` as spare
> keys, which is exactly the arrangement the grammar replaced. They were brought up one at a
> time as each class was finished, the Bulwark last, on 2026-09-15.
>
> **Rows marked "bound" are what the game does**; everything else is intent for the shape of
> that class's full kit. `cargo run -p sim --bin frametable` is the authority for the first
> group, and the kit documents in [`kits/`](kits/) for the second.

## Dual mage — built 2026-09-13

The mechanic is on the primary buttons, and it is not optional.

**The autos have sides and nothing else does.** `L` is dark, `R` is light, and throwing one
sets which force she is *carrying*; every other input pushes the bar further along whichever
force that is. See [dual-mage.md](dual-mage.md#the-last-auto-is-the-force-you-are-carrying--revised-2026-09-13),
which is where the rule and the reason for it live.

What is bound today:

| Input | Result |
| --- | --- |
| `L` | **Dark auto** — a punch with the left arm. Steers dark by 5, **on the press**, and she is now dark |
| `R` | **Light auto** — the same punch with the right arm. Steers light by 5, and she is now light |
| `shift` + `L` | **Lance**. Steers 12, in whichever force she is carrying — not dark for being on the left button |
| `shift` + `R` | The light auto again. The light *form* of the committed cast is not built |
| `Q` | **Judgement**, the finisher. **Not gated** — the depth gate was removed 2026-09-13 |
| `E` | **Sweep**. Steers 12, in whichever force she is carrying |

**Right click is an attack on this class**, and by now on most of them. `R` is guard only
where there is a shield to raise, and only the Bulwark has one — so on the three classes with
no shield the button was doing nothing while half of this class's mechanic had no input. It is
not a special case in the code either: which move a click asks for is one function
(`state::clicked_move`), which asks the mechanic rather than the button, and the Champion's
three weapons already needed it.

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
| On foot, hit 1 | Sword — a diagonal, stepping in | Hammer — arc down | Spear — a one-armed jab |
| On foot, hit 2 | Backcut — the mirror diagonal | Uproot | Skewer — wind up, then dash |
| On foot, hit 3 | Upcut — rising | Earthbreaker — unblockable, and it launches | Whirl — swept round the body |
| Airborne | Air sword | Air hammer — spikes | Air spear — a fan around the aim |
| Rushing | Rush slash | Rush sweep — along the floor | Rush stab, or Pole vault aimed at the floor |
| `space` + weapon | Rising cut | Uppercut | Pole drive |

| Input | Result |
| --- | --- |
| `E` | **Rush.** One charge. A dash, and it cancels any recovery |
| `space`, airborne, holding somebody | The uppercut's second leap. Both of you go higher |
| `space` during Earthbreaker | Go up with them: a bigger knock-up, and you leave the floor on the frame it lands. The only place in the game the jump button is read inside another move's frames — see §Open |

Nineteen moves, three buttons, one modifier key, no chords. The full kit is in
[kits/champion.md](kits/champion.md).

### Two rows that are new, 2026-09-14

**The first three rows are one three-hit string.** Connect and the same three buttons throw
the next row; connect again and they throw the last. **Every hit is a free choice of all
three weapons**, which is the class fantasy as an input scheme rather than as a fiction —
sword into spear into hammer is an ordinary thing to do. Nothing remembers what the last
hit was made of.

The string is a **hit confirm**: a connected link cuts its own recovery short so the next
one can begin, and a blocked, parried or whiffed one pays in full. That is why blocking one
hit of a string is worth doing, and why every number in the frame table stays true against
somebody who did.

**`space` plus a weapon is that weapon's takeoff.** It is the one place in the grammar where
two buttons combine on this class, and it is not a chord in the awkward sense: `space` is
already under the thumb and the weapon is already under a finger. The window is a few frames
wide on purpose — *jump then weapon* is the order that works, and both at once is the same
frame — because "attack as you jump" is one intention and two buttons and nobody presses two
buttons on the same tick. A takeoff spends the window, so one jump buys one of them.

**This is the first real use of `space` as a modifier anywhere in the grammar**, and it is
worth flagging as a precedent rather than as a Champion detail: if it reads well here it is
a whole row of options every class could have, and if it reads badly it should not spread.

**The mid-animation swap still needs no new input**, and it is now easier to say what it
means: press a different weapon's button during active frames and the move ends in that
weapon. It is not built, and the chain has taken most of what it was for — see
[champion.md](champion.md#the-core-addition--mid-animation-form-swap).

The directional variants below are not built on this class and may not be wanted — the
stance rows already do the work that `w`/`a`/`d`/`s` plus a click was meant to do, and
three rows of three is as much as one class should ask a player to hold.

| Input | Not built |
| --- | --- |
| `w` / `a`/`d` / `s` + click | Charge, lateral, low — directional knockback as the combo backbone |
| `shift` + click | The six-ability kit: Drive, Sweep, Throw, Brace |

## Bulwark

Current as of 2026-09-23, and the five rows marked **bound** are what the game does. This
table read `M` for the mechanic and `shift` + click for the whole kit until then, which was
the pre-2026-09-11 scheme rather than anything in the game.

| Input | Result |
| --- | --- |
| `L` | **Bash** — the shield strike, and the safe poke. **Bound** |
| `M` | **Slam** — the shield driven into the ground, spending its weight. A crouch does not duck the shake. Thrown in the air it lands with the feet. **Bound** since 2026-09-23; it had no input from 2026-09-16 |
| `Q` | **Grapple** — the command grab, and the answer to a turtle. **Bound** |
| `R` (hold) | **Guard.** Opening frames are the parry, and it is the mechanic that gates it rather than the button: `R` guards while the shield is in hand and does nothing while it is not. **Bound** |
| `E` | **Throw** when held, **Recall** when planted, **leap to it** when it is in flight — and the shield turns to meet the leap, so he arrives with it in hand. One key, three states, no frames — the mechanic fires on the press. **Bound** |
| direction + `L` | Basic moves |
| `LR` | Reserved — the candidate slot for a dedicated ally-cover stance |

Shield position lives on `E`, so the whole three-state mechanic is one key with context —
which is the ordinary arrangement rather than an exception: the shield is a state to change,
and changing it is free. Guard on the right button matches every game where alt-fire is the
defensive option. There is no separate off-hand auto: `L` is the shield itself.

> **The auto and the committed slot are one rung lower than this table used to show.** Bash is
> left click rather than `shift` + left, because the shared grammar gives every class a poke on
> the bare click. Slam was `shift` + left until shift stopped modifying clicks, and is the third
> click now. What the class does *not* have yet is the sixth ability the kit
> document specifies — see [kits/bulwark.md](kits/bulwark.md).

## Shadow Reaver

Current as of 2026-09-14, and the four rows marked **bound** are what the game does.

A half-caster. Click abilities should feel like real melee — strong individually rather than
combo-dependent — and the shadow abilities should reward being close and fast.

| Input | Result |
| --- | --- |
| `L` | **Slash** — the melee auto. The shadow throws it too, a beat later, for a quarter. **Bound** |
| `R` | **Send shadow** — out to the crosshair; pressed again it dashes home through anybody in the way, and drags an open lotus with it. Cuts any recovery short, and the press is remembered for a few frames rather than dropped. **Bound** |
| `Q` | **Guillotine lotus** — twelve blades out of the shadow, held open, then chasing it home. **Bound** |
| `E`, or `shift` + `L` | **Executioner** — the committed melee, and an overhead. **Bound** |
| `shift` + forward, crosshair on the shadow | **The dash to it.** Invulnerable across the gap, and it collects the shadow. Straight to wherever it is standing, up onto a dais included; airborne, it is the airdodge that does it, and it spends the airdodge |
| `space`, in the frames after a dash lands | **The dash jump.** Takes the speed she arrived with up with her, and cuts the dodge's tail short. A press, and only after a dash |
| `shift` + forward, anywhere else | The ordinary dodge |
| — | Unplaced: Deadly mistake, which has no button left. Right click ignores `shift` and `shift` + `E` is Executioner |

**The mechanic is on the mouse and the melee is on the key.** That is the whole of what is
unusual here, and it reads as an exception to the grammar when it is the opposite — it is
sentence six of it, applied:

> **The mouse means *where*.**

Sending the shadow is the only thing in this kit the crosshair aims. It is a grounded cast:
you point at a patch of floor and the second body goes and stands there, and where you put it
is the decision the whole class is made of. That wants the hand already doing the pointing.

### Right click answers whatever else she is doing — added 2026-09-14

It is the second cancel in the game, and the same shape as the first: **Send shadow ends the
recovery of any move it is pressed during**, exactly as the Champion's Rush does. And the press
is **remembered for twenty-four frames** rather than read on one frame and thrown away, so it
comes out on the first frame she can spend it. Being hit throws it away: the memory is there so
her own kit cannot eat the mechanic, and a stun is not her own kit.

Both come from the same place. Every other button here is an attack, and an attack eaten by
another move's frames is the game correctly saying *you were busy*. Right click is not an
attack — it is where her second body stands, which is her way out of both the ordinary limits
on where she can be and the ordinary limits on what she can reach. A mechanic the rest of the
kit can lock her out of is a mechanic behind a timing test.

**Recovery only, and the press never reaches back into a startup or an active frame.** Recovery
is the part that is over — the animation finishing, not the decision. Cancelling a wind-up
would let her take a committed swing back after throwing it, which is whiff punishment deleted.
Being *hit* is not cancellable either: that is the opponent's reward.

**It buys tempo, not safety.** Send shadow costs nineteen frames and the longest recovery it
can cut short is Executioner's twenty-six, so the exchange nets her seven — she is busy for
nearly as long either way, and what changes is that the frames do something. A blocked
Executioner goes from −12 to about −5 and stays a punish. What is *pinned* in
`crates/sim/tests/reaver.rs` is that relationship rather than either number:
`cutting_a_recovery_short_does_not_rescue_her_from_the_punish` fails if the cancel ever turns
a blocked commitment safe, so the two frame counts can be tuned and the rule cannot be lost.

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

Current as of 2026-09-14. **The row is where her feet are** — the same three buttons mean one
thing standing up and another off the floor, which is the Champion's grid read one class
further. See [kits/elementalist.md](kits/elementalist.md) §"In the air".

| Input | Result |
| --- | --- |
| `L` | **Bolt** — the auto. A beam along the crosshair, and whatever it meets first. **Bound** |
| `shift` + `L` | **Fissure** — the committed ground skillshot. **Bound** |
| `Q` | **Fire pillar**, planted where the crosshair is. **Bound** |
| right click | **Cataclysm.** A slow, long-range heavy along the same beam: breaks a structure into thrown debris, turns a fire pillar into a travelling tornado, or lands a real hit on a fighter. **Bound** |
| `E` | **Raise.** Spawn a structure at the crosshair — the mechanic, and an instant with no frames at all. **Bound** |
| `L` in the air | **Air bolt** — a small, fast, long-range shot of air. The air row's poke. **Bound** |
| right click in the air | **Gale** — a disc of air that grows as it travels and is worth what it has become. **Bound** |
| `E` in the air | **Landfall** — the descending slam, and a slab of rock levered out of the floor in front of her. **Bound** |
| direction + click | Quake, Ice blast |
| the rest | Flame spitter, and the heavier elemental work |

`shift` is spent on the ground and **ignored in the air**: shift plus left click is Fissure, a
crack that races along the *ground*, and there is no airborne version of it to reach for. Up
there left click means what left click means.

`E` was listed as `R` here until 2026-09-14, which was the table remembering an older scheme —
Raise was on right click before Cataclysm took the button, and the mechanic key is where it
actually lives.

### Blood mage

Current as of 2026-09-13, and the four rows marked **bound** are what the game does.

| Input | Result |
| --- | --- |
| `L` | **Bloodletter** — the auto. A blade out to a fixed distance and back, cutting on both passes and paying out on the catch. **Bound** |
| `shift` + `L` | **Rend** — the committed melee rake. **Bound** |
| hold `Q` | **Grasp** — four arms out in a cone that arc inward to converge at a depth you chose by how long you held the button, from melee to ten metres over half a second along the line the crosshair picked; all four catch. **Bound** |
| `E` | **Black spike** — a spike in a draining, slowing field, placed at long range. **Bound**, and the one place in the game where the mechanic key is an ability rather than a state change |
| direction + click | Cripple, Affliction, and the reactivating projectile Rend was meant to be |
| the seals | Unplaced. Seal of the unforgiven wants a button of its own and there is not an obvious one |

Every bound ability costs health on the press and returns a share of its damage on the hit.
That is the class mechanic, and it is the only class with a cost in the move table.

**Holding a cast button is new, and it means one thing only: range.** The Grasp is the first
move in the game wound up by how long its button is down, and what the hold buys is how far
out the arms converge — from melee at a tap to ten metres at half a second, with a marker that
leaves the caster's chest and travels outward to show where it currently is. It is aiming, so
the mouse still turns you through it, and the aim locks on release like every other move's
does. The press is what costs health; there is no backing out of it. Both ends of the slider
and its length are move-table knobs, so a second channelled move gets the grammar for free —
see [kits/blood-mage.md](kits/blood-mage.md).

**A channelled move is a skillshot whose line is solved once.** The crosshair picks the line at
the move's full reach, every frame, and the hold picks a point along it — so the line does not
move while the mouse does not, and the marker slides rather than jumps. **The line is read for
its direction only:** how far out the marker gets is the hold and nothing else, so a full hold
aimed at a wall two metres away still reaches ten, through the wall. See
[aiming.md](aiming.md).

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
- ⚠️ **A button pressed *inside* another move's frames — new 2026-09-15.** Jump, during the
  Champion's hammer finisher, is the only one in the game: it is banked while the move winds
  up and spent on the frame it connects, and what it buys is a bigger knock-up and both of
  you leaving the floor together. Everything else in the grammar reads a button when you are
  free to act; this reads one when you are not.

  It is a good input for what it does — "am I going with this one" is a real question and
  the move it hangs off is a twenty-frame telegraph with room in it to decide — but it has
  no precedent and nothing on the HUD suggests it exists. Three ways it could go: it stays
  the one move's rule and is taught by the kit document; it becomes the *hammer's* rule,
  applying to the uppercut as well, which is where the second half of the same idea already
  lives; or it becomes a grammar-wide "commit harder" modifier, which is the ambitious
  reading and needs four other classes to have something to say with it.
