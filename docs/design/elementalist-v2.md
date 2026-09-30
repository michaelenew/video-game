---
status: built 2026-09-30 — every decided item is in the game and unplayed; Blast, Hover and the Trail wait on a word
decided: 2026-09-30
supersedes: the input map, the air row and the "Fissure has no button" state of kits/elementalist.md, once built
sources: kits/elementalist.md, controls.md, exploration/0001–0004, docs/archive/combat-design/elementalist-skills.md
---

# Elementalist — v2: space into damage

**Identity.** Terrain author, as before. What v2 adds is the payoff for authoring it well:
**she converts distance into damage.** Her two placement buttons charge when held, at the
crawl, in full view, and every tenth of a second of charge costs about seven tenths of a metre
of the gap between her and a walking opponent. A player who has built the space gets a strike
nobody else in the roster has; one who has not gets hit out of it. Everything else in v2 is
either how she builds that space or how she spends it.

**The action plan for building this is [plans/elementalist-v2.md](plans/elementalist-v2.md).**
The design thread is [exploration/0001–0004](exploration/SUMMARY.md); this document is the
decisions, and where it and the notes differ, this wins.

## What is wrong today

She plays best of the six and she is five inputs deep. Fissure has had no button since shift
became one verb. The air row is three projectiles that compose with nothing she built. Nothing
in the kit rewards having spaced well beyond not being hit. Her stones are cover for the
opponent — the one ruling in [elementalist.md](elementalist.md) — and she has no answer to it.
And when somebody closes and starts a string, she has a pillar, which does not stop a string.

## Inputs

Four inputs are new to the game — the two mouse side buttons and `F` and `R` — and middle click
is bound for her for the first time. [exploration/0001](exploration/0001_control_budget.md)
is the argument that these are the four the hand can reach without moving; exactly four bits
are free in the input word.

|  | Standing | Airborne |
| --- | --- | --- |
| `L` | **Bolt** — as built | **Air bolt** — more knockback; **lights** through fire |
| `R` | **Cataclysm** — as built | **Gale** — more knockback; **pushes stones**; lights through fire |
| `M` | **Cinder spray** | Cinder spray — the same move; only where it bursts changes |
| `M4` | *proposed:* Blast | *proposed:* Blast |
| `M5` | **Quake** | — |
| `Q` | **Fire pillar** · **held: the Strike** | Fire pillar; a charge begun standing continues while she falls |
| `E` | **Raise** · **held: Fissure** | **Landfall** — as built |
| `F` | **Updraft** | **Downdraft** |
| `R` | **Tremor** | *proposed:* Hover |
| shift, into a stone | **Break through** | Break through |
| shift, through fire | *proposed:* the Trail | *proposed:* the Trail |

Bold is decided. Italic is carried from the exploration notes as a candidate and is not part of
this build unless the person says so. Keyboard stand-ins for the two side buttons, as `J`, `K`
and `U` stand in for the clicks, so a trackpad plays her.

## The mechanic

Structures, cap of three, as built. Two additions:

- **Fire on a stone lights it.** A Strike landing on its top, a Cinder spray bursting on or
  beside it, a burning Gale passing it. A lit stone glows at the seams for a while, burns
  whoever stands on it, and **bursts into burning debris when anyone shoves or breaks it** —
  the beam's kick, a Gale's push, Cataclysm, or the break-through below. This is the answer to
  "her stones are the opponent's cover": **earth builds the field and fire decides who may
  use it.**
- **Rough terrain.** A patch of broken ground that slows whoever crosses it. Fissure's scar
  leaves it; so does the break-through. It is one effect used in several places, and it is
  the setup the Strike wants.

## The charges

Both are the `channel` the move table already has — *what a channel chooses is a number the
move otherwise takes from the table* — and both obey the rule that **a hold deforms a move
along one axis and never selects a different one.** A pillar and a Strike are the ends of one
shape; a stone and a Fissure are the ends of another.

**The startup is the tap window.** The press begins the move's own startup. A release inside it
is the ordinary move; a hold past it is the charge. A tap costs nothing it did not already.

**She crawls while she holds** — the committed speed, the slowest she can move — and she cannot
jump or dodge. The aim is live through the hold and locks on release, as the Grasp's does.

