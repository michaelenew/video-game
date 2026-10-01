---
status: built 2026-10-01 (bestiary P1 and P8)
---

# Species — a creature is a table

The bestiary ([bestiary.md](bestiary.md) §3) found the Ridgeback's code true to
its promise in the *machinery* and false in the *data*: every creature table was
a Ridgeback constant, and a handful of moves were special-cased by name. P1 and
P8 fix that. This document is what was built, the decisions it made, and **the
recipe for adding a creature** — which is the part the next ten builds read.

The bar was that the Ridgeback comes out **bit-identical**, and it does: the two
pinned hunts (`crates/sim/tests/ridgeback_pin.rs`, every class against noise;
`crates/hunt/tests/pin.rs`, the scripted hunter over four seeds) hash every
frame of the snapshot and match the code before the change; the fight report
for every class, twelve seeds each, solo and duo, is byte-for-byte the same; and
the re-baked pose table is the same table.

## 1 · What a species is

`sim::species::Species`, one `static` per creature:

| Field | What it is |
| --- | --- |
| `bones` | Name, parent, rest offset (metres at scale one), side. Parents before children; bone zero is the root. Up to `beast::MAX_BONES` (40). |
| `mirror`, `neck`, `follows` | The left/right pairs a mirrored clip swaps; the bones the head's tracking is spread across (empty for none); the bone each `Follows` number in the move table means (`follows[0]` is the body). |
| `parts` | A box on a bone, with flags: `mountable`, `solid`, `breakable`, `weak` (fills poise), and `vuln`, the own-knob holding its damage multiplier. Built with `part(..)` and `.mountable() .soft() .breakable() .weak_point()`. Up to `beast::MAX_PARTS` (48). |
| `breakable` | Which parts carry a health of their own, worked out from the flags at compile time (`beast::breakables`). Up to `beast::MAX_BREAKABLE` (12): health is kept for those parts only, so a forty-part rig does not spend the snapshot on armour. |
| `legs` | Any number, zero included: upper part, foot part, hip and knee bones, front or back, side. The break, the gait's lean, the turn penalty and the sag all walk this. |
| `moves` | `MoveDecl`: a name, the clip it plays, and **flags in place of special cases** — `mirrors_to_target_side` (the sweep), `scaled_by(knob)` (the shake's force). Up to `species::MAX_MOVES` (16). Every number about a move is a knob. |
| `clips`, `stock` | `ClipDecl` (name, looping, phased) per clip, and which clips stand for idle, walk, gallop, flinch, stumble, topple and dead. |
| `span`, `rows`, `row` | The baked pose table, from the species' own `baked.rs`. |
| `own`, `tuned`, `tuned_path` | Its own knobs, and its own baked tuning file. |

`Monster` keeps a `SpeciesId` (one byte) in the snapshot and reads the rest
through `Monster::sp()`. `beast::Rig` carries the `&'static Species` it was built
from, so every part and bone lookup reads the right table.

**What used to be by name, and is now declared:**

| Was | Now |
| --- | --- |
| `kind == SWEEP` picks a side | `MoveDecl::mirrors_to_target_side` |
| `kind == SHAKE` scaled by `shake_force` | `MoveDecl::scaled_by(Knob::ShakeForce)` |
| `part == RIDGE \|\| part == NAPE` | `Part::weak` |
| `vulnerability()`, a `match` on part names | `Part::vuln`, an own-knob per part (shared: the four feet are one) |
| `BREAKABLE`, four named feet | `Shape::breakable`, and `Species::breakables()` |
| `follow_bone()`, a `match` | `Species::follows` |
| `NECK_CHAIN`, the lame layer's lever through `SPINE` and `CHEST` | `Species::neck`; the lever is the leg's hip chain up to the root |
| `Clip::of_move`, `Clip::phased`, `beast_baked` | `MoveDecl::clip`, `ClipDecl`, the species' own table |
| the hunter's `bucks()`, the report's "SWEEP \| SLAM \| SHAKE" | the species' `hunt::plans::Card::bucks` |

## 2 · Knobs

Three kinds, stored in that order in each species' own store and its own baked
file, `crates/sim/src/species/<species>/tuned.rs`:

- **`species::Common`** — the numbers every creature has, because the shared
  machinery reads them: size, health, breakable-part health, walk, gallop, turn,
  glance, think, decisiveness, poise, topple, stumble, flinch, the leg sag, the
  strain thresholds, the gait, spawn distances, the windup's tracking, braking
  and launch, the grace at the start of a hunt. Read as `sp.scale()`,
  `sp.glance_frames()` and so on (`species/common.rs`, where each one's note now
  lives). Shown in the palette under the species' name: "Ridgeback",
  "Ridgeback · mind".
- **Its own** — declared in the species' file with `species_knobs!`. The
  Ridgeback's are its hide (a multiplier per part) and its shake force.
- **Its moves** — one row of `oven::MonsterField` per move.

`Knob::Species(SpeciesId, Tunable)` is the palette's handle on all three. The
Oven keeps a row of cells per species id; the bake (`bake_tuning`, and the Bake
button) writes `tuned.rs` and every species' own file. **A species that has
never been baked** starts from the Ridgeback's common numbers and the bottom of
every other knob's range, so a new species begins with an empty `tuned.rs` and
`bake_tuning` writes the rest.

The Ridgeback's knob ids are unchanged except one: its breakable-part health is
`ridgeback.breakable_part_health` (was `ridgeback.foot_health`), because the
number is now every creature's, and its move ids are `ridgeback.<move>.<field>`
(was `monster.<move>.<field>`). No value moved.

## 3 · Two creatures in the world

`World::monsters: [Option<Monster>; 2]` (`monster::MAX_MONSTERS`). The Pair need
two; every other creature uses slot zero. Everything that touches a creature
walks the slots in order, so a hunt with one creature in slot zero is exactly
the old one-creature game.

- **Riders.** `Player::mount` packs the slot in its top two bits and the part
  in the low six (`monster::mount_of`, `mount_slot`, `mount_part`). Slot zero
  packs to the old part number, so the Ridgeback's snapshots did not change.
  The creature's turn reaches the rider's look from the creature they are on.
- **Hits.** `aim::Contact::Quarry` names the slot. An effect remembers the
  second creature in its own bits above everything the first three victims use
  (`effects::quarry_victim`), so the first creature's bits are where they were.
- **The end.** The hunt is won when every creature is down.
- **Spawning.** Creatures stand at their species' spawn distance, a keep-out
  (`Common::Margin`) apart either side of the line when there are two. A first
  guess for the Pair to replace.
- **The snapshot.** 2,856 bytes, from 2,680: the second slot is about 180 bytes
  that every fight pays. `Monster` itself went from 184 to 180 (health on
  breakable parts only, lockouts room for sixteen moves). `budget.rs` runs a
  two-creature scene for the frame budget and the no-allocation check.
- **The hash.** `World::checksum` writes nothing for empty slots after the last
  creature, so a one-creature world hashes as before; `World::state_checksum` is
  the same hash without the Oven's, which is what the pins compare.
- `World::hunt(classes)` is still the Ridgeback; `World::hunt_of(classes,
  species)` and `World::hunt_with(classes, [Option<SpeciesId>; 2])` are the rest.
  `World::monster()` is the first creature, for code about one.

## 4 · Where a creature's files are

| Crate | File | What |
| --- | --- | --- |
| `sim` | `src/species/<species>/mod.rs` | The table: bones, parts, legs, moves, clips, own knobs, `SPECIES` |
| `sim` | `src/species/<species>/baked.rs` | Its pose table — generated by `bake_beast` |
| `sim` | `src/species/<species>/tuned.rs` | Its knobs — generated by `bake_tuning` |
| `anim` | `src/beast/<species>/mod.rs`, `clips.rs` | Its recipes, its bones' looseness groups, its pose builders (a trait on `beast::Pose`), where it bakes to |
| `hunt` | `src/plans/<species>.rs` | Its hunter plan, its `Card`: which moves buck, what the report calls its parts |
| `game` | `src/species/<species>.rs` | Its `Look`: paint, joints left out, the head, any decoration |

And **two lines in each of four registries**, every planned creature's already
written and commented out, a blank line from the next — so two branches each
uncommenting their own creature's lines do not conflict:

1. `crates/sim/src/species/mod.rs` — `pub mod <species>;` and its arm in `lookup`.
2. `crates/anim/src/beast/mod.rs` — `pub mod <species>;` and its arm in `authored`.
3. `crates/hunt/src/plans/mod.rs` — `pub mod <species>;` and its arm in `card`.
4. `crates/game/src/species/mod.rs` — `pub mod <species>;` and its arm in `look`.

(The ids — `SpeciesId::GNAWERS` to `SpeciesId::SIEGESHELL` — are already fixed,
so nobody has to choose a number.)

## 5 · Adding a creature — the recipe

Say the Pair, whose id is `SpeciesId::PAIR` and whose lines are waiting.

1. **The table.** Copy `crates/sim/src/species/ridgeback/mod.rs` to
   `species/pair/mod.rs` and make it the cat: bones (the same topology with new
   rest offsets, for the Pair), parts and their flags, legs, moves and their
   flags, the `Clip` enum and `CLIPS`, `stock`, its own knobs. Set `id`, `name`
   and `tuned_path`. Uncomment the two lines in `species/mod.rs`.
2. **Bootstrap its generated files**, so the table compiles before anything has
   been baked. `tuned.rs`:

   ```rust
   #[rustfmt::skip]
   pub const KNOBS: [i32; 0] = [];
   ```

   and `baked.rs` (no rows yet; every clip reads as the rest pose):

   ```rust
   use super::{CLIP_COUNT, bones};
   pub const CHANNELS: usize = crate::beast::channels(bones::COUNT);
   pub const ROWS: usize = 0;
   pub const SPAN: [(u16, u16); CLIP_COUNT] = [(0, 0); CLIP_COUNT];
   pub static FRAMES: [[i32; CHANNELS]; ROWS] = [];
   ```

3. **Bake its knobs**: `cargo run -p sim --bin bake_tuning`. It starts from the
   Ridgeback's common numbers and the bottom of every other range -- every hide
   multiplier zero, every move zero frames -- so set its hide, its moves' frame
   data and hit volumes in the Oven (F7) and bake again. Until then
   `tests/species.rs` says it cannot be hurt, and the creature never attacks.
4. **Its animation.** `crates/anim/src/beast/pair/` with an `Authored` impl
   (copy the Ridgeback's) and its recipes; uncomment its two lines in
   `anim/src/beast/mod.rs`; `cargo run -p anim --bin bake_beast -- --species
   pair`. `cargo run -p anim --bin preview_beast -- --species pair --all` draws
   it.
5. **Measure it.** `cargo run -p sim --bin beastcheck -- --species pair` prints
   every surface against every class's hop, read off the table.
6. **Its hunter.** `crates/hunt/src/plans/pair.rs`: a `Plan` and a `CARD`;
   uncomment its two lines in `plans/mod.rs`. `cargo run -p hunt --bin fight --
   --species pair`.
7. **Its look.** `crates/game/src/species/pair.rs` with a `LOOK`; uncomment its
   two lines in `game/src/species/mod.rs`.
8. **Check.** `cargo test -p sim --test species` checks every registered table
   is well formed and that the creature stands, moves and can be hurt;
   `tests/oven.rs` that its bake is current. The Ridgeback's pins must not
   move.

This recipe was dry-run on 2026-10-01 with a copy of the Ridgeback registered as
the Pair: steps 1 to 6 and the checks in 8 compiled, baked and measured with no
edit outside its own files and its registry lines.

9. **Its arena.** Follow [arenas.md](arenas.md) §4: a table in
   `crates/sim/src/arena/<creature>.rs` naming the species, its two registry
   lines, and optionally a dressing in `crates/game/src/arenas/`. Until it
   exists the creature is hunted in the proving ground.

What you should **not** need to touch: `monster.rs`, `beast.rs`, `state.rs`,
`oven.rs`, the report. If you do, it is a place the rig was not data yet — fix
it there, for every creature, and say so in this document.

## 6 · Decided while building, for a person to review

- **Sizes are maximums, not generics.** `MAX_BONES` 40, `MAX_PARTS` 48,
  `MAX_BREAKABLE` 12, `MAX_MOVES` 16, chosen against the Siegeshell (26 bones,
  ~40 parts). Per-species const generics would make `Monster` a different type
  per creature and the world could not hold two of them in one array. Loops run
  to each species' own counts.
- **Breaking a breakable part stumbles the creature**, whichever part it is.
  True of the Ridgeback's feet; the Broodmother's sacs will want a flag.
- **Common knobs share one range each.** A creature that needs a health below
  500 or a size outside 0.5–3 widens the range for everybody.
- **The fight report reads the first creature.** A report per creature is the
  Pair's to add.
- **Hunter intents are names** (`hunt::Intent`, a string), so each plan has its
  own without a shared list every creature branch edits.
- **`beastcheck`'s routes are generic now**: the lowest surface in each lowering
  state (stumbling, toppled, front or hind feet broken, every move), rather than
  the six Ridgeback routes it named. Its numbers are unchanged where they
  overlap (shoulders stumbling 2.665 m, front feet broken 3.807 m, through the
  slam 3.23 m, nose to tail 13.406 m).
- **Not here:** critters (P3), the `sheds` part flag the Pair asks for, a
  perception filter (P5). Each lands with the creature that needs it. Arenas as
  data (P2) landed after this, ahead of the creatures:
  [arenas.md](arenas.md) is the same kind of recipe for a creature's arena, and
  a creature branch follows both.
