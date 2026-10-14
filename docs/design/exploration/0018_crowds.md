---
status: built 2026-10-14 as far as one session reaches — the crowd field in the snapshot with its lookahead gate and first reader, cells and leaders read off the world, aggregates on the match's lines by agreement; what remains is a peer running only its regions (regions.md §"The session layer")
started: 2026-10-14
---

# 0018 — Crowds: a world of any size, no server, and nobody rolling back for a stranger

*How a peer-to-peer rollback game can hold an essentially arbitrary number of
players on one map, with no relay server we operate. The ask: the world
divided into zones, players streaming inputs only to their zone and its
neighbours; past that, cells of players with elected, redundant leaders that
relay the cell's aggregate to other cells' leaders, cells of cells for scale;
and as the crowd grows, everyone's every input collapsed into a few bytes a
second that the world reacts to slowly enough that the hops never force a
rollback.*

[0005](0005_open_world_netcode.md) and [regions.md](../regions.md) (branch
`brave-cray-d08zzb`, 2026-10-02) are the first half of this: the zones, with
the inequality that makes them exact. This note is the second half, and it
starts by saying what the first half already proved and what it left as "a
later problem, by agreement": **crowds**. A peer simulates everyone in its
regions, so fifty players in one square is fifty in everyone's rollback.

## 1 · What is settled, and the one inequality

The world is tiled by overlapping hexagons. A peer runs **full rollback** for
every region it is a member of -- it has every member's inputs -- and takes
its neighbours' **confirmed outcomes** `D` frames late. That is exact (every
peer computes the state of one imaginary global simulation, bit for bit)
whenever

> **overlap ≥ h = r + c·D**: the farthest anything acts in one go, plus the
> farthest anything can travel in the time an outcome takes to arrive.

This is a textbook result and the literature says exactly when it holds.
Chandy, Misra and Bryant (1979) call `h` the **lookahead**: a guaranteed
minimum delay before anything in one piece can affect another. Conservative
simulation with positive lookahead everywhere is deadlock-free and never
rolls back; its cost is the "null messages" (heartbeats) that say *nothing
happened through frame f*, and its performance collapses as the lookahead
goes to zero (Fujimoto, *Parallel and Distributed Simulation Systems*, 2000).
HLA (IEEE 1516) built a whole standard on it, with every federate declaring a
lookahead and regions in a "routing space" deciding who hears whom.

**The condition is the whole design.** One instantaneous influence across
the overlap -- a hitscan, a teleport, a world-wide rule -- is zero lookahead
and breaks the guarantee for the entire map. So the first job of this note
is an audit (§6), and the branch's own watchdog already found the shape of
the problem: it declares a speed of light of 60 m/s, and the Champion's
javelin flies at 70.

## 2 · The crowd is a second layer with its own lookahead

Here is the reframing. The regions treat every influence as **exact** and
pay for it with raw inputs and rollback. A crowd three hundred metres away
cannot touch you -- its reach is 24 m -- but you want to *know* it is there:
dust on a ridge, a herd stampeding away from it, a gate that has fallen. That
knowledge does not have to be exact to the frame. It has to be **the same on
every machine** and it has to be **there before the world reads it**.

So: a second layer, carrying **aggregates** rather than inputs, with its own
declared lookahead `L`, measured in frames and much larger than `D`.

- Every region `R` emits, once an **epoch** (`E` frames; a second), one
  fixed-size **aggregate** of its confirmed state at the epoch's frame: how
  many fighters, where their middle is, which way they are moving as a whole,
  how much blood is being spilt, which doors are down (§4).
- Aggregates travel by **cells and leaders** (§5), any number of hops, with
  whatever delay the hops cost: call the worst case `Δ`.
- The world keeps a **crowd field** `F`, and the simulation at frame `t` may
  read `F` **only through aggregates stamped at or before `t − L`**.

Then the two claims, and they are the same claim as §1 with different
numbers:

> **(S1) Rollback-free.** If `L ≥ Δ`, every aggregate the frame `t` reads
> had arrived before `t` was first simulated, so nothing about `F` is ever
> predicted and nothing about it is ever corrected. A peer may advance frame
> `t` once every stream it subscribes to has said what happened through
> `t − L` -- the ledger's gate, with one lookahead per stream instead of one
> `D` for all.
>
> **(S2) Exact.** `F(t)` is a pure function of the ordered aggregates with
> stamp `≤ t − L` and the frame. Two peers holding the same aggregates
> compute the same field, bit for bit, and it enters every region's checksum
> like anything else in the world.

