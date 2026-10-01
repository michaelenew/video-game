---
status: built 2026-10-01 (bestiary P4, P5, P6, P7, A5, and the rest of F3b)
---

# The hunt's shared machinery — floor, senses, falls, defended things

[bestiary.md](bestiary.md) §3 lists what has to exist before the creature cast
can be built two at a time. P1 (species), P2 (arenas), P3 (critters) and P8 (the
harness) landed first. This is the rest: **floor hazards** (P4), **perception**
(P5, with the aim change A5), **one fall rule** (the half of P6 every creature
shares), **objectives** (P7), and five smaller things the creature documents
asked of shared code — creatures that collide with solids, the eye under a
ceiling, lengths across a 240 m valley, the sparring bot's walls, and room in
the snapshot for all of it. No creature is built here; each piece is stood up
on a dev creature, **the sentinel**, the way the gnats stood up packs.

The bar was that the Ridgeback, versus and every pinned fight come out
**bit-identical**, and they do: both pinned hunts unchanged, and the fight
report for every class, twelve seeds each, solo and duo, and the gnats, byte
for byte the same.

## 1 · The hunt's lore: one region, laid out per fight

Every creature after the Ridgeback wants a little state of its own, and only
one fight is ever loaded. So the `World` keeps **one fixed region**,
`World::lore` (`sim::lore::Lore`): 24 sixteen-byte **cells**, and which
species' layout they are in. The fight's species declares the layout in its
table (`FightDecl::layout`):

```text
| hazards (P4) | noises (P5) | objectives (P7) | the species' own cells |
```

The three generic lists are one cell per entry, read and written by
`sim::hazard`, `sim::noise` and `sim::objective`. The rest is the species', as
plain words (`Lore::word`, `set_word`, `int`, `set_int`, or whole cells
through `Lore::own`): its file says what each word means. Plain integers
rather than a union of typed structs, because `sim` has no `unsafe` and no
dependencies and a word is the same on every machine.

