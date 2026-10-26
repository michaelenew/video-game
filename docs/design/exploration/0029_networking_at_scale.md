---
status: exploration — diagnosis measured, plan proposed; NET-1 and NET-2 answered 2026-10-26 (peer-to-peer only; tens in a world, eight to a fight); step 1, the load harness, next
started: 2026-10-25
---

# 0029 — Networking at the scale of a world, not a fighting game

The owner (2026-10-25), after the tutorial halls became a shared space:

> We are clearly headed down a path of not treating the networking like an
> MMO will need. Distance means lower granularity of sharing, we will
> eventually need many fights active at once, and we need to figure that out
> sooner rather than later. I imagine our networking is super inefficient
> right now because it assumes a fighting game style small arena.

This note measures what the netcode costs today, says which of those costs
grow with the number of players (and how fast), and proposes the order to
change it in. Most of the hard thinking is already in the repository --
[0005](0005_open_world_netcode.md), [0018](0018_crowds.md) and
[regions.md](../regions.md) -- and was built as far as test harnesses. What
has not happened is the game moving onto it, and one whole tier of the
design (seeing people you cannot touch) was never written.

## 1 · What the game plays on today

**Lockstep rollback, the fighting-game kind** (GGRS, "Good Game Rollback
System"). Every machine runs the *entire* world, every frame, from everybody's
inputs. When a friend's input is late, each machine guesses it (the last one
repeated) and plays on; when the real input arrives and the guess was wrong,
it loads the world from before the mistake and plays every frame since
again. Every machine sends its input to every other machine, every frame, and
a frame is final only once *everybody's* input for it has arrived.

That is exactly right for two to eight people in one arena, and it is what
makes the fighting feel sharp: there is no server between your button and
your fighter.

## 2 · Measured (2026-10-25, the owner's 44-minute tape, two players)

| What | Measured | Where from |
| --- | --- | --- |
| A frame of the valley, two fighters | 70–80 µs typical, ~160 µs at the 99th percentile | the tape replayed, timed per frame by place |
| A frame of an arena or a hall | 40–70 µs | same |
| A frame that rebuilds the world (a trip) | 1–3 ms | same: every spike over 1 ms was one |
| Each fighter more, walking about | +5–10 µs | Hearth and the proving ground, one to four fighters |
| A creature | +~20 µs | Ridgeback, Gnawers, Hornback hunts |
| The link | 13–15 kbps each way, one friend | the tape's `net` lines |
| Rollbacks | 6,424 in 44 min (2.4 a second), 1.7 frames each | same |
| Waiting on the friend | 100 ticks in 44 min | same |
| The snapshot | 22,000 of its 22,528-byte cap | `size_of::<World>()` |

**With two players nothing is slow.** A twelve-frame rollback of the valley
is about a millisecond. The owner's instinct is right about the shape, not
about today's numbers: the costs that matter are the ones that grow.

## 3 · What grows, and how fast

Write **N** for the players in one session. Every line below is a property
of lockstep, not of this code:

1. **Everyone waits on the slowest connection anywhere.** A frame is final
   when all N inputs are in. One player on bad hotel wifi at the far end of
   the valley stalls everybody. Grows with N (the worst of N links).
2. **Everyone sends to everyone.** Each machine sends N−1 streams: about
   1.7 KB/s each as measured, so ~80 KB/s up at N = 50 and ~170 KB/s at
   N = 100, and N·(N−1) connections in all. Home upload links and NAT
   traversal both give out well before that.
3. **A stranger's guess rolls back your world.** Rollbacks come from any
   remote input guessed wrong: measured 2.4 a second for one friend, so
   about 2.4·(N−1) for N, i.e. one nearly every frame at N = 25 -- each one
   re-running the *whole* world, every body in it, for a stranger a
   kilometre away who could not have touched you.
4. **Every machine simulates every body.** Cheap per body (§2) but it adds:
   N fighters plus every live fight, times (1 + rollback depth).
5. **The snapshot is built for a match.** Four seats (`MAX_PLAYERS`), two
   creature slots and one hunt's lore and pack, one coordinate frame for
   the world. That is why there can be one live fight at a time: the world
   *cannot describe* two.

Items 1–3 are what kill it, long before CPU does. They all have the same
cure: **only people who can affect each other should have to agree, and only
they should wait on each other.**

## 4 · Three tiers by distance

