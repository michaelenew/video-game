# Regions — the world's checksum logic, run by every build

**Status: decided 2026-10-02, built the same day on a branch; brought onto
main 2026-10-14** with the valley, four fighters and everything since, and
with the experiment the note asked for built and passing
(`crates/regions/tests/session.rs`). The thinking behind it is
[exploration/0005](exploration/0005_open_world_netcode.md); what comes after
it -- crowds, cells and leaders, the slow aggregate -- is
[exploration/0018](exploration/0018_crowds.md). This document is what is built
and what it must keep true.

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
| **speed** `c` | the fastest anything travels | 90 m/s: a blink is six metres in four frames; the Champion's javelin flies at 70, the dashes at 50 |
| **delay** `D` | frames a neighbour's outcome may be old (at most 30, the tape's length less one) | 8 (`net::MAX_ROLLBACK_FRAMES`) |
| **overlap** `ε` | how far a checksum zone reaches past its hexagon | 4 m |
| **hexagon size** `S` | circumradius of a hexagon | 96 m |
| **neighbour lag** | frames a simulated neighbour's heartbeat is behind | 2 |

Derived (`sim::region::Grid`):

- **horizon** `h = r + c·D/60`: how far influence can travel before a newer
  outcome arrives. 36 m as tuned.
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

**On the valley's map the grid is laid in map coordinates.** The world runs
in the coordinates of the place it is in and moves to the next place's when
the fighters cross ([atlas.md](atlas.md)); every position a region is asked
about has `World::map_origin` added first, in the checksum
(`World::each_body`) and in the watchdog alike, so a change of frame is not
a body moving and the grid holds still under it.

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
  which regions are *live* (someone is in them), and -- for **this peer's
  own regions and their neighbours only** (`Ledger::books`, since
  2026-10-14) -- each one's checksum. A region further off is somebody
  else's to sum up, so the cost of the books is the cost of one
  neighbourhood however many are playing: at most three regions of one's
  own and the ring round them, sixteen at the outside. The ledger is kept
  for a seat (`Ledger::new(world, me, watch)`, `Ledger::sit` when a match
  seats the machine), and *live* is still every region anybody is in,
  because the aggregates summed from it are fed to the world, and every
  peer of a match has to feed the same.
- **Heartbeats.** Every region in the books has a *heard-through* frame:
  the newest frame it has said "this is what happened" about.
  - One of my own, or a neighbour somebody stands in, is heard through the
    frame just confirmed: its inputs are in hand.
  - **Any other neighbour is simulated.** It has no peer behind it in a
    one-arena game, so the ledger plays one: a clock that ticks once per wall
    tick, heard through *clock minus neighbour lag*. It ticks whether or not
    the world advanced, the way a real peer keeps going while you wait.
- **The gate.** `Ledger::may_advance(next)` is true when every region in
  the books is heard through at least `next − D`. Both drivers ask it
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
4. **Locality**, the real test. One region of the books per frame, in turn: take the
   tape's world from `D` frames ago, re-run the `D` frames with the inputs of
   every fighter who was *not* in the region replaced by nothing pressed, and
   compare the region's checksum with the real one. If they differ, something
   outside the region reached its checksum zone faster than `r` and `c` say it
   can. The report names the region, the frame, and which bodies differ.
   Windows containing a trip or a round reset are skipped.
5. **Stalls**: a frame the gate held, with the region that held it.

Every finding is a line in a fixed ring of the latest sixteen, with a running
count per kind. The HUD shows the grid, the books (own regions starred) and their checksums,
the heartbeats, the counts and the latest findings, along the bottom of the
screen. Each finding is printed the frame it is found: to the terminal on the
desktop, to the browser console as a warning (`platform::say`).

**Getting the locality test to bite.** With the tuned grid, an arena 28 m
across sits wholly inside every region near it, so no fighter is ever outside
a region and the test only proves the re-run is deterministic. To exercise it,
lower *reach* and *speed* in the Oven (F7). Regions shrink, fighters end up
outside each other's regions, and every report is an effect that outruns the
numbers you declared. Writing those down is how the real knobs get set.

## The session layer, as an experiment (`regions::session`, 2026-10-14)

What GGRS cannot provide, in the shape the wire will carry: a [`Peer`]
runs **one region's world** with only its **members'** inputs (everybody
else pressing nothing), and takes its neighbours' **outcomes** -- the
fighters standing in a neighbour's checksum zone at a frame, exactly -- by
**rolling back to that frame, overwriting those fighters, and playing
forward again** with the inputs it recorded. So the strip between a zone and
the membership radius is driven by the neighbours, `D` frames late, and the
zone is exact at every frame an outcome has arrived for.

`tests/session.rs` is 0005's experiment: one global run, three regions
across the proving ground on 8 m hexagons, two fighters walking through all
three and meeting on a boundary to shove. **Every zone agrees with the
truth at every frame, 690 zone-frames of it; and with the reach declared
zero, it does not.** Two things it taught, both now in the code:

