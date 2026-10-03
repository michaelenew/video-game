---
status: exploration — run
started: 2026-10-03
run: 2026-10-03
---

# 0006 — Numbers as differences in kind

*Does moving a creature's knobs inside the Oven's ranges make a different
fight, or only a harder one? The measurement, written down before it is run.*

**Run, 2026-10-03** (§8; tables in [0006_results.md](0006_results.md)): kind
exists, and by default it is modest and concentrated. Mutated genomes of equal
difficulty make measurably different fights -- more different than the hardest
temper -- but about as different as a temper in size, and in timing and
choice rather than geometry: 94 of 247 knobs carry it, 142 carry nothing at the
level of a whole fight. New animals have to come from the body.

## The question, and why it decides things

A bred creature, in the smallest version, is its species with a different
vector of knobs ([0005](0005_body_plans.md) §1, layer one). Tempers are already
a line through that space: four knobs, three steps, and the ladder measured on
the Champion went 20, 12, 8 and 1 hunts won of 40. That is a difficulty slider,
and a slider is what No Man's Sky's planets turned out to be — endless
variation along axes that do not change what you do.

The breeding thesis needs more than that. It needs directions in knob space
that are **orthogonal to difficulty**: two genomes equally hard to beat that are
beaten *differently* — different moves matter, different answers get used,
different classes win, the fight happens in a different place. If those
directions exist and the fight report can see them, bred lines are new animals
and the world can grow out of them. If they do not, breeding is tempers with
names, and the world design should lean on parts, traits and moves (0005
layers two to five) instead. Either answer is useful; the point is to have it
before a genome is designed.

The test is cheap. Six Ridgeback hunts of about a minute each run in 0.47 s in
a release build on this machine (`fight --repeats 6`), so **one hunt costs about
80 ms** and a quarter of a million hunts is an afternoon on four cores.

## 1 · Definitions

**Genome** `g`: a value for each of a species' knobs — `species::Common`, its
own, and the twenty-five `MonsterField`s of each move; for a pack creature its
`PackKnob`s and `CritterField`s; for one with senses its `FightField` row.
Every one is an Oven cell with a range, enumerated by `oven::species_knobs`.

**Hunt** `h(g, class, seed)`: one run of the scripted hunter for that class
(`hunt::class`) against the species in its own arena at temper zero, solo,
under `fight`'s default frame cap. Deterministic: the seed reproduces it.

**Difficulty** `D(g)`: a scalar. Over all classes and seeds, three numbers —
win rate, mean health left per hunt, mean frames to kill among wins (an
`Unresolved` hunt counts as a loss with its health left). Each is z-scored
against the baseline genome's seed scatter and the three are averaged. Beside
it, the **per-class win vector** `W(g)` in `[0, 1]^6`, because a genome that
changes *which* class wins is a difference in kind by itself.

**Signature** `S(g)`: the fight's shape, from the `Report`, as shares and rates
so that fight length does not leak in. Per hunt, then averaged over seeds, per
class, then concatenated across the six classes:

| Group | Components |
| --- | --- |
| Move mix | each move's share of starts; each move's landed-per-start |
| Rhythm | the five `Threat` window shares; `reactable_share`; `openings_per_minute`; `mean_opening`; `idle_share`; `dominant_share`; `entropy`; `coverage` |
| The ride | `ride_share`; rides per minute; `mean_ride`; bucked (`thrown`) per ride; `fled` per ride |
| The route | `topples` per minute; `ridge_hits` per minute; `legs_broken`; `foot_damage` as a share of `dealt` |
| The hunter | `swings` per minute; `connected / swings`; `spread`; the class layer's `Uses` lines as rates |
| Where | mean flat range at commit and the aboard share of commits, from the `timeline` beats |

**Noise** `N`: the baseline genome (the species as tuned) run over many seeds.
Split the seeds into two halves of `S` repeatedly; the distances between the
two half-means are what two genomes that are *the same* look like at `S` seeds
apiece. Its 95th percentile is `N95`.

**Distance**: Euclidean, on components each divided by the baseline's per-seed
standard deviation, so one unit along any component is one seed's worth of
noise. Plainly: a distance of one means "as different as two runs of the same
creature usually are".

## 2 · The measures

**M0 · The contract rate.** For every sampled genome, the twelve assertions in
`crates/hunt/tests/fight.rs` — most of what it throws can be answered on sight,
it uses its whole move set, no single move is the whole creature, an opening is
long enough to punish, it is never just standing there, it is dangerous most of
the time and open the rest, the back is reachable and not a safe room, a ride
is long enough to do something with, breaking its poise happens, nothing hits
from where it could not be read, the hunt reaches a conclusion, the fight is
neither free nor hopeless, the fight uses the arena — evaluated on the genome's
hunts. The share of genomes that break any, per regime, is how much of the knob
space is *fair*, and so how hard a breeding filter has to work.

