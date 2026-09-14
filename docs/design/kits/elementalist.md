---
status: proposed
decided: 2026-09-09
sources: docs/archive/combat-design/elementalist-skills.md, docs/archive/combat-design/class-builds.md
---

# Elementalist — kit

**Identity.** Terrain author. You build the battlefield, then combo through what you built.
Ranged control that creates its own targets.

**Shape of every ability — settled 2026-09-11, from play.** Long, telegraphed startups;
devastating, large follow-through; and only *moderate* frames after, because the cost was
already paid on the front end. That last clause is the part that makes the class playable
rather than merely slow: a long wind-up you also pay for afterwards is a move nobody throws.

The telegraph is not a drawback to be minimised. It is what makes her terrain *fair* — the
opponent gets to see it coming and decide — and it is what makes landing one feel earned. It is
also the first thing the curve harness was built for: a structure now holds barely out of the
floor for the first half of its rise, then erupts, and the duration did not change.

## Mechanic — structures

Physical objects you spawn. Every ability has a second behaviour when it hits one.

- **Cap of three on the field.** Spawning a fourth collapses the oldest. That cap is the
  resource — structures are spent by being consumed in combos, and hoarding them costs you
  new ones.

> **Implemented** (`E`). One press raises one — they **climb out of the floor** over about a
> quarter second, because they are earth. Structures stand in the arena and **have no clock** — the cap is the
> only cost, exactly as written above. They briefly had a lifetime, which meant the fire
> pillar (gated on having one out) silently stopped working ten seconds after you raised one.
>
> **They are solid, and they move.** A stone stops whoever walks into it, holds up whoever
> stands on it, and is pushed by whatever arrives where it already is — up if the new stone
> comes up underneath, sideways if it comes up beside. One raised under another pops it about
> a metre clear; one raised under its *edge* flips it away instead of balancing it there. A
> stone thrown into another hands over the speed it was carrying and both come out slower, so
> a stone cannot be relayed the length of the arena through a row of them.
>
> **Standing on them is the class's floor.** Terrain you cannot get on top of is only cover. A
> stone still climbing carries whoever is on it at the speed its top is climbing, which is
> already worth about 2.9 m against a 2.2 m full hop — that is the seed of the mobility the
> class is meant to get, not the finished thing. It waits on moves that launch stones properly.
>
> **They come up where you are pointing.** `structure_ahead` became a *reach*: the stone rises
> at the spot the crosshair is on, out to 4 m. Look down and it comes up at your own feet,
> which is what the ability description below has always said and what it could not do while
> the stone went a fixed distance straight ahead. See [../controls.md](../controls.md).
- **Contested, not owned.** Enemies can use them as cover, destroy them, and displace them
  short distances with attacks, but cannot combo through them nearly as well. See
  [../elementalist.md](../elementalist.md).
- **The rise is a telegraph with teeth.** The first half is the ground **churning** underfoot —
  a slight slow on anyone standing over it, felt before it is seen, and cleared by getting off
  the floor. The second half is the **eruption**: a little damage and a stagger, once per
  fighter, because a stone erupts once. Never the Elementalist herself — she raises them under
  her own feet on purpose.

> **Implemented.** The two phases come off the rise *curve* rather than a frame count, so
> reshaping the rise moves the telegraph with it. A warning that can drift out of step with
> the thing it is warning about is worse than no warning.

The two overlapping kit versions in the archive are reconciled here; where they disagreed
this kit takes the `elementalist-skills.md` version, which is the later document.

## Element loadout

**Earth is always equipped** — it is what generates structures. A second element slots
alongside it. This kit specifies **earth plus fire**; ice, air, and lightning are the
specialisation axis for later.

Input map in [../controls.md](../controls.md).

## Auto attack

A **beam**, not a bolt. A short wind-up, and then an instant line from her hand
to *whatever the crosshair is on*, out to a short-to-middle distance. Nothing
travels, so there is nothing to lead and nothing to dodge once it is thrown —
what there is instead is a shot that goes exactly where you are pointing.

It is the game's one **skillshot** in the sense [../aiming.md](../aiming.md)
means it, and that document is where the rule lives. The short version: the shot
ends on the point the camera's ray through the crosshair reaches first, and when
that point is the floor it is raised **off that floor** by a fixed height — the
middle of a fighter standing there — so a shot aimed at the ground goes through
whoever is on the spot rather than into the dirt.