- **A member sends inputs from one horizon further out than it is exact**
  (`Peer::admits`): the outcome that puts a newcomer's state right arrives
  `D` late and is played forward with the inputs recorded since, so those
  inputs have to be in hand `D` frames before the body could first touch
  the zone. Inputs flow from `membership + h`; exactness needs `membership`.
- **The overlap repairs as far as it is wide.** An outcome *overwrites* the
  bodies in the strip, members included, so a lie about reach smaller than
  `ε` is corrected before anyone checksums it. The dishonest run had to thin
  `ε` to a decimetre to break. A margin, not a licence: the inequality is
  what is held to, and the watchdog's *outran* is what reports a lie.

What an outcome does not yet carry: effects, bolts, creatures. A fighter is
the one body whose future depends on an input the peer lacks; the rest are
functions of fighters and frames, and carrying them is the next step, with
the stable ids 0005's list asks for.

## The crowd field, and the cells that carry it (2026-10-14)

Layer two, as [exploration/0018](exploration/0018_crowds.md) argues it, and
as built the same day:

- **`sim::crowd`.** Once an epoch (`Crowds, epoch`: 60 frames) a region's
  confirmed state is summed into a 22-byte `Aggregate` (fighters, their
  middle and drift, how hurt, creatures, doors), associative under
  `Aggregate::sum`. The world keeps a `Field` of the streams it follows --
  the neighbours of every present fighter's region, decided in the step so
  every peer follows the same -- each a last accepted aggregate and the slow
  quantities filtered from it (`crowd`, `heat`, `flow`, time constants
  `Crowds, crowd and flow` 240 frames and `heat` 600). **Aggregates are fed,
  not kept**: `World::advance_with(inputs, due)` takes the frame's due
  aggregates beside its inputs, from the ledger's tape of them, and an
  aggregate of epoch `e` is due at frame `e·E + L` (`Crowds, lookahead`: 210
  frames) and never earlier. The field is in the checksum once it has heard
  anything, and a world that has heard nothing hashes as it did.
- **The ledger** keeps the aggregates offered to it until they are due,
  feeds them, speaks for a followed stream nobody is behind (an aggregate of
  nothing, on its simulated clock), sums every live region's aggregate at
  each epoch's frame (what a leader posts), and **holds the gate** for a
  stream with a peer behind it (`Ledger::peer_behind`) until the aggregate
  the next frame reads has come.
- **The first reader**: a calm pack **hears the valley fill next door**
  (`Crowds, a pack hears`: three fighters by the field, in the regions round
  its home, never its own) and takes it as an alarm. Inert in every pinned
  fight, which has nobody next door.
- **`net::cells`**: a cell is a region's membership, read off the world; its
  three leaders this epoch are the lowest `hash(id, region, epoch)`, so no
  election and no message decides it, and a member gone is gone from the
  list on the same frame everywhere. Super-regions are hexagons of seven on
  the lattice `(2, 1), (-1, 3)`, every region within a step of exactly one
  centre; their aggregate is the sum of their children's.
- **`net::crowd`**: a leader's aggregate rides the match's own lines, tagged
  so GGRS would drop it, into a post box the mesh shares with the game; a
  receiver believes it when two leaders agree or one stands unchallenged for
  an epoch, and believes neither of two at odds. The game's online driver
  posts what it leads to the leaders next door and offers what it collects.

**The experiments** (`crates/regions/tests/crowd.rs`, `net/tests/meet.rs`):
two peers handed the same aggregates at different times -- one at the
epoch, one after a random trip up to the whole lookahead -- agree bit for
bit at every frame, and the herd on both hears the crowd on the same frame;
a stream with a silent peer behind it holds the gate at exactly the frame
that would read it, and the aggregate releases it; `τ ≥ L` holds at the
Oven and a value read `L` late is within `L/τ` of range; four on the mesh
post as leaders and collect in agreement with no disputes, following the
same streams.

**What is not yet true**: the *game's* session is still GGRS, so every
peer in it simulates the whole world, a region with anybody in it is live
on every ledger, and the wire's aggregates duplicate the ledger's own. The
transport, the agreement and the gate are exercised; they *matter* once a
peer runs only its regions, which is the session below -- built on the
loopback lines, not yet what the game plays on.

## The session layer, on the wire (`net::region`, 2026-10-14)

`regions::session` with lines under it: a [`Session`] is one peer of the
open world, holding one world, a tape, and a line to every peer it can
reach. GGRS's rules where they fit, and its own where GGRS's players being
fixed for the match does not:

- **Inputs go to the members of my regions.** Every frame, one packet to
  every line: the last eight frames of my inputs (a lost packet costs
  nothing until eight go in a row) and the regions I am a member of this
  frame -- my home hexagon and whichever neighbours are within
  `membership + h` of my feet, which is what the no-sockets experiment
  found a member has to send from. 84 bytes. A member's missing input is
  predicted as its last, and corrected by rollback when it comes.
