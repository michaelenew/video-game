---
status: decided
decided: 2026-09-12
revised: 2026-10-01 (small bodies: A1 and A2 of the bestiary; the top of a part you stand on: A3)
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

**The angle the ray is built from is `Input::aim`, and that is the whole look.**
Usually it is just the mouse. Standing on the creature it is the mouse plus how
far the animal has turned underneath you, because the ground turning turns your
whole frame of reference with it — `World::advance` folds that in before
anything reads it, so there is one angle rather than one per reader. See
[monsters.md](monsters.md) §3; the failure it prevents is the parallel-ray
mistake wearing different clothes, a camera and a body describing two different
people.

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

**Except the top of a part you could stand on, seen from above** (A3,
2026-10-01, asked by the [Siegeshell](creatures/siegeshell.md) §6). Aboard a
shell twenty metres up, the floor *is* the creature, and with the whole body
off the ray a crosshair on your own feet went through the plateau to the valley
floor: a grounded cast came out twenty metres below, and a skillshot at a rider
two plates over flew at the dirt under them. So the top face of a mountable
part, met by the ray going **down** through it, is a place -- ground, the same
as the top of a platform (`Rig::top_along`, asked only by `aim`). Met from
below or on a side it is still a body and the ray goes through it, which keeps
what the 2026-09-13 change was for: under the rim, the crosshair on the
creature's chest still goes to the floor behind. A part that is buried, not
boardable this frame or too steep to stand on is not a place either -- the same
rules the ride's landing uses.

What it changed, measured: the Ridgeback's bit-for-bit pins did not move (the
pinned hunts never cast down at its back), and of its twelve-seed reports per
class only the Elementalist's and the Blood mage's moved, by a hit or two in a
move's landed count, with the same outcomes -- the cases where a caster on a
platform or aboard puts a grounded cast on its back.

**Fire is also not on the list**, for its own reason: you can see through flame,
so a fire pillar never steals the crosshair — but a shot that *travels through*
one still notices it.

### Ground, and everything else

The one distinction the raycast draws is whether the surface it hit **faces
upward**: the floor, the top of a platform, the top of a stone. That is "the
ground". The side of a platform, the side of a stone and the range sphere are
not. Bodies do not come up: they are not on the ray.

**To an attack, the top of a stone is not ground** (`aim::sight_for_attack`,
2026-09-26). It is ground for *placing*, because it is where the next thing
goes. For a shot it was a trap: somebody standing just in front of a stone puts
the crosshair through them and onto its lid, a body's height up, and a skillshot
raised half a body above *that* went over their head — Cataclysm whiffed at
point blank. So an attack meets a stone exactly where the crosshair touches it,
lid or side, like a wall. The stone stays on the ray rather than coming off it,
because where on a stone you point is a mechanic: the Bolt kicks one along the
line it was shot along.

**To a placement, the Elementalist's stones are not there at all**
(`aim::grounded_path`, 2026-10-02, from play) — **Raise included.** The ray goes
through them, lid and side, to the floor behind, and a point under one settles on
the floor rather than on its lid. Stones used to be ground to a placement, and in
practice that took agency away: a stone under the crosshair moved where the next
thing landed, faster than the player could notice it was there and adjust, and
the opponent could not predict it either — so raising stones to spoil aim was a
viable strategy, which is a degenerate one. **No aiming treats a structure as
ground.** Where a placement lands depends on the floor and the arena, never on
what has been built on it. The cost: stacking a stone on another's cap is gone;
aimed at the floor under one, the new stone erupts beneath it and lifts it. A
Bulwark's planted shield is not one of her structures and is still a wall to
every placement.

## The five lines of effect

Two are **skillshots**: they start with the raycast above and go where it lands.
Three are not: they are pointed by something the player already decided — which
way their body is facing, where they put the mechanic, or where they raised the
stone a crack races from. The last of those does read the ray, but only for a
direction: where it starts and how far it goes were decided already.

Every move declares which, in the move table (`aim::Kind`). There is no sixth.

*Was four until the Elementalist's v2 (2026-09-30)*: Fissure, a crack racing out
of a held stone, was given a line of its own, `aim::racing_path`, and this
section kept saying "there is no fifth" for a week. Brought up to date
2026-10-01 from what `aim.rs` contains.

## The two kinds of skillshot

### Grounded

