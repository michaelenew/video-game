---
status: built 2026-10-01 (bestiary P2, world W0)
---

# Arenas — a place is a table

[bestiary.md](bestiary.md) P2 and [world.md](world.md) §6 W0. Every creature
after the Ridgeback fights somewhere of its own — a 240 m valley, a plateau with
a 12 m tower, a cave under a vault, sand with rock islands — and the world is,
for now, those arenas picked from the dev harness. This is what was built, the
decisions it made, and **the recipe for adding an arena**, which is the part
the creature branches read. It is the arena half of [species.md](species.md).

The bar was that the first arena comes out **bit-identical**, and it does: the
two pinned hunts (`crates/sim/tests/ridgeback_pin.rs`,
`crates/hunt/tests/pin.rs`) did not move, the fight report for every class,
twelve seeds each, solo and duo, is byte-for-byte the same, and
`tests/arena.rs` checks the proving ground's numbers against the constants it
replaced.

## 1 · What an arena is

`sim::arena::Arena`, one `static` per arena:

| Field | What it is |
| --- | --- |
| `id`, `name` | Its `ArenaId` and its name; the slug (`proving_ground`, `range`) is what `--arena` takes. |
| `creature` | The species it is for. `arena::for_species` reads it; a creature with no arena yet is hunted in the proving ground. |
| `bounds` | The playable rectangle. A projectile leaves by it (`Arena::inside`) and a creature is clamped to it less its keep-out. It does **not** stop bodies: author walls, banks or cliffs as solids. |
| `floor`, `regions` | What the floor is made of: a `Material` everywhere, then rectangles and discs of other materials, later ones winning. |
| `solids` | Axis-aligned boxes, at most `MAX_SOLIDS` (64). Every face stops a body, every top is standable however high, and each has a `Material` for its top. |
| `spawns` | Two versus marks; and the hunt's marks (two hunters, a creature per slot), or `None` for the species' own spawn distances along x, which is how the proving ground places a hunt. A `Mark` is a point and a facing, written as "stand here, face there". |
| `sites` | Where a fight's defended things stand (bestiary P7): a route of points and a box's size each -- a wall's place, a cart's road. What stands there is the species' ([hazards.md](hazards.md) §5). Empty for most arenas; the range has a gate and a road. |

