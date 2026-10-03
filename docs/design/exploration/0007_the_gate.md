---
status: exploration — proposed
started: 2026-10-03
---

# 0007 — The gate

*What a generated creature must pass before it is born. Written from what broke
in [0005](0005_body_plans.md)'s lab and [0006](0006_differences_in_kind.md)'s
sweep, as properties a body has to have rather than shapes it has to be.*

## The question

Every creature so far was built by an AI and looked at by a person, and both
of them caught things the other did not. A breeding program takes the person
out of the loop for every creature but the first few, and the AI out of most
of it too: a bred creature is made by code, at a breeder's request, possibly
on their machine. So what used to be judgement has to become a check, and the
check has to be **fast** (a breeder is waiting), **deterministic** (two peers
must agree a creature is valid), and **not anatomical** (nothing in it may
say "a creature has four legs and a tail", or the world can only grow animals
it already had).

The answer this note proposes: **check function, not form.** A body is valid
because every part of it does a job the fight can use, and does it within
limits the fight can survive -- not because it looks like something.

## 1 · Roles are jobs, not limbs

Today a bone's meaning is its name and its index ([0005](0005_body_plans.md)
§7, E4): `tail3`, `bones::HEAD`, `FOREFOOT_L = 14`. The lab had to guess roles
from names to move a pose between skeletons, and the guess was right for legs
and spines and wrong for anything else -- the Galewing's wings and the
Mantis's blades had no role to receive the Ridgeback's moves, and the Mantis
took the bite as a nod, because a pitch on its one short neck is not the
Ridgeback's reach on two long ones.

The vocabulary a generator and a gate need is what a part **does in the
fight**. Six jobs cover every part of every creature built:

| Job | What it does | Its mechanical contract |
| --- | --- | --- |
| **Support** | Bears the body on the floor | Its sole is on the floor when planted, its stance sweep matches the ground covered, and the body's mass sits over the polygon the supports make |
| **Striker** | Carries a hit volume | On the frame the blow comes out, the striker is in or at its volume, and the volume moves with it |
| **Surface** | Is somewhere a fighter stands | Its top is level enough to stand on, its height is where the creature's sentence says the route is, and consecutive surfaces touch |
| **Weak point** | Fills poise when hit | It is reachable by the route, and the hunter knows it is there |
| **Breakable** | Costs the creature something when broken | It is reachable from where a fighter can be, and breaking it does something the player can see |
| **Sensor** | Is where the creature perceives from | The senses it declares have the knobs and lore cells they need |

A part can have several jobs (the Ridgeback's foot is support, breakable and
a striker for the stomp). A bone's name is for people. A generated body is a
tree of bones whose parts carry jobs; nothing in the gate reads a name, and a
seven-legged thing with two strikers on its back is as valid as a horse if its
jobs hold.

## 2 · The checks

Six layers, cheapest first. Each says what it checks, what failed it in the
experiments, and roughly what it costs. Thresholds are first guesses taken from
the creatures as built, which pass every one of them.

### G1 · The table is well formed (load time, microseconds)

- A tree: parents before children, one root. Within `MAX_BONES`, `MAX_PARTS`,
  `MAX_BREAKABLE`, `MAX_MOVES`.
- **Every per-limb table is derived from the limb count.** Today it is not:
  the Broodmother's eight-entry stance array and the Siegeshell's six-entry
  name, rim and mirror tables are written out by hand, so four or ten legs do
  not compile and four Siegeshell legs compile with a mirror table that points
  past the skeleton (0005 §7, E7). `tests/species.rs` caught the last one.
- Mirror pairs within the skeleton and on opposite sides; the neck chain and
  follow bones within it; a leg's hip, knee and foot are a chain.