*Corrected 2026-09-14.* This said "raised to the height the shot leaves her at",
which is the same number on flat ground and nothing like it anywhere else: from
a platform, or from the air, it meant level over the plane **she** was on rather
than over the place the crosshair was on, and the shot sailed over everybody.
The rule was always meant to be a lift measured from the ground the ray met; the
code says so now and so does this.

> **Implemented** (`L`). What the line reaches **first** is the whole move,
> checked every active frame and spending the move's one hit on whatever it
> finds:
>
> - **a fighter** — small damage, and it takes the move they were winding up.
>   **No stagger at all**: they are free again on the very next frame, and all
>   they have lost is the charge. That trade is the move's identity in neutral.
>   It is why the auto is worth throwing at someone who has already committed
>   rather than only at someone standing still, and it is why it is the
>   cheapest hit in the class — what it buys is an interrupt, not damage;
> - **a structure** — the stone is sent **along the line**: through the ground
>   when she is aimed down it, and up into the air when she is aimed above it.
>   The kick dies off over the back quarter of its travel rather than skidding
>   to a stop on friction alone, so a stone caught early in its flight hits like
>   a boulder and one caught late barely nudges anyone. What it deals to whoever
>   it is still moving fast enough to catch — damage and a stagger — is a
>   function of its speed **relative to the target**, the same quantity two
>   colliding stones already hand each other. The beam never reaches past the
>   structure: aimed through one, it does not also poke whoever is standing
>   beyond it;
> - **fire** — a pillar is a hazard, not a wall, so it does not stop the beam.
>   It **lights** one. A fire bolt leaves the pillar along the same line: fast,
>   small, long range, low-to-middling damage and a little stagger. It is the
>   one thing she throws that has a speed, and it starts *at the fire* — a bolt
>   that came out of her hand instead would make the whole interaction
>   invisible.
>
> **The range is the move's own.** The move table's `reach` is the max-range
> sphere the aiming ray stops at and its `radius` is the line's thickness,
> rather than knobs of their own: the beam *is* the move, and a second copy of
> its range would only be a number the frame table could disagree with. The
> shot is often shorter than the sphere, because it ends on whatever the
> crosshair found. The fire bolt is what carries it *past* that range, which is
> the trade a pillar buys — put fire between you and them and the poke stops
> being a point-blank tool.
>
> **Height decides this one.** It is the first fighter-on-fighter hit in the
> game where it does: a crouch ducks a shot aimed over the head, a shot aimed
> over a stone passes over it instead of kicking it, and someone standing on a
> platform is out of reach of a level shot and in reach of one pointed at them.
> Every other attack compares flat distance and says nothing about height.
>
> **What the shot looks like is what the shot is.** The line is drawn in the
> game as the thin cylinder it is, from her chest to wherever it stopped, and
> her body tilts on to it — spine, chest, shoulders and head — so a shot fired
> forty degrees up is thrown forty degrees up. Both come off `state::hitbox`
> and the aim in the snapshot, so the picture and the rule cannot drift.
>
> This replaces the version recorded in [../feel-log.md](../feel-log.md) as
> "the autos are due a pass": a flat circle at a fixed distance in front of
> her, which meant aiming up did nothing whatsoever, and a fire interaction
> that was an instant long-range hit rather than a projectile. The melee
> classes' equivalent pass, and the timing pass across all six, are still
> outstanding.

