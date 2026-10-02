# Regions — the world's checksum logic, run by every build

**Status: decided 2026-10-02, built the same day.** The thinking behind it is
[exploration/0005](exploration/0005_open_world_netcode.md). This document is
what is built and what it must keep true.

## Why every build runs it now

The open world will be divided into overlapping regions, each with its own
checksum, and peers will only talk to the peers near them. That logic is
complicated, and the time to find out it is wrong is while the game is one
arena and two fighters. So **single player, the dev harness, the browser and
the two-peer game all run the same region logic on every frame**. Where there
is nobody to hear from, neighbouring regions are simulated locally and send
heartbeats like any other peer. In dev mode a watchdog checks the logic's
promises every frame and reports, on screen, the moment one is broken.

Nothing here changes what happens in a fight. The simulation (`World::advance`)
does not read any of it. The region layer watches confirmed frames, keeps the
books, and decides one thing: whether the next frame may go ahead.

## The shape

The floor is tiled by **pointy-topped hexagons** of circumradius `S` (the Oven's
*Regions, hexagon size*). A hexagon's id is its axial coordinate `(q, r)`. One
hexagon's corner sits at the origin, the middle of every arena, so the three
regions meeting there all cover the fight. That is the busiest spot the grid
has, on purpose.

From the Oven:

| Knob | Meaning | Tuned |
|---|---|---|
| **reach** `r` | the farthest anything reaches in one go | 24 m (the fire bolt) |
| **speed** `c` | the fastest anything travels | 60 m/s (the Reaver's dash is 50) |
| **delay** `D` | frames a neighbour's outcome may be old (at most 30, the tape's length less one) | 8 (`net::MAX_ROLLBACK_FRAMES`) |
| **overlap** `ε` | how far a checksum zone reaches past its hexagon | 4 m |
| **hexagon size** `S` | circumradius of a hexagon | 96 m |
| **neighbour lag** | frames a simulated neighbour's heartbeat is behind | 2 |

Derived (`sim::region::Grid`):

- **horizon** `h = r + c·D/60`: how far influence can travel before a newer
  outcome arrives. 32 m as tuned.
- **A body's regions** are the hexagons within `h + ε` of it. Its peer sends
  inputs to all of them and runs all of them with full rollback.
- **A body's checksum zones** are the hexagons within `ε` of it. Neighbouring
  zones overlap by `2ε`, so a body near a line counts toward both checksums.
- **The geometry is valid only if `h + ε < S/2`.** That is what keeps a body in
  at most three regions: the middle of an edge is `S/2` from the two hexagons
  at its ends. Breaking it is the first thing the watchdog reports.

Why a checksum zone is the hexagon plus `ε`, and the regions plus `h` beyond
that: a peer running region R has the inputs of everything within `h + ε` of
R's hexagon. So everything within `ε` of it can be computed exactly, because
nothing it lacks can reach that far within `D` frames. The strip beyond is
simulated but in no checksum.

## A region checksum

`World::region_checksum(grid, id)` hashes the frame, the id, and every body
whose position is in the zone, each by its index and its own state hash: the
fighters, the creatures (by their root position), the critters, bolts, debris,
gusts and effects. The per-body hashes are the same bytes `World::checksum`
writes. Both are built from one set of `hash_*` helpers, so they cannot drift
apart, and `World::checksum` is bit for bit what it was before.

**Global state belongs to the fight's home region**, the one whose hexagon
holds the origin: the phase, the arena byte, the pack brain and the lore. That
is the honest answer for one arena, and it is item 6 of the exploration's list
(nothing global) written down as a known debt. The watchdog sees its
consequence: if a body outside the home region changes the phase, the home
region's check fails.

## The ledger (`crates/regions`)

A crate with no dependencies beyond `sim`, so it builds for the window and the
canvas alike, and allocates nothing after it is made. It holds:

- **The tape.** The last `D + 1` confirmed frames, each the world before the
  frame and the inputs that advanced it. The local driver writes it on every
  advance. Online, `net::handle_requests` writes every advance, a re-simulated
  frame overwriting its prediction, and a frame counts once GGRS confirms it.
- **The books.** For each confirmed frame: which regions each body is in,
  which regions are *live* (someone is in them), and every live region's
  checksum.
