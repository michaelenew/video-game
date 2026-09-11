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

Field of view and distance are **settings, not constants** — 58 degrees vertical from seven
metres is a starting point, not an answer. Bevy's default projection is 45 degrees, which is
a portrait lens pointed at an arena you are meant to be moving around inside.

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

### Pitch is two different cameras, blended

Looking down and looking up are not the same problem, and one arm length cannot serve both.

**Down is close.** Looking at the ground means looking at the ground *near you* — that is what
the gesture means. With a seven-metre arm the ray reaches the floor before it reaches the
fighter, so steep angles put the middle of the screen behind your own heels. So the arm
shortens and the eye drops toward the fighter's head as pitch goes down: by the bottom of the
range the reticle sits just in front of their feet, and the fighter has risen to near the
centre of the frame. That is the whole of "mouse down means bring the aim closer".

**Up runs out of third person.** The ordinary answer — walk the arm down toward the ground
behind the fighter — reads well for the first forty degrees and then stops working: the arm is
on the floor, the body is between you and the sky, and every bump in the terrain shoves the
view. Past `sky_start` the rig climbs into the fighter's own eyes, the floor and geometry
clamps fade out, and the body stops being drawn. You are panning the sky, which is what you
were trying to do. Hidden rather than faded, because these are untextured primitives and a
half-transparent one reads as a rendering fault rather than as your own body.

Both blends are smoothstepped rather than linear. The blend swaps the whole rig over, and a
linear handover makes the camera visibly change its mind at exactly the angles where the player
is holding the mouse still.

### Why the camera is not in the Oven

Every magnitude in the *simulation* is an Oven knob and a test enforces it. The camera is
deliberately outside that rule, and the reason is the checksum: tuning values are folded into
`World::checksum()` so mistuned peers desync loudly, which is right for anything that decides
what happens and wrong for anything that decides what you see. Two people playing each other
must be able to run different fields of view and different camera distances without the match
falling apart. Those three already live in `settings.conf` per player; the rig's own numbers
sit beside them in `RigConfig`.

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

## Persistent effects

Until the classes were filled in, every attack was an instant: a hitbox that existed for a few
frames and was gone. Two moves are built on the opposite idea — a fire pillar that grows where
it was planted, and a drain field that punishes standing still. Those have to outlive the move
that made them, and that makes them simulation state.

`crates/sim/src/effects.rs` holds them in a **fixed array, not a `Vec`**. Effects are
snapshotted and restored on every rollback, so a heap allocation per re-simulated frame would
be the single most expensive thing in the tick. The cap also turns "what happens when you
spam it" into a decision rather than an emergent property: the oldest one goes.

Three rules follow from rollback and are worth stating because they are not obvious:

- **Growth is a function of `age`, never accumulated.** A pillar's radius is computed from how
  many frames it has existed, so replaying frame 90 twice produces the same pillar both times.
  An effect that grew by adding to itself each tick would drift with every rollback.
- **Effects act after both fighters have stepped.** Standing in a fire pillar costs you whether
  you walked in or were knocked in. Checking before movement would let someone walk through a
  pillar untouched on the frame they entered it.
- **An effect never hurts its owner.** A fire pillar you cannot stand beside is a fire pillar
  you cannot use.
- **Eviction reaches your own effects only.** A full board drops *your* oldest. Reaching across
  owners would let one fighter delete the other's setup by holding a button.

### The fire pillar is two volumes, not one

A wide base and a taller, only slightly wider column above it. They are different threats: the
base is what catches someone walking past, the column is what stops them jumping over. Growing
them together would collapse two decisions into one, and the pillar would end up either
useless against a jump or unavoidable on the ground. The base spreads **out** as it ages; the
column reaches **up**.

### Structures are deliberately *not* effects

The Elementalist's structures are a cap-of-three resource owned by her mechanic, with no clock.
They were briefly put in the effects array, which looked like reuse and was not. Two things
came with it, and both were bugs:

A **lifetime they never had.** The fire pillar is gated on having a structure out, so ten
seconds after raising one the class's own special stopped working, silently, with no way to
tell why.

A **second list to disagree with.** Being in the array meant the mechanic's slots and the array
both claimed to know what was standing, which needed a reconciliation pass every frame to keep
them honest — about eighty lines existing purely to paper over a decision that should not have
been made.

