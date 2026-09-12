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
> class is meant to get, not the finished thing. It waits on moves that launch stones properly,
> and on Raise gaining a way to target your own feet.
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

Ranged bolt, low damage. A poke, not a win condition — except that it reads
what it is aimed through, which is the terrain-author identity showing up in
the one input every class throws constantly.

> **Implemented** (`L`). What the shot is aimed through decides what it does,
> checked once, on the frame it fires:
>
> - **Aimed through a structure first** — the structure blocks the shot the
>   way it blocks anything else, and the shot becomes the structure's problem
>   instead of the target's. It **kicks the structure forward**, fast, along
>   her facing. The kick dies off over the back quarter of its travel rather
>   than skidding to a stop on friction alone, so a stone caught early in its
>   flight hits like a boulder and one caught late barely nudges anyone. What
>   it deals to whoever it is still moving fast enough to catch — damage and a
>   stagger — is a function of its speed **relative to the target**, the same
>   quantity two colliding stones already hand each other. The bolt itself
>   never reaches past the structure: aimed through one, it does not also poke
>   whoever is standing beyond it.
> - **Aimed through a fire pillar first** — a pillar is a hazard, not a wall,
>   so it does not stop the shot the way a structure does. It **charges** it
>   instead: the same bolt, empowered, still capable of reaching and hitting a
>   fighter beyond the pillar.
> - **Aimed through neither** — the plain poke, unchanged.
>
> Structure and pillar are compared by whichever the line reaches first, so a
> structure sitting in front of a pillar screens it, and a pillar with nothing
> in front of it still empowers a shot that goes on to land. The aim itself
> reaches far further than the poke's own short hit-range against a fighter —
> it has to, to find terrain that may be well past where the poke could ever
> land — which is what makes this the class's first real point-blank-versus-
> range distinction: what she is aimed through can matter long before what she
> is aimed *at* is even in reach.
>
> This is the pass recorded as open in [../feel-log.md](../feel-log.md) under
> "the autos are due a pass" — the Elementalist's autos interacting with
> structures and persistent effects, and the hitscan-circle-at-reach test
> starting to give way to something shaped like an actual shot. The melee
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
**Startup** medium · **Recovery** medium · **Range** medium · **Mechanic** on a structure,
detonates it for a much wider blast

A focused pillar of flame with a small staggering core and a moderate surrounding area.
The bread-and-butter structure detonation.

> **Implemented** (`Q`). The pillar is two volumes rather than one: it starts narrow and
> short, then the **base spreads out** while the **column reaches up** and widens only
> slightly. The base is what catches someone walking past it; the column is what stops them
> jumping over. It burns everyone but the Elementalist, on a tick rather than every frame, and
> it stands long after her recovery frames are over.

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

- **Settled for the auto, open for everything else.** Yes — the bolt is blocked by a
  structure in its way, and that self-obstruction is a real cost worth keeping. Fissure,
  Quake and Ice blast are unbuilt skillshots and have not been given the same answer;
  Ice blast in particular *wants* to reach structures rather than be stopped by the
  nearest one, so "blocks" cannot simply mean the same thing for every ability that
  travels.
- Should the aim-through check on the auto also read *black spike*, or anything else a
  future element adds to `effects.rs`? Right now it only recognises fire pillars,
  because fire is the only element that currently ships with the class. The dispatch is
  written so a second kind is one more branch, not a rewrite — nothing has needed the
  second branch yet.
- Raise places a stone 2.5 m ahead, so "cast beneath yourself to launch into the air" above
  still has no input. The lift exists; the targeting for it does not.
- A stone lifted off centre rides up on the shoulder of the one below rather than sliding off
  it. That is what the arena's own platforms do, and it may want revisiting once moves are
  throwing stones around in earnest.
- Structure durability and displacement force are the tuning knobs, per
  [../elementalist.md](../elementalist.md). Both need a prototype.
- Three may be the wrong cap. It is the number that keeps the arena readable in third
  person, which matters more than the combo ceiling.