- **Membership is declared, not inferred, and remembered by frame.** A peer
  knows exactly where its own fighter stands, so its packet says which
  regions it is in, and everybody takes its word; frame `f` needs the
  inputs of every seat whose declaration *as of `f`* shares a region with
  mine as of `f`. That history is asked again on every replay, because a
  declaration arrives after its frame was first played -- the first frame
  of every pairing, every time -- and an answer frozen at first play left
  that frame without its partner's input for good: a one-frame lag inside a
  zone where only inputs count, which the first run of the test found.
- **Confirmed** is the newest frame everybody who has ever spoken has
  declared for, and every member of it has sent an input for. The peer runs
  at most eight frames past it (`WINDOW`), as GGRS does.
- **Outcomes go next door.** Every four confirmed frames, for each region I
  have been in for a while, the fighters in its checksum zone at that
  frame, to every peer that declared it or a neighbour of it and has *not*
  been in it for a while (`SETTLE`, sixteen frames). A receiver rolls back
  to that frame, puts those fighters where I say, and plays forward again
  -- `regions::session::Peer::accept`, on real packets. A fighter is 544
  bytes, so this is the expensive packet, and it goes only across a
  boundary and only to those who cannot know better.

  The settling window is the second thing the test found. A new member's
  state at the frame it joined was a neighbour's guess between two
  outcomes, up to three frames stale; without a few more outcomes *after*
  it declared, the old members' word never reaches it again and the whole
  chain it then shoves is a frame behind forever.
- **The books go with the outcomes** (`Packet::Books`). At the same
  confirmed frames, a peer sends the checksums of the regions it is an old
  member of -- the only ones it sums up -- to every peer that declared one
  of them or a neighbour of one. A receiver holds them against its own
  only where its own are exact: a frame it has *confirmed* (a frame still
  predicted is nobody's books; the first run compared one and cried desync
  at the frame everybody let go of the stick), and in it a region it is an
  old member of, or a neighbour's whose outcome for that very frame it
  took. Anything else is not its to judge and is dropped. So a peer sums
  up its own regions and its neighbours', and nothing further, in this
  layer and the ledger alike; and a bit flipped in one game is found by
  its neighbours and itself within a few frames (`net/tests/region.rs`).
- **A newcomer asks for the world** (`Packet::Want`) and is sent it at the
  asked peer's confirmed frame, in 1 KB chunks (`Packet::Snapshot`; a world
  is 19,000 bytes); it keeps asking a different peer every ten frames for
  the chunks still missing, which the asked peer serves off its tape, and
  the inputs since are still in everybody's redundancy window. The
  declarations it heard while waiting are kept, so the frames it then plays
  know who was in what.

The state crossing in an outcome or a snapshot is the struct's own bytes
(`net::raw`), sound only between two copies of the same build -- which the
meeting's terms already demand of two peers before a frame is played. It is
the one `unsafe` in the networking, in one file, and a stopgap for a `sim`
that cannot take a serialiser.

**The experiment** (`crates/net/tests/region.rs`): four peers on loopback
lines losing one packet in ten, four fighters walking the line through
three 8 m hexagons from both ends, pairing up on both boundaries and
shoving into one chain in the middle. Every peer's regions' zones equal
the global truth's at every frame the tape holds, with no stalls and a
handful of rollbacks each, and every book held against a neighbour's
agrees; a fifth peer that joins at frame 40 from a snapshot, through the
same loss, is exact from then on, and the old hands are exact with it
among them; and one fighter a point of health short in one game only is
a disagreement on all four within three frames.

**What is not yet true.** A disagreement is counted and the first kept,
not yet acted on (which side is right is the owner's question, exploration
0018 §7b). Every peer still *advances* the whole world --
the regions it is not in with nobody pressing anything -- so the saving is
in what crosses the wire and in what is exact, not yet in the frame's cost;
an outcome carries fighters only (effects, bolts and creatures are the next
thing to carry, with the stable ids 0005 asks for); a peer that falls
silent holds confirmation for everybody who ever heard it, with no timeout
yet; the lines are a full mesh handed in, not found (the meeting still
makes a GGRS session); and the game does not play on it. The game plays
four on GGRS with a whole-world checksum, which is right for four, and this
is the machinery for the day it is more.

## Wire

GGRS compares whole-world checksums between the peers it has, and a region
checksum is a pure function of the world, so two worlds that agree have
regions that agree. What the game plays on today changes nothing on the
wire. The region session above has a wire of its own, exercised on the
loopback lines; putting it under the game is the step after this one.

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
- `net/tests/region.rs`: the session on the wire, above -- four peers
  through loss against the truth with their books agreeing, a late joiner,
  a desync found by the books, and the size of an input packet.