The rule that came out of it: **reuse storage only when the lifetimes match.** An effect expires
on a clock; a structure is spent by raising a fourth. Those are different rules, so they are
different homes. Structures render from a pool driven straight off the mechanic, which is the
only thing that owns them.

### The mechanic fires on the press, and the edge lives in the snapshot

Held, the mechanic button used to re-fire every frame, and every class was wrong in its own way:
the Bellator's form became a function of how many frames you happened to hold it, the Reaver's
shadow toggled itself back off, the Bulwark's shield was pinned mid-throw and never planted, and
the Elementalist spent all three structures on one spot in three frames.

"Was it down last frame" is a **single bool on the fighter**, not a renderer-side `just_pressed`.
Rollback re-runs these frames, so the edge has to be recomputed from the snapshot; a flag kept
outside the simulation would report a press on every re-simulated frame and none of the above
would be fixed. It costs one bit in the checksum.

The attack buttons deliberately still repeat while held. Mashing a poke is normal for the genre
and the move's own recovery frames are the rate limit. The mechanic has no recovery frames,
which is why it needed the edge and they do not.

### A grab is a state, not a stun

`Action::Held` pins the victim to their captor at arm's length and moves them with him. It is
not hitstun with a longer timer: hitstun is something you recover from where you stand, and a
grab is something that takes you somewhere. That is what makes it worth beating guard with,
and what makes whiffing it a commitment for both fighters.

## The debug overlay draws what the rules use

**F1.** Hitboxes while they are out, hurtboxes always, guard arcs, facing, and wherever the
class mechanic is sitting.

The attack volume comes from `sim::state::hitbox`, which the hit test itself calls. That is
not tidiness — it is the difference between an overlay and a *second implementation of the
rules that can disagree with the first*. The previous version rebuilt the box from the move
table and so ignored the Bellator's weapon form, which multiplies reach: it drew a spear as
though it were a sword. An overlay that can drift is worse than none, because it is
confidently wrong at the exact moment you are using it to work out why something missed.

Three things it has to get right, and did not before:

**It is a cylinder, not a sphere.** The hit test compares *flat* distance and says nothing
about height. You cannot duck under an attack or jump over one — whether an overhead beats a
crouch is a property of the move (`hits_crouching`), not of its geometry. A sphere would imply
a vertical extent the rules do not have.

**Hurtboxes are drawn too.** The test threshold is the attack radius *plus the defender's body
radius*, so drawing only the attack circle makes every hitbox look smaller than it acts. With
both cylinders drawn, "do these touch" is exactly "does this connect".

**Hue and brightness carry different facts.** Hue says what kind of attack it is — ordinary or
overhead. Brightness says whether it still has its hit. Folding the two together lost the
overhead signal the moment a move connected, which is precisely when you are stepping through
frames to see what happened. Worth knowing what the dim state actually shows: at point-blank
range a move connects on the very frame its box appears, so a landed hit is dim for every
frame you can see it. Bright means *out and still looking for someone* — a whiff.

## Help that cannot go stale

`./scripts/help.sh` — or `cargo run -p game -- --help` — prints every command, key, flag and
environment variable.

`crates/manual` holds the tables, and is **the source of truth rather than a document about
one**. The in-game legend is generated from the same entries, and `--help` returns before Bevy
starts because the crate has no dependencies.

Help maintained separately from the thing it describes is wrong within a month, and wrong help
is worse than none: it sends you looking for a feature that moved. So four tests read the
game's own source and fail if it responds to anything the manual does not mention — every
`KeyCode`, every environment variable, every command-line flag, and every binary and script in
the repository. The legend was already a hand-kept copy of the key handlers that had drifted
once; it is now assembled from the entries, so there is nowhere for the two to disagree.

The interesting failure was in the tests rather than the code. The first flag check scanned all
of `crates/game/src` and reported `--abbrev-ref` and `--cached` as undocumented features — they
are git's flags, passed through by the bake step. A completeness test has to know whose surface
it is describing.

## The Oven: tuning while it runs

**F7.** Every tuned number in the game — 307 of them — editable in a palette that floats over
the arena, with a **bake** button that writes them back to the repository and pushes.

Feel work is a loop: change a number, play it, change it again. The loop is only as fast as
its slowest step, and with the values compiled in that step is a rebuild — so in practice you
change one number, wait, and lose the comparison you were trying to make. Worse, a session of
tuning ends as something you have to remember and retype.