**Materials** are `Ground`, `Grass`, `Rock`, `Stone`, `Sand`, `Snow`, `Ash`,
`Peat`, `Water`, `Wood`. The simulation can ask `Arena::material_under(pos)`
(the top of the solid you are standing on, or the floor's region) and
`Arena::floor_at(x, z)` (the floor, ignoring solids), and two questions are
answered already as first guesses for the creatures that ask them:
`Material::soft` (a burrower moves under it: sand) and `Material::takes_prints`
(snow, ash, sand). Nothing in the fight reads materials yet — that is the
Sandmaw's and the Veilstalker's to do — but the renderer colours the floor by
them, so what you see is what the simulation will answer.

**A ceiling is a solid.** A cave's vault is a box hanging from the roof, its
underside the ceiling. Bodies are pushed down out from under it by the resolve
that lands them on a platform; the crosshair's ray meets it as it meets a wall;
the camera's arm is pulled in under it as it is pulled in from a wall; and
`ground_under` does not count a hanging solid as ground beneath a point below
it. A vault that comes down toward the walls is more hanging boxes. No second
mechanism to keep in step with the first.

`World::arena` is an `ArenaId`, one byte in the snapshot. Everything that asks
about geometry reads `World::arena()`: the resolve for fighters and stones, the
floor under a planted shield or a burst, projectiles leaving, the creature's
walls, the aiming ray (`aim::Scene::arena`), the camera (`Surroundings::arena`)
and the renderer. The hash writes the arena only when it is not the proving
ground, so a fight there hashes as it always did.

## 2 · The arenas there are

- **The proving ground** (`arena/proving_ground.rs`): the first arena, ported
  as data. 28 m square, 1.5 m walls, two 1.5 m platforms. Versus, the training
  dummy and the Ridgeback fight here.
- **The range** (`arena/range.rs`): a dev arena with one of everything, at the
  largest size anybody asks for — 240 × 50 m (the Siegeshell's valley); sand
  with three rock islands and a boulder (the Sandmaw); snow with ash pits, a
  stream and five 5 m trunks (the Veilstalker); a 6 m square tower 12 m tall
  with eight ledges a hop apart (the Galewing); a 3 m bank with a step; a cave
  under a 12 m vault coming down to 8 m at the walls, with two pillars (the
  Broodmother); grass and 2 m boulders. `--arena range`. It is where
  `tests/arena.rs` asks its questions and where `tests/budget.rs` measures the
  worst case.
- **The Commons** (`arena/gnawers.rs`, 2026-10-01): the Gnawers' den, the first
  creature arena. A 36 × 30 m meadow, a 3 m bank along the north with the den a
  notch in it, low walls and a six-metre hedge round the rest, a fallen trunk
  the pack scrambles and two boulders too tall to climb. See
  [creatures/gnawers.md](creatures/gnawers.md) §11.
- **The low meadow** (`arena/hornback.rs`, 2026-10-01): the Hornback herd's.
  48 × 40 m of grass, a 1.5 m bank with steps along the north, thicket and slope
  as the edge (outside the bounds, so a charge pulls up short of them rather
  than meeting them), the ford in the east the herd leaves by. Its four boulders
  are not arena solids: they are the herd's hazard cells, laid on the first
  frame, which crack and shatter. See [creatures/hornback.md](creatures/hornback.md) §11.
- **The crossing** (`arena/hornback.rs`, 2026-10-01): the Hornback's defend
  variant, and **the first creature with two arenas**. 80 × 36 m, a road along
  its middle with a site for the cart (bestiary P7), three boulders beside it.
  `arena::for_species` still answers the first arena that names the creature
  (the meadow); the crossing is reached by name.
- **The Cliffs** (`arena/galewing.rs`, 2026-10-01): the Galewing's. A 44 × 44 m
  plateau that is a **12 m solid** standing on the floor -- the floor of the
  game is at zero, so a drop has to be a solid -- with the shelf four metres
  wide round three sides of it twelve metres down, six stairs of 2 m steps back
  up, a rock face eight metres above it on the north, the perch tower (6 m
  square, 12 m above the plateau, eight ledges 1.5 m apart spiralling round it)
  and four standing stones. Two sites, `circle` and `perch`, tell the bird where
  things are. The sun is overhead (`game::arenas::sun`). See
  [creatures/galewing.md](creatures/galewing.md) §11.

## 3 · The picker (W0)

- `--hunt <creature>`, `?hunt=<creature>`: hunt that creature, in its arena.
  `--hunt` alone is the Ridgeback, as it always was. A name that is not built
  yet says so and hunts the Ridgeback.
- `--arena <name>`, `?arena=<name>`: another arena, for versus or (with
  `--hunt`) for the hunt.
- `--hunt <creature>-<mode>`, `?hunt=<creature>-<mode>`: a creature's other
  arena, by that arena's name or by `escort` for the one with something to
  defend (`picker::mode`): `hornback-escort` is the Hornback's crossing, the
  same as `--hunt hornback --arena crossing`.
- `H` swaps between hunting the Ridgeback and fighting each other; `Shift+H`
  steps to the next registered creature, in its own arena
  (`species::after`, which skips ids nobody has built).

**The keys are a simulation event, not a fresh `World` built by the client.**
`H` and `Shift+H` set `Input::travel`, one byte on the wire beside the buttons
(`sim::input::Travel`), and `World::advance` builds the new fight on the frame
it arrives — same classes, frame number kept, because the rollback session owns
it and checks it. The reason is that inputs are the only thing two peers agree
on per frame: a client that replaced its own world on the frame its player
pressed `H` would do it on a different frame from its peer, or alone. As an
input it is predicted, confirmed and rolled back like a button
(`crates/net/tests/rollback.rs` has a peer's trip discovered inside a rollback
and landing on the same frame). A trip to an unregistered creature is no trip.
`Tab`, `Backspace` and the class pickers still rebuild the world locally, as
they did; they keep the arena and the creature now.

## 4 · Adding an arena — the recipe

Say the Galewing's cliffs, whose id is `ArenaId::GALEWING` and whose lines are
waiting.

1. **The table.** Copy `crates/sim/src/arena/proving_ground.rs` (or the range,
   for more kinds of thing) to `arena/galewing.rs`. Set `id`, `name`,
   `creature: Some(SpeciesId::GALEWING)`, `bounds`, `floor`, `regions`,
   `solids` (centimetres, `Solid::cm(min, max, material)`), `spawns`.
   Uncomment the two lines in `arena/mod.rs`: `pub mod galewing;` and its arm
   in `lookup`.
2. **Its dressing**, if it has any: `crates/game/src/arenas/galewing.rs` with a
   `DRESSING` (a sky colour and props nothing collides with — bones, reeds, a
   banner); uncomment its two lines in `game/src/arenas/mod.rs`. Without one it
   is drawn bare, which is honest: every solid and region is drawn from the
   table anyway.
3. **Check.** `cargo test -p sim --test arena` checks every registered arena:
   solids with volume, at most 64 of them, marks inside the bounds and out of
   the solids, facing level, its creature registered. `cargo test -p sim
   --test budget` measures a fight in the range; an arena bigger or busier than
   the range should be added to its scenarios.
4. **Look at it.** `cargo run -p game -- --hunt galewing` (or `--arena galewing`
   before the creature exists), and `./scripts/screenshot.sh` with the same
   flags.

What you should **not** need to touch: `state.rs`, `aim.rs`, the camera, the
renderer's drawing. If you do, it is a place the arena was not data yet — fix it
there, for every arena, and say so here.

## 5 · Decided while building, for a person to review

- **Arguments, not a global.** Every query is a method on `&Arena`, and the
  arena reaches it through the `World` (or `aim::Scene`, or the camera's
  `Surroundings`). A thread-local "current arena" would have been fewer edits
  and a hidden input to a function the architecture says is pure.
- **Bounds are not walls.** Every arena authors its own edge as solids, so a
  cliff, a bank and a 1.5 m wall are the same kind of thing, and the edge is as
  standable as any other top. The bounds only bound.
- **64 solids, scanned linearly.** A fight in the range (34 solids, two
  creatures) measured 142 µs a frame in the worst class against 102 µs in the
  proving ground, with a 520 µs budget. No grid until an arena needs one.
- **Lengths over 181 m saturate** in 16.16 (`V3::len`). The range is 240 m
  long, so two things at its two ends read as 181 m apart. **Done 2026-10-01**
  ([hazards.md](hazards.md) §6): `math::wide_len` and its siblings are exact at
  any distance and bit-identical below a hundred metres, and the glance, the
  creature's walk and the straight-line tests use them.
- **Done 2026-10-01, in F3b** ([hazards.md](hazards.md)): the simulation's eye
  is held under a ceiling (`camera::eye_under`) and the drawn camera starts from
  it; creatures collide with solids when their species says so
  (`FightDecl::collides`) -- the Ridgeback does not, and stays bit-identical; the
  sparring bot reads its walls off the arena; and the floor's material is read
  by the fight for the first time (a footfall on rock or stone is a louder
  noise). **The ground as it stands** (`arena::Terrain`) -- the arena, the
  solids a fight raises at runtime, and the floor hazards -- is what every query
  takes now; with nothing raised it is the arena exactly.
- **A Reaver's shadow starts at the versus mark in a hunt**, as it always has:
  the hunt moves her to the hunters' mark afterwards and the shadow eases over.
  Fixing it moves the Reaver's pinned hunt, so it was left for a change that
  means to.
- **Arena geometry is data, not knobs.** Like a species' bones, an arena is a
  table in centimetres, edited in its file rather than the Oven; the knobs
  test's two exemptions for the old `ARENA_HALF` and `WALL_HEIGHT` are gone
  with them.
