---
status: exploration — run
started: 2026-10-03
run: 2026-10-03
---

# 0005 — Can a body plan be bred?

*What of a creature's body is data today, what it would cost to make each layer
heritable, and the experiments that say whether it is worth it.*

## The question

The world thread (2026-10-03, not yet a note of its own) wants creatures to be
**bred**: a line that changes over generations, that two players' lines can be
crossed, and that the frontier of the world is populated by. Tempers already
prove the smallest version — four knobs, three steps, no new content. The
question this note answers is the largest version: **can the body itself
change** — longer legs, a different weak point, a cousin's move, a different
number of legs — so that a bred line can be a new animal rather than a retuned
one? And at what cost, against the budgets in
[architecture.md](../architecture.md): a float-free deterministic simulation, a
4 KiB snapshot, no allocation in a frame, and a browser build that is the same
program.

The answer in one line: **three of the body's six layers are already runtime
data or a small change from it, one is a medium change with a free first step,
and two are large and should not be genes yet.** The rest of this note is the
evidence and the experiments.

**Run, 2026-10-03** (§7, tables in [0005_results.md](0005_results.md)): every
layer can be changed today, and nothing crashes when it is -- which is the
problem. Proportions keep their poses and lose their footing, their route and
their aim; moves cross a skeleton family cleanly and lose whatever lived in the
donor's hooks; across families, copied angles keep the bones and lose the move;
declared senses do nothing without their prerequisites; and the bake is
deterministic on one machine and not between two. What a generated creature
must pass is [0007](0007_the_gate.md).

## 1 · What a creature's body is, in code

Six layers, from cheapest to vary to dearest. "Runtime" means the simulation
reads it through a value that can differ per hunt; "static" means a `&'static`
table compiled into the species.

| Layer | Where it lives | Today | What a gene here would need |
| --- | --- | --- | --- |
| **Knobs** | Oven cells per species, `oven::species_raw`, read through `Species::common`, `::own`, move rows | Runtime | A per-hunt override and a way to agree it between peers. Tempers are the precedent. |
| **Proportions** | `Bone::rest`, `Shape::min`/`max` — metres at scale one, multiplied by `species.scale()` in `Rig::build`, `Rig::rest_at`, `Species::rest`, `Species::shape` | Static tables, uniform runtime scale | A *build* vector multiplied in those four places. The feet are the risk (§2). |
| **Parts** | `Part::{weak, vuln, mountable, solid, breakable, hollow, sheds}`; `breakable` folded into health slots at compile time (`beast::breakables`, `MAX_BREAKABLE` 12) | Static flags; `vuln` points at a runtime knob | A per-hunt mask over the flags. Health slots stay the species'. |
| **Traits** | `FightDecl`: data fields (`hears`, `perceives`, `hazards`, `objectives`, `collides`, `lands_on_bodies`, `rolls_over`, `steepest`, `bodies`, `keeps_height`, `pack`) and hook fields (`appetite`, `prowl_to`, `commit`, `hide`, `struck`, `landed`, `marks`, `presence`, `clip`, `frame`, the guard, the sight) | Static | The data fields are **portable**: a mask. The hooks are **native**: code written against one species' part indices and lore layout, and they stay with the body they were written for. |
| **Moves** | `MoveDecl` = name + clip + flags + a knob row; a clip is baked rows of angles, one per bone, for *this* skeleton (`Species::row`, `channels(bones)`) | Static | Same skeleton: address a clip by `(species, clip)` and it plays. Different skeleton: re-bake the recipe (§2). |
| **Topology** | `Species::bones`, `parents`, `legs`; `Rig` and `Monster` carry `&'static Species`; the renderer's `Look` and the hunter's `Card` are per species | Static, and `&'static` everywhere | A species becomes an instance. Large; not a gene for now (§4). |

## 2 · Three things in the code that change the cost picture