And a third, which is the design consequence the person asked for:

> **(S3) Slow by construction.** The field is a quantity with its own
> **time constant** `τ`: a count that fills and drains, a centroid that
> drifts, a pressure that decays. If `τ ≥ L`, a value read `L` late differs
> from the current one by at most `L/τ` of its range, so the world's reaction
> to the far crowd is as it would be with the crowd in view -- just slow,
> the way a tide is slow. The error bound is the one co-simulation theory
> gives for coupling two systems through a signal exchanged every macro-step:
> safe when the signal varies slowly relative to the step.

The point of (S3) is that **"the world reacts slowly" is not a compromise
forced by the network, it is what makes the network free.** A quantity that
must react within a frame belongs in layer one and costs an input; a
quantity that may react over seconds belongs in layer two and costs bytes a
second. The design decision per thing in the world is which, and it is made
by writing down `τ`.

### What the far crowd is not

It is **not bodies.** Donnybrook (Bharambe et al., SIGCOMM 2008) ran 900
players of Quake III with no server by giving everyone five full-rate peers
and replacing the rest with "doppelgängers": local bots steered by 50 bytes
a second. That is the empirical ceiling for this kind of thing, and the
recipe for the byte budget -- but a doppelgänger is a *locally divergent*
bot, which is fine for a picture and fatal inside a region we checksum. The
far crowd here is a field, not a crowd of bots. Nobody far away is drawn as
a fighter; what is drawn is what the field says (§4), and when they come
near they come into a region, their inputs start flowing at `h + ε` before
they can touch anything, and they are exact from the first frame they
matter. There is no moment where a bot becomes a body.

## 3 · Scale, in numbers

Per peer, per frame, layer one costs what it always did: the inputs of
everyone in the one to three regions it stands in. Layer two costs the
aggregates it subscribes to: its regions' neighbours (six), its super-region's
neighbours (six), its super-super-region's (six)... Each aggregate is about
24 bytes an epoch. Three levels is 18 streams × 24 B/s: **under half a
kilobyte a second** for the entire rest of the world, however big it is.

What does not scale is a **hot region**: a hundred players in one hexagon is
a hundred in everyone's rollback there, and no hierarchy helps, because they
can all reach each other. The literature has one shipped answer -- EVE's
time dilation slows the region's clock -- and ours has to be the same kind:
a region past its population budget is a design problem (the hexagon is too
big, the arena draws too many), not a protocol one. The hexagon size is the
dial, and §7 says what to measure.

## 4 · The aggregate and the field

**The aggregate** of region `R` at epoch `e` (frame `e·E`), computed from
`R`'s confirmed state by any member -- so every leader of `R` produces
identical bytes, and a receiver can check two leaders against each other:

| Field | Bytes | Meaning |
| --- | --- | --- |
| region, epoch | 4 + 2 | which hexagon, which second |
| fighters | 2 | how many are standing in it |
| middle | 2 + 2 | their centroid, in decimetres from the hexagon's centre |
| drift | 1 + 1 | their mean velocity, in decimetres a second, clamped |
| blood | 2 | health lost in the epoch, in hundreds |
| down | 1 | fighters knocked down in the epoch |
| doors | 2 | a bit per waystone door or gate in the region, 1 = open |
| creatures | 1 | creatures alive in it |
| | **22** | |

**Aggregation is associative.** A super-region's aggregate is the sum of its
children's: counts add, centroids add weighted by count, drifts add weighted,
blood adds, doors OR. So "cells of cells" is just summing, deterministic, and
a level-`k` leader need hold nothing but its children's latest aggregates.
(RAND's multi-resolution modelling work -- Davis & Hillestad, 1993 -- is the
warning on the other direction: an aggregate cannot be disaggregated. We
never need to; a body re-enters exactness by entering a region.)

**The field** is what the world keeps, per region it subscribes to: the last
aggregate accepted for each stream and its stamp, plus the filtered
quantities the simulation actually reads:

