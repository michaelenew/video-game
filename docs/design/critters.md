---
status: built 2026-10-01 (bestiary P3, and the aim changes A1 and A2)
---

# Critters — small bodies and the packs they run in

[bestiary.md](bestiary.md) P3. Four creatures are many small bodies, or bring
them: the [Gnawers](creatures/gnawers.md) are a pack, the
[Hornback](creatures/hornback.md) a herd, the
[Broodmother](creatures/broodmother.md) breeds hers, and the
[Siegeshell](creatures/siegeshell.md) carries parasites on its back. A full
`Monster` is two hundred bytes of skeleton nobody stands on, so these get a
**critter** -- a box on its feet -- and a **pack brain** that drives up to ten
of them. This is what was built, the decisions it made, and **the recipe for a
critter species and a pack**, which is the part the creature branches read. It
is the small-body half of [species.md](species.md); a creature branch follows
both, and [arenas.md](arenas.md).

What was proven, and how: a dev pack, **the gnats** (`--hunt gnats`; six gnats
0.6 m at the crown and a queen that leads them), stands up every piece below in
`crates/sim/tests/critters.rs`. The Ridgeback, versus and every pinned fight are
**bit-identical**: both pinned hunts unchanged, and the fight report for every
class, twelve seeds each, solo and duo, byte for byte the same.

## 1 · A critter