| Fight | Cells it needs (from its document) | Of 24 |
| --- | --- | --- |
| Siegeshell | 8 vents, 12 falling things, anchors, siege state, the wall | 19 |
| Mireback | 16 pools, braziers, swallow | 18 |
| Veilstalker | 32 footfalls, paint, veil, 4 hazards | 15 |
| Sandmaw | noise ring (8), two sinkholes, spine, attention, swallow | 17 |
| Broodmother | 6 patches, 4 strands, 6 sacs, list | 13 |
| Mantis | sight ring, guard, habits | 7 |
| Hornback | bull's extra, boulders (the rest is in `Pack::memo`), the cart | 6 |
| Galewing | flight, wind, ride cycle (falls are every fighter's) | 3 |
| The Pair | roles, scar, pair brain | 2 |
| Gnawers, Ridgeback | — | 0 |

**The World is 3,912 bytes of 4,096**: 388 of them the region, 8 the fall
height on the two fighters, and 184 to spare (architecture.md §"Where the 4 KiB
goes"). The region is hashed only when a cell is not zero, so the Ridgeback's
fight -- whose lore is never written -- hashes as it did. `tests/lore.rs`
checks every registered species' layout fits and the snapshot with every cell
full.

## 2 · Floor hazards (P4)

`sim::hazard`. **One list, both drawn and hit-tested.** A hazard is one cell:
a kind (an index into the fight species' kinds, plus one), a state byte, an
anchor, a radius in twentieths of a metre (up to 12.75 m), an age, a point in
centimetres (±327 m, which every arena is inside) and, for a line, its other
end. Each frame the world turns the list into a **`Floor`** -- every hazard
placed in the world, anchored ones carried through their creature's pose --
and that one value is what the fighters' feet, the creatures, the aiming ray,
the perception filter, the renderer and the overlay all read.

**Kinds belong to species.** A species declares its kinds in its own file
(`HazardDecl`): a name, a **shape** (a disc, or a strand -- a line with a
half-width, the Broodmother's web), who it **reaches** (fighters, creatures,
critters), and what it is:

| Declared | What it does |
| --- | --- |
| `blocks_sight` | A creature's glance and `aim::in_view` cannot see through its column |
| `ignites_into(k)` | Fire touching it turns it into kind `k`, its age starting again: tar into burning tar, web or smoke into a flash |
| `burning` | It is fire: it lights every igniting kind it touches, every `Spread` frames |
| `becomes(k)` | When its `Life` runs out it becomes `k` (burning tar into slag), or is gone |
| `solid(material)` | It stands as a box of the arena's own kind, `Solid` metres a layer, `state` layers high; a kind becoming a solid where one of its kind stands adds a layer to it (tar burnt on slag) |

and every number about it is a knob, a row per kind in the Oven ("Mireback ·
tar"), `hazard::HazardField`: how long it lasts, how far above the floor it
reaches, **walk, dodge-distance and jump multipliers** (all three at zero is a
root), damage a tick and frames a tick, **pull** toward its centre or its
line, **lift** up its column, how often fire spreads from it, the radius it
takes on becoming this kind, a solid's layer height and most layers, and the
most of it at once (one more and the oldest goes). An unbaked kind's
multipliers start at one, so a new creature's first hazard does nothing until
it is tuned rather than rooting everyone in it.

**What it does to a body.** `World::feel_the_floor`, once a frame before the
fighters step: the walk through the ordinary slow; damage ticking; a grounded
body dragged toward the pull point; anybody in a lift's column pushed up --
only a kind whose lift is above nothing: a solid hazard with no lift (the
Hornback's boulder) once held every body over it in the air. The
dodge and the jump read the same floor where they are thrown
(`step_player`): a dodge from inside tar carries half as far and keeps its
frames and its invulnerability. A slow, a pull and a root are things the floor
does to *feet* -- an airborne fighter keeps the speed it took off with, which is
the Sandmaw's "the air is not sand". Creatures take damage (`Monster::scorch`:
health and strain, no part) and the slow; critters take damage.

**Fire** is anything `EffectKind::ignites` (a fire pillar, a fire tornado,
embers, the fire ring -- declared per kind, like a move's line of effect), a
lit stone, and a fire bolt in flight (`World::fires`). Fire spreads along
touching igniting hazards one hop a frame at most, decided against the list
as it stood at the top of the frame. Judgement's field is light, not fire.

**Solids at runtime.** `sim::arena::Terrain` is **the ground as it stands
this frame**: the arena's table, the solids the fight has raised on it (slag,
and any objective that is a solid), and the placed floor. It replaced `&Arena`
in every query that walks geometry -- the resolve for fighters, stones and
critters, `ground_under`, `material_under`, the aiming ray, the projectiles
leaving, the creature's walls -- and derefs to the arena for its bounds and
marks. With nothing raised it answers every question bit for bit as the arena
alone did (`tests/arena.rs`). At most `MAX_RAISED` (8) solids are raised; a box,
not a cylinder, because the box is the one shape the arena's collision is
written for.

**What a species does with its kinds** -- where a pool lands, whether two
merge, what trips on a strand, when a vent blows -- is its own code: its
`FightDecl::frame` hook runs once a frame after the creatures and the pack have
moved and before the fighters do, with the whole `World`, and `hazard::place`
puts one down (in the first free slot, or over the oldest of its kind).

## 3 · Perception (P5) and A5

`sim::perception`. **Each species has a perception filter**, a function in its
own file (`FightDecl::perceives`), and the glance does not sample a fighter
the filter rules out. A glance that perceives nobody keeps the old sample and
goes on leading it -- which is what makes breaking line of sight a real thing
to do. The filter is handed the creature, its **head** (`Monster::head`, the
last bone of its neck), the fighter as `Quarry` sees them, the scene and the
lore, and is built from pieces:

| Piece | Reads | For |
| --- | --- | --- |
| `sees_all` | -- | The Ridgeback, and every species that says nothing |
| `sees_nobody` | -- | The Sandmaw underground |
| `in_sight_cone` (and `bearing`, for a filter of its own) | `SightCone` | A creature that sees ahead only |
| `in_blind_arc(p, side)` | `BlindArc` | The Pair's scarred cat (its side lives in its lore) |
| `in_line_of_sight` | `aim::sight_clear` | The Pair, and anything that should not see through a wall or smoke |
| `felt` | `FeelRadius`, `FeelHeight` | The Sandmaw's feel radius |

The numbers are the species' **senses row** (`species::FightField`, "Sentinel ·
senses"), which a species has when `FightDecl::row` is set. Whether a straight
line reaches something is the aiming model's question, so the filter asks
`aim::sight_clear` rather than casting a ray itself.

**Hearing.** A species that `hears` is handed, each frame, the noise that
reached its head with the most to spare since its last glance -- its loudness
to that species less its distance -- and when its glance perceives no body it
takes its sample from that: where the noise was, no velocity, and who made it.
The noise ring (`sim::noise`) is a run of cells in the lore: a kind, who, how
big, when, and where. **Noises are made by diffing each fighter's body across
the frame** rather than by a hook at each loud place in a five-thousand-line
step: a footfall when the stride crosses a half-cycle on the floor and not
crouched (on rock or stone, a louder kind), a landing (how far fallen, from
the landing speed), a dodge at both ends, a blow landing (at the swing's own
volume, or where the shot's path ends -- never a new aim), a Rush step, a
stone coming up, a shield planted (its weight), a quake. How far each carries
is the hearing species' own row. A fight with no noise cells makes none.

**The Veilstalker's half** is the other way round -- what the *hunter* can see
-- and two things are shared for it. **`aim::in_view(who, look, at, cone,
scene)`** (A5): is a point inside a cone round a fighter's look, from the eye,
with nothing solid and no smoke in the way (`aim::in_view_of` from an eye and a
look already known). And **`World::shown(slot, part)`**, nought to one, a
species' `FightDecl::shown` hook over its own lore state: the renderer does not
draw a part shown at nothing, and the report and the scripted hunter can read
the same function. How a part shown at *some* is drawn -- a shimmer -- is the
Veilstalker's look to add.

**As built** (the Veilstalker, 2026-10-01): it used all 24 cells -- five for six
hazards (coals ×4, its smoke cloud, a flash), eight for 32 footfalls, one
for four paint marks, and ten of its own words (the glances, the view, the
quills, the mimic, the braziers, the perch, the report's counters). The smoke is the one floor hazard that
`blocks_sight`, and fire turns it into a flash (`ignites_into`). A creature's
view of a hunter is `aim::in_view_from`, from where the hunter stood and the
way they faced, a glance old; the drawn shimmer is a translucent silhouette
in four strengths.

Packs keep their own glance (`pack::Seen`) and perceive every fighter; no pack
creature asked otherwise.

## 4 · One fall rule (P6's shared half)

The Galewing's document had falls free below 7.5 m at 30 a metre, the
Siegeshell's free below 6 m at 18; both keyed to **height fallen**, which is
right -- terminal velocity arrives after four metres, so speed cannot tell a
ledge from the sky. One rule, one pair of knobs ("Air · Fall, free up to",
"damage per metre past it") and a third for the Dual mage's slow fall:

> **A fall is measured from the last thing stood on** -- the floor, a solid, a
> creature's back -- **lowered by any push up on the way down** (a second jump,
> a wing beat, an Updraft, a launch) and never raised by one. Free up to
> **9 m**; **25 a metre** past it, less the height landed on; **halved** for a
> landing slower than **12 m/s**. A creature that carries a fighter up and
> lets go sets where the fall starts (`Player::falling_from`).

Why from the footing and not the apex: a fighter's own jump is not a fall. A
hop off a platform, the Champion's leap (7.9 m in a random-input run), the
Dual mage flying are all free however high they go, and a Galewing's drop or a
shell's crown still hurt.

**Why 9 m**, which moves the Galewing's 7.5: the free height has to be above
everything the proving ground can stand a fighter on, or the pinned hunts would
take fall damage. The Ridgeback's back is 5.5 m standing, but **its shoulders
reach 8.73 m at the top of the rear** (measured over every frame of every clip,
`tests/falls.rs`), and the scripted hunter rides there (8.54 m, across every
class's twelve seeds). So nothing in the proving ground ever leaves the excess
at zero, the field is hashed only when it is not zero, and the pins did not
move. The buck's own damage is the only cost of being thrown off it, so it is
not counted twice. What the numbers make of the two creatures that asked: the
Galewing's 12 m tower top is 75, a talon drop at 12 m 75, its ride at 20 m 275;
the Siegeshell's 14 m rim 125 and its 24 m crown 375. A person should look at
both (§7).

Stored as `Player::fall_over`, **the excess over the free height** -- zero
for every fall that cannot hurt -- 4 bytes a fighter. `state::fall_rule` is the
rule, `state::landing_damage` what a landing costs (the report counts falls
with it).

## 5 · Objectives (P7)

`sim::objective`. A defended thing is **a box standing on the floor, with
health, at a point or somewhere along a route**, split the way a fight already
is: **where** is the arena's (`Arena::sites`: a route of points in
centimetres and the box's size), **what** is the species' (`ObjectiveDecl`:
which site, whether it rolls while escorted, whether it is a solid, whether
breaking it loses the hunt or arriving wins it) with a row of knobs (health,
speed escorted, how near a hunter must be, how much of a creature's blow it
takes), and **how it stands** is one lore cell (health, how far along, what it
has taken, broken, arrived, which creatures' moves have struck it).

- An escorted one rolls at its speed while any standing hunter is within its
  escort distance (the Hornback's cart).
- A creature's move reaches it **with the volume it reaches a fighter with**
  (`Monster::reaches`, and a critter's `Critter::reaches`), once a move, for
  the move's damage times what it takes (the Siegeshell's beam on the wall,
  the bull's charge on the cart).
- A solid one is raised into the `Terrain`: bodies stop at it, the aiming ray
  meets it, a colliding creature walks into it.
- `World::advance` ends the hunt: lost when one that loses it breaks, won when
  one that wins it arrives (or, as before, when every creature is down).
- Drawn as its box with a bar over it for what it has left; the report has a
  line per objective.

The range has two sites for the dev creature's gate and cart. A species
declares objectives for its own arena's sites; where the arena has none, there
are none.

## 6 · Smaller things

- **Creatures collide with solids, opt-in** (`FightDecl::collides`): pushed out
  sideways, at the senses row's `BodyRadius`, from every standing solid taller
  than `StepOver`; never stood on one, never pushed down. The species'
  `bumped` hook is told the push. Critters already stop at solids (P3); a
  pack's `PackMind::bumped` is told when one walks into one, before it slides
  round the face -- which is where the Hornback's bull, a critter, is stunned
  by the rock it charged. **The
  Ridgeback does not collide**, which is what keeps it bit-identical; whether it
  should is an open question (§7).
- **The eye under a ceiling.** `camera::eye_under` holds the eye
  `Aim · Eye, held under a ceiling by` below the lowest hanging solid over the
  eye or the fighter; every eye `aim.rs` takes is this one, and the drawn camera
  starts from it. Where nothing hangs overhead it is the eye exactly.
  [aiming.md](aiming.md) §"The eye under a ceiling".
- **Lengths across the valley.** `math::wide_len`, `wide_flat_len`,
  `wide_flat_dist`, `wide_normalized`: exact at any distance 16.16 holds, and
  **bit-identical to the short ones below a hundred metres a component**, so
  switching to them moved nothing in a 28 m arena. The glance, the creature's
  walk and the straight-line tests (`clear_between`, `blink_to`, `line_clear`,
  `sight_clear`) use them. `math::flat_segment_gap` and
  `math::segment_meets_column` are the strand's and the cloud's shapes.
- **The sparring bot reads its walls off the arena** (`hunt::duel::by_the_wall`
  and `to_the_wall` take the arena's bounds); in the proving ground it is the
  arithmetic it always was, which `crates/hunt/tests/duel.rs` checks number
  for number.

## 7 · The sentinel, and the recipe

**The sentinel** (`--hunt sentinel`, species id 12) is the Ridgeback's body and
moves -- its own statics, and a baked file that began as a copy of the
Ridgeback's -- with everything a creature can bring besides: it sees inside a
72° cone, not through a solid or a cloud and not into a scarred side (its own
word 0); it hears; it collides, and a move that runs it into a solid is knocked
out of its stride; it has every generic hazard kind (tar that burns into slag,
a snare, a sinkhole, smoke and the flash that burns it off, a vent, a strand);
and a gate and a cart where the arena has sites. On the first frame of a round
it lays one of each hazard round the proving ground, and a vent on its own back.
`tests/hazards.rs`, `senses.rs`, `falls.rs`, `objectives.rs` and `lore.rs` stand
everything up on it; `tests/budget.rs` measures it in the range with every
hazard slot full (26–70 µs a frame in release, against 520).

**Adding a creature's share of it** -- the Mireback, say:

1. In `species/mireback/mod.rs`, a `FightDecl` (copy the sentinel's) and
   `fight: &FIGHT` in the table, or `Species::pack_only(..).fighting(&FIGHT)`
   for a pack. Its `layout`: so many hazard cells, noise cells, objective
   cells, and its own. Its kinds (`HazardDecl`), objectives (`ObjectiveDecl`),
   filter, `hears`, `collides`, and its hooks.
2. `cargo run -p sim --bin bake_tuning`, then `--set mireback.tar.walk_speed_in_it_(x)=0.4`
   and so on: the rows are appended after everything the species had, so its
   existing knobs keep their indices.
3. Its own state: give each word a name in a `mod word` and read it through
   `Lore::word` / `int`. `tests/lore.rs` says whether the layout fits.
4. Its rules in its `frame` hook: place, merge, cap, trip -- `hazard::place`,
   `hazard::get`/`set`, `objective::strike`.
5. Its arena's sites, if it defends something: `sites` in `arena/<it>.rs`.

What you should **not** need to touch: `hazard.rs`, `noise.rs`,
`objective.rs`, `perception.rs`, `lore.rs`, `state.rs`, `aim.rs`, the
renderer (`game/src/ground.rs` draws every list), the report. If you do, it is
a place the machinery was not generic yet -- fix it there, for every creature,
and say so here.

## 8 · Decided while building, for a person to review

- **One region laid out per fight** rather than a field per feature (§1). The
  cost is that a species' own state is words, not a struct.
- **The fall rule measures from the footing, free to 9 m, 25 a metre** (§4).
  The Galewing's 7.5 m is gone because the Ridgeback's rear lifts a rider to
  8.73 m; the per-metre number is between the two documents'. Both creatures
  should re-read their fall tables against it.
- **The Ridgeback does not collide with solids.** Turning it on would let it be
  walled by the platforms -- a new fight, and new pins. A person's call.
- **Hazards affect creatures by their position, not their footprint**: a
  burning pool under the toad's middle burns it, one under its flank does not.
  The Mireback decided for itself, in its own `frame` hook: it burns by its
  **footprint**, up to three pools at once (`burn_self_pools`), because a
  12 m animal whose flank stood in fire and felt nothing read as a bug. The
  shared rule is unchanged.
- **Fire spreads after `Spread` frames, not on the first** (the Mireback,
  2026-10-01): a pool that had just caught lit its neighbours the same frame,
  so a fuse ran the arena in a second. Spread is measured shape to shape
  (`Placed::nearest_to`), so a burning strand (the coals) lights what touches
  its line and not only its ends. `hazard::ignite` lights one hazard by hand
  (the belch, a backfire).
- **A slow from the floor lingers one frame** after the feet leave it, because
  it rides the ordinary slow, which counts down at the end of the frame.
- **Noises are made by diffing** the fighters across the frame; a hit's noise
  is at the swing's volume or the shot's end. Footfalls are fighters' only;
  the Veilstalker's own prints are its own lore.
- **The sentinel is a registered species** (id 12). It has no hunter plan, so
  `fight --species sentinel` says so. It was in the picker's `Shift+H` cycle,
  the way the gnats were; *since 2026-10-01* neither is (`SpeciesId::is_dev`,
  [review.md](review.md) `REV-C1`), and `--hunt sentinel` reaches it.
- **A defended thing's box is axis-aligned**: a cart turning a corner turns
  its route, not its box.
- **Not here**: flight and the long ride (the rest of P6, the Galewing's and
  the Siegeshell's), part-frame critters on a moving surface (P3 has them),
  the Hornback's swept charge stop (its `bumped` hook is the place), a
  shimmer pass for a part shown at less than whole, and a debug key that drops
  a hazard under the crosshair.