- The baked table's channel count is the skeleton's. Today that is a type, so a
  stale table does not compile -- good -- but it also means a new skeleton
  cannot compile until it has a table, and it cannot have a table until it
  compiles (species.md's stub breaks the loop by hand).

### G2 · It stands (statics, milliseconds)

Posed **through the creature**, not read off the table: the Siegeshell's and
the Broodmother's feet are planted by a runtime hook, and the raw clip had the
Siegeshell ten metres in the air (0005 §7, E0).

- Idle: the lowest sole within a few centimetres of the floor (every creature
  built: -0.08 to +0.02 m). Stretched legs with the hips left alone put the
  Ridgeback 2.8 m into the floor or 1.3 m above it.
- Walking and galloping: no sole more than 0.3 m into the floor.
- Through each move: no sole more than about 0.6 m into the floor unless the
  move declares that it buries a foot. The Ridgeback's rear-and-slam read onto
  the Broodmother put her feet three metres down.
- **Skating**: the stride the walk animates against the ground it covers.
  Today every creature's planted feet sweep back at about half the speed the
  stride knob says the ground moves (skate about 0.5 on all of them), and the
  number moves with leg length (0.64 at 0.7x, 0.12 at 1.6x). For a generated
  body the stride is not a knob: it is **computed from the leg's stance
  sweep**, and the gate checks the two agree.
- **Balance** (not measured yet): the body's mass, taken as its parts'
  volumes, sits over the polygon of its planted supports. This is what makes a
  three-legged or a top-heavy thing valid or not without saying what it is.

### G3 · Its blows come from its body (statics, milliseconds)

- On the frame each body blow comes out, some part of the body is within half
  a metre of the volume. Every creature built passes; the Ridgeback's stomp
  read onto the Siegeshell missed by four metres.
- **Drift**: the bone a move rides and its volume move together. A hit volume
  is authored in body-space metres and carried only by the bone's motion from
  rest (`Monster::hit_volume`), so a longer neck takes the head somewhere the
  bite's volume does not go, and the Size knob grows the body and not one of
  its volumes (0005 §7, E1). For a generated body the volume is **placed from
  its striker** -- the part's box at the contact pose, inflated by a reach --
  rather than authored. Measured: 0.8 to 1.5 m of drift for a neck, tail or
  trunk changed by a third, and 2.2 to 2.7 m for the Size knob at 0.6x and 1.5x.
- A move whose meaning lives in a hook travels with the hook or not at all.
  The Pair's pounce, perch and dive, carried to the Ridgeback, were never
  thrown, because what flies a cat through the air is the Pair's frame hook
  (0005 §7, E3).

### G4 · The route exists (statics, milliseconds)

The creature's sentence says where the fight is. The Ridgeback's -- *you
cannot reach the thing that kills it from the ground* -- is a statement about
heights, and the gate checks it:

- The lowest surface's height against every class's hop: who can get on from
  the floor (one class as built; five at 0.7x legs; none at 1.3x).
- Every weak point reachable by the route, and every breakable reachable by
  somebody standing on the floor or on a surface. With nothing mountable, or
  feet that do not break, the scripted hunter lost all thirty-six hunts.
- What the declared sentence needs is the breeder's choice -- a creature you
  can climb from the floor is a different fight, not an invalid one -- but it
  must be **a** sentence the checks can read: a list of routes and the window
  each earns.

### G5 · What it declares, it has (load time, microseconds)

- A perception filter and hearing need the senses row of knobs, and hearing
  needs noise cells in the lore. Without them they are ignored, silently: a
  Ridgeback declared blind and hearing fought exactly as before; given the row
  too, the hunters won 10 of 36 instead of 6.
- A hazard kind needs a hook that makes hazards; declared alone it is inert.
- A species' hooks are written against its own part indices and lore layout.
  Handed to another body they do not crash -- nothing in the lab crashed -- they
  do nonsense: the Sandmaw's or the Pair's fight on the Ridgeback's body
  produced thirty-six hunts that never ended. **No hook crosses bodies.** A
  hybrid takes one parent's hooks with that parent's skeleton, or none.

### G6 · The fight holds (simulation, seconds)

The fight contract (`crates/hunt/tests/fight.rs`), run for every class:
every move answerable in its own way, a threat at every range, openings long
enough to punish, nothing unanswerable, the hunt ends, nobody wins all of them
and nobody none. Seventy-two hunts take about six seconds.

- Half of the mutated genomes in 0006 broke a clause the tuned creature keeps;
  every wild one did. So the gate is not a formality: it is the filter that
  makes random variation fair.
- Each move is **thrown** and **lands sometimes**. The Pair's tail trip on the
  Ridgeback was thrown sixty-nine times and landed none; the Ridgeback's bite
  on the Galewing was thrown 585 times and landed twenty-two.
- **The evaluator has to read the body.** The scripted hunter is a plan per
  species: it climbs to where the Ridgeback's ridge is. Moved to the head, the
  weak point was never hit, and the hunt read the same as one with no weak
  point at all. A generated creature needs a hunter that finds weak points,
  routes and breakables from the jobs in §1, or G6 measures the plan instead
  of the creature.

### G7 · Two machines agree (the bake)

- The bake is deterministic on one machine (two runs, two build profiles,
  identical), and six of the nine committed tables reproduce exactly.
- **Three do not** -- the Galewing (by up to 19 degrees on a channel), the
  Mantis and the Sandmaw -- not even at their own commits, and the one-bit test
  below says machine maths cannot explain differences that large: they were
  baked from tuning that was never committed. Nothing checks a committed table
  against its recipe today.
- Nudging every non-exact maths result in the bake by one bit (what a
  different platform's maths library does) moves up to 0.4% of a table's
  values, by up to 0.2 degrees on the Broodmother. A bake run on two players'
  machines would disagree and desync. **A generated creature's table is either
  baked by a deterministic solver** (fixed point, or a software maths library
  every platform runs identically) **or baked once and shipped.**

## 3 · The four hard problems, against the gate

The person's list, and where each stands after the experiments.

1. **Coalescing the fight into body plans that make sense.** The gate does not
   generate; it rejects. What it gives the generator is a target that is not
   anatomy: G2 (it stands and its feet are planted), G3 (its blows come from
   its strikers), G4 (its sentence is true). A generator that composes jobs
   rather than limbs, then solves proportions until G2 to G4 pass, can make
   things no animal is and still have them read as bodies.
2. **New moves for those bodies.** A move is three things today: a clip, a
   volume and a row of numbers, and they are authored separately. The
   experiments say the volume must come from the striker (G3) and the numbers
   from the body's size; only the clip is content. A move for a generated body
   is then *a striker, a path for it, and a tempo* -- and the path is what the
   role trait of [0005](0005_body_plans.md) §5 B3 would let a recipe express
   without bone indices. [0006](0006_differences_in_kind.md) §8 says which of
   those a fight feels: the tempo (frame data is what reshapes a fight without
   making it harder) and when the move is chosen, far more than the size or
   reach of its volume, which the rest of the move set absorbs.
3. **Moves that survive a change of topology.** Within a skeleton family they
   already do: the Pair's and the Veilstalker's moves played on the Ridgeback
   with their feet on the floor and their blows on the body. Across families,
   copying angles by role keeps the bones in place and loses the meaning (the
   Mantis's nod). What survives is the **intent** -- this striker, to this
   point, at this time -- re-solved on the new body, which is a bake, which is
   G7.
4. **Animations that work with the body plan.** Proportions need no re-bake
   for the pose to be the right shape (poses are angles), but they need the
   hips re-seated (G2), the stride recomputed (skating), and the contact poses
   re-solved for feet that plant (`plant_leg` solves for the authored leg). The
   runtime already re-poses legs for breaks and the Siegeshell plants its feet
   in a hook; a **planting layer for every legged creature**, solved in fixed
   point at runtime, would make most of G2 true by construction rather than by
   check.

## 4 · What to build, in order

1. A test that every committed creature table equals its re-bake (G7), and
   re-bake the three that drift.
2. Per-limb tables derived from the limb count (G1); the Siegeshell's mirror
   and the Broodmother's stance first.
3. A declaration check for traits (G5): a filter or hearing without the senses
   row is a load error, not a no-op.
4. The statics as a library beside `beastcheck` -- the lab's `measure` module,
   moved into `sim` in fixed point -- and run on every registered creature in
   `tests/species.rs`, so the creatures as built are held to G2 to G4 too.
5. A body-reading hunter (G6), so the fight contract measures a creature
   rather than a plan.
6. The deterministic bake (G7).
7. Then generation.