The first principle: **how exactly two players must agree falls with how
soon one could affect the other.** Somebody within reach of a blow must
agree with you to the bit, every frame. Somebody across a valley only has to
be drawn roughly in the right place. Somebody ten kilometres off only has to
be counted.

| Tier | Distance (as tuned) | What is shared | Exact? | Rolled back? | Cost |
| --- | --- | --- | --- | --- | --- |
| **Touch** | within the region horizon, ~46 m | every input, every frame | bit for bit | yes, only by people in it | the fighting-game cost, for a handful |
| **See** | out to the draw distance, a few hundred m | each body's position and pose, a few times a second, fewer the further | no | never | bytes per body per second |
| **Know** | anywhere | a region's *aggregate* once a second: how many, where, how hurt, which doors | yes, but slow | never (the lookahead covers the hops) | ~0.5 KB/s for the whole rest of the world |

- **Touch** is [regions.md](../regions.md): hexagonal regions 96 m across,
  overlapping, each with its own checksum; a peer gets every input of the
  regions it stands in and runs those with rollback, and takes its
  neighbours' confirmed outcomes. Proven exact by the no-sockets experiment
  (`regions/tests/session.rs`) and, with lines, four peers through packet
  loss (`net/tests/region.rs`). The underlying result is old and solid:
  conservative distributed simulation (Chandy–Misra–Bryant, 1979) -- if
  nothing can cross the overlap faster than an outcome arrives, the pieces
  agree without ever being told.
- **Know** is [0018](0018_crowds.md), built as far as the snapshot: the crowd
  field, fed by aggregates with a lookahead of 210 frames.
- **See is missing.** Today a body is either in your exact simulation or not
  drawn at all. An MMO needs the middle: the player standing on the next
  ridge, drawn walking, at a cost of bytes. This is what every shipped
  server-based game does under the name *interest management* or
  *relevancy*: Tribes' networking model (Frohnmayer & Gift, 2000) sends each
  object at a priority that falls with distance, inside a per-client
  bandwidth budget, newest state only, never resent. Here it would be one
  more packet kind on the region session, drawn by the renderer and never
  read by the simulation, so it can be as lossy as it likes and can never
  desync anything.

**The handover between tiers is the delicate part.** Someone you can see
walks towards you; before they could touch you they must be in your exact
simulation. 0005 already sized this: inputs start flowing a horizon *before*
contact (`Peer::admits`), and the snapshot of the body comes with them, so
there is never a frame where a ghost becomes a body by guesswork.

## 5 · Many fights at once

With the tiers in place, **a fight is just the people within touch of it.**
Two hunts a kilometre apart share no region, so nobody in one ever waits on,
rolls back for, or simulates the other: each is a small fighting game of its
own, and the rest of the world sees it as ghosts and knows it as an
aggregate ("blood is being spilt over there").

What has to change for the world to *describe* several: a hunt's state --
its creatures, its pack, its lore, its objectives -- moves from being the
world's (`World::monsters`, `World::lore`, `World::pack`) to being a
region's, with **stable ids** for bodies instead of fixed slots (0005 §"What
it would need", item 2, and regions.md's own "what is not yet true"). That
is the one structural refactor this whole note waits on, and it is why the
"one live hunt at a time" rule exists.

## 6 · The question this note cannot answer: is there a server?

The repository's stance so far is **no server of ours**: peers meet through
free public brokers and talk directly (WebRTC, UDP), and 0018 is written
under that rule. That is coherent, and Donnybrook (Bharambe et al., 2008)
showed 900 players of Quake III with no server. But everything after "the
game plays on regions" gets much harder without one:

- **Who holds a region nobody stands in?** Somebody has to keep the world
  where everyone has left -- a creature mid-roam, a door broken. 0005 §7:
  plain storage, or regrown from the seed.
- **Who is right when two peers disagree?** The books find a disagreement
  within frames (`net/tests/region.rs`) but cannot say who is lying.
- **How do a hundred browsers find each other?** Today a room is a link and
  a full mesh.

A server does not have to *simulate* anything for these: a relay that
forwards packets, holds empty regions' state and settles disputes by
majority keeps the fighting peer-to-peer and fixes all three. That is a
decision about money and hosting, so it is the owner's (NET-1).

## 7 · The order to do it in

Each step is useful on its own and can be judged by a number.

