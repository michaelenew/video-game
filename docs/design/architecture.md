---
status: decided
decided: 2026-09-10
---

# Architecture

Rust. Three crates. The simulation is a pure function and knows nothing about
rendering or networking.

```
crates/sim    Deterministic simulation. Zero dependencies, no floating point.
crates/net    Rollback session (GGRS) + the headless soak binary.
crates/view   Presentation logic: interpolation, camera framing, posing. No engine.
crates/game   Bevy app. Rendering only -- it owns no gameplay state.
crates/web    WebAssembly build and the browser frame-data tool.
```

`view` sitting between the simulation and Bevy is what keeps the engine choice
reversible, and it means the parts that must be *correct* rather than pretty --
interpolation, camera framing, posing -- are unit tested without a window.

## Toolchain

**Rust 1.85 or newer.** The floor is Bevy 0.16's declared MSRV, not our use of
edition 2024 -- dropping the workspace to edition 2021 would not lower it.

`rust-toolchain.toml` pins the channel and pulls in the wasm target, so
`rustup update` is all a stale install needs. The symptom of being behind is a
manifest parse error naming `edition2024`, which is misleading: the edition is
the first thing Cargo trips over, not the actual constraint.

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

## Animation under rollback

**Pose is a pure function of simulation state.**

```text
pose = f(action, frames_into_action, speed, grounded, sim_frame)
```

No accumulated animation time, no independently ticking player. Rollback
re-simulates past frames, so anything animating on its own clock pops and slides
every time a rollback happens. `sim_frame` is part of the snapshot, so driving
cyclic motion (walk cycles, idle breathing) from it stays deterministic.

**This is the reason primitive standins come before glTF.** Writing transforms by
hand forces the pure-function shape. Reaching for Bevy's `AnimationPlayer` the way
the documentation shows -- play a clip, let it advance -- builds exactly the thing
rollback breaks. When skeletal animation lands, the rule is that the player is
never allowed to advance itself; its time is set explicitly from simulation state
every frame.

One nuance keeps this from being painful: **gameplay-relevant pose must be pure;
cosmetic smoothing may be renderer-local and is allowed to pop.** A rollback is
one to eight frames -- 16 to 130 ms. A cross-fade that hiccups across that window
is imperceptible. Blend state does not belong in the snapshot.

A test replays a frame the way a rollback would and asserts the pose is identical.

## The animation factory

`crates/anim` is an **offline** tool. It never runs in the game.

Hand-keyed poses read as a slideshow, because the parts that make motion look
alive -- an arm trailing the shoulder it hangs from, a swing carrying past its
target and settling back -- are precisely the parts that are miserable to key by
hand. They are, however, exactly what a spring-damper produces for free.

So an animation is authored as **a handful of poses and a looseness setting**,
and the solver fills in everything between them:

```text
recipe (keys + looseness)  --[springs, offline]-->  a table of poses, one per frame
```

`cargo run -p anim --bin bake` runs the solver and writes
`crates/view/src/baked.rs`. **Playback is then an array index by frame**, which
is why this changes nothing about rollback: `pose = f(state)` still holds, and a
rollback re-indexes the same table with the same frame and gets the same pose.

Generated Rust source rather than a data file, on purpose: no loader, no asset
path, no runtime parsing, and a diff shows exactly what changed when an
animation is retuned.

### Looseness is expressed in frames, not in spring frequency

Each part is described by **lag** (how many frames it runs behind the keys) and
**ring** (how far it overshoots on arrival, where `1.0` never overshoots).

This is not cosmetic API taste. A damped spring chasing a moving target settles
into a steady lag of about `2·ring/frequency`. The first pass at this file
expressed weight as a *low frequency*, which does not mean heavy -- it means
late. The Bulwark's slam has a 14-frame startup and its silhouette had barely
moved by frame 5, so there was nothing on screen for the opponent to read while
they were supposed to be deciding whether to block. **Weight must read as
follow-through, never as delay.** Splitting lag from ring makes that mistake
hard to repeat, and two tests pin it.

## Render interpolation

