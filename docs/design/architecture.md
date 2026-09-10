---
status: decided
decided: 2026-09-10
---

# Architecture

Rust. Three crates. The simulation is a pure function and knows nothing about
rendering or networking.

```
crates/sim    Deterministic simulation. Zero dependencies, no floating point.
crates/net    Rollback session. GGRS wired, plus a dependency-free local harness.
crates/game   Headless soak / CI determinism check.
crates/web    WebAssembly build and the browser sandbox.
```

## What rollback actually demands

Three requirements, and they decide the engine question — not performance, and not
framework bloat:

1. **Bit-exact determinism.** Same inputs produce the same state on every machine.
2. **Cheap snapshot and restore.** Rolling back means reloading frame *N* and
   re-simulating forward.
3. **Simulation fully decoupled from rendering.** Eight simulation steps may run inside
   one rendered frame.

The third has a consequence worth stating early: **animation state must live inside the
simulation.** Anything the engine animates independently will pop when a rollback occurs.
Actions are frame-data state machines, not blend trees. That is how fighting games are
built anyway, but it removes one of the main reasons to reach for a large engine.

## Why not Unreal or Unity

Not because they are slow. Unreal is not slow, and frame time will not be the problem.

Because their networking model is **client-server replication, not rollback**, and their
core assumptions run against all three requirements above. Rollback games have shipped on
Unreal, by fighting it. You would carry the full complexity of the engine while bypassing
the parts you are paying for — including the animation pipeline, which rollback partly
disqualifies.

The useful version of the "frameworks are not worth it" instinct is narrower than the
general claim:

- **For the simulation, write it yourself.** Engines actively obstruct determinism.
- **For the renderer, do not.** Skeletal animation, asset pipelines, shadows and platform
  windowing are enormous, solved, and not where this game's value is. Writing a renderer
  from scratch is how solo projects die.

## The layering pays for itself

Because `sim` is a plain library with no engine dependency:

- **The renderer is swappable**, including to Unreal later as a pure view over simulation
  state, if fidelity ever demands it. The project is not a bet on an engine.
- **Rollback correctness is testable with zero graphics.** Run the simulation twice with
  identical inputs and compare checksums; inject artificial rollbacks and compare against
  ground truth. This is the hardest thing to retrofit and the cheapest thing to build first.
- **Determinism has exactly one home** — one crate, no dependencies — rather than being a
  property you hope the engine preserves.

## Determinism, concretely

### No floating point in the simulation

IEEE 754 basic operations are reproducible if you control rounding and contraction, but
**transcendentals are not standardised across libm implementations** — and peers will be on
Windows x86 and Apple Silicon ARM simultaneously.

`sim` uses 16.16 signed fixed point (`Fx`) throughout, with its own square root and a
quarter-wave trigonometry table built at compile time. Range is +/-32768 at a resolution of
1/65536, which is ample for an arena measured in metres.

**Overflow saturates rather than wrapping.** Both are deterministic; saturating degrades
into a stuck character instead of a teleport, which is far easier to notice. Division by
zero saturates too — a panic during rollback re-simulation is worse than a wrong number.

A guard test walks `sim/src` and fails on any `f32`, `f64` or float literal outside a line
marked `RENDER-ONLY`. It is the cheapest defence available against cross-platform desync,
and it has been verified to fire.

### No physics engine

This falls out of the game design and it is what makes the whole approach tractable. Arena
combat with capsule hitboxes: no ragdolls, no stacked rigid bodies, no general constraint
solving. What is needed is capsule-versus-capsule, capsule-versus-static-geometry, and
kinematic movement — on the order of one to two thousand lines.

That matters because **most physics engines are not deterministic across platforms.** PhysX,
Havok and default-configuration Bullet all fail here; Rapier claims cross-platform
determinism behind a feature flag, with caveats. Writing this ourselves removes the single
largest determinism risk in the project and is *less* work than validating someone else's.

### Zero dependencies in `sim`

Load-bearing, not incidental. Every dependency is a determinism risk: unstable iteration
order, hidden floating point, platform-conditional code paths. Adding one requires a
determinism argument.

