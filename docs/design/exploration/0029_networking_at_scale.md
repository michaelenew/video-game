---
status: exploration — diagnosis measured, plan proposed, NET-1 to NET-4 open
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

## 8 · Open

- **NET-1** Is there a server of ours -- a relay that forwards, stores and
  settles, simulating nothing -- or is the world peer-to-peer to the end?
- **NET-2** The target: how many players in one world, and how many in one
  fight? (Tens in a fight is a design problem -- a hundred people who can
  all hit each other cannot be cheap however it is networked; EVE slows the
  clock in such a place. Hundreds in a world is the networking problem this
  note is about.)
- **NET-3** The See tier's distance and rate: how far should another player
  be drawn, and how smoothly?
- **NET-4** Is the region size right for the world as it is now (96 m
  hexagons, a 46 m horizon set by the 24 m fire bolt and a 90 m/s blink)?
  Every long reach or fast move widens the horizon for the whole world.