**M1 · Separability at matched difficulty.** For every pair of genomes whose
`D` differ by less than half a baseline standard deviation, the distance
between their mean signatures against `N95`. The **kind rate** is the share of
matched pairs further apart than `N95`. If the space is a slider, matched
pairs are the same fight and the kind rate sits near five per cent.

**M2 · Identifiability.** Hold out one seed per genome. Classify its signature
to the nearest genome mean among genomes in the same difficulty band. Accuracy
against chance (one over the band's size). A fight you can tell apart from a
single run is a fight a player will feel.

**M3 · Which knobs carry kind.** One knob at a time, every other at tuned:
seven levels across the knob's whole Oven range and five across ±30 % of its
tuned value. Per knob, two numbers: the change in `D` across the sweep (its
**degree** effect), and the displacement of `S` orthogonal to the difficulty
direction (its **kind** effect — the difficulty direction `v̂` is the regression
of `S` on `D` over all the random genomes; kind is `|ΔS − (ΔS·v̂) v̂|`). That
ranks every knob as *kind*, *degree*, or *dead*. Along each sweep, also: the
largest jump between adjacent levels as a share of the sweep's total
displacement (**a cliff** when over a half), and whether `D` is monotonic. The
Elementalist's stone chain and temper I's first values both say the space has
cliffs; this finds them.

## 3 · Sampling

**Species.** The Ridgeback first: eight moves, pinned bit for bit, the
best-understood fight, and the scripted hunter plays all six classes against
it. Then, as a second pass for generality, one pack creature (the Gnawers:
`PackKnob` and `CritterField` genes) and one hook-heavy creature (the Mireback)
— if the Ridgeback's answer is interesting either way.

**Knobs.** The Ridgeback's fifty-seven `Common` knobs, its own (the hide
multipliers and the shake force), and two hundred move fields. Left out as not
continuous magnitudes about the fight: `Follows` (which bone a volume rides —
categorical, a trait), `Unblockable` (a flag — a trait), `Margin`, `Spawn` and
`HunterSpawn` (where things stand before the fight), `HuntGrace` (the opening
pause), and the pose row (`GaitStride`, `BreathRate`, `HeadTrack`: how it
looks, not what it does). About two hundred and thirty-five genes.

**Regimes.**

| Regime | What it models | Draw |
| --- | --- | --- |
| **R1 · Mutation** | An offspring a few generations from stock | Each knob varies with probability ¼; a varying knob is uniform in `[0.7, 1.3] ×` tuned, clamped to its Oven range, frame counts at least one |
| **R2 · Wild** | The whole space, to find its edges | Every knob uniform over its full Oven range |
| **R3 · One at a time** | Each knob's own effect | Seven levels across the range, five across ±30 % of tuned |

**Sizes**, at twelve seeds × six classes = seventy-two hunts a genome:

| Run | Genomes or settings | Hunts | Time at 80 ms, one core |
| --- | --- | --- | --- |
| Baseline noise | 1 × 240 seeds × 6 classes | 1,440 | 2 min |
| R1 | 400 | 28,800 | 38 min |
| R2 | 200 | 14,400 | 19 min |
| R3 | 235 knobs × 12 levels | 203,040 | 4.5 h |

Four cores bring the lot under two hours. Everything is deterministic, so any
row is reproducible from its genome and seed.

## 4 · Thresholds, written down first

- **Kind exists** if the R1 kind rate (M1) is at least 30 % *and* M2 accuracy
  is at least three times chance in bands of eight or more genomes.
- **Degree only** if the kind rate is under 10 % and M2 is under one and a half
  times chance.
- **Between**: kind is carried by a few knobs. M3's ranking names them, and the
  genome's gene list is those knobs and nothing else.
- **The filter**: if more than half of R1 breaks the contract, every offspring
  must be run through the twelve assertions before it is born (seventy-two
  hunts, about six seconds — affordable in the breeding loop, and the breeder
  is told which clause it failed). If more than nine in ten break it, the
  mutation band shrinks to ±15 % and R1 is run again.
- **Cliffs**: a knob with a cliff is either not a gene or a gene with discrete
  alleles at the levels either side of the cliff.

**Predictions**, so the result can surprise. The mind and nerve rows (glance,
lead, decisiveness, strain) are degree — that is what tempers found. Health
and damage are degree. Size is kind, because it moves the routes
(`beastcheck`). The move cones (`IdealRange`, `RangeSpan`, `AimCos`, `AimSpan`)
and the frame data are kind, because they decide which move is chosen at which
range and whether it can be answered on sight. `Cooldown` is kind, because it
changes the move mix. `Travel` and `Advance` are kind.

## 5 · The tool

`crates/hunt/src/bin/spread.rs`, beside `fight`.

