---
status: exploration
started: 2026-10-03
---

# 0006 — Numbers as differences in kind

*Does moving a creature's knobs inside the Oven's ranges make a different
fight, or only a harder one? The measurement, written down before it is run.*

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