### Rules for simulation code

- Pure function of `(state, inputs)`. No clocks, no ambient randomness, no I/O.
- Any randomness is seeded from state and advanced deterministically.
- No iteration over unordered collections. Fixed arrays and sorted vectors only.
- Everything affecting gameplay lives in `World`. Everything else lives in the renderer.

## Networking

**Peer to peer, rollback from day one.** The prototype needs no server: create a room and
connect two peers directly by IP.

**GGRS is wired.** It is the Rust reimplementation of GGPO and handles input prediction,
rollback, and the periodic checksum exchange, which means **desync detection is largely
free**. `handle_requests` services its save / load / advance requests against the
simulation; that function is the entire integration.

GGRS requires its input type to be `serde`-serialisable, so the wire type `NetInput` lives
in `net` rather than putting a dependency on `sim` — whose zero-dependency status is a
guarantee, not an accident. The wire format is **two bytes per player per frame**.

**GGRS SyncTest is the strictest determinism check available.** It replays locally with
forced rollbacks and compares its own checksums across re-simulations. It runs in
`crates/net/tests/ggrs_synctest.rs` over 1200 frames and should be the first test to fail
if the simulation ever stops being a pure function.

`LocalSession` is a second, dependency-free harness that runs the same predict-and-rollback
loop against a simulated peer. It is kept because it is readable — when SyncTest reports a
desync, `LocalSession` is where you can watch one happen.

**Honest about "no server ever":** true for the prototype and for LAN. Direct-IP
connections between arbitrary home networks eventually need NAT traversal, which means a
small STUN or relay service. Not needed now; worth not being surprised by later.

## The browser sandbox

`crates/web` compiles the simulation to WebAssembly and `build-sandbox.sh` inlines it into
one self-contained HTML file — no server, no fetch, no CORS. Open the file.

It is a **training mode**, not a demo: hitbox and guard-arc overlays, a frame-data timeline
per player, pause and single-frame stepping, quarter-speed, dummy modes, and the live desync
checksum. Frame-stepping is the point — it is how frame data gets tuned.

The exported surface is a handful of C-ABI functions returning raw fixed point; JavaScript
divides by 65536 to draw. **No wasm-bindgen**, so the module has zero imports and comes to
about 21 KB. Float conversion happens on the far side of the boundary, which keeps the
simulation integer-only.

It runs the *real* simulation. Anything felt in the browser is what the game does.

## Current state

Everything below builds and passes today.

| Piece | State |
| --- | --- |
| `Fx` fixed point, vectors, trig | Working, tested |
| `Input` bitfield | Matches [controls.md](controls.md) |
| `World`, tick, hitboxes, guard, parry, hitstun | Bulwark stand-in: Bash 4/3/10, Slam 14/4/24 |
| GGRS integration + SyncTest | Passing over 1200 frames |
| `LocalSession` readable harness | Passing against ground truth |
| Test suites | 15 tests |
| Headless soak (`cargo run -p game`) | 3600 frames, 900 rollbacks, converges exactly |
| Browser sandbox | `./crates/web/build-sandbox.sh` |

The move set is a **Bulwark stand-in**, not a finished class: a fast poke, a committed slam,
and the guard/parry layer from [defense.md](defense.md). Both players use it, so a sandbox
match is a mirror. It exists to make the frame vocabulary concrete.

The point of the scaffold is that **the determinism harness existed before the gameplay
did**, so every class implemented from here is checked from its first commit.

## Next

1. **Finish the Bulwark**, then a second class. The Bellator exercises the form-swap window,
   which nothing else uses. Tune against the browser sandbox.
2. **Capsule-versus-static geometry.** Player bodies already separate; the arena is still a
   flat plane with walls.
3. **Direct-IP peer to peer.** GGRS is wired; what is missing is the socket and a room
   handshake.
4. **Bevy front end** for the 3D view. The browser sandbox covers frame-data work, so this
   is about the third-person camera and the visual reward of using abilities — which is a
   different job, and a later one.
