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
ends on the point the camera's ray through the crosshair reaches first, and
when that point is the floor it is raised to the height the shot leaves her at,
so a shot aimed at the ground flies level over the spot rather than into it.

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
> along that line decides what it does:
>
> - **A structure** is destroyed outright and throws a cone-shaped blast out from where it
>   stood, with knockback heavy enough to be the class's real punish rather than a poke.
> - **A fire pillar** is not damaged -- it is transformed. The pillar itself is consumed and
>   a fire tornado is lit in its place, racing off along the direction Cataclysm was aimed,
>   pulling in and burning anyone it catches until its own clock runs out or it leaves the
>   arena. See `crate::tornado`.
> - **A fighter**, hit directly with nothing in the way, just takes a real hit -- heavier
>   than the auto's poke, with its own stagger rather than none.
>
> The wind-up is long enough to be read and punished; the payoff is why you would still
> throw it. Earth plus Fire again: Raise or Fissure to seed a structure, then Cataclysm to
> decide whether it becomes a blast or, by way of a fire pillar first, a moving hazard that
> keeps threatening the space after the swing is over.

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

## Playing it

Raise or Fissure to seed the field, then read the opponent's position and detonate the
structure that catches them. Structures are simultaneously your damage, your cover, and
their cover — the skill is placing them where they serve you more than the opponent.

## Open questions

- **Settled for the auto, open for everything else.** Yes — the beam is blocked by a
  structure in its way, and that self-obstruction is a real cost worth keeping. Fissure,
  Quake and Ice blast are unbuilt skillshots and have not been given the same answer;
  Ice blast in particular *wants* to reach structures rather than be stopped by the
  nearest one, so "blocks" cannot simply mean the same thing for every ability that
  travels.
- **The beam ignores the arena.** Walls and platforms are not traced against, so a shot
  aimed down at the floor passes through it and whiffs rather than stopping short of one.
  Deliberate for now — stones are the one piece of terrain the shot is *for* — but it is
  the obvious thing to revisit once the arena is more than a blockout.
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