### `Q` held — the Strike

The fire gathers in her hands, brighter and tighter the longer she holds, visible across the
arena. On release, a column of flame **in the pillar's exact shape** lands at the crosshair's
point and is gone in a few frames. The number the channel chooses is **how much of the pillar's
fire arrives at once**: a tap is all burn and no Strike, a full hold is all Strike and no burn,
and in between a flash and then a shorter pillar. The Strike's total against the burn it
replaces is a knob and the one ratio the feel harness pins.

The opponent sees *that* she is charging and how far it reaches, not where it lands. Their
answers are the honest ones: close and hit her while she crawls, leave the range, or put a
stone in the lane — and a Strike that meets a stone lands on its top and lights it.

### `E` held — Fissure

The patch the crosshair picked **churns and does not erupt** — the rise's first half, held. On
release the earth travels: a crack races from the patch along the direction she is looking,
staggering and slowing along its length, and the stone erupts where it stops — at the first
body it meets, or at the end. The number the channel chooses is **how far the crack runs**, the
Grasp's own axis. Where it ran is rough terrain for a few seconds.

Fissure is a Raise that went somewhere. It is a stone placed at range with a telegraph, which
the instant Raise never was, and its scar is what holds somebody in the Strike's footprint.

### The arithmetic

At a walk of 7.0 m/s against her crawl of 1.4, a charge costs the gap about 0.7 m per tenth of
a second held. A full hold of a second is safe at about seven metres, less what a stone in the
lane, rough terrain underfoot or a stagger buys. `frametable` prints the **safe charge
distance** per hold length against the fastest walk in the roster, so the number is read rather
than guessed.

## Quake and Tremor — one effect, two placements

**Quake, `M5`, aimed.** A patch of floor at the crosshair (grounded path, medium range) shakes
through a slow, visible wind-up: anyone *moving* through it staggers, anyone standing still is
fine. Then it erupts — moderate damage to everyone in it — and leaves a stone at its centre.
Area denial that punishes movement, and a third way to place a stone.

**Tremor, `R`, on her.** The same effect centred on her own feet. The ring shakes, movers
stagger, it erupts, and the stone comes up **under her**, taking her with it — the structure
jump with a telegraph attached, for when somebody has closed. Standing on a stone already, it
pops that one instead. One implementation; Tremor is Quake with its centre set to her feet.

## Updraft and Downdraft — `F`, on her body

A cylinder of air about two body widths across, centred on her. It is not aimed: a column at a
point in the air had nowhere for the crosshair to rest, and a column on her needs no crosshair.
(An offset along her flat facing, so that one edge sits at her feet, is a knob at zero; see
`aim::planted_ahead` if it moves.)

**Updraft, standing.** The air gathers for a medium wind-up — dust rising in the footprint — then
one hard gust. **Everything in it goes up, her included**, each by their own class gravity: she
rises a full jump's worth, the Bulwark barely leaves the floor, a stone lifts a body height and
drops. It is her "jump higher" at the cost of a cast rather than a slot, and a body lifted
beside her has one dodge and no guard.

**Downdraft, airborne.** The same cylinder drawn falling, under her. She and everything in it are
driven down. Two things happen if she **lands while it is still blowing**:

- **Over plain ground: a ring of air.** It breaks outward from her feet and shoves everyone
  nearby away a body width or two, no damage. That is the fast defensive tool the kit lacked:
  caught in the air, Downdraft puts her on the floor now and opens space on arrival, and the
  space is what the next charge needs.
- **Into any fire area — a pillar, a Cinder patch or cloud, a Trail, a firestorm's embers: the
  fire goes out and a ring of fire races outward from her.** Fast, low, brief, damage and a
  small outward knockback, and it consumes the source. It is the Strike's idea done with her
  body instead of a hold: a lingering area concentrated into one instant.

A lofted stone in the column comes down as a meteor; a grounded one is pressed into the floor and
leaves rough terrain.

## Cinder spray — `M`, both rows

A thrown ember that **bursts at a fixed distance along the crosshair**, or on the first thing it
meets. It needs nothing for the crosshair to rest on: the range sphere *is* the burst point, a
firework rather than a shot. In the air it leaves a hanging cloud of sparks a couple of metres
across for a couple of seconds; on the floor it leaves a low burning patch. One move; the row
changes only where it pops.