Things that come out of the floor, whatever the caster was doing when they cast
them: a structure, a fire pillar.

- **Hit the ground** — cast it *exactly* there. Not a pixel different.
- **Hit the max-range sphere** — cast it at max range on the ground, in the
  direction the mouse is facing. Settling the sphere's own point instead would
  make an upward aim land short, which reads as the ability refusing to go where
  it was pointed. **Mounted, "the ground" is the footing** (A3): the creature's
  back under that point if it reaches that far, the floor beyond it if not.
- **Hit the top of a creature's part** (A3) — exactly there, on the shell: it is
  a place the floor does not know about, so nothing settles it.
- **Hit anything else** — a wall — it drops to whatever is underneath,
  because the thing being placed can only exist on the floor.
- **The Elementalist's stones are not on this ray**, for any placement,
  Raise included — see [Ground, and everything else](#ground-and-everything-else).
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
- **Hit terrain that is not ground, a structure (its top included), or the
  range sphere** — the point of intersection, exactly.
- Either way the ability follows a **straight line from the caster to that
  point**, and that line is its whole reach. There is no separate range number:
  the sphere is part of the raycast.

## The three that are not skillshots

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

**Against the surface underfoot** — added 2026-10-01, **A4**, by the
[Galewing](creatures/galewing.md) §6. The horizon the dead zone is measured
from is the surface's, not the world's: `swing_path` takes the `up` of what
the fighter stands on (`aim::underfoot_up` -- `+y` on the floor and on
anything in the arena, the mounted part's own `+y` on a creature) and reads
the look against the plane that up defines, along the facing: the facing laid
onto that plane, its lean out of the world's horizontal, the look less the
lean dead-zoned, and the lean put back. The swing leaves from cast height
*along that up*. On the floor the up is exactly `+y` and the code takes the
old path, so every swing there is bit-identical; a rider on a back banked
forty degrees who looks at the wing root at their feet swings along the back,
not into one wing and over the other. The Ridgeback's shake gets it too.
`a_swing_on_a_banked_back_is_level_with_the_back`.

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

### Racing

From the stone the Elementalist is holding churning — or her own feet, if there
is none — **flat toward the crosshair's spot on the ground**, for as far as the
hold bought. One move: Fissure, `E` held. `aim::racing_path`.

The place it starts was aimed already, with the crosshair, when the stone was
raised; what is chosen now is a direction and a distance, and the distance is
the hold's. The direction is the line from the stone through the spot a
grounded cast would land on (`grounded_path`, asked out to the stone's distance
plus the crack's reach, so the far end of the run is on the floor rather than
the range sphere). The crack runs along that line, past the spot or short of
it as the hold decides: it has no pitch to be given, and one that went shorter
because she happened to be looking down would be aiming twice. A crosshair on
the stone's own spot names no direction, and there the yaw of her look is used.

*Was the yaw of her look until 2026-10-08.* That is parallel to the crosshair
rather than through it, so from a stone off to one side the crack ran past what
the reticle sat on by the stone's whole offset — the chest-ray mistake, made
from a stone. What it meets on the way is `first_along`'s
question, asked when the crack comes out.

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
| **Grounded** | Fire pillar, Black spike, Judgement, Send shadow, Quake |
| **Skillshot** | Bolt, Cataclysm, Air bolt, Gale, Bloodletter, Grasp, Lance |
| **Swing** (the ones worth naming) | Reaping sweep — the Blood mage's scythe, whose reach and width grow with her grey and are drawn, as essence around the weapon, at the size they hit at |
| **Swing** | every melee attack: Bash, Slam, Grapple, Slash, Executioner, Rend, Landfall, the Dual mage's Sweep and both of her autos, and all nineteen of the Champion's |
| **At the mechanic** | Guillotine lotus |
| **Racing** | Fissure |

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

### The questions that are not lines of effect

Everything else in `aim.rs` answers a question about where something is or what
can be seen, and points nothing anywhere: twenty-seven functions as of
2026-10-01, beside the five lines. The raycast itself is `aim::sight` (and
`aim::sight_for_attack`, below); where a cast leaves the body is `aim::origin`
and, for a one-armed move, `aim::hand_origin`; what a path runs into is
`aim::first_along` (§"What the path runs into"). The rest, one question each:

**Is the crosshair on the shadow?** `aim::pointing_at` answers it, and the
Reaver's forward dodge reads the answer to decide whether it is a dodge or the
dash to her second body. It is not a fifth kind of aiming — it points nothing
anywhere — but it lives in `aim.rs` for the same reason everything else here
does. The obvious alternative is an angle between the look direction and the
line to the shadow, worked out beside the dodge, and that is the parallel-ray
mistake in its usual disguise: it agrees with the crosshair at long range and is
out by a whole body at short.

**Is the crosshair on a pool on the floor?** `aim::pointing_at_disc` answers that for the
Blood mage's blink — the same ray, against a short cylinder standing on the pool's own disc
rather than the column a body makes. A disc is as wide as it is drawn and has no height to
speak of, so the slack it needs is above it, not around it, and asking `pointing_at` about a
puddle with a body's column made a blink aimed at the sky go through. It is here for the reason
`pointing_at` is: built from the eye and the look direction.

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

**Where must the mouse be to point at that?** `aim::look_onto` answers that
one — added 2026-09-26, for the sparring bot ([sparring.md](sparring.md)). It is
the raycast run backwards: given a body and a yaw, the pitch that puts the
crosshair on a point. It is not a line of effect and the simulation never calls
it; it is for whatever *plays* — a bot choosing where to put its mouse, the
Elementalist rehearsal aiming at her own feet. It lives here because the answer
depends on where the eye is, and the eye moves round the body as the pitch
changes, so it is settled by a few rounds of asking the camera. A bot that
aimed from its chest along the line to its target would be making the mistake
this document exists to prevent, with a player's hands.

**Which way does a copy thrown from somewhere else face?** `aim::shadow_faces`
answers that one — added 2026-09-23, for the Reaver's v2. Her shadow out on the
field copies her swings, and the copy is still a **swing**: it keeps her pitch,
her shape and her frames. The one thing it cannot take from her is *which way is
forward over there*. On her yaw it landed only on somebody standing at exactly
her offset from it, so it turns to the nearest body within the move's reach plus
a slack, measured to the edge of a fighter's column or to the nearest point of
the creature from the height the swing leaves at. `aim::copied_swing` then
carries her line over to the shadow and turns it onto that yaw — along and
across her facing, the same two amounts along and across the new one, and no
angles. A yaw to the nearest body worked out beside the shadow would be the
mistake again, so it lives here.

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

**Where does a body sent along the floor stop?** `aim::blink_to` answers that
one — the Dual mage's blink, which puts her where the dodge would have ended on
its first frame. Stricter than `clear_between` on purpose: the dash to a shadow
goes wherever the shadow is, up included, so one clear corner line is a way
through; a blink goes along the floor to a spot at her own height, so she stops
where the **first** of the four lines meets something, less her own radius.
**What does this point stand on?** `aim::settle` drops it onto whatever is under
it — a stone's top, a ledge, the floor — for anything that lands a thing
somewhere it was not aimed (a thrown body, a mark on the ground).

**Can a body stand there, and if not, where nearby can it?** `aim::standable`
says whether a settled point is footing: anywhere but a course's drop, whose
floor is a pit rather than ground. `aim::footing_toward` is the Reaver's send
asking it (added 2026-10-04, from play: the shadow dived into the abyss). The
grounded point itself when it is footing; otherwise the first footing scanning
back toward her along the floor, for at most `Send shadow, forgiveness back`
(3 m), then a body's width further onto it; otherwise nothing, and the send
is refused. A small forgiveness and then a refusal, never a search for the
nearest footing anywhere: a placement that goes somewhere the player did not
point is what this document exists to prevent.

**Which way is up underfoot?** `aim::underfoot_up` — `+y` on the floor and on
anything in the arena, the mounted part's own `+y` on a creature. What the
swing's dead zone is measured against (§"Swing", bestiary A4); added
2026-10-01 for the Galewing, whose back tilts under a rider.

**Where was a remembered look pointing?** `aim::in_view_from` is `in_view` for a
look a creature *remembers* — where a fighter stood and which way they faced, a
glance old, with the camera level — and `aim::off_look` how far off that look a
point is, in turns of bearing. The Veilstalker decloaks only where that view
would have shown it, and the report's thirds of the screen read the same
number; both build their eye from `camera::eye_under`, as everything here does.
See §"Two questions about seeing" for `in_view`, `in_view_of` and `on_screen`.

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

### Two questions about seeing (2026-10-01, bestiary P5 and A5)

**Can something at this point see that one?** `aim::sight_clear` answers it:
`aim::line_clear` -- nothing solid between the two points: the arena's solids,
the ones a fight has raised (slag, a wall) and the stones -- and no floor hazard
that **blocks sight** on the way. A cloud is an upright column: a line through
the part of it between its floor and its top is blocked, a line over it is not,
and a watcher standing inside one is blinded by it. Bodies do not block it, as
they are not on the crosshair's ray. A creature's perception filter asks it from
its head (`perception::in_line_of_sight`): the Pair do not see you through a
pillar, the dev sentinel not through smoke.

**Is that point on this fighter's screen?** `aim::in_view(who, look, at,
half_angle, scene)` -- **A5**, asked by the [Veilstalker](creatures/veilstalker.md)
so that it never reveals itself off-screen. Inside a cone of `half_angle` round
the look, from the eye, and `sight_clear` **from the character's chest** to the
point. Built from the eye and the look, so it lives beside `pointing_at`;
`aim::in_view_of` takes an eye and a look direction already known -- the look a
creature glanced some frames ago. A new question rather than a changed answer:
nothing that aimed before aims differently.

`aim::on_screen` is the cone alone, for what the renderer draws over
everything (2026-10-01, the Galewing's report): the floor markers are drawn
on top of the arena, so a lane on the plateau is on the screen of a fighter
standing on a ledge above it with the ledge between them. `in_view` is
`on_screen` and then `sight_clear`.

*Changed 2026-10-01, by the Gnawers' report.* The line of sight ran from the
eye, which this module places nine metres behind and above a standing fighter
-- through any wall at their back. Every bite on a hunter backed against a
wall counted as begun off screen. The renderer's camera is pulled in off that
wall, so what can hide a point is what stands between the character and it:
the cone stays the eye's, the line is the character's.

### The eye under a ceiling (2026-10-01)

The ray starts at the eye, and the eye rides a sphere round the body that can
rise several metres above it. Under a cave's vault that put the start of every
aim inside the rock. **`camera::eye_under` holds the eye
`tuning::eye_under_ceiling` below the lowest ceiling over the eye or over the
fighter** -- a solid hanging from the roof whose underside is above the
fighter's head -- and moves it nowhere else. Where nothing hangs overhead it is
`camera::eye` exactly, which is every arena but a cave, so nothing outside one
moved. Every eye `aim.rs` takes is this one, and the drawn camera
(`view::camera`) starts from it too, so the crosshair stays on the line the ray
follows. `look_onto` still settles against `camera::eye`: it has no arena to
ask, and a bot aiming under a vault is off by the clamp at worst.

**Sloped at the edges since 2026-10-04.** On a jump course every island hangs, so
every island is a ceiling, and the footprint test switched the hold on and off
the frame the eye or the body passed under an edge: the camera and the aim
jumped together ("the camera jumped on me... it can mess someone's aim up").
The ceiling is now `Arena::ceiling_near`: a hanging solid's underside, plus
`Eye, ceiling slope away from an edge` (1 m per metre) for each metre outside its
footprint. Directly under a vault nothing changed; near an edge the eye glides.
`arena::walking_under_an_island_edge_never_jumps_the_eye` walks every course's
islands in 5 cm steps. The drawn camera's arm, pulled in when geometry comes
between it and the fighter, now lets go slowly (`view::camera`, `ARM_OUT`)
instead of snapping back out; the pull-in stays immediate, since anything slower
puts the eye inside the rock.

## Small bodies: how tall "there" is

**Added 2026-10-01, with the critters** ([critters.md](critters.md); bestiary §6
A1 and A2). Everything above was written when the only things that stood were
fighters, so "the middle of whoever stands on that patch" was a constant, and a
level swing at the shoulder met everybody. A knee-high body breaks both: every
skillshot aimed *through* a gnawer was raised to a fighter's middle and sent over
its back, and every standing swing pointed at it stayed level over its head
(the Gnawers' §1a has the arithmetic). The fix is one idea, and it is in this
file so it is true of every ability at once: **"there" has a height, and the
height is whatever stands there.**

- **`aim::stands_at`** casts the crosshair's own ray to the terrain it meets and
  reports the **last body it passed through** on the way -- its height, and where
  its feet are (`aim::Stand`). Fighters count at their *standing* height, so where
  only fighters stand the answer is a fighter's height, exactly as before; crouch
  stays the move table's `hits_crouching`. A creature's skeleton is not a body
  here: it is a place you aim at, and what a shot meets on it is `first_along`'s.
  **Bodies are still not on the ray** -- nothing stops it short, and the
  creature-filling-the-screen bug cannot come back.
- **A ground-aimed skillshot** goes to `standing_middle(ground, height)`: the
  middle of what stands there. On a gnawer that is 0.3 m up, and the line from
  the caster's hand to it passes through the gnawer.
- **A standing swing pointed at something short dips to meet it.** `aim::stoop`
  says how far below the shoulder it should meet the body -- `cast height × (1 −
  h / fighter height)`, the same share of the body's height a level swing meets a
  fighter at -- and `swing_path` tilts the line by the angle that drop makes over
  the distance to the body (or the swing's reach, if shorter). Only standing,
  only looking down, never shallower than the dead zone's own answer, and zero
  where only fighters stand. **It reads the crosshair, not the pack**: a swing
  that looked for the nearest short body and ducked for it would be aim assist.
  Point over a gnawer at the Big One's head and it swings level.

  The Gnawers' document proposed the *middle* of the body for the swing too. The
  instrument (`critcheck`) said otherwise: a swing leaves the hand at cast height,
  not at a fighter's middle, so aiming its line at a 0.6 m body's middle tipped it
  steeper than it meets a fighter and lost reach for nothing. The share of the
  height is what makes a short body behave like a small fighter.
- **Three volumes read the stoop rather than the path's pitch**, because they do
  not follow the pitch: the Dual mage's wing lies flat at the shoulder when she
  stands, and comes down by the drop; a flat disc has no height at all, and goes
  out level along the facing rather than being pulled in toward her; a thrust
  lowered at a gnawer still reaches as far across the floor as it would level.
  All three are exactly what they were among fighters (`Player::stoop` is zero).

**The small bodies are on `first_along`'s list (A2).** With the creatures, under
`Targets::quarry`: a critter's box, swollen by the travelling thing's girth.
What a shot runs into changed, not how it is aimed.

**`aim::line_clear`** is the question the pack asks when it cuts a ring round a
fighter -- is there nothing solid on the straight line from here to there -- and
**`aim::look_onto_closely`** is `look_onto` settled for as many rounds as a steep
look needs: six rounds leave the crosshair a metre above a point forty degrees
below the horizon, which is exactly where a gnawer at three metres is. The bot's
own six rounds are left alone; its pinned fights were played on them.

`cargo run -p sim --bin critcheck` is the instrument: one critter at 1, 2, 3 and
5 m, the crosshair on its middle, every class's every move pressed, beside the
same move against a fighter standing there. Every move touches a 0.6 m gnat
wherever it touches a fighter, except where a lunge carries the fighter through
it (fighters pass through critters) and the Guillotine, whose blades are flat at
0.9 m above the shadow's feet by their own design -- a question for a person, in
[critters.md](critters.md).

## What the path runs into

Separate from the aiming ray, and separate on purpose. The camera's ray says
*where the player is pointing*; this says *what is in the way of the thing they
threw*. The two lines are not the same line — the camera is behind and above —
so a body the camera could not see is still a body the shot passes through.

Each ability states which kinds of thing its path can meet. The Elementalist's
beam meets bodies, stones and **fire**; the fire bolt that a pillar lights meets
bodies and stones but not fire, or it could not leave the pillar that lit it.

**The arena is on the list too, and nothing asks for it — added 2026-09-17.** It
was not there at all before, which was invisible rather than wrong: the
crosshair's ray already stops on terrain, so a shot is aimed at a point the
geometry allows and asking a second time along its own path would only ever
agree with the first answer. What wanted it is a different kind of question —
**is there anything to pull on** — and the Blood mage's Grasp is the ability
asking it. Her blink asks the same question from the other side: is there a wall
in the way of where I am about to be. Both are answered here rather than beside
either of them, so the two cannot disagree about where a wall is.

That is also why `Contact` grew `is_an_anchor`. Terrain, a structure and the
creature are things a thrown rope could hold on to; a fighter and a fire are not
— one of them moves and the other is not there. It is a fact about the world
rather than about the Blood mage, so it lives beside the enum.

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