- **Heartbeats.** Every live region and every neighbour of one has a
  *heard-through* frame: the newest frame it has said "this is what happened"
  about.
  - A live region a fighter stands in is heard through the frame just
    confirmed: its inputs are in hand.
  - **Any other neighbour is simulated.** It has no peer behind it in a
    one-arena game, so the ledger plays one: a clock that ticks once per wall
    tick, heard through *clock minus neighbour lag*. It ticks whether or not
    the world advanced, the way a real peer keeps going while you wait.
- **The gate.** `Ledger::may_advance(next)` is true when every neighbour of
  every live region is heard through at least `next − D`. Both drivers ask it
  before a frame. With the neighbour lag under `D` it is always true. Raise the
  lag past `D` in the Oven and the game holds, then runs `D` frames behind the
  slowest neighbour, paced by its heartbeats. That is what it will do in the
  open world.

## The watchdog (dev mode)

On with `--dev` (or `?dev`), off otherwise; the ledger's books and gate run
either way. Each confirmed frame it checks:

1. **Geometry**: `h + ε < S/2`.
2. **Membership**: every body is in one to three regions and one to three
   checksum zones.
3. **Speed of light**: no body moved further in one frame than `c` allows.
   Frames that rebuild the world (a trip, a round reset) are skipped. A blink
   is not: a body crossing many metres in one frame is exactly the kind of
   thing the speed knob has to know about, and the report names the body and
   the distance.
4. **Locality**, the real test. One live region per frame, in turn: take the
   tape's world from `D` frames ago, re-run the `D` frames with the inputs of
   every fighter who was *not* in the region replaced by nothing pressed, and
   compare the region's checksum with the real one. If they differ, something
   outside the region reached its checksum zone faster than `r` and `c` say it
   can. The report names the region, the frame, and which bodies differ.
   Windows containing a trip or a round reset are skipped.
5. **Stalls**: a frame the gate held, with the region that held it.

Every finding is a line in a fixed ring of the latest sixteen, with a running
count per kind. The HUD shows the grid, the live regions and their checksums,
the heartbeats, the counts and the latest findings, along the bottom of the
screen. Each finding is printed the frame it is found: to the terminal on the
desktop, to the browser console as a warning (`platform::say`).

**Getting the locality test to bite.** With the tuned grid, an arena 28 m
across sits wholly inside every region near it, so no fighter is ever outside
a region and the test only proves the re-run is deterministic. To exercise it,
lower *reach* and *speed* in the Oven (F7). Regions shrink, fighters end up
outside each other's regions, and every report is an effect that outruns the
numbers you declared. Writing those down is how the real knobs get set.

## Wire

Nothing changes on the wire in this step. GGRS already compares whole-world
checksums between the two peers, and a region checksum is a pure function of
the world, so two worlds that agree have regions that agree. Exchanging region
checksums and outcomes between peers needs messages of our own, which is the
session layer GGRS cannot provide. It is the next step, not this one.

## Tests

What the watchdog said when it was built: a 1500-frame Ridgeback hunt with the
tuned grid, 1,493 region re-runs, nothing found. The same hunt with reach 2 m,
speed 5 m/s and 16 m hexagons: regions outrun, and fighters, bolts and gusts
too fast. SyncTest (forced rollbacks every frame) and the localhost peers over
real UDP keep the books and find nothing.

- `sim/tests/regions.rs`: geometry (every point of a large patch is in 1–3
  regions and 1–3 zones while `h + ε < S/2`; ids round-trip; the origin is a
  corner), region checksums (stable, unchanged by bodies outside, changed by
  bodies inside, global state only in the home region), and `World::checksum`
  unchanged (the pinned hunts already cover it).
- `regions/tests/gate.rs`: heartbeats and the gate (a neighbour lag five
  frames past `D` holds for five ticks, then paces one frame a tick).
- `regions/tests/watch.rs`: the watchdog, quiet on a bot hunt with the tuned
  grid, and reporting *outran* and *too fast* when the speed of light is
  declared smaller than what the fight does.
- `net/tests/ggrs_synctest.rs` and `p2p_localhost`: the books kept through
  rollbacks, and over real UDP, with nothing but *too fast* allowed.