**Poses are angles, and proportions are applied when the rig is built.** A baked
row is three angles per bone plus a hip offset; `Rig::build` places each bone at
`parent + rot(rest × scale)`. So a longer thigh is a different `rest`, read at
build time, and every baked clip plays on it unchanged. Proportions do not need
a re-bake. The one thing authored against lengths is the planted feet:
`Pose::plant_leg` solves hip and knee angles offline so the sole meets the floor
for the authored leg. Stretch the leg at runtime and the same angles put the
sole below the floor by the stretch times the cosine of the knee. That is the
whole risk of layer two, and it has a precedent for the fix: the simulation
already re-poses legs at runtime for breaks (`LegDrop`, `LegFold`, `LegBuckle`
— the sag), so a runtime correction of the knee for leg length is the same kind
of thing in the same place.

**Three species share one skeleton.** The Pair's bones are "the Ridgeback's, in
the Ridgeback's order", and so are the Veilstalker's (`species/pair/mod.rs`,
`species/veilstalker/mod.rs`). A baked row is angles by bone index, so **a Pair
clip plays on a Ridgeback body as it is**, and a Veilstalker's on either. That
is a cross-species move transplant with no new animation work, inside a family
of three. The move's knob row (hit volume in body-space metres, frames, damage,
cones) comes with it and wants scaling by the size ratio between donor and
recipient.

**Recipes are written against roles, not bones.** `RidgebackPose` is a trait of
`root`, `spine`, `chest`, `head`, `neck`, `tail`, `leg(which, …)`; the Pair,
Veilstalker, Mireback, Mantis, Galewing and Broodmother each have their own
near-identical trait and their own `stride(p, which, u, reach, lift)`. One
shared `BeastPose` trait with optional roles would let a recipe bake onto any
skeleton that has the roles it uses: a Ridgeback bite on the Mireback's
thirteen bones. That is offline work in `anim` and opens the body *across*
families.

**The determinism catch.** The solver is `f32` and runs offline; the tables it
writes are deterministic because they are committed. A clip baked at breed time
on two machines can differ in the last bit of one channel and desync on the
first frame it plays. So a bred clip is one of:

- (a) **never baked at runtime** — transplant within a family only; the rows are
  already in the binary;
- (b) **baked once and shipped as a table** with the genome: three phases ×
  thirty-two samples × fifty-eight channels × four bytes ≈ 22 KB per attack for
  an eighteen-bone body — too big for a link, fine for a peer exchange or a
  shared log;
- (c) the solver made fixed point — a project.

(a) first; (b) when the role trait exists; (c) only if (b) proves too heavy.

## 3 · What "the body plan changes" can mean, cheapest first

1. **Build.** Five to eight multipliers: foreleg, hindleg, neck, tail, trunk
   length, trunk height, head. Visible at a glance, and it changes the fight in
   the way the Ridgeback's extra metre did — which surfaces a class can reach
   from the floor (`beastcheck`), how far a bite reaches, where the back is.
   Size already runs half to triple in the Oven; a build is size with a
   direction.
2. **Parts.** Which parts are weak, which are soft (`vuln`), which can be stood
   on. Moves the route: a line whose ridge has hardened and whose throat has
   opened is a different climb.
3. **Portable traits.** Hearing, a perception filter from the named set, a
   hazard kind, whether it rolls over. A Ridgeback line that hears is a
   different approach.
4. **Moves from a cousin.** Within the family now; across families once the
   role trait exists. Changes the answer table, which is the contract's second
   clause.
5. **The body-parent rule for hybrids.** A hybrid has one parent's skeleton,
   native hooks, health slots, `Look` and `Card`. From either parent it may take
   knobs, build, part flags, portable traits and moves. That is how real
   hybrids read, and it keeps every hook running against the indices it was
   written for.
6. **Topology** stays out. Not because it is impossible — the Broodmother and
   the Siegeshell already generate their leg tables in `const fn` for eight and
   six legs — but because `&'static Species` is load-bearing in `Rig`,
   `Monster`, the renderer and the hunter, and a runtime species is a refactor
   of all four for a gene whose visible effect the build vector mostly gives.

## 4 · The genome's body section, as proposed