### One representation for three hundred numbers

Every knob is an **`i32`**. Fixed-point values are their raw 16.16 bits, frame counts are
frames, health is health, flags are 0 or 1. A `Unit` says how to read it back.

That uniformity is what makes the rest cheap: one store, one editor widget, one file format,
one search index. Three hundred parameters each needing their own would have been a UI project
rather than an afternoon.

### Two ways in, because three hundred needs both

**Families** group knobs the way you think about them — Movement, Air, Defence, Body, Match,
one per class's air stats, one per move. **Search** matches labels, families and identifiers
for when you already know the name. Either alone is unusable at this size: families with no
search means scrolling past two hundred things, search with no families means you can only find
what you can already name.

### `sim` stays float-free

The Oven lives inside `sim`, which has no floating point anywhere as a determinism guarantee.
An editor is not a good enough reason to put the first one in, so slider bounds are stored in
raw units and the baked file's human-readable comments are formatted with integer arithmetic.
The `f32` conversion for the sliders lives in the palette, on the renderer side of the line.

### Mistuned peers desync loudly

Tuning values are **rules, not state**: they never change during a frame, so rollback neither
saves nor restores them. But two peers running different rules would diverge silently and look
exactly like a netcode bug. The tuning hash is folded into `World::checksum`, so a mismatched
Oven is a desync on the first frame — an error message instead of a mystery.

### Class pickers

A click-to-cycle button beside each health bar, showing that player's class. **F8**, and on
under `--dev`. They sit next to the health bars because that is where you are already looking
to know who is who, and the button *is* the class name, so it labels itself.

They need a free cursor, which is what the Oven gives you — and they route through the same
`UiFocus` the Oven does, so clicking one neither throws a poke nor re-grabs the mouse for the
camera. That is the third thing sharing the pointer now, which is why the ownership question
below has a single answer rather than one per widget.

### Who owns the mouse

The game and the editor share one window, one mouse and one keyboard, so something has to say
which of them an input belongs to. egui tracks it; `UiFocus` copies the answer somewhere the
game systems can read, a frame behind — which is fine, since a pointer over the panel this
frame is still over it next.

**F7 hands the cursor back. Clicking the arena takes it again. Escape returns it to the Oven.**

Three lines, and it took three attempts to get there. Releasing only while the pointer hovers
the panel leaves the first click after F7 spent on getting the pointer back. Never capturing
while the Oven is open means closing the panel to play. What both missed is that *clicking the
arena* is a different event from *clicking* — once the rule distinguishes them there is nothing
left to learn.

The keyboard keeps playing throughout, so a value can be dragged and then felt with W and J
without closing anything.

Pointer and keyboard are claimed separately. Hovering a slider must not stop the fighter
responding; a focused text field must stop it entirely, because `J`, `K` and `L` are attack
keys and searching the Oven for a move name would otherwise play a sequence.

### Families are gathered, not adjacent

The palette draws one collapsing header per family, keyed by name. Grouping walks the whole
registry and collects a family wherever its members sit, rather than treating a family as a run
of neighbouring knobs.

That distinction is load-bearing rather than fussy. **Appending a knob to the end of the
registry is what keeps every existing index in `tuned.rs` valid**, so a new knob almost always
arrives a long way from its relatives. The first version grouped by adjacency and drew a second
"Air" header the moment two air scalars were appended — egui flagged it as a duplicate widget,
correctly.

### Baking

`crates/sim/src/tuned.rs` is generated and is the single source of truth for values. `moves.rs`
keeps the move *names and prose*; its numbers live in the Oven, because a second copy would
drift and a move table that disagrees with the game is worse than no table.

The generated arrays carry `#[rustfmt::skip]` so the file stays byte-identical to what the
emitter produces — otherwise `cargo fmt` re-columnises it, the file stops matching its
generator, and the test that catches a stale bake has to be weakened to a fuzzy comparison.
That test (`the_committed_file_is_what_the_oven_would_write`) is what catches a hand-edited
`tuned.rs` or a bake that wrote the file but never got committed.

Bake writes the file, formats it, commits it and pushes on whatever branch is checked out. It
reports each step separately: a bake that wrote and committed but could not push is a
*different* outcome from one that worked, and blurring the two would cost someone an
afternoon.