The simulation is locked to 60 Hz; displays are not. `view::interp` keeps the
previous and current snapshots and draws at `lerp(prev, cur, alpha)`.

Without it a perfect 60 Hz simulation still looks choppy on a 144 Hz monitor, and
it reads to the player as "the game feels bad" rather than as a missing feature.

Only continuous quantities are blended. The action and its frame counter come
from the newer snapshot -- there is no meaningful halfway point between startup
frame 3 and startup frame 4. Movement beyond a threshold snaps rather than
slides, because blinks and swaps are real moves in several kits.

## The camera

Third-person, behind the fighter, and the player owns it. **Aim is the camera**: where you
look is where you are pointed, and where you are pointed is where your attacks go.

It used to be an auto-framing rig that sat perpendicular to the line between the two
fighters, keeping both in profile the way a 3D fighting game does. That was a good idea
under a control scheme where nothing needed mouse-look — but it was a *consequence* of that
scheme rather than an independent choice, and once attacks are aimed the premise is gone.
It also only ever worked for 1v1; "frame both fighters" means nothing in coop against four
monsters. It has been deleted rather than kept behind a flag. A lock-on camera is a
controller feature and can come back when there is a controller.

Three rules hold it together:

**Yaw and pitch are never smoothed.** They are the mouse. Any filtering between the hand and
the crosshair is felt immediately even when it cannot be named. Only the focus *position* is
smoothed, so the camera glides over the character's footsteps instead of jittering with them.

**The eye sits off one shoulder; the aim point does not.** At melee range an opponent stands
directly behind your own fighter from a centred camera, and raising the camera does not fix
it — a body is wider than a sightline. So the eye slides sideways while the point at the
centre of the screen stays on the look axis, straight ahead of the fighter. The offset costs
nothing in aiming precision; it only moves the character out of the way. It is folded in
*before* the geometry check, not after — a camera slid sideways after being cleared has not
been cleared. The test asserts
the *aim point*, not the camera's own axis, because the two are deliberately different.

**The camera is not in the snapshot.** Aim reaches the simulation as input, so peers agree on
gameplay without the camera ever being rolled back.

### The floor is not an obstacle to dodge, it is a surface to rest on

Look up far enough and the camera arm wants to swing below the ground. There are two ways to
refuse that, and only one of them feels like anything.

Shortening the arm is the wrong one. It hauls the camera in toward the fighter's head while
its height never changes, so the rig reads as a **pole of fixed length** with the camera
sliding down it — which is exactly what it felt like when the clamp compared the arm against
the ground without counting the eye lift, and so fired about three metres early. A ten-degree
glance upward pulled the camera halfway in.

The right one is to clamp the eye's *height* and leave the arm alone. The camera descends,
settles onto the ground, and rides along it at full distance. It still closes on the fighter
as you keep looking up — but only once it is actually on the ground with nowhere left to go,
which is the point at which closing in is the only thing left to do.

### The camera pulls in rather than turning away

Level geometry must never get between the camera and the fighter. The arm marches back from
the focus and stops short of anything solid. It does not swing around the obstruction,
because the angle is the player's and taking the mouse away to dodge a wall is worse than
briefly sitting close to the character's back. The minimum arm length is deliberately tiny:
an arm that refuses to shorten will happily hold the camera *inside* a wall when a fighter
stands against one. The floor gets its own clamp, since it is a plane the simulation handles
rather than an entry in `SOLIDS`.

### The crosshair is not painted at screen centre

It is projected from the point the fighter is pointed at, one aim-length ahead — the same
distance the camera aims at, so the two coincide exactly when facing matches aim.

That sounds like a long way round for "draw a cross in the middle", and it is the whole
point. Facing locks when a move starts and lags while guarding, so for a meaningful fraction
of every match the camera is pointed somewhere the attack will not go. A reticle nailed to
the centre would be confidently wrong precisely when the player needs it to be right. This
one drifts off centre instead, and dims while you are committed.

A test checks it against `CameraRig` itself rather than against a restatement of the same
arithmetic: if the rig's aim point and the crosshair's ever diverge, a still crosshair stops
meaning anything.

## Aim is an input, not a camera read