**Open, deliberately not built now.** `L` and `R` could both become autos with
longer, independent animation frames — enough that a single click reads as
committed on its own, but landing both in quick succession (left then right,
or the reverse) chains into a stronger combo that a single button mashed
twice cannot reach. The two windows would not overlap, so it is a skill
input — a real read-and-execute — rather than a way to double the DPS of
spamming one button. Right now `R` is Raise, the mechanic input, so this
would also mean finding Raise a new home; it is a kit-wide control question,
not an Elementalist one, and belongs with the rest of [the open control
questions](../controls.md#open-since-the-dodge-moved) rather than being
decided here.

## Abilities

### Raise — mechanic input
**Startup** fast · **Recovery** short · **Range** short · **Mechanic** spawns a structure

Spawn a structure at the cursor. Cast beneath yourself to launch into the air. Cast at an
existing uncaptured structure to kick it forward through the ground for low damage.

> **Implemented, at the cursor.** "At the cursor" is now literal — the stone comes up on the
> first thing the crosshair's line meets, out to Raise's reach. "Beneath yourself" is looking
> down, and the eruption carries you with it.

### Fissure
**Startup** medium · **Recovery** medium · **Range** long · **Mechanic** spawns a structure
at the point of impact

A skillshot that races forward through the ground and stops at the first enemy hit,
staggering them. Leaves a slowing field along its path for several seconds.

### Quake
**Startup** slow, telegraphed · **Recovery** medium · **Range** medium · **Mechanic** spawns
a structure at the centre

A small area shakes immediately, staggering anything moving through it, then erupts after a
delay for moderate damage. The telegraph is the point — it is an area denial tool that
punishes movement, not a damage spell.

### Fire pillar
**Startup** medium · **Recovery** medium · **Range** medium

A focused pillar of flame with a small staggering core and a moderate surrounding area,
planted wherever the crosshair is. It does not need a structure out, and does not touch one
if there happens to be one there.

> **Implemented** (`Q`). The pillar is two volumes rather than one: it starts narrow and
> short, then the **base spreads out** while the **column reaches up** and widens only
> slightly. The base is what catches someone walking past it; the column is what stops them
> jumping over. It burns everyone but the Elementalist, on a tick rather than every frame, and
> it stands long after her recovery frames are over.
>
> **Does not require a structure.** It briefly did — gated on having one out, the same way
> Ice blast and Flame spitter below are written to need one — which was wrong for this move
> specifically: nothing about a pillar of flame planted at the cursor has anything to do with
> a structure being there, and it does not detonate or otherwise touch one that happens to be.
> Earth plus Fire is the loadout, not a dependency chain where Fire only works after Earth has
> gone first.

### Cataclysm
**Startup** slow · **Recovery** slow · **Range** long, skillshot

The right-click heavy. A long wind-up, thrown along the same line the auto follows, that
turns whatever field effect it meets into something worse rather than just damaging it.

> **Implemented** (right click). It reads the same beam as the auto and Fire pillar's
> targeting, and shares their aim -- point it, don't lock onto anything. What it meets
> along that line decides what it does, and neither answer invents a new hitbox to do it:
> both reuse something that already exists rather than detonating an instant bubble.
>
> - **A structure** is destroyed outright and thrown outward as several pieces of debris, in a
>   cone around the line Cataclysm was aimed rather than one blast that lands everywhere at
>   once -- a real cone standing in space, square to the line of effect however it is pitched,
>   not an arc swept flat around the world's vertical axis. Each piece is its own small
>   projectile with its own flight time, so what actually connects depends on how close you
>   were standing and whether you were inside the cone -- a shotgun rather than a bomb, and one
>   you can see coming rather than one that has already landed by the time you notice it. See
>   `crate::debris`.
> - **A fire pillar** is not damaged -- it is transformed. The same `Effect`, the same two
>   volumes a standing pillar already tests against, cut loose from the ground and sent
>   racing along the direction Cataclysm was aimed. It grows on exactly the curve the pillar
>   it came from was already growing on -- one that had barely erupted keeps widening as it
>   goes, one that was already mature stays that size -- rather than snapping to full size or
>   back to nothing the instant it starts moving. The first frame it reaches somebody it lands
>   a real stagger, the same eruption a fire pillar already throws the moment it is cast, and
>   that stagger is not a flourish: a fighter free to act sets his own velocity from the stick
>   every frame, which would cancel the pull below before it ever moved him. Stunned, his
>   velocity only decays, and the pull can win inside that window -- toward the tornado's own
>   live centre, for as long as he is standing in either volume, dragging him along with it
>   rather than merely burning him where he stands. Once the stagger runs out he is free again:
>   walking clear means outrunning the wide base, and jumping or air-dodging clear of the
>   narrower column above it is the faster way out. See
>   `crate::effects::EffectKind::FireTornado`.
> - **A fighter**, hit directly with nothing in the way, just takes a real hit -- heavier
>   than the auto's poke, with its own stagger rather than none.
>
> The wind-up is long enough to be read and punished; the payoff is why you would still
> throw it. Earth plus Fire again: Raise or Fissure to seed a structure, then Cataclysm to
> decide whether it becomes a spray of debris or, by way of a fire pillar first, a moving
> hazard that keeps threatening the space after the swing is over -- the tornado in particular
> is the class's answer to somebody standing at mid range refusing to close: catch him with it
> and he is dragged further from you as it travels, not toward you, which is the opposite of
> what every other catch in this kit does and is the point of it.

### Flame spitter
**Startup** fast · **Recovery** medium · **Range** medium, channelled · **Mechanic** on a
structure, melts it into a lasting magma field that damages and slows

Channelled flame toward the cursor. Damage increases further from the caster, so the
spacing is inverted from most channels — you want them at the tip.

### Ice blast
**Startup** fast · **Recovery** short · **Range** short cone · **Mechanic** launches
structures it hits as projectiles

A quick cone. Structures caught in it are knocked forward, dealing extra damage and
staggering whatever they hit. This is the class's answer to a blocking opponent — the
structure is the guard breaker.

## In the air — built 2026-09-14

**Implemented.** Three moves, and they are the same three buttons she already uses. The row
is where her feet are:

```text
                left click      right click     E
  standing      Bolt            Cataclysm       Raise — the mechanic, an instant
                (shift: Fissure, Q: Fire pillar)
  in the air    Air bolt        Gale            Landfall
```

That shape is the Champion's grid read one class further, and it is deliberately the
[README's](../README.md) open **Aerials** question answered rather than dodged: *airborne
attacks are variants of their grounded counterparts, not a separate move list.* Left click is
still the cheap thing you throw constantly, right click is still the committed one, `E` is
still earth. A player who has learnt her standing up has learnt most of her in the air.

**Shift does not reach up there.** Shift plus left click is Fissure, a crack that races along
the *ground*; there is no airborne version of it to reach for, so the modifier is ignored and
left click means what left click means. Ignoring it has to come out as the Air bolt rather
than as silence, or the input is simply eaten.

### Why air

Earth is the thing she is standing on, and off the floor she is not standing on it. So the two
things she throws with her hands up there are **air** — the element listed above as a later
specialisation axis, borrowed for the one situation where the element she has cannot reach.
The way *back* to earth is to go and hit it, which is Landfall, and it is the only one of the
three that leaves a structure behind.

**Both shots travel**, which nothing she throws standing up does. Bolt and Cataclysm are
instant lines resolved on the frame they come out; these have a speed. The reason is the
situation rather than the element: she is falling while she throws them, and an instant hit
taken from a position she cannot hold would be free. A flight time is what makes being in the
air a trade. See `crate::gust`.

### Air bolt — left click, airborne
**Startup** fast · **Recovery** short · **Range** long, skillshot

A small, fast bolt of air thrown along the crosshair. Low damage and a little stagger — the
poke, and the same trade the grounded auto makes with a body added to it.

> **What it buys is reach.** It is the longest thing in the kit by some way, longer than
> Cataclysm and twice the beam, and that is the whole of what leaving the floor pays for: she
> is committed to an arc, she cannot walk out of what she started, and what she gets for it is
> the ability to touch somebody who thought they were out of the fight. It is slower than a
> fire bolt on purpose — a poke you can see coming is a poke that can be answered, and a
> long-range one that could not be would simply be the correct button.
>
> It carries the aerial shove the other pokes in the game get, so throwing it is *part of*
> moving in the air rather than a pause in it.

### Gale — right click, airborne
**Startup** slow · **Recovery** medium · **Range** long, skillshot

A **frisbee** of air, thrown flat, that opens as it goes and hits harder the wider it has got.

> **It is thrown, not pushed.** The disc's own axis — the one a frisbee spins about — is square
> to the line of effect that aimed it and lies in the *vertical plane containing that line*. So
> a Gale thrown level lies flat, and one thrown down at the floor is tipped nose-down by exactly
> the angle it was thrown at: it slices along its own path, edge leading. It is a thing going
> past you, not a wall of air coming at you.
>
> The hit test agrees and always did. What the shot occupies is a thin disc of its current
> radius riding its line, with its width lying *across* the throw — the victim's standing
> cylinder is swollen by the girth in radius and never in height — so the same clearance that
> saves you by stepping aside does not save you by standing above it, and vice versa. Tipping
> the disc into the plane its path already lies in costs that nothing: the width runs along
> `dir × axis`, which is horizontal whichever way the throw is pitched.
>
> Getting there took two wrong drawings. It was face-on to its own travel first — a picture of
> the one volume the game does not have, and it read from the seat as a disc turned to face
> her. Then it was a flattened *sphere*, which in a barely-opaque material has no flat face and
> no rim and so read as a glowing orb whatever it was scaled to. It is a disc mesh now, with an
> edge you can see it turn on. [../../../CLAUDE.md](../../../CLAUDE.md)'s overlay rule is what
> all of this is in service of, in a place that is not technically an overlay.
>
> **The size is the move.** It leaves her hand at a fraction of its listed radius and comes up
> to full size over a distance of its own, and damage and knockback ride that same fraction —
> so a disc caught at point blank is a puff of air and one caught out at size is the heaviest
> shove in the kit. Every other projectile in the game is worth the same wherever it lands;
> this one is worth what it has *become*.
>
> **How far it goes and how fast it opens are two knobs**, not one. They were one number for a
> day — the growth measured against the reach — and the trouble showed up the first time the
> range was bumped: doubling how far it flew silently halved how big it was everywhere a
> fighter actually stands. `Gale full size after (m)` owns the opening; the move row's `reach`
> owns the flight. It is full size well before it expires, and stays that way, which is the
> shape a thrown disc has anyway: it opens, and then it is open.
>
> That inverts the spacing, and it is the same sentence **Flame spitter** below is already
> written around — *"you want them at the tip"* — said as a thing that flies. The answer to it
> is to close, which is the answer this class least wants you to have and most deserves to be
> given.
>
> **The stun does not scale.** How long a hit holds somebody is frame data, and frame data
> that changed with distance would be a move nobody could learn. Only what it is worth moves.
>
> It is still the slowest thing she throws, and the longest-lived: a disc that has to be walked
> away from is only a decision if there is time to make one.

### Landfall — `E`, airborne
**Startup** slow, telegraphed · **Recovery** medium · **Range** melee, around where she lands
· **Mechanic** drives a structure up in front of her

A descending slam. She hangs, then comes down hard; the arrival staggers a patch of floor for
low damage and levers a slab of rock out of the ground in front of her at forty-five degrees,
throwing whatever was standing there up and away.

> **The wind-up ends on the floor, not on a number.** Landfall is the only move in the game
> whose startup does — the frames in its row are what she is guaranteed to owe (a hang at the
> top, then the drop), and the descent is over when her feet arrive, which from four metres up
> takes longer than from one. So the telegraph is exactly as long as the height she chose to
> open up. Going higher is buying reward with time the opponent gets to use.
>
> **And they can use it.** Everything about the descent is an ordinary startup, so a hit lands
> on her the way it lands on anybody mid-wind-up: she is knocked out of it, and the slab she
> was about to drive up never appears. That is the whole of it, and it needed no rule of its
> own. Contesting the space she is coming down into is the counterplay, and the length of the
> plunge is what makes it findable rather than a read.
>
> **The slab is the payoff, and it is not raised — it is driven.** Two differences from an
> ordinary structure, and they are the same difference twice. It comes out of the floor in
> well under half the time, because the telegraph was the plunge rather than the rise. And it
> comes out **leaning**, forty-five degrees above the floor pointing away from her, so whoever
> is standing over it is thrown up and back along the lean instead of merely staggered where
> they stand. An ordinary eruption does damage and a stagger and leaves you where you were;
> this one clears the space she has just landed in.
>
> It is a structure like any other and it spends the **cap of three**, collapsing the oldest
> if she is already carrying three. A move that raised a fourth for free would be a way around
> the only cost the mechanic has.
>
> **It comes up in front of her, not at the crosshair** — the one placement in the class the
> mouse does not decide, because she is landing rather than aiming. That is `aim::planted_ahead`,
> which lives with the rest of the aiming model for the reason everything else there does; see
> [../aiming.md](../aiming.md).

### What it is for

The slam and the slab are one option, not two: the stagger holds somebody still exactly where
a slab is coming up beside them, and the lean throws them out of the space rather than into
it. Against somebody who has closed on her — which is the position this class least wants to
be in — that is a reset she can take from above rather than a trade she has to win on the
floor. The two shots are the other half: the Air bolt is how she pokes at range she cannot
reach standing up, and the Gale is what she throws at somebody who has to come *through* it.

## Playing it

Raise or Fissure to seed the field, then read the opponent's position and detonate the
structure that catches them. Structures are simultaneously your damage, your cover, and
their cover — the skill is placing them where they serve you more than the opponent.

## Open questions

- **The air row wants playing before any of its numbers are believed.** Three in particular.
  Whether the Gale's near end should be as weak as it is, or whether a disc that is nearly
  worthless at her own feet reads as a bug rather than as spacing. Whether the plunge's length
  *scaling with height* is the right trade or a free reward for pressing it low down — from a
  short hop the telegraph is barely longer than the hang. And whether a fourth structure's
  worth of terrain arriving every time she leaves the floor is more than the cap of three can
  absorb.
- **Landfall's aerials share the grounded clips.** The Air bolt plays Bolt's flick, the Gale
  plays Cataclysm's two-handed throw and Landfall plays Fissure's *hands driven into the
  ground* — each of which is recognisably the right shape, and none of which was authored for
  the air. Clips are a contract (`view::clips`) and the bake refuses to run with one missing,
  so four new ones is a real piece of work rather than a line; it is the obvious next step and
  is logged in [../feel-log.md](../feel-log.md).
- **Do the air shots interact with fire?** A bolt of air through a fire pillar is the
  interaction the loadout is asking for, and nothing has been built: they are stopped by a
  structure like everything else and pass through a pillar without noticing it. The dispatch
  in `crate::gust` is one more branch away from having an answer, which is exactly the shape
  the beam's own fire branch has.
- **Should the Gale kick a stone the way the beam does?** It is the heaviest push she has and
  a wall of moving air arriving at a boulder plainly ought to move it. Today it is stopped by
  one, which is the same answer the fire bolt gives and has the same open question against it.
- **Raising a stone mid-air is gone, and nobody has missed it yet.** `E` off the floor used to
  be Raise, which made a stone under your own feet a sort of second jump. It is Landfall now.
  Whether that pseudo-double-jump was load-bearing for her mobility is a thing to find out by
  playing, not at a desk — and the **Double jump** row in [../README.md](../README.md) is
  where the general version of the question lives.
- **Settled for the auto, open for everything else.** Yes — the beam is blocked by a
  structure in its way, and that self-obstruction is a real cost worth keeping. Fissure,
  Quake and Ice blast are unbuilt skillshots and have not been given the same answer;
  Ice blast in particular *wants* to reach structures rather than be stopped by the
  nearest one, so "blocks" cannot simply mean the same thing for every ability that
  travels.
- **The beam ignores the arena, and so does everything else she throws.** Walls and
  platforms are not traced against, so a shot aimed down at the floor passes through it and
  whiffs rather than stopping short of one. The same is true of Cataclysm's debris, of the
  fire bolt and of both air shots: the only solid any of them knows about is a stone.
  Deliberate for now — stones are the one piece of terrain the shots are *for* — but it is
  the obvious thing to revisit once the arena is more than a blockout, and it has already
  cost one confusing test failure. A stone raised inside the dais's footprint is pushed up
  **on to** it and stands a metre and a half in the air; debris then flies under the stone
  *and through the platform holding it up*, which looks from the outside exactly like debris
  punching through a stone. See `CLEAR_LANE` in `crates/sim/tests/cataclysm.rs`.
- Should the first-thing-it-meets check on the auto also read *black spike*, or anything
  else a future element adds to `effects.rs`? Right now it only recognises fire pillars,
  because fire is the only element that currently ships with the class. The dispatch is
  written so a second kind is one more branch, not a rewrite — nothing has needed the
  second branch yet.
- A fire bolt is stopped by a structure and expires at its range. Whether it should
  instead *kick* one the way the beam does has not been played against: a bolt of fire is
  not a shove, but a stone taking a hit and not moving reads oddly.
- Raise places a stone 2.5 m ahead, so "cast beneath yourself to launch into the air" above
  still has no input. The lift exists; the targeting for it does not.
- Do structures block your own projectiles? Almost certainly yes, and that self-obstruction
  is a real cost worth keeping. They block *bodies* now, the Elementalist's included.
- A stone lifted off centre rides up on the shoulder of the one below rather than sliding off
  it. That is what the arena's own platforms do, and it may want revisiting once moves are
  throwing stones around in earnest.
- A stone stands a whole body height and abilities come out of the chest, so from the ground
  you only ever see a stone's *side*. Stacking one on another by aiming needs you above the
  cap — honest geometry, but it may want an answer.
- Structure durability and displacement force are the tuning knobs, per
  [../elementalist.md](../elementalist.md). Both need a prototype.
- Three may be the wrong cap. It is the number that keeps the arena readable in third
  person, which matters more than the combo ceiling.