`cargo run -p sim --bin bake_tuning` does the same write without launching the game.

### Every magnitude is reachable from the Oven, and a test says so

A feel number written straight into the code is invisible to the Oven *and* to the bake, so the
only way to change it is a recompile — which is the exact thing the harness exists to avoid.
Worse, it can silently disagree with the knob meant to control it. `crates/sim/tests/knobs.rs`
was written after finding precisely that: `arena.rs` collided against a hardcoded body radius
while the hit test used the Oven's, so tuning the body made fighters a different size to walls
than to attacks.

The rule it enforces: **in the simulation, an `Fx` built from a literal, or a `const` of a
numeric type, is a tuning value.** It belongs in the Oven, or it belongs in the test's `EXEMPT`
table with a sentence saying why it is not. A second test asserts every exemption has a real
reason, because an exemption nobody had to justify is just a way to silence the check.

It found thirty-two on the first run — the Bellator's nine form multipliers, which are most of
that class; the Bulwark's shield speed, range, damage and knockback; the Reaver's leash; all
three velocity decays; the Dual mage's whole meter. None of them were tunable, and all of them
are the kind of number you want to move while watching the game move.

Four files are excluded and the reasons are in the test: `input.rs` is the wire format, and
`curve.rs`, `fixed.rs` and `math.rs` are arithmetic — the `3` in a cubic Bézier is the
definition of a cubic Bézier, and changing it would not change how anything feels, it would
stop the curve being a curve.

The known gap is inline integers: a `damage: 85` written in a struct literal is not caught,
only a named `const`. Fixing that means parsing rather than scanning, and the named-const rule
already covers the shape these actually take.

## Curves: what the time is spent doing

A single number says *how long*. A curve says what happens during it. A structure rising out of
the ground over a quarter second can be a steady slide or a hold followed by an eruption, and
those are different things to play against even though the duration is identical.

The shape is a **cubic Bézier from (0,0) to (1,1) with two handles** — the same four numbers as
a CSS easing. Four because it is the smallest thing that can express hold-then-burst *and* be
edited by dragging two points, which is what the curve editor will do when it exists. Until
then they are four ordinary knobs in the Oven, baked like everything else: the data model is
already right, only the widget is missing.

Evaluation solves for the curve parameter by **bisection with a fixed iteration count**, not
Newton. Newton converges faster, but its step count depends on the handles, so the answer would
vary with the shape — and a value that depends on how hard it was to compute is not something
two peers can agree on. Twenty halvings puts the bracket below what 16.16 can represent, at the
same cost every time.

### Not yet

Curves. Several of these want to be splines rather than scalars — a knockback that varies with
damage, an acceleration that eases. The store is integers precisely so that adding a curve type
later is a new `Unit` and a new editor widget rather than a rewrite. Animation authoring is the
other half the Oven is eventually meant to hold; today that lives in `crates/anim` and is baked
offline.

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
| Test suites | 163 tests |
| Headless soak (`cargo run -p game`) | 3600 frames, 900 rollbacks, converges exactly |
| Browser frame-data tool | `./crates/web/build-sandbox.sh` |
| **Bevy prototype** | **`cargo run -p game`** — 3D arena, standins, HUD, debug overlay, local 2P |
| Bulwark kit | Bash, Slam, Guard, parry, Grapple — which actually **holds** you — shield throw/recall/leap |
| Persistent effects | Fire pillar (two growing volumes), black spike (drain + slow), structures that expire |
| Uppercut | Leaps, and takes whoever it catches into the air with it |
| Universal movement | Variable-height jump, shift-dodge with i-frames, once-per-jump airdodge, crouch |
| Air movement | Quake-style acceleration, per-class jump/gravity/fall/steering, per-move aerial hang |
| **Mouse look** | **Third-person camera, camera-relative movement, aimed attacks** |
| Crosshair | Projected from facing, so it is honest during a committed move |
| Settings | `~/.config/arena/settings.conf` — sensitivity, field of view, camera distance |
| **The Oven** | **F7** — 396 live tuning knobs, searchable, with bake-and-push, each move headed by the key that throws it |
| Help | `./scripts/help.sh` — generated, and tested against the game's own source |
| Dev mode | `./scripts/dev.sh` — wireframes, the Oven and the class pickers |
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