```text
species        body parent (1 byte)
build          8 × i16, percent of the species' own rest lengths
parts          weak | mountable | soft, a bit per part (48 bits each), and a
               vuln share per part (i16 percent)
traits         portable FightDecl flags, and a perception filter by name
moves          up to 16 × (donor species, donor clip, knob row deltas)
knobs          (index, i16 share) deltas over the species' Oven cells
```

**Where it lives.** As *rules*, the way Oven tuning does: applied before the
hunt, folded into `World::checksum` so two peers with different genomes desync
on frame one with a message rather than a mystery, never saved or restored by a
rollback because it never changes mid-hunt. The snapshot carries a hash of it
(8 bytes, in the hunt's lore or beside `Monster::temper`). The picker's travel
byte has seven bits spoken for, so a genome is a new message at hunt start,
not a bit field — the one piece of netcode this needs. Versus has no creature
and carries nothing.

**What it costs against the budgets.** Snapshot: the hash. Frame: `Rig::build`
already multiplies every rest offset by the scale; a build vector is one more
multiply per bone per frame, eighteen to twenty-six bones, in fixed arrays — no
allocation, no measurable time. Browser: the same program; a genome arrives by
query string when small or by the shared table when it carries clips. Pins:
with every share at one hundred, every mask the species' own and no moves
transplanted, the Ridgeback must come out **bit-identical** — that is the
parity bar every build below is held to, as every creature's was.

## 5 · The experiments

Each is small enough to build in a sitting, and each has a measurement and a
stop condition. B0 to B2 are independent of each other; B3 waits on nothing but
is the largest; B4 needs the first three.

### B0 · The build vector

- Add `beast::Build` (eight shares) and read it in `Rig::build`, `Rig::rest_at`,
  `Species::rest` and `Species::shape`, through the creature. `plant_leg` in
  `anim` keeps reading the authored table: the bake does not change.
- A dev flag on `fight`, `beastcheck` and `preview_beast`:
  `--build legs=130,neck=80,tail=120`.
- **Measure:** `beastcheck` at five builds — which surfaces each class's hop
  reaches, and whether that *set* changes (a class gaining or losing the floor
  route is a change in kind). A new line in `beastcheck`: the lowest sole over
  the idle, walk and gallop rows, in centimetres below the floor. The fight
  report for five builds × six classes × twelve seeds, read through 0006's
  signature. A `preview_beast` sheet at ±25 % legs, neck and tail for a person
  to look at.
- **Stop:** if soles sink more than about 0.3 m at ±25 % legs and the sag
  machinery's knee correction does not take it back, the build's leg range
  shrinks to ±10 % and body plans lean on parts and moves instead.

### B1 · The part mask

- Per-hunt overrides of `weak`, `mountable` and the `vuln` share, read where
  `Part` is read. Health slots unchanged.
- **Measure:** `ridge_hits`, `topples`, `rides`, `legs_broken` and the windows
  across a dozen masks; the contract's clause nine (nowhere safe) and clause
  seven (there is a window, and a route to it) as the fight assertions already
  state them.
- **Stop:** none expected; this layer is flags.

### B2 · A move from a cousin

- Address a clip by `(species, clip)` so a `MoveDecl` on one species can name
  another's rows; scale the donor's knob row by the size ratio.
- Give the Ridgeback one Pair move and one Veilstalker move; give the Pair the
  Ridgeback's back kick.
- **Measure:** landed/thrown for the new move; `reactable` share; **unanswerable
  must stay zero**; where the hit volume sits against the body (`frametable`'s
  overlay) — a cat's pounce volume on a thirteen-metre body is the thing most
  likely to be wrong.
- **Stop:** if volumes cannot be made right by the size ratio alone, moves
  carry their volumes as body-space shares rather than metres, which is a
  change to `MonsterField` for every creature at once.

### B3 · The role trait

- One `BeastPose` trait with optional roles, the seven species' traits
  implementing it; one `stride`. Bake a Ridgeback recipe on the Mireback and a
  Mantis recipe on the Ridgeback; look at the sheets.
- **Measure:** eyes, and whether `tests/species.rs`'s stands-and-moves holds
  on the baked rows.
- **Decide:** (a), (b) or (c) from §2 for bred clips. The expected answer is
  (b).

### B4 · A hybrid in the harness

- `--hunt ridgeback --genome <file>`; the report; a person plays it.
- **Measure:** does it read as a new animal in the first thirty seconds. The
  fight report says whether it is fair; only a person says whether it is new.

## 6 · Open questions

1. **Health slots.** A hybrid taking the donor's breakable parts needs the slot
   table at runtime, inside `MAX_BREAKABLE`. Cheap, but it moves a compile-time
   fact. Do it only if B1 shows breakables are where the kind lives.
2. **Paint.** The renderer's `Look` is the body parent's. A line wants to be
   recognisable: colour by the genome's hash is cheap and enough for now.
3. **The scripted hunter.** Its `Card` lists which moves buck and what to call
   parts, per species. A transplanted move it has no line for is still
   measured — unanswerable is computed from the telegraph, not the card — but
   its plan will not answer it well, so a hybrid's win rate reads low. Note it
   in 0006's confounders.
4. **Where the genome is exchanged.** Out of band at hunt start is the
   smallest change. Whether it rides the same channel a future signalling
   service uses is for the netcode, later.

## 7 · What broke, 2026-10-03

Run in full: the lab (`crates/lab`, outside the workspace, through a `lab`
feature in `sim` that lets a species be installed at runtime and is off in
every build the game and the tests make), and source mutations in a scratch
checkout. Every table is in [0005_results.md](0005_results.md); the
constraints it adds up to are [0007](0007_the_gate.md). The experiments ran as
designed with one change: B3, the role trait, was not built -- instead the
lab moved baked poses between skeletons by a role map guessed from bone names
and the leg table, which is what can be done today without new code.

**Nothing crashed.** Eighty-odd bodies, whole fight declarations put on the
wrong skeleton, moves played on animals they were not written for: not one
panic. Every failure below is silent -- a hunt that never ends, a trait that
does nothing, a foot three metres underground -- which is why the gate has to
measure rather than wait for an error.

### Proportions (E1)

- **The pose survives, the stance does not.** Poses are angles, so a stretched
  leg plays every clip in the right shape -- and stands in the floor. Legs 1.6
  times longer put the soles 2.8 m down; 0.7 times, 1.3 m up. Re-seating the
  hips fixes standing for every length tried
  ([naive](0005/e1-legsx1_6naive.jpg), [re-seated](0005/e1-legsx1_6kept.jpg)).
- **The fight's sentence is a statement about proportions.** At 0.7x legs the
  back is at 4.0 m and five of six classes hop onto it from the floor; at 1.3x
  nobody does. The Ridgeback's whole premise moves with one number.
- **Blows do not follow the body.** A longer or shorter neck, tail or trunk
  moves the bone a move rides 0.8 to 1.5 m away from where its blow lands; the
  existing Size knob, 2.2 to 2.7 m. Hit volumes are authored in absolute
  metres and carried only by the bone's motion from rest.
- **Every gait skates today**, by about half (§E0), and how much moves with
  leg length -- 0.64 at 0.7x, 0.12 at 1.6x. The stride is a knob; for a body
  that changes it has to be computed.
- Fights: longer legs and a longer trunk made it much harder (1 or 2 hunts of
  36 won, from 6); a 1.5x Size knob broke the contract outright (nobody climbed
  it). Neck, head and tail lengths moved the fight little.

### Parts (E2)

- With nothing to stand on, or feet that do not break, the scripted hunter lost
  all 36 hunts and the contract broke in six clauses: the route is the fight.
- **The hunter cannot find a moved weak point.** The head made the weak point
  and the back not scored exactly as having none: its plan climbs to where the
  Ridgeback's ridge is. A generated body needs a hunter that reads the body.

### Moves within a skeleton family (E3)

- **Animation transplants cleanly.** Every Pair and Veilstalker move played on
  the Ridgeback with its feet no deeper than the Ridgeback's own moves put them
  (worst 0.64 m against 0.53 m) and every body blow on the body.
- **Behaviour does not.** Moves whose choosing or flying lives in the donor's
  hooks were never thrown (the pounce, the perch, the dive, the ambush) -- five
  of the Pair's eight chosen moves. The tail trip was thrown 69 times and never
  landed. And balance is fragile: the Veilstalker's rake on the Ridgeback was
  thrown 138 times, landed 108, and the hunters won 1 of 36.
- The Ridgeback's moves on the Pair were thrown once each: the Pair's appetite
  hook decides what a cat does, and it has never heard of a bite.

### Moves across skeletons (E4)

- The role map drove 9 to 28 of a recipient's bones and left 5 to 10 of the
  Ridgeback's eighteen with nowhere to go -- the tail on anything without one,
  everything but the legs and neck on the Galewing.
- **Copying angles keeps bones in place and loses what the move is.** The
  Mantis takes the bite as a nod of its short neck
  ([sheet](0005/e4-mantis-bite.jpg)); the Broodmother's eight legs, each given a
  Ridgeback leg's angles, lift off the floor in the bite and go three metres
  under it in the slam ([sheet](0005/e4-broodmother-bite.jpg)); on the
  Siegeshell, whose hips are twice as high, the scaled volumes landed up to
  four metres off its body.
- In a fight the transplanted bite was ignored by some brains (the Mireback,
  the Siegeshell, the Sandmaw never threw it) and spammed by another (the
  Galewing threw it 585 times and landed 22).

### Traits (E5)

- **Declared senses do nothing without their row.** A Ridgeback that sees
  nobody and hears fought exactly as one that sees everybody, until it was
  also given the senses row of knobs (and noise cells in its lore): then the
  hunters won 10 of 36 instead of 6, in fights half as long again.
- A hazard kind declared without the hook that makes hazards is inert.
- **Hooks do not cross bodies.** The Mireback's whole fight on the Ridgeback's
  body made a creature nobody beat in 33 seconds; the Sandmaw's and the Pair's
  made 36 hunts each that never ended.

### A hybrid (E6)

The Ridgeback's body, legs 1.15x with the hips re-seated, the head as its weak
point, the Pair's swat, the Veilstalker's tail spear, hearing with its row's
prerequisites half met: it stands, its blows come from its body, both new
moves are thrown and land (41 of 68, 27 of 51), and the hunters won 6 of 36 --
the as-built rate. It broke the contract only where the hunter could not find
its new weak point ([sheet](0005/e6-hybrid.jpg)). **A hybrid is possible today
when it stays inside one skeleton family and one parent's hooks.**

### The bake (E7)

- Deterministic on one machine: the same tables twice, and from debug and
  release builds.
- Six of the nine committed tables reproduce exactly. **Three do not** -- the
  Galewing by up to 19 degrees, the Mantis and the Sandmaw -- and not even at
  their own commits, so they were baked from tuning that never reached a
  commit. Nothing checks this.
- **One bit is enough to desync.** Nudging every non-exact maths result by
  one unit in the last place moves up to 0.4% of a table's values (the
  Broodmother, by up to 0.2 degrees; the rest by one raw unit). Two players'
  machines baking the same genome would not agree. That is why the bake has to
  become deterministic before anything is bred at runtime, and it is also why
  the Galewing's 19 degrees are tuning drift, not a platform.

### Topology in the source (E8)

- **Changing a leg count does not compile, and the compiler says where.** The
  baked table is a typed array, so the simulation will not build until it is
  rebaked, and the bake will not run until the simulation builds; species.md's
  stub breaks the loop.
- After the stub: the Broodmother at six or four legs fails on one
  hand-written eight-entry stance table in `anim`; at ten, on her four-entry
  hip and eight-entry name tables. The Siegeshell at eight fails the same way;
  **at four it compiles, rebakes, passes its own 26 tests and fights** -- and
  `tests/species.rs` catches its mirror table, still written for six legs.
- **A new bone is free.** A horn on the Ridgeback: rebake, all 51 tests pass,
  and six hunts came out exactly as before.
- **A removed limb is fourteen compile errors in two files** -- the animation's
  pose helpers and the tests -- plus two in the renderer: every coupling is a
  named constant the compiler finds. Index coupling is loud. What is quiet is
  everything in the bullet lists above.