- `crowd(R)`: fighters, filtered with time constant `τ_crowd` (3 s);
- `heat(R)`: blood per second, filtered with `τ_heat` (10 s), decaying;
- `flow(R)`: the drift, filtered;
- `doors(R)`: not filtered -- a door is a fact -- but read `L` late like
  everything else.

The filters are fixed-point first-order lags stepped once a frame from the
accepted aggregates: integer arithmetic, no allocation, in the snapshot, in
the checksum. `τ ≥ L` is asserted, not hoped.

**What reads it**, each with its `τ` written down -- the list is short on
purpose, and grows one entry at a time:

| Reader | Time constant | What it does |
| --- | --- | --- |
| A pack's mood | `τ_crowd` | A herd beside a region whose `crowd` is rising turns toward the ford: it hears the valley fill. Its alarm radius already reads the Pack's `seen`; this is one more sense, slow. |
| The valley's gates | none (`L` late) | A waystone door in `R` is open for everyone the frame `R`'s aggregate says so, `L` late. Two parties a kilometre apart agree on the door within a second and a half, and nobody rolls back to open it. |
| The sky and the ground | `τ_heat` | Dust on a ridge over a region with `crowd`; a glow where `heat` is high. Drawn, not played: the renderer reads the field. |
| The hunt's "every creature down" | none | Per region, never world-wide (0005's item 6). |

## 5 · Cells and leaders

**A cell is a region's membership.** The peers who are members of region `R`
already have lines to each other (layer one needs them: full rollback within
`R`), and membership is a function of confirmed positions. So the cell needs
no protocol to form: every member knows who is in it, and they all agree,
because it is in the world.

**Leaders need no election.** The `k` leaders of `R` for epoch `e` are the
`k` members with the lowest `hash(id, R, e)`. Every member computes the same
list from the same confirmed world. A leader that drops out of the match is
gone from the world (`World::away`, 2026-10-14) on the same frame on every
peer, and so gone from the list on every peer on the same frame. The hash
rotates the job round the cell every epoch, which spreads the upload and
means no one peer is the relay for long. `k` is chosen the way Kademlia
chooses its bucket size: so that all `k` dropping inside an epoch is
negligible. Three.