```text
spread --species ridgeback --regime mutate --genomes 400 --seeds 12 \
       --classes all --band 30 --sparsity 25 --seed 1 --shard 0/4 --out target/spread
```

- For each setting: `oven::reset_to_baked()`; then for each gene
  `Knob::Species(id, tunable).set_raw(v)`, `v` clamped to `.range()`; then for
  each class and seed `play_card_in(card, None, 0, [class; MAX_PLAYERS], 1,
  limit, seed, |_| {})`. One CSV row per hunt with the `Report` fields in §1,
  plus outcome, frames, health left; one row per setting in a genome file —
  setting id, `Knob::id()`, value.
- The Oven's cells are process-global atomics, so parallelism is **processes**,
  not threads: `--shard i/n` splits the settings and writes its own file.
- It never writes `tuned.rs`; `tests/oven.rs` cannot notice it ran. Its random
  numbers are its own small generator seeded from `--seed`; it is a tool, not
  the simulation, so `no_floats.rs` and `knobs.rs` do not apply to it.
- `--regime oat` takes `--knob <id>` or sweeps them all; `--regime wild` and
  `--regime mutate` take `--genomes`.

`scripts/spread.py`, standard library only (this machine has no numpy): reads
the shards, computes `D`, `W`, `S`, `N95`, M0 to M3, and prints the tables in
§6. A `--pairs 5` flag prints the five most-separated matched pairs with both
genomes' changed knobs and both reports' move mixes and class wins, for a
person to read.

## 6 · What comes out

Appended to this note when run:

1. The contract rate per regime, and which clauses break most.
2. The kind rate, M2 accuracy against chance, and band sizes — the verdict
   against §4.
3. The knob ranking: the twenty strongest kind knobs, the ten strongest degree
   knobs, the dead list, the cliff list with the level pairs that jump.
4. The five most-separated matched pairs, described in prose from their
   reports: which moves carried each, which classes won, where the fight sat.
   The numbers say *different*; the prose is where a person checks it reads as
   *a different animal*.
5. The decision: the genome's gene list, bands and alleles for the first
   breeding build — or the finding that knobs are a slider, and the world
   leans on 0005's body layers.

## 7 · What the measurement cannot see

- **One plan per class.** The scripted hunter is a decent player with one plan
  and does not adapt. A genome whose kind lies in a weakness the plan never
  probes is invisible, so every kind measure here is a **floor**. Where a card
  has a second plan (`--gamble`), run both and take the better hunter.
- **Pairs of knobs.** R3 misses interactions; R1 samples them statistically.
  If M3 names ten kind knobs, a pairwise pass over those ten is forty-five
  sweeps, an hour.
- **Correlated components.** The signature's components are not independent,
  so the distance over-counts some directions. Acceptable for a first pass;
  principal components are the refinement if the verdict is close.
- **One arena, one temper, solo.** Genome × arena, genome × temper and coop are
  later passes; each is a flag the tool already has.