`sim::critter::Critter`, **48 bytes**: position, velocity, yaw, health, a state
and its timer, an **animation clock**, its kind, the move it is throwing, flags
(holds a token, hit spent, leader, airborne, struck by each fighter's swing, a
detour's side), a role, a ring place, a target, and -- for standing on a
creature -- a mount byte and a perch. `World::critters` holds ten
(`critter::MAX_CRITTERS`: the Gnawers' coop pack is nine and the Big One) and
says whose they are; `World::pack` is the brain. Dead bodies keep their slot
and their clock, so a corpse fading out does not pop on a rollback, until the
slot is needed. The snapshot is **3,520 bytes** (from 2,864): ten critters
480, the pack brain 164, and 8 for the stoop below.

**The hit shape: a box standing on its feet, yawed with it** -- length along the
facing, width across it, height from the soles to the crown. The bestiary asked
for one shape (§6, "Critter hit shape") and had two proposals, the Gnawers'
capsule lying along the yaw and the Hornback's box that reaches the floor. The
box wins for the Hornback's reason -- a level shot under a quadruped is the
Ridgeback's lesson, and nobody is meant to shoot between a cow's legs -- and it
costs the gnawer nothing: its capsule touched the floor anyway, and the box is
the same body with corners. It is also what `aim::stands_at` reads the crown
off, so "how tall is what the crosshair passed through" and "what does a swing
hit" are one number. `Critter::body` is its one description; the hit test, the
aiming ray, the overlay and the renderer read it.

**One hit test, the overlay's.** A fighter's attack is `state::hitbox` -- the
volume the debug overlay draws -- tested against each critter's box by
`critter::Body::touched_by` (a capsule against a box, a flat disc against its
footprint, a ring section against its column). **Each body is struck once per
swing**, every body the swing touches and not only the first: a sweep through a
heap catches the heap. The memory is on the critter (`flag::STRUCK`), so the
creature's one-connection-per-swing rule is untouched. Projectiles meet critters
through `aim::first_along` (A2) and are hurt through `pack::hurt`; effects -- a
pillar, a burst, the Quake, a tether's line, the Guillotine's blades -- reach
them by the same column, line or slab they reach a fighter with. A thrown blade
cuts a critter **once per throw** (`Critter::seared`; an effect's own memory has
no bits for ten more victims), so out and home is one cut, where it is two on a
fighter. `tests/critters.rs` checks, for every swing in the roster at a step and
at two, that a critter loses health on exactly the frames the drawn volume
touches its box.

**Critters hit fighters** with their move's volume: an upright cylinder in the
critter's own frame (`HitX` ahead, `HitZ` across, `HitLow`/`HitHigh`), the same
shape and the same knobs as a monster's move (`oven::MonsterField`), carried
along the facing by `Travel` and the body by `Advance`. Guard and parry are the
monster's: guarding in its arc blocks, a parry knocks the critter out of it.
`Critter::telegraph` is what will land, from the windup, for markers.

**Fighters pass through critters** (the Gnawers' choice: seven bodies that could
wall somebody in would be an unanswerable trap). A kind can **yield** instead --
it steps out of a fighter's way; a fighter is never moved by one.

**Arena solids stop them**, by the same `resolve_sized` that stops fighters and
stones, and **stones** push them out. There is no path-finding: blocked, a
critter slides along the face it hit toward its goal -- or, square on, the way
its slot number says, so a pack splits round an obstacle -- and keeps that side
until it is clear (`flag::DETOUR`).

**Standing on a creature** (the Siegeshell's parasites): a critter carries the
creature slot and part it stands on and a **perch** -- its point in that part's
frame, in centimetres -- and, as for a rider, **the perch is authoritative**:
its world position is worked out through the creature's pose every frame, so
the animal moving carries it. It walks in the world and is carried back;
walking off the top it falls, and anything falling onto a mountable top lands on
it. `pack::perch_on` puts one there.

## 2 · The pack brain

`sim::pack`, one per fight (`World::pack`), in the order the Gnawers' §5 asks:

- **One glance for the pack**, every `GlanceFrames`: each fighter's position,
  velocity and facing (`pack::Seen` -- nothing about buttons), projected by
  `Lead`. Each critter is after the nearest fighter still standing.
- **A ring, not a chase.** Members without a move out are given places on a ring
  of `RingRadius` round their fighter's lead point, the best `RearBias` toward
  the back; a place with a solid between it and the fighter (`aim::line_clear`)
  or outside the arena scores nothing, which is what makes a wall at your back
  work. At `LineBelow` members or fewer it is an arc in front. Places are handed
  out greedily, best place to nearest member, in a fixed order.
- **Tokens.** A move a kind declares as needing one (`CritterMove::token`) is
  only thrown with one of `Tokens` in hand; it comes back when the move ends
  and rests `TokenRest`. `Pack::rally` lends more for a while (the howl). A
  monster that owns the pack holds `OwnerTokens` while its own move is out, so
  the Broodmother's slam and her brood's bites share one budget. A move that
  needs none (the gnats' leap; the Gnawers' pile-on) is thrown regardless.
- **Thinking is staggered**: critter `i` decides on frames where `frame %
  ThinkEvery == i % ThinkEvery`, and a pack-level coin decides whether this one
  goes now, so which one bites is a draw and how many is a rule.
- **A leader, optionally** (`PackDecl::leader`): it hangs back `LeaderHangback`
  behind the ring, on its pack's side. If `LeaderRouts`, its death breaks the
  pack for good.
- **Morale, optionally.** A death scatters the rest `ScatterDistance` for
  `ScatterFrames`. At `RoutShare` per cent dead it routs for home and regroups
  after `RegroupFrames` if nobody stands within `RegroupClear` of the den,
  counting again against who came back. A share of zero never routs (a brood).
  Broken, the survivors run home and leave by it (`is::GONE`); the hunt is won
  when every creature is down and no critter is left in the fight.
- **Owned by a monster** (`Pack::owner`): its death breaks the pack. The
  monster's species is the pack's, and a body that brings a pack owns it.
- **Spawning into a freed slot** (`pack::spawn`): an empty slot, a gone one, or
  a corpse that has finished fading -- never over a fresh body -- without
  allocating. The Broodmother's sacs burst through this.
- **Alarm and grace**: `Alarm` zero hunts at once, otherwise it is calm until a
  fighter comes within it or a critter is hit; `Grace` holds attacks off at the
  start, as the monster's does.

Its per-frame cost is bounded by constants: 45 separation pairs, at most twelve
ring places per fighter scored once a glance, no per-frame ray. A full pack
costs about what the Ridgeback does (the dev pack with every slot filled, in
release: 15–63 µs a frame against the Ridgeback's 11–61 µs; with the Ridgeback
in the range as well, 22–114 µs; the budget is 520). `budget.rs` runs both
shapes for every class, and they allocate nothing.

**What a species supplies** is `pack::PackMind`, every method defaulted to the
generic pack above: `appetite` (how much a critter wants a move now -- the
default is the monster's tent on range and bearing times its appetite),
`steer` (where it goes, given the generic answer), `frame` (pack-level rules,
with the herd to read an owner by), `hurt`, `died`, and `bumped` (it walked
into a solid -- added with [hazards.md](hazards.md), for the Hornback's bull). Its own state lives in
`Pack::memo`, twelve words the brain never reads, so a creature adds no field
to the world. Roles (`Critter::role`) are the species' to mean.

## 3 · Knobs

A species that brings a pack keeps, after its moves: the pack's own numbers
(`pack::PackKnob`, "<Species> · pack" and "· morale") and a row per kind of
critter (`critter::CritterField`: health, length, width, height, walk, dart,
acceleration, turn, when the windup stops tracking, the damage that flinches it,
the flinch, the knockback it takes, how long the body lies). Critter **moves**
are the species' own moves, one `MonsterField` row each -- the same row a
monster's move has. A species that has never been baked starts at the bottom of
every range; **`cargo run -p sim --bin bake_tuning -- --set <id>=<value>`** sets
knobs by the identifier their baked line carries, in its unit, and bakes: the
palette's slider from a terminal, for a first set of numbers without a window.

## 4 · Aiming at something short (A1 and A2)

[aiming.md](aiming.md) §"Small bodies" has it in full. In short: `aim::stands_at`
is the height of the last body the crosshair's ray passed through (bodies still
never stop it); a ground-aimed skillshot goes to the middle of that body, and a
standing swing pointed at it dips by `aim::stoop` -- to meet it at the same share
of its height a level swing meets a fighter at. Critters are on `first_along`'s
list. Zero change where only fighters stand.

**`cargo run -p sim --bin critcheck`** is the instrument: a critter at 1, 2, 3
and 5 m, the crosshair on its middle, every class's every move pressed, beside
the same move against a fighter standing there. On the dev pack's 0.6 m gnat
**every move touches it wherever it touches a fighter**, with two kinds of
exception, both written down:

- **Lunges run through it.** The Champion's Skewer at 1–2 m and the Dual mage's
  dark auto at 1–2 m carry the fighter forward past a body a fighter would have
  stopped. Geometry of the lunge, not the aim, and the cost of fighters passing
  through critters.
- **The Guillotine passes over it**, at every distance: its blades are flat at
  0.9 m above the shadow's feet (`lotus_height`), by design, and their slab's
  bottom is above a 0.6 m crown. A question for a person, below.

The queen (1.1 m) is touched by everything wherever a fighter is.

## 5 · What you see

- **The renderer** (`crates/game/src/critters.rs`) draws every critter as a few
  boxes -- body, head, four legs, tail, eyes -- **sized from its kind's own
  knobs**, so the body on screen is the hit test's box. It is posed from the
  snapshot: a trot from its clock and speed, the crouch and wiggle of a windup,
  the lunge, the flinch, lying on its side and fading as the corpse runs out,
  and **the tail up and lit while it holds a token**. Interpolated between
  snapshots like a fighter. A species adds only paint: one `CritterPaint` per
  kind in its `Look` (`critters`).
- **The overlay** (F1) draws each body's box (orange while it holds a token),
  each windup and bite from `Critter::telegraph`, every ring place, and the den.
- **The HUD's quarry bar** is what is left of the pack when there is no monster.
- **The fight report** (`crates/hunt`) has a THE PACK section whenever there is
  one: kills (and whether the leader), spawns, blows landed on critters, hits
  and damage taken from them, the most attacking at once, the share of the fight
  with two or more, "tokens live", scatters, routs, regroups and breaks. Critter
  moves are counted in the move table with the species' numbering. The dev pack
  has a minimal hunter (`plans/gnats.rs`): closes, swings at a crouch first,
  aims with the crosshair on the body.

## 6 · Adding a pack creature — the recipe

The Gnawers, say: id `SpeciesId::GNAWERS`, all four registry lines waiting.
Follow [species.md](species.md) §5, with these differences.

1. **The table** (`crates/sim/src/species/gnawers/mod.rs`). A creature that is
   only a pack is `Species::pack_only(id, name, &MOVES, OWN, &tuned::KNOBS,
   path, &PACK)` -- no skeleton, no clips, no `baked.rs`, no anim recipe. Copy
   `species/gnats/mod.rs`: the moves (`MoveDecl`), the kinds
   (`CritterKind`: name, which moves and whether each needs a token, the
   starting role, whether it yields), the `PackDecl` (kinds, who it musters
   with, which kind leads, the mind), and a `PackMind` with the species' own
   rules. A creature with a body *and* a pack (the Broodmother) is an ordinary
   species table with `pack: Some(&PACK)`; it owns the pack.
2. **Its numbers**: bootstrap an empty `tuned.rs`, run `bake_tuning`, then
   `bake_tuning -- --set gnawers.gnawer.height=0.6 --set ...` until the moves,
   kinds and pack say what its document says.
3. **Its look** (`crates/game/src/species/gnawers.rs`): a `Look` whose
   `critters` holds one paint per kind (`NO_BODY` for the monster paints if it
   has no body). Its tail-up and eyes are the mark paint.
4. **Its hunter** (`crates/hunt/src/plans/gnawers.rs`): a plan, and a `CARD`.
   The report's pack lines come for free.
5. **Measure it**: `critcheck -- --species gnawers --kind gnawer` and `--kind
   "big one"`; `fight -- --species gnawers`.
6. **Its arena**: [arenas.md](arenas.md) §4.

What you should **not** need to touch: `critter.rs`, `pack.rs`, `state.rs`,
`aim.rs`, the renderer, the report. If you do, it is a place the machinery was
not generic yet -- fix it there, for every pack, and say so here.

**For the four that use it, specifically:**

- **Gnawers**: pile-on is a move with no token and an appetite that waits for
  `Seen::slowed` or `down` (the gnats' leap is the pattern); the howl is
  `Pack::rally`; scatter, rout, regroup and the leader's death are knobs; the
  den is `Pack::home` (set it in `frame` if it is not where they mustered).
  Scramble and gnaw are the species' own.
- **Hornback**: the bull is a kind with a role and its extra state in
  `Pack::memo`; the herd's five states and flocking are `PackMind::frame` and
  `steer`; cows that push are `yields`. The charge's swept stop, solids with a
  health, and a ride on a critter are its own to build.
- **Broodmother**: her species has a body and `pack: Some`; the brood never
  routs (`RoutShare` 0); `OwnerTokens` is her share of the budget; her sacs
  spawn through `pack::spawn` from `PackMind::frame`, which is handed the herd
  to read her by.
- **Siegeshell**: parasites spawned and `pack::perch_on` a plate; a perch is
  carried by the pose. Falling, they land on any mountable top under them.

## 7 · Decided while building, for a person to review

- **The box, not the capsule** (§1).
- **The swing meets a short body at the share of its height a level swing meets
  a fighter at**, not at its middle as the Gnawers' document proposed -- the
  instrument found the middle too steep (aiming.md).
- **The Guillotine passes over a 0.6 m body.** Lowering `lotus_height` below
  0.6 m would fix it and changes versus; leaving it makes the Reaver's flower
  a Big One tool and her Slash the gnawer tool, which the Gnawers' §7 almost
  says anyway. A person's call.
- **Lunges run through critters**, because fighters pass through them. An attack
  step that stopped at a critter would be a second rule for what blocks a body.
- **A blade cuts a critter once per throw**, not once per pass.
- **The Quake now hits critters**; it does not hit the Ridgeback, and was left so.
- **Critters do not take the Grasp's hold or the tether's leash** -- a line that
  catches one cuts it and does not hold it. A Blood mage hauling the Big One out
  from behind its ring (Gnawers §7) needs the Grasp to learn to hold a critter.
- **One pack per fight**, ten bodies. Two packs would be the Pair of packs; none
  is planned.
- **The dev pack is a registered species** (id 11, after the ten creatures),
  so it is in the Oven, the picker's `Shift+H` cycle and the report, the way
  the range is an arena. It has no arena of its own and is hunted in the
  proving ground.
- **`aim::look_onto` keeps six rounds**; `look_onto_closely` settles a steep
  look. Six leave the sparring bot a metre high forty degrees down; changing it
  moves the bot's pinned fights, so that is a change for whoever means to.

## 8 · Not done here

(Since built, in [hazards.md](hazards.md): critters take damage from floor
hazards that reach them, a critter's move strikes a defended thing as a
creature's does, and `PackMind::bumped` is told when one walks into a solid.
Packs keep their own glance and perceive every fighter -- P5's filter is the
monsters'.)

Each the creature's own: telegraph floor markers for critter moves (the data is
`Critter::telegraph`); a "swings over" and "hidden commits" line in the report
(Gnawers §9); scrambling up a platform's edge (critters slide round solids, they
do not climb them); critters riding a critter (the Hornback's cow ride); a
creature hit or held by the Grasp. The renderer's critter is boxes; a species
that wants a silhouette of its own adds it to its look.