This is the part that makes camera-relative movement compatible with rollback.

Movement and attacks resolve relative to where the player looks. That makes the look angle
gameplay state, and every peer must agree on it bit-for-bit or the simulations diverge. There
were two ways to get that:

1. Make the camera deterministic and part of the snapshot.
2. Send the look angle as input.

The second is much cheaper and does not constrain the camera at all. Aim travels in the input
packet next to the buttons, so it is predicted and corrected by the machinery that already
exists, and the renderer stays free to do whatever it likes.

It is quantised to **1/65536 of a turn, as an integer**. Integer turns rather than radians for
three reasons: every `u16` is a valid angle, adding past a full turn wraps exactly and for
free, and because `Fx` is 16.16 the raw `u16` *is* the fractional part of a turn already — so
converting to the simulation's angle type is a widening cast with no rounding for two machines
to disagree about. Trigonometry goes through the existing `sin_turns` / `cos_turns` lookup
table, which is already determinism-safe.

**Pitch is deliberately not sent.** It moves the camera and changes nothing in the simulation,
so it stays renderer-local. Splitting the two along "does this decide anything?" keeps the
wire format honest: four bytes per player per frame.

## Arena geometry

`sim::arena` is a fixed array of axis-aligned boxes resolved along the axis of
least penetration. Not a physics engine, and not a placeholder for one: capsule
versus box and capsule versus capsule is all an arena fighter needs.

Geometry is a constant rather than part of `World` -- it never changes during a
match, so it does not belong in the rollback snapshot. The renderer builds its
meshes from the same array, so if you can see it, you collide with it.

The walls are deliberately low. Tall ones read as a box and put geometry between
the camera and the fight, which is exactly the thing an auto-framing camera
cannot solve.

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
| Test suites | 87 tests |
| Headless soak (`cargo run -p game`) | 3600 frames, 900 rollbacks, converges exactly |
| Browser frame-data tool | `./crates/web/build-sandbox.sh` |
| **Bevy prototype** | **`cargo run -p game`** — 3D arena, standins, HUD, debug overlay, local 2P |
| Bulwark kit | Bash, Slam, Guard, parry, Grapple, shield throw/recall/leap |
| Universal movement | Jump, dodge with i-frames, crouch that ducks overheads |
| **Mouse look** | **Third-person camera, camera-relative movement, aimed attacks** |
| Crosshair | Projected from facing, so it is honest during a committed move |
| Settings | `~/.config/arena/settings.conf`, sensitivity on `-` / `=` |
| Round flow | Knockout, round wins, reset |
| **Peer to peer** | **`game --port N --peer ADDR`** — verified over real UDP |
| Headless screenshots | `./scripts/screenshot.sh` — Xvfb + lavapipe, no GPU needed |
| **All six classes** | **`game --p1 bellator --p2 elementalist`**, or Tab to cycle |
| Feel harness | `crates/sim/src/tuning.rs`, `tests/feel.rs`, [feel-log.md](feel-log.md) |
| Frame table | `cargo run -p sim --bin frametable` — every move, on-block and on-hit |
| **Animation factory** | **`cargo run -p anim --bin bake`** — F2 toggles baked playback |
| Repeatable capture | `SHOT_FRAME=N` stops on an exact frame; `BAKED_ANIM=0` for procedural poses |

Each class has its **class mechanic** and **three exemplar moves** -- a poke, a committed
move, and a special -- not a finished kit. Enough to find out how the classes feel against
each other, which is the only question a dummy cannot answer.

The point of the scaffold is that **the determinism harness existed before the gameplay
did**, so every class implemented from here is checked from its first commit.

## Next

1. **Play it against a person.** Everything below is downstream of that. The open
   questions in [feel-log.md](feel-log.md) are written so an answer can be recorded
   against them rather than lost.
2. **More clips.** Five baked animations cover the shared vocabulary; per-class moves
   still fall back to procedural poses.
3. **glTF standins.** The pose function's signature does not change, only what it returns.
   Kenney and Quaternius have CC0 rigged low-poly characters.
4. **NAT traversal**, when the game leaves the LAN.