**What a leader does.** Once an epoch, it computes `R`'s aggregate, signs it
with its id, and sends it to the leaders of every neighbouring region it
knows a line to, and to the leaders of its **super-region** (the hexagon of
hexagons `R` sits in: seven regions, radius one). A super-region's leaders
are chosen from the leaders of its live child regions by the same hash; they
sum what they are sent and send the sum up and out. Depth is `log₇` of the
number of live regions. A receiver accepts an aggregate for `(R, e)` when
**two leaders agree** on it, or one leader's stands unchallenged for an epoch;
a disagreement is reported and the stream marked suspect, which is the NEO
rule (GauthierDickey et al., 2004) -- a move counts when a majority saw it --
applied to a stream instead of a move. It does not stop collusion; nothing
cheap does (Corman et al., 2006; Webb's RACS), and layer two is where a lie
costs least: a forged aggregate moves dust on a ridge.

**Heartbeats.** A leader with nothing to say still says "through epoch `e`".
A stream nobody has heard from for `L` frames **holds the gate**, exactly as
a silent neighbour region does today, and the HUD says which. A region that
has no leader because it has no members has no stream: a subscriber treats
an unlive region as empty, which the hierarchy tells it (the super-region's
aggregate counts it as zero).

**Lines.** Layer one's lines are a full mesh within a cell, as now. Layer two
needs a line from a leader to each neighbouring cell's leaders: `k × 6` lines
per leader, opened through the same board and the same offer/answer the
meeting uses (`meet.rs`), kept while the peer is a leader, and the rotation
keeps that short. A peer that is not a leader holds no layer-two lines at
all, which is what keeps a crowd's cost at the crowd's edge.

**Rollback window.** `L` covers the hops. One hop is an epoch plus a trip:
with `E = 60` frames and a 150 ms trip, three levels is about `3·(60 + 9) ≈
210` frames: three and a half seconds. The valley's gates open three and a
half seconds late for somebody a kilometre away; the herd turns over the
next ten. Nobody rolls back.

## 6 · The audit: everything with zero lookahead

The regions' guarantee is only as good as the declared reach and speed, and
the branch's own watchdog is the instrument. Read against `tuned.rs` today:

| Thing | Number | Verdict |
| --- | --- | --- |
| Fire bolt range | 24 m | the declared reach; fine |
| The Champion's javelin | **70 m/s** | outruns the declared 60: the watchdog would report it every throw. Declared at 70, `h = 24 + 70·8/60 = 33.3 m`, and `h + ε = 37.3 < 48 = S/2` still holds |
| The Reaver's dash, the Dual mage's Flurry dash, Thousand cuts | 50–52 m/s | inside 60 |
| Blinks: the Blood mage (6 m in 4 frames), the Dual mage's Phase, Shadowstep to a body | 6 m in a few frames | 90 m/s. Declared at 90, `h = 36 m` and `h + ε = 40 < 48`: the tuned grid holds with the declaration raised to the truth. What matters is that the declaration is **asserted against the Oven** (`tests/regions.rs`), not remembered |
| The valley's frame shift (`World::shift`) | the whole world moves | **not an influence**: a change of coordinates, which the grid must be laid in map coordinates to ignore (`World::map_origin`); the branch laid it in world coordinates, before there was a map |
| "The hunt is won when every creature is down" | world-wide | per region, or it ties every region to every other |
| A pack's brain reads every fighter's `seen` | the fight's room | per region once rooms are regions; today a room is inside one |
| `Travel`: a trip, a class change, a pause | world-wide, instant | layer one within a region; across the map these are not influences but *agreements*, and the rule is that they happen in the region that asked and reach the rest through the field (a door) or not at all (a pause is yours) |

The test that holds this: the watchdog's *outran* finding, run over every
pinned hunt and every class's sandbox with the tuned grid, must stay empty;
and `tests/regions.rs` asserts `reach + speed·D + overlap < S/2` and that the
declared speed is at least the fastest number in the Oven that moves a body.

## 7 · The experiments

No sockets, in this order, each a test that stays:

1. **The region experiment** (0005's, unchanged): one global simulation, two
   overlapping regions as separate simulations fed their own inputs and the
   other's outcomes `D` late; checksums equal every frame while bots cross;
   shrink the overlap until it breaks, where the inequality says.
2. **The field experiment**: the global simulation with the crowd field read
   at lookahead `L`, against a world fed the same aggregates `Δ ≤ L` late over
   a simulated chain of `k` hops with jitter; checksums equal every frame;
   then `Δ > L`, which must *hold the gate*, never diverge.
3. **The filter bound**: for each reader's `τ`, the field read `L` late
   against the field read now, over the pinned hunts: the difference under
   `L/τ` of range, as (S3) says.
4. **The population budget**: `sim/tests/budget.rs` with `N` fighters in one
   region, for the `N` at which a rollback burst no longer fits the frame.
   That number sets the hexagon.
5. **The cell experiment**: the loopback mesh with twelve peers in two cells,
   leaders by hash, one dropped mid-epoch; every survivor's field identical,
   and the gate held for exactly the frames the hop cost.

## 8 · What this does not solve, said plainly

- **A hot region.** See §3. The dial is the hexagon; the measurement is 7.4.
- **Joining a match under way.** A newcomer needs a region's snapshot from a
  member, and the branch's "snapshot fetch on entry" is still unbuilt. It is
  layer one's problem and the same size it was.
- **Cheating in the aggregate layer.** Two-leader agreement catches a lone
  liar; it does not catch two. The literature's answer is unaffiliated
  referees, which is a cost we do not pay for dust on a ridge.
- **Crossing a boundary between a dilated region and a normal one**, if time
  dilation is ever the answer to a hot region. Nobody has published it.

## Open questions

- **CROWD-1.** `E`: an epoch of one second, or a half? Halving it halves `L`
  and doubles the bytes; both are small. Play decides.
- **CROWD-2.** `k = 3` leaders, chosen by hash per epoch. Is a leader's
  rotation worth its line churn, or should a leader keep the job while it
  holds it?
- **CROWD-3.** The first reader. The herd hearing the valley fill is the
  prettiest; the gates are the most useful. Which goes in first decides what
  the first playtest is about.
- **CROWD-4.** What the field *draws*. Dust and glow are placeholders; the
  right picture of "a crowd over there" is an art question.
