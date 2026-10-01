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
| `fight` | What it brings to the fight besides its body and pack: its hazard kinds, its defended things, its perception filter and whether it hears, whether it collides with solids, the layout of its share of the hunt's lore, and its hooks (`frame`, `bumped`, `shown`, and `signs` -- the shapes it draws on the floor, `sim::sign`, added with the Hornback; and the Mireback's seams below). `FightDecl::PLAIN` for none of it, which is the Ridgeback. See [hazards.md](hazards.md). |

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

10. **What it brings to the fight** -- hazards, senses, defended things, its
   own state in the hunt's lore: [hazards.md](hazards.md) §7. The dev
   creature that uses all of it is the sentinel (`--hunt sentinel`, id 12).

11. **Its tempers come free** (world W2, `crates/sim/src/temper.rs`). A
   temper is a share of the creature's *own* glance, lead, decisiveness and
   strain desperation (`species::Common`) -- or, for a pack, of its pack's
   glance, lead and thinking cadence (`pack::PackKnob`) -- set once for every
   creature in the Oven under "Tempers". So a new species declares nothing:
   it has three tempers the moment it is registered, its trophies are kept by
   its id, and `fight --species <it> --temper 3` measures the hardest one.
   What it must do is **read those knobs through the creature, never off the
   table**: `m.glance_frames()`, `m.lead()`, `m.decisiveness()`,
   `m.strain_desperation()`, `pack.glance_frames()`, `pack.lead_frames()`,
   `pack.think_every()`. A species' own mind that reads `sp.glance_frames()`
   directly is a creature whose tempers do nothing to it. A creature whose
   difficulty lives in a knob of its own (the Veilstalker's veil, say) and
   wants a temper to move it too is a change to `temper.rs`, for every
   creature at once, not a special case beside the species.

What you should **not** need to touch: `monster.rs`, `beast.rs`, `state.rs`,
`oven.rs`, the report. If you do, it is a place the rig was not data yet — fix
it there, for every creature, and say so in this document.

**The seams a creature that changes its own fight adds** (the Mireback,
2026-10-01). Every one is `None` or off in `FightDecl::PLAIN`, so every other
creature, and both pins, are unchanged:

| Seam | What it is for |
| --- | --- |
| `appetite(m, move, score, mind)` | its own terms in the scoring, after the shared ones; `monster::Mind` is what the brain may read (the glance, the `Terrain`, the lore) |
| `prowl_to(m, mind)` | where it walks when free, if not at its target (the toad walks to its tar) |
| `commit(m, move, mind)` | as a move commits: where a lobbed move lands, which way a leap goes |
| `hide(m, part)` | a multiplier on a part's vulnerability from its own state (the tar coat) |
| `struck(m, part, dealt)` | a hit has landed; returning true skips the shared ladder (the sac tearing, a wallow broken) |
| `landed(world, slot, fighter, move, guarded)` | one of its moves landed on somebody: what it does besides hurt (the tongue's grab) |
| `marks(world, out)` | what it draws besides hazards and telegraph: rings, kindling, the braziers (`species::Mark`) |
| `lands_on_bodies`, `rolls_over` | a creature that lands *on* fighters pushes them out sideways, not into the floor; a part facing the floor is no surface on a creature that rolls over |
| `MoveDecl::lobbed`, `stops_at_aim`, `harmless`, `never_chosen` | a volume landing at a point chosen at commit (`Brain::aim`), a reach that stops at its aim, a move that hits for nothing and still lands, a move only the species starts |
| `Part::hollow` | a part you are inside (the stomach): mounted only by being put there (`state::put_inside`), no jump, no buck |
| `Monster::own` | four words of its own on the body, hashed only when not zero |

**The seams a creature that is not always there adds** (the Sandmaw,
2026-10-01), on the same terms -- `None` in `FightDecl::PLAIN`, both pins
bit-identical:

| Seam | What it is for |
| --- | --- |
| `presence(m, rig)` | which parts have no body this frame (`beast::Presence`): **buried** parts have no hurtbox, are not solid and are not stood on; **unmountable** ones are solid but not a back. The worm's answer is geometric: a part whose box is wholly under the floor is buried |
| `clip(m)` | the clip its posture plays, ahead of the speed-picked stock -- a worm circling slowly under the sand swims rather than standing up out of it |
| `hearing(m, lore)` | a multiplier on what it hears (`World::heard_by`): hunger, the deafness after a breach |
| `from_inside(world, slot, fighter, input)` | the input of a fighter inside one of its hollow parts, and what they may still do with it -- the swallow's gulp press |
| `radius(m, move, r)` | a move's hit radius from its own state: the rise-bite smaller once the tooth ring is broken |
| `steepest` | its own knob for the steepest face that is still floor, and a face below the floor is never one: a standing column is a wall, a beached back is a ridge |
| `MoveDecl::own_hit` | a move whose hit the species tests itself in its `frame` hook (the spit's cone, the lash's half-ring), from the same discs its `marks` draws |
| `MarkLook::{Sand, Fin, Heard, Feel}` | a raised wake, a dorsal fin, a ring where it heard you, the disc it feels in |
| `noise::nth(lore, cell)` | a noise by its cell, for a hook that keeps per-noise state of its own beside the ring |

**The seams a creature with two bodies adds** (the Pair, 2026-10-01), on the
same terms -- `PLAIN` is one body and none of the rest, and both pins, the
twelve-seed Ridgeback reports and the Mireback's, Sandmaw's and Gnawers'
reports came out byte-identical:

| Seam | What it is for |
| --- | --- |
| `bodies` | how many of it a hunt places (`World::hunt_of` fills the slots); one for everybody else |
| `keeps_height` | the walk leaves its height to its own hooks: a cat on a platform stays up there |
| `pace(m)`, `glance(m)` | a multiplier on its speed and turn, and its frames between glances, from its own state: the enrage, two hunters |
| `lob_height(m)` | the height a lobbed volume sits at, for one that lands on a top rather than the floor |
| `Part::sheds` | nobody stands on it and nobody mounts it: its top face is a slope in `Rig::resolve`, not a floor |
| `Presence::passable` | parts with a hurtbox and no body: a leaping cat can be hit but not run into, so a dodge under the arc goes through |
| a lobbed volume's skid | a `lobbed` move with a `Travel` slides its volume along the facing for the frames it has flown; zero for every lobbed move before it |
| `Tally::observe_with(before, after, bots)` | a species' report lines that need the hunters -- what is on their screen; defaults to `observe` |

**The fight report reads every creature.** Each slot keeps its own commit
(the move, how far each hunter was, where it stood), a hit is the slot whose
volume connected that frame, and the windows read the smaller of the slots'
`frames_until_free`: the pair is free when either cat is. One creature reads
exactly as before. **"Unanswerable" needs a connection**: health lost on a
frame no creature's volume connected -- a Blood mage paying for her own
spells -- was being charged to the last commit, however far away it was. No
pinned report moved; a Blood mage's against the Ridgeback can (one hunt in
twelve read a phantom one).

**The seams an unseen creature adds** (the Veilstalker, 2026-10-01), on the
same terms -- every one off in `PLAIN`, and no other creature's code moved:

| Seam | What it is for |
| --- | --- |
| `FightDecl::apparition` / `World::apparition()` | a decloak drawn where no body is -- the mimic's ghost: where it stands, which way, which rear it plays and its frame, from the lore. What the renderer draws of a decloak, real or not, and the only thing the hunter reads to see one |
| `FightDecl::shown` (P5's, first used here) | how strongly each part is drawn, `0` cloaked to `1` whole: the veil, derived each frame from the move, the fade, speed, paint, mottle and hazards (`fight::shown`) |
| `aim::in_view_from`, `aim::off_look` | *is that point on a screen looked along this yaw, from here, and not behind anything* -- the creature's view gate, built from the eye `aim.rs` builds every other eye from |
| `Tally::until_free(w, slot, free)` | a species' own correction to `frames_until_free` for the windows: the Veilstalker adds its decloak floor, so a stalk is free but not threatening until it is in reach and in view; identity for everybody else |
| `MonsterField::Cooldown` range to 900 | the smoke's fifteen seconds |

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
- **The fight report reads every creature** (the Pair): one report for the
  hunt, the slots summed, rather than one per creature.
- **Hunter intents are names** (`hunt::Intent`, a string), so each plan has its
  own without a shared list every creature branch edits.
- **`beastcheck`'s routes are generic now**: the lowest surface in each lowering
  state (stumbling, toppled, front or hind feet broken, every move), rather than
  the six Ridgeback routes it named. Its numbers are unchanged where they
  overlap (shoulders stumbling 2.665 m, front feet broken 3.807 m, through the
  slam 3.23 m, nose to tail 13.406 m).
- **Not here:** a perception filter (P5) -- it landed with its own document;
  the `sheds` part flag came with the Pair. Critters (P3) landed after this: [critters.md](critters.md) is the
  recipe for a creature that is a pack or brings one -- a `Species` gained a
  `pack` field (`None` for the Ridgeback), and `Species::pack_only` is a table
  with no skeleton. Each lands with the creature that needs it. Arenas as
  data (P2) landed after this, ahead of the creatures:
  [arenas.md](arenas.md) is the same kind of recipe for a creature's arena, and
  a creature branch follows both.
