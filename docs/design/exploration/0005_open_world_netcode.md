---
status: exploration — regions built as the books every build keeps (regions.md); the look on the wire decided and built
started: 2026-10-02
---

# 0005 — Rollback in an open world, and what goes on the wire

Two threads that came out of one conversation. The first is how peer-to-peer
rollback could stretch over a large world with no server computing anything. It
moved into [regions.md](../regions.md) on 2026-10-02: every build now keeps
the region books, with simulated neighbours and a dev-mode watchdog, while the
arenas stay separate ([world.md](../world.md) §7). Peers exchanging region
checksums and outcomes is the step after. The second is what each peer sends each frame. That one is small, true for
the two-player game today, and decided: it moved into
[architecture.md](../architecture.md) §"The look on the wire".

## Part one — overlapping checksum regions

### The idea

The world is covered by **overlapping regions**, hexagons enlarged so that
neighbours overlap. Hexagons because at most three meet at any point, so a body
is always inside one, two or three regions and never four.

- **Each region has its own checksum**, computed by every peer standing in it.
  Not one checksum per session or per player.
- **A peer receives every input of everything in every region it stands in**,
  and runs those regions with full rollback.
- **A peer also receives the confirmed *outcomes* of the regions next to its
  own**, and trusts them: what happened, happened. Outcomes are needed because
  what happens next door may have been caused by somebody further away still,
  whom this peer will never hear from.
- **Crossing is monkey bars.** The neighbours' outcomes are already in hand, so
  the moment a peer steps into a new region it can start rolling in that
  region's inputs. As long as joining and leaving a region is fast and decided
  the same way on every machine, crossing never desyncs.

The overlap matters for play, not just bookkeeping. Anything you can touch has
to be simulated by a peer who has your inputs. If regions only touched edge to
edge, a monster a metre over the line would ignore you.

### Where it comes from

Nobody has shipped rollback this way, as far as this note knows. Two other
fields have the same idea, and their results say exactly when it is safe.

- **Domain decomposition with halo cells.** Big physics simulations (weather,
  fluids) are split into overlapping tiles, one per machine. Each tile is
  updated exactly, and the overlap (the *halo*) is refreshed from the
  neighbour's results. A halo *k* steps of influence deep lets a tile run *k*
  steps between refreshes with its middle still exactly right.
- **Conservative distributed simulation** (Chandy, Misra and Bryant, 1979).
  Separate simulators agree without deadlock as long as there is a guaranteed
  minimum delay, called *lookahead*, before anything in one piece can affect
  another. The overlap is that lookahead, measured in metres.

Hashlife, a fast algorithm for Conway's Game of Life, uses the same fact.
Nothing in Life moves faster than one cell per step, so the future of the
middle of a square depends only on the square. **Every simulation has a speed
of light**, and this design depends on it.

### The one inequality

Let

- **r** be the farthest anything reaches in one go: the longest ability range,
  a creature's perception or hearing radius, the length of the biggest body,
- **c** be the fastest anything travels: a dash, a bolt, knockback,
- **D** be how old a neighbour's confirmed outcome is by the time it arrives
  (their time to confirm plus the trip),
- **h = r + c·D**.

Then:

- **Neighbouring regions overlap by at least 2h.**
- **A peer also simulates a strip h wide outside its regions**, fed by the
  neighbours' outcomes. That strip is not in this region's checksum, because
  the neighbour checksums it. It sits well inside the neighbour's region.

With that, nothing outside a region can reach the part it checksums before a
newer outcome arrives to account for it. So every peer computing a region gets
the state of one imaginary global simulation, bit for bit. That includes peers
in an overlap computing it twice, once per region.

It also rules out the monster that ignores you. A monster in region S that you
can reach is within r of you, so you are inside S and sending S your inputs.
Anything you can touch is in a region you belong to.

**Rough numbers from `tuned.rs`:** the fire bolt reaches 24 m, the Reaver's dash
to her shadow moves at 50 m/s, and D is about 8 frames (133 ms). So
h ≈ 24 + 50 × 0.133 ≈ 31 m, and the overlap is about 62 m; call it 80 m with
margin. With regions a few hundred metres across, a peer is in one region most
of the time.

### What it costs and where it breaks

1. **Outcomes in the strip must be bit-exact state, not summaries.** The strip
   is simulated, so it needs everything: brain state, random-number state,
   animation clocks. Send only what changed, and only for bodies in the strip.
   A summary ("the monster died") is fine for regions further out, as display
   only and in no checksum.
2. **The longest instant reach sets the strip width for the whole world.**
   Perception and hearing radii, max-range ground casts, a body standing across
   a line. This wants a test like the aiming rule's, failing when an effect
   reaches further than the strip allows. Otherwise one generous perception
   radius makes every region more expensive.
3. **The rollback window grows to D.** The newest state that is exact is as old
   as the oldest outcome still awaited, so a peer re-runs D frames of
   everything in its regions and strips. The current budget is 8 frames, so a
   neighbour's outcome has to arrive within about 133 ms. The strip is the
   dial: wider tolerates slower neighbours and costs more simulation.
4. **Quiet regions must still report.** A peer cannot confirm a frame until
   every neighbour has said "nothing happened through frame f". These
   heartbeats are Chandy–Misra's *null messages*. Without them, an empty region
   stalls everything around it.
5. **Joining and leaving must be decided by the simulation.** "Is this body in
   region R on frame f" is a function of confirmed positions with a margin, so
   nobody flips in and out on a line. Fetching a region's snapshot has to
   start well before the overlap. A 50 m/s dash crosses 80 m in under two
   seconds.