- **Air shots through it ignite.** An Air bolt comes out as fire: more damage and a small burst
  on hit. A Gale comes out burning, a firestorm disc that leaves embers along its path.
- **A stone in it is lit.**
- **It is backup fire** — the pillar is far or burnt out; this is fire where she is looking, in
  a second — and **it is the flare**: fleeing in the air, pop one behind you, and whoever follows
  flies through embers.

## The air row

Nothing lifts. **Air bolt** gains knockback and the fire branch. **Gale** gains knockback, the
fire branch, and **pushes stones** it passes — kicked along its travel, the beam's shove made
wide — so a Gale shoving a lit stone into somebody is the class's air combo, built from three
things she placed. **Landfall** is unchanged. `Q` in the air is the pillar, and a charge begun
on the ground continues while she falls, since she is committed either way.

## The dodge into what she built

**Into a stone — break through.** The dodge carries her into the stone and out the other side;
the stone **breaks down** as she passes, its slot freed, and where it stood is rough terrain.
Whoever was chasing her now has broken ground between them and her, and a lit stone leaves
burning ground instead. It costs the dodge and a structure, which is the right price for a
tool that beats a string rather than a hit.

**Through fire — the Trail** is carried from the notes as a proposal and is not in this build:
the dodge drags a burning line along its path, a wall drawn with her body. It is the natural
partner of the break-through and waits on the person's word.

## Why this shape

Three sentences carry the whole of it, and each has a test behind it.

1. **Distance is the resource the charge spends.** The safe charge distance is printed, and
   the Strike is worth more than the burn only when the hold was paid for.
2. **Every button is a verb typed by what it meets.** The beam already does this; the Gale,
   the Cinder spray and Downdraft now do too. A move whose dispatch table has one row is a
   projectile and needs a reason.
3. **The air row is the ground row with the vertical added or flipped.** Raise and Landfall,
   Updraft and Downdraft, Quake and Tremor. A player who has learnt her standing has learnt
   most of her in the air.

## What it costs to build

- **Four input bits and their bindings**, on both platforms and in the manual's tables, with
  keyboard stand-ins for the side buttons. `read_input` and the web build's controls panel.
- **A channel that chooses different numbers.** The Grasp's channel chooses reach; the Strike
  chooses a fraction and Fissure a distance. What a channel chooses becomes per move.
- **Effects.** Rough terrain (a slow field, also Fissure's scar), the Quake patch, the Cinder
  cloud (the first effect that hangs in the air — effects are grounded today), the fire ring
  (expanding, brief) and the lit-stone flag. Each is bounded, because the snapshot is.
- **A cylinder that lifts by class gravity**, her included — `self_lift` exists, lifting others
  by their own weight does not.
- **The dodge redirected by class**, as the Reaver's and the Blood mage's already are: pointed
  at a stone within reach, it breaks through.
- **Ignition** as one more branch in `gust`'s dispatch, the shape the beam's fire branch has.
- **Clips.** The bake refuses a missing clip, so every new move is animation work as well: a
  charge pose for each hand, the Quake stamp, the Updraft palm, the Cinder throw.
- **Tests.** Safe charge distance; the Strike-to-burn ratio; a hold deforms one axis; every
  air-row move changes a height; the fire ring consumes its source; break-through frees the
  slot and leaves the field.

## Open questions

- **Live aim on the Strike, or locked on press.** Live is the spacing reward; locked is the
  fairer telegraph. Play both.
- **The charge lengths**, and the Strike's full-hold worth against the burn. More is a free
  upgrade, less is a trap; the harness pins the ratio and a person picks the value.
- **Updraft against the structure jump.** Cheaper (no slot) and slower (a wind-up); whether that
  is the right pair or a redundancy.
- **The Downdraft fire ring's radius and speed**, and whether consuming the source is a cost
  the player feels.
- **Break-through's price.** A slot and the dodge; whether the broken ground should also
  stagger, and whether a lit stone's burning ground is too much for a dodge to leave behind.
- **The three candidates** — Blast on `M4`, Hover on `R` airborne, the Trail through fire —
  each waits on a word, and none is needed for the rest to stand.