- **The frame cap.** `Unresolved` hunts are counted as losses and reported
  separately; a regime that produces many of them is a finding (a line that
  cannot be killed is a wall, 0005's thesis), not noise.

## 8 · What came out, 2026-10-03

Run in full: the baseline (240 seeds), the temper control, 400 mutated
genomes, 200 wild ones and every knob alone at twelve levels -- about 258,000
hunts, every class over the same twelve seeds. The tables are
[0006_results.md](0006_results.md); this is what they say.

### Two corrections to the plan, made before reading the verdict

- **A control was added: the tempers.** They change only how the creature
  thinks and were built to be the same fight, harder. They came out at 1.2 to
  1.4 times the noise floor N95 -- so §4's bar, "further apart than N95", is
  passed by a pure change of difficulty, and cannot by itself be the test of
  kind. Every kind rate below is also given against the hardest temper's own
  shift orthogonal to difficulty, which is the bar a difference in kind has to
  clear.
- **A knob's kind is its best of eleven levels**, and a best-of-eleven
  clears a 95% bar by chance 43% of the time. The per-knob bar is the larger
  of the noise a best-of-eleven reaches (1.13 x N95) and the hardest temper
  (1.37 x N95), and degree needs three noise sds rather than two.

And two lenses a player would recognise were added beside the distance: how
much of the creature's move mix differs, and how far the six classes' win
rates move.

### The verdict

**Kind exists, and it is modest by default and concentrated in a few places.**

- **By §4's rule, kind exists**: the kind rate is 100% (at least 30% asked)
  and one run names its genome among ten of equal difficulty 6.8 times as often
  as chance (at least 3x asked). **Against the tempers it still holds**: 90%
  of mutated pairs at matched difficulty -- 85% of those that keep the fight
  contract -- shift further away from difficulty than the hardest temper does.
- **But the size of a random mutation's difference is a temper's size.** Two
  mutated genomes of equal difficulty differ in about 11% of the moves they
  throw (temper III: 12%), and their class win rates move by 0.42 summed over
  six classes (temper I: 0.42), beyond noise in only one pair in five. A
  breeder would feel a mutated line as the same animal in a different mood.
- **The big differences are in who it is for.** The most separated fair pairs
  are the ones that flip which class wins: one genome lets the Bulwark win 11
  hunts of 12 and the Champion 1, another of the same difficulty gives the
  Bulwark none and the Champion 8. That is a difference in kind a player would
  name, and random mutation finds it rarely.
- **Wild genomes are different and broken.** Every one of 200 broke the fight
  contract, half their hunts never ended, and their move mix differs by more
  than half. The space has room for very different fights; almost none of the
  room is fair.

### Where kind lives (M3)

Of 247 knobs, **94 change the fight beyond the bar, 28 of them at the same
difficulty ("pure kind"); 11 change only difficulty; 142 change neither** --
over their whole range, one at a time.

- **Timing carries it.** 23 of 24 frame-data knobs are kind and 11 are pure:
  every move's recovery, the sweep's and spray's windup and active frames.
  How long a move commits the creature is what reshapes a fight without making
  it harder.
- **Choice is next**: half of the knobs that say when a move is chosen (its
  ideal range, range tolerance, bearing and cone) are kind, and two appetites
  and the glance are pure.
- **Geometry is nearly dead**: 44 of 56 hit-volume and travel knobs, and 38 of
  48 knobs for what a hit does (stun, knockback, launch), change neither the
  fight nor its difficulty. The exception is how far a move carries the body
  (advance), kind on five of eight moves. Not because nothing happens -- with no radius the
  bite lands 3 times in 409 instead of 110 in 257 -- but because the creature
  makes it up with its other seven moves, and the hunt as a whole absorbs it.
  This measure sees whole fights; a change to one move's local life is a
  smaller thing it is built not to see.
- **The nerve row is dead** (6 of 7), and so is most of the mind row beyond
  the glance and the pause after a rear: the wounded aggression, the repeat
  penalty, the closing appetites.
- **Difficulty lives in health**, by far (26 noise sds across its range),
  then the flinch threshold, the per-rider appetites, the pause between moves
  and the feet's hide -- each of which also changes the fight, so none is pure
  degree.
- **Six cliffs**: the turn rate, poise and the flinch threshold near the
  bottom of their ranges, and the per-rider appetites of the bite, sweep and
  kick between 2,000 and 3,300, where the creature stops choosing anything
  else while somebody is aboard.

### Against the predictions in §4

| Predicted | Found |
| --- | --- |
| Mind and nerve rows are degree | **Wrong**: mostly dead. The glance and the turn after a rear are pure kind; lead and decisiveness are kind with a change of difficulty; how far the strain thresholds fall -- the fourth temper knob -- is dead |
| Health and damage are degree | **Half**: health is the strongest difficulty knob there is, and it changes the fight too; damage is mostly dead, except on the rear-and-slam, the back kick and the sweep, where it is pure kind |
| Size is kind | **Right**, though not at matched difficulty: bigger is harder |
| Cones and frame data are kind | **Right**: frame data nearly all, cones half |
| Lockouts are kind | **Wrong**: 2 of 8 |
| Travel and advance are kind | **Half**: how far a move carries the body (advance) is kind on five of eight moves, with a change of difficulty; how fast a volume travels is dead on seven of eight |

### The decision

- **Knobs alone are not a slider, but they are not new animals either.** A
  line bred by knob mutation would feel like the same creature in a different
  mood; that is worth having, and it is not what the world thread asks for.
  New animals have to come from 0005's body layers, under 0007's gate.
- **The gene list for knob breeding is the 94 kind knobs** -- weighted toward
  the 28 pure ones -- and not the 142 dead ones, which are genes that carry no
  phenotype and would only add noise to a lineage. Frame data and the
  choosing knobs first.
- **Difficulty stays with tempers and health**, not with lines: a bred line
  should differ in what it is, and how hard it is should be the hunter's
  choice.
- **Every offspring passes the fight contract before it is born.** Half of
  the mutations broke it; that is the filter's work, and at seventy-two hunts
  -- six seconds -- it is affordable in the breeding loop. The cliffs are
  either excluded or given discrete alleles either side.
- **What kind random variation rarely finds, a breeder can select for.** The
  class-flipping pairs exist at matched difficulty; a breeding program that
  shows the breeder which classes their line favours would find them on
  purpose.

### What the measurement still cannot see

As §7 said, and confirmed: one plan per class (the hunter does not adapt),
whole fights rather than single moves, one arena, one temper, solo. Signature
components that rarely move in the baseline (the Bulwark's mean ride) can
dominate a distance by themselves, which is why every distance is also given
clipped; the verdict reads the same either way.