1. **A load harness first.** N bot peers on loopback lines walking the
   valley, no rendering, reporting per peer: bytes a second each way,
   rollbacks a second, frames waited, simulation time per frame. Run it on
   GGRS and on `net::region`. Every later step is judged against these
   curves, and the budget tests get a line per tier.
2. **Bodies with stable ids; state by region.** Fighters in a pool keyed by
   id rather than `[Player; 4]`; a hunt's state owned by its region; the
   snapshot a set of region chunks, so a rollback restores and re-runs only
   the regions it is about. This removes the four-seat cap and the
   one-hunt rule together. The biggest step, and the one everything after
   waits on.
3. **The game plays on the region session**, not GGRS: inputs only to
   peers in your regions, waiting only on them, rolling back only for them.
   The harness's curves for items 1–3 of §3 should go flat in N.
4. **The See tier**: ghosts of the bodies beyond your regions, by distance
   and priority inside a byte budget, interpolated, drawn and never
   simulated.
5. **Many fights**: hunts woken in any region, any number at once.
6. **Whatever NET-1 decides**: a relay for empty regions, disputes and
   discovery -- or the peer-to-peer answers to each.

Two players today would notice nothing until step 3, and then only a little
less waiting; the gain is in what becomes possible, and in numbers the
harness can show.

## 8 · Answered (2026-10-26)

- **NET-1: peer-to-peer only.** No server of ours, not even a relay. Empty
  regions, disputes and discovery each need a peer-to-peer answer (§6), and
  step 6 is those answers.
- **NET-2: tens of players in a world, eight in a fight.** Eight is the size
  every per-region budget is set for: a region's rollback must fit eight
  fighters and its fight, and the harness measures up to it.
- **Next: the load harness** (§7, step 1).

## 9 · The load harness, first run (2026-10-26)

`cargo run --release -p net --bin load` (`crates/net/src/bin/load.rs`): bot
peers in the valley's town on simulated lines -- each packet held three
frames each way (a 100 ms ping), one in thirty lost -- a bot's input held a
quarter to two-thirds of a second and then changed. Fighters **together**
(3 m apart) or **apart** (300 m, different regions). Per peer, 900 frames:

| netcode | layout | n | up KB/s | rollbacks/s | tick p50 / p99 µs |
| --- | --- | --- | --- | --- | --- |
| GGRS | together | 2 / 3 / 4 | 3.5 / 7.7 / 12.2 | 1.1 / 2.9 / 4.8 | ~100 / ~400 |
| GGRS | apart | 2 / 3 / 4 | 3.5 / 7.7 / 12.2 | 1.1 / 2.9 / 4.8 | ~180 / ~900 |
| regions | together | 2 / 3 / 4 | 5.7 / 11.5 / 17.4 | 2.2 / 4.3 / 6.3 | ~40 / ~200 |
| regions | apart | 2 / 3 / 4 | 13.1 / 34.4 / 30.3 | 9.5 / 16.4 / 9.5 | ~130 / ~1,100 |

What it says:

- **GGRS is exactly §3**: about 4 KB/s up per other player, and rollbacks
  in proportion to them, whether they are 3 m away or 300.