6. **Nothing global inside the simulation.** One shared random-number stream,
   a world-wide count, or a world-wide rule would tie every region to every
   other. `World::hunt_won`'s "every creature down" is the example to avoid.
   Already local: random numbers are kept per body (`brain.rng`, `p.rng`) and
   the frame count is the same everywhere. New bodies would need ids made from
   (region, frame, who spawned it, count).
7. **Regions nobody stands in are computed by nobody.** The last peer out hands
   the region's state to plain storage (the broker could hold it without
   computing anything), or the region regenerates from the world seed.
8. **"Trust what happened" means trusting peers.** An outcome could count only
   once the region's members have signed the same checksum. A region holding
   one cheater can still lie about itself.
9. **Crowds.** A peer simulates everyone in its regions and strips, so fifty
   players in one square is fifty in everyone's rollback. A later problem, by
   agreement.

### What it would need

- **`World` per region**, with fixed slots for bodies joining and leaving, plus
  the strip of incoming outcomes. The 4 KiB cap restated per region.
- **Inputs keyed by a stable id**, not by a seat in `[Input; MAX_PLAYERS]`.
- **One checksum per region**, covering only the exact part.
- **A session layer GGRS cannot provide**: input feeds per region, outcome feeds
  per neighbour, null messages, snapshot fetch on entry, and stalling when an
  outcome is older than D. `LocalSession` is a better place to start than GGRS.
- **A speed-of-light test** that keeps r + c·D under the strip width as the Oven
  is tuned.

### The experiment that would settle it

No sockets needed:

1. Run one global simulation on a test map.
2. Run two overlapping regions as separate simulations, each fed only its own
   inputs plus the other's outcomes D frames late.
3. Assert that each region's checksum matches the global run on every frame
   while bots walk back and forth across the line.
4. Shrink the overlap until it breaks; the inequality should say where.

## Part two — the look on the wire

### What was wrong

`Input` carries the yaw and pitch at full precision every frame. GGRS guesses a
late remote input by repeating the last one it saw, and a mouse in a human hand
is almost never still to 1/65536 of a turn. So almost every frame of a remote
player's input was a wrong guess and a rollback. The rollbacks were not caused
by anything the player decided, only by their hand moving.

### What the simulation reads the look for

| Where | What for | When it has to be exact |
|---|---|---|
| `step`, `facing = look` (three copies, one per kind of ground) | which way the body faces | never: a degree off is invisible |
| `move_dir(input.aim_turns(), ..)`, about fifteen sites | which way `W` walks | never: the same |
| `aim::*_path` when a move begins | where an ability goes | on the frame it begins |
| a channel ("a channel is the aiming") | aim that keeps following the crosshair | every frame of the channel |
| the spear's vault, the Reaver's forward dodge, the blink | pitch or "is the crosshair on it" at a press | on the press |
| `pack.rs` and the Veilstalker (`in_view`, `led_look`) | is that on this fighter's screen | never: a screen is 90° wide |

The rule that follows: **the look only has to be exact on frames where it
decides something, and those are frames where the player is doing something
with a button.**

### The decision

The simulation does not change. What the sender puts in `Input::aim` and
`Input::pitch` changes, in one place, `sim::input::WireLook`:

- **Exact** on any frame where a button in `Input::PRESSES` goes down or comes
  up, on every frame one of them is held, and on every frame the local fighter
  is channelling.
- **Otherwise, the look last sent**, until the real look has moved more than
  `sim::input::LOOK_BAND` away from it. Then the real look is sent and becomes
  the new reference.

So the look on the wire is never more than the band away from the crosshair,
and it is exact whenever a button is involved. Between those, a hand resting on
the mouse sends the same value frame after frame, and repeating the last input
guesses it right.

This is "set state to", not "change state by": every value sent is a whole
look, so a lost or late packet heals itself on the next one that arrives.

The camera is unaffected. It is drawn from the local mouse (`view::camera`),
not from the `World`, so the screen still moves at full precision. Only what
crosses the wire, and what this peer's own simulation sees, is held.

The band is **1/1024 of a turn, about 0.35°**. That is the error a buffered
cast can have if the button was let go before the move came out and the mouse
kept moving: about 15 cm at the fire bolt's 24 m. Whether the band can grow is
a feel question to play, not to argue (see the feel log).

### Why not send world coordinates instead

The idea in the conversation was to send the resolved point a move goes to,
rather than the look that resolves to it. It carries the same information on
the same frames, so it saves no extra rollbacks. It would move the aiming
raycast out of the simulation into the sender, and the simulation would then
have to check every point sent (inside the range sphere, on the kind's rule)
or a modified client could send any point. What it would buy is *what you saw
is what you got*: the shot goes where your crosshair was in your predicted
world, even if a correction later moves a stone. That is worth having, and
what is built here is a strict first step toward it. The sparse look stays the
same, and the press frames would carry a point instead of a look.

### What it does not fix

- **A player turning fast.** A turn faster than the band per frame (about 21°
  a second) still changes the look every frame. Guessing the next look by
  continuing the current turning speed, instead of repeating the last value,
  would cover that. That is a change to GGRS's prediction, which GGRS does not
  expose, so it waits for the session layer of part one.
- **A cast that comes out of a buffer after the mouse moved.** It aims within
  the band, not exactly. Latching the look at the press inside the simulation
  would fix it, and would also change when a buffered cast reads the
  crosshair, which is a feel change, so it is not done here.