- **The region session, as built, does not yet use distance at all -- and
  apart is worse.** By packet kind, apart at n = 3: inputs 10.2 KB/s (the
  same as together: **every peer's inputs go to every line, every frame**),
  outcomes 22.9 KB/s (**outcomes reach peers 300 m off**, two hexagons
  away, who are no neighbour of anything they stand in), books 1.3. And
  taking an outcome is a rollback, which is why apart rolls back more.
- Four is all the snapshot seats (`MAX_PLAYERS`), so the curves stop there;
  that is step 2's to lift.

So before the game can move onto it, the region session needs three
changes, each measured by this table: inputs only to peers who share a
region (and a slow heartbeat to the rest, which is also where the See
tier's ghost goes); outcomes only to a peer standing next to the region;
and an outcome that changes nothing taken without a rollback.

## 10 · Distance in the region session (2026-10-26)

The first two of §9's three changes, built in `net::region`:

- **A peer too far to matter hears a heartbeat.** Every `HEARTBEAT` (6)
  frames: where I am, no inputs, and a promise that it holds through
  `PROMISE` (12) frames ahead (`Packet::Inputs::until`). The receiver counts
  my declaration as good that far (`Known::through`) when it confirms a
  frame. **Who is far is the speed of light** (`region::far`): the peer
  stands within membership of every region it declared, so it is at least
  the farthest of those gaps less membership from me; two bodies can share
  a region only within two memberships and a hexagon's width; and over the
  promise and a trip (`LATENCY`, 18 frames) each of us closes at most the
  grid's speed. Near -- or never heard from -- hears every input, every
  frame, as before.
- **An outcome goes only to a peer whose regions its fighters could
  reach**: within membership and a horizon of one of them. Before, a peer
  two hexagons off had them at 544 bytes a fighter four times a second.

**With the tuned grid the far line is about 420 m** (two memberships of
46 m and two hexagon radii of 96 m are 284 m; the promise and a trip at
90 m/s both ways add the rest). That is the regions' geometry, not the
heartbeat's: two bodies 284 m apart can stand in one hexagon's
membership. NET-4 is where it would change.

`net/tests/region.rs` (`far_peers_beat_and_are_exact_when_they_meet`):
two peers 80 m apart on the small test grid, far past its line; 507 bytes
in the first second where every input would have been about 5 KB, and
every zone either is in agrees with the truth through the walk, the
meeting and the shove (3,796 zone-frames).

The harness, apart now on a 600 m square:

| netcode | layout | n | up KB/s | rollbacks/s | tick p50 / p99 µs |
| --- | --- | --- | --- | --- | --- |
| GGRS | apart | 2 / 3 / 4 | 3.5 / 7.7 / 12.2 | 1.1 / 2.9 / 4.8 | ~100 / ~400 |
| regions | apart | 2 / 3 / 4 | **0.25 / 0.51 / 0.76** | **0 / 0 / 0** | ~40 / ~70 |
| regions | together | 2 / 3 / 4 | 5.8 / 11.6 / 17.5 | 2.2 / 4.3 / 6.3 | ~40 / ~200 |

**Apart, a player costs a quarter of a kilobyte a second to everybody
else, and nobody rolls back for anybody.** Together the region session
still sends more than GGRS (84 bytes a frame against GGRS's compressed
inputs): an input repeated eight times for redundancy, uncompressed --
GGRS sends each input as its difference from the last, and the same
would halve it or better. That, and §9's third change (an outcome that
changes nothing taken without a rollback, which the session already does
-- what rolled back apart was outcomes that were not needed), are next.

## 11 · Eight fighter slots (2026-10-26, step 2 begun)

`MAX_PLAYERS` is eight, NET-2's fight. The pools fighters fill -- 128
effects, eight bolts, eight gusts, four blasts of shards -- stay at four
fighters' worth (`state::POOLED_FOR`), each giving up its oldest when full,
rather than doubling with the slots: the snapshot is 25,520 bytes and its
cap 26 KiB (architecture.md). The masks that count fighters were already as
wide as the seats; the creatures' per-fighter memory was already laid out
by the seat count. Every pinned hunt's *hash* moved -- eight slots hashed
where four were -- and every one was compared frame by frame against the
code before (fighters, creature, phase, effects) and was identical. A
replay writes only as many seats a line as the tape used. `allies` counts
only fighters in the fight.

The harness to eight, 600 frames, apart on a 450 m grid:

| n | GGRS up KB/s | GGRS rollbacks/s | regions apart up KB/s | regions apart rollbacks/s | regions apart tick p50 µs |
| --- | --- | --- | --- | --- | --- |
| 2 | 3.5 | 1.2 | 1.0 | 0 | 38 |
| 4 | 12.4 | 4.8 | 1.5 | 0 | 218 |
| 8 | 37.7 | 9.1 | 3.7 | 0 | 253 |

(450 m is just over the far line, so some pairs are still near.) What is
left growing is the tick: every peer still simulates every fighter in the
world -- step 3's local work.

## 12 · Groups, not one world (2026-10-26, the owner's call)

Many fights at once had two roads. **One world of regions**: a pool of hunt
slots in the snapshot, each peer working only its regions -- no seams, but
the creature and hunt code assumes one hunt in about 21,000 lines
(`self.lore` alone 547 times). **Groups**: the players near enough to
touch share a world, which is exactly the world a match has always been --
eight fighters, one hunt, one frame of reference -- run in lockstep among
its members; far groups share nothing, so a fight each runs at once with
no new simulation of fights at all. The cost is the seam: groups **merge**
when they come near, at an agreed frame, and **split** when they part;
until a merge, two groups near each other see one another but cannot
touch. The owner chose groups.

**G1, the simulation's half** (`sim::group`, `tests/groups.rs`):

- **A split needs nothing new**: each side goes on with a copy of the
  world, the other side's fighters fed `Travel::LEFT` from then on -- what
  a seat whose player has gone has always been.
- **A merge** (`group::merge`) makes two groups' worlds one: the base is
  the one in a hunt if either is; the other's fighters go into its free
  slots in its coordinates (`World::shift` by the difference of the two
  frames' origins), with their effects, bolts, gusts, shards and ground,
  and what they say of each other (who holds, whose bleed, whose curse)
  said of their new slots. **Two hunts are refused**, as are two places
  with no map in common and more fighters than slots.
- **`World::ids`**: who each slot is, beyond the world -- what a merge
  keeps a fighter by when it moves them.

What the tests hold: a group of two in the town and one on the High Plane,
300 m up, half a kilometre off and fifty frames ahead, merge with everybody
exactly where they were on the map; and **the moved fighters then play on
for 300 frames exactly as their own world would have played them**, frame
difference and all. Split four into two pairs, play them apart 200 frames,
merge again: everybody back where their own group had them, by id.

**What the first version of the split test found**, and why groups are
right: in *one* world, two fighters a hundred metres off walked into a
creature's range, which started its hunt and moved the frame of reference
for everybody -- and a fighter who was nowhere near moved a centimetre
differently for it. A world is everybody's; a group's is only its own.

Next, **G2**: the network half -- each peer's group, a lockstep session
among its members, heartbeats (where each player is) to everybody else,
and the merge and split agreed on the wire.

## 13 · Groups on the wire (2026-10-26, G2)

`net::group::Session` is one peer's group. Within it, nothing new: every
member's input every frame, the last eight resent in every packet, a wrong
guess corrected by playing again -- what a match has always been. To
everybody outside it a peer sends only a **heartbeat** every six frames:
where its fighter is on the map, its group's leader (the lowest id in it),
the members, and whether the group is hunting. A fight far off is nobody
else's to wait on, send to or roll back for.

- **A split needs no word.** Every 60 frames, at a frame every member has
  confirmed, each member cuts the members' fighters into groups by 200 m
  steps; whoever is not joined to it is fed `Travel::LEFT` from the next
  frame. Every member cuts the same world the same way, so all of them
  agree without a packet. A group in a hunt never splits.
- **A merge is agreed**, when two groups' fighters come within 120 m (the
  gap between the two distances keeps a pair standing at the edge from
  merging and splitting every second). The base is the group in a hunt,
  else the one with the lower leader; its leader calls the other's. That
  leader names a frame, in the inputs its members already exchange; at it
  they stop, and it sends its world there to the base's members in
  kilobyte chunks, resent until it arrives. The base's leader names a frame
  of its own; at it every base member makes the same merged world
  (`sim::group::merge`), plays on from it, and the leader sends it to the
  newcomers, who take it and play on as members.
- **Frames are the base's** from the merge on: the newcomers drop what
  they had heard and pressed by their old count, which would otherwise be
  read as the new one.

What `tests/groups.rs` (in `net`) holds: four peers on lines that lose one
packet in twenty, two in the town and two on the High Plane half a
kilometre off. Apart, each pair agrees within itself, and each is a group
of its own. The plane's pair fast-travels to a cairn by the town:
**all four are one group 27 frames later**, holding one world. The pair
travels to the furthest cairn: **two groups again**, each agreeing within
itself, with no stall anywhere.

**What the test found**: the first cairn it
met at was beside a gnawer den, whose pack comes out about five hundred
frames after anybody arrives -- for a group alone just the same. The
merged group was then in a hunt, and fast travel is refused in a hunt
(`World::peaceful`), so the pair that pressed for the far cairn never
left. That is the existing rule doing its job -- a fight is everybody's in
it until it ends -- and it means a group in a hunt cannot be pulled apart
by a trip either. The test meets at the nearest cairn that stays quiet as
long as it plays there.

Next, **G3**: the game plays on group sessions instead of GGRS, and the
load harness gets a groups mode to measure them beside the other two.

## 14 · Open

- **NET-3** The See tier's distance and rate: how far should another player
  be drawn, and how smoothly?
- **NET-4** Is the region size right for the world as it is now (96 m
  hexagons, a 46 m horizon set by the 24 m fire bolt and a 90 m/s blink)?
  Every long reach or fast move widens the horizon for the whole world.
