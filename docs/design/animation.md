---
status: decided
decided: 2026-09-11
---

# Animation

How a character moves, from the bones outward. Three files decide everything:

```
crates/view/src/skeleton.rs   The frame: bones, joints, limits, per-class builds
crates/view/src/clips.rs      The contract: every animation the game can play
crates/anim/src/clips/*.rs    The content: the poses and timing of each one
```

Plus two tools:

```
cargo run -p anim --bin bake                Recipes -> crates/view/src/baked.rs
cargo run -p anim --bin preview -- <clip>   A PNG contact sheet of one clip
```

…and the **animation hub**, `F9` in the running game, which is the same
authoring surface with a mouse on it.

## The rule everything rests on

**Pose is a pure function of simulation state.**

```text
pose = f(action, frames into it, distance walked, airtime, turn rate, health)
```

No accumulated animation time, no independently ticking player. Rollback
re-simulates past frames, so anything animating on its own clock pops and slides
every time a rollback happens. Every input above comes out of the snapshot, and
the five that are purely for the renderer — distance walked, airborne frames,
frames since landing, the parry flourish, and what the current stun started at —
live in `Player` for exactly that reason.

Cosmetic smoothing may be renderer-local and is allowed to pop: a rollback is
one to eight frames, and a cross-fade that hiccups across 16 ms is
imperceptible. Nothing that reads as gameplay may.

## The frame

Sixteen joints. Two members per limb, as a body has: a thigh and a shin, an
upper arm and a forearm. The torso is two — a spine that bends at the waist and
a chest that turns on top of it — because one rigid torso cannot both fold into
a lunge and twist into a swing, and every melee animation wants both at once.
Hands and feet are along for the ride: they cost three channels each and they
are what makes a footfall land.

```text
                  Head
                   |
  Hand ─ Forearm ─ Arm ─ Chest ─ Arm ─ Forearm ─ Hand
                          |
                        Spine
                          |
            Thigh ────── Root ────── Thigh
              |                        |
            Shin                     Shin
              |                        |
            Foot                     Foot
```

A pose is **fifty-one numbers**: three for where the hips are, and three per
joint. No positions anywhere else — an elbow cannot be anywhere except at the
end of its upper arm, and saying so in the data structure is what lets one
authored clip play on six differently-proportioned bodies.

### The sign conventions

Each joint carries three *named* angles rather than a raw Euler triple:

| | means | positive is |
| --- | --- | --- |
| **swing** | forward and back | forward, on every joint that can go forward |
| **spread** | out from the body | *away from the midline*, on both sides |
| **twist** | rotation along the bone | — |

The mirror lives in the bone, not in the animation. `shoulders(0, 40, 0)` puts
*both* arms out to the sides; a symmetric pose is a symmetric set of numbers,
and mirroring a clip is a swap of the two sides and nothing else. Every
hand-authored pose in the version of this that stored raw Euler angles had at
least one sign wrong somewhere.

Knees and elbows are hinges: positive is **folded**. A knee folds the heel back,
an elbow brings the hand forward. Their other two channels are pinned near zero,
because an elbow that can also be splayed sideways is how a rig starts producing
broken dolls.

### Joint limits

Every joint has a range, and **the solver clamps to it**. That is not belt and
braces: springs overshoot on purpose, and overshoot on a knee is a broken frame.
Clamping at the end of the solve means ring can be tuned for feel without being
audited for anatomy.

An *authored* key that breaks a limit is a different thing — it means somebody
asked for something the body cannot do, and a test says so by name rather than
silently overruling them.

### Per-class builds

Six numbers per class: overall scale, bulk, limb length, shoulder width, stance
width, head size. The Bulwark is short, wide and thick; the Dual mage is tall
and slight. Weight is the most legible thing a character can have — you read it
across the arena in the first second, before you know a single move — and
because a pose is angles, the same clip is correct on all six without being
re-authored.

The hips sit exactly as high as the legs are long, derived rather than typed, so
every build stands with its soles on the floor and no one has to tune it.

## Authoring a pose

In degrees, through named methods, because that is how a person describes a
body:

```rust
// Weight back, sword arm cocked, front foot light.
Pose::rest()
    .hips(0.0, -0.05, -0.10)
    .spine(-8.0, 0.0, -22.0)
    .chest(4.0, 0.0, -18.0)
    .head(0.0, 0.0, 20.0)
    .shoulder_r(-40.0, 28.0, 0.0)
    .elbow_r(95.0)
    .hip_l(18.0, 6.0, 0.0)
    .knee_l(22.0)
```

The full surface, all in degrees except `hips`, which is in metres:

| | |
| --- | --- |
| `hips(x, y, z)` | move the hips, in metres from standing |
| `root(lean, tilt, turn)` | rotate the whole body at the hips |
| `spine(bend, side, twist)`, `chest(…)` | the two torso members |
| `head(nod, tilt, turn)` | positive turn looks to the character's own right |
| `shoulder_l/r(swing, spread, twist)`, `shoulders(…)` | |
| `elbow_l/r(bend)`, `elbows(bend)`, `forearm_l/r(bend, twist)` | |
| `wrist_l/r(bend, spread, twist)` | |
| `hip_l/r(swing, spread, twist)` | |
| `knee_l/r(bend)`, `knees(bend)` | |
| `ankle_l/r(point, roll, turn)` | positive `point` drops the toe |
| `set(Joint::…, a, b, c)` | any joint by name |

### Let the kinematics do the work

Four helpers exist so that the things which are *arithmetic* are not keyed by
hand. Using them is not optional polish — it is the difference between a walk
cycle that skates and one that does not.

| | |
| --- | --- |
| `plant_l/r([x, y, z])` | put the **ankle** at a point in character space and solve the leg for it. Ground contact is `y = ANKLE_ON_GROUND`. |
| `reach_l/r([x, y, z])` | the same for the wrist. |
| `toe_l/r(degrees)` | level the foot with the floor, then tip it. A heel strike is `-12`, flat is `0`. Tipping the toe *down* is capped at where the sole meets the floor, because that is all a foot can do without lifting its heel — which is the next line. |
| `toe_floor_l/r()` | roll the foot onto its toe, whatever height the ankle is at. What the end of a stride wants, and the only way a heel comes off the ground without the toe going through it. Solved against the box's actual corners, because a foot pitched forty degrees puts its front-bottom corner several centimetres below where its centreline says. |

`plant_*` and `reach_*` **clamp to the joint limits**. The closed form will
happily return a hundred degrees of hip adduction for a foot placed behind and
across the other leg; the limb reaches as far as the joint allows and stops,
which is what a body does. An out-of-reach target is not an error, it is a
strain.

A planted foot given the same target on consecutive frames **does not move**,
however much the hips do. That is what "no foot skate" means mechanically, and
it is why `plant_*` exists.

A hand given a target inside the working envelope lands within two centimetres
of it, and `a_hand_goes_where_it_is_sent` holds the solver to that over a grid
of 1544 targets. It is worth knowing why that test exists, because the failure
it caught was silent: for a while a wrist sent 30 cm above its own shoulder
came out 47 cm away, hanging by the ribs, and every pose in three Champion
clips was quietly a different pose from the one written. Three separate things
were wrong, all of them the kind of thing this layer is supposed to absorb:

- **The elbow was pinned.** With shoulder and wrist both fixed, the elbow can
  still travel a whole circle around the line between them. Hanging or
  punching it sits behind and below; overhead it swings out to the side. A
  fixed pole named one point on that circle, and asking for a hand above the
  head with the elbow still pointed backward only builds with the upper arm
  pointing up behind the shoulder, which no shoulder does. `hand_to` walks the
  circle and scores each candidate on where the wrist actually lands.
- **The elbow folded for a direction the shoulder had refused.** Clamping the
  shoulder and then bending the forearm as if it had not been clamped is what
  turns a few centimetres out of range into most of a metre. The bend now comes
  from where the elbow really ended up.
- **Two angles name a direction twice.** Swing past the pole and spread half a
  turn the other way is the same direction; add half a turn of twist and it is
  the same rotation. `aim_within` tries both readings, so a limit that refuses
  one no longer clamps the limb for nothing.

The last one has a consequence for clips as well as for single poses, and it is
the one to remember: **an arm pointing straight forward sits exactly where the
two readings meet**, the way longitude is undefined at the pole. Two keys can
hold the same shoulder to the last decimal and be 180 degrees apart in the
numbers, and since the numbers are what get interpolated, the frames between
them are a limb going the long way round. The bake's `unwound` pass puts every
key of a clip on one reading before anything is interpolated, so this is
handled — but if a pose ever looks right in the hub and wrong in motion, this
is the first thing to suspect.

Two more for combining poses: `mirrored()` (the same pose on the other side) and
`blend(&other, t)`.

## Authoring a clip

A recipe is a handful of keys, an easing per gap, and a looseness setting.
Everything between the keys comes from the solver.

```rust
Recipe {
    clip: Clip::BulwarkCommitted,
    looseness: Looseness::HEAVY,
    notes: "The overhead. Heavy, telegraphed on purpose, and the telegraph has \
            to be legible from frame two.".into(),
    keys: vec![
        Key::eased(0, neutral(), Ease::ANTICIPATE),
        Key::eased(3, coil(), Ease::SNAP),
        Key::eased(11, overhead(), Ease::STRIKE),
        Key::eased(18, landed(), Ease::OUT),
        Key::eased(41, neutral(), Ease::SMOOTH),
    ],
}
```

### The ease is the move

An ease shapes the time **out of** its key and into the next one. Two poses
eight frames apart can be a steady slide, a hold and then a burst, or a
pull-back before a commitment — three completely different moves to play
against, from identical frame data. This is where most of the character of a
move lives.

| | |
| --- | --- |
| `LINEAR` | constant speed. Correct for a stride and almost nothing else. |
| `SMOOTH` | eases in and out. The default. |
| `IN` / `OUT` | gathers / arrives and settles. |
| `SNAP` | almost nothing happens, and then it all happens. The telegraph. |
| `STRIKE` | all of it immediately, then a long tail. What a contact frame wants. |
| `ANTICIPATE` | pulls *away* from the target before going. |
| `OVERSHOOT` | carries past and comes back. |
| `HOLD` | stays put until the last instant. |

Handles may go outside 0…1 vertically; that is what anticipation and overshoot
are. `Ease::new(x1, y1, x2, y2)` for anything the presets do not cover.

### Looseness is in frames, not in spring frequency

Each part of the body gets a **lag** (how many frames it runs behind the keys)
and a **ring** (how far it carries past on arrival; `1.0` never overshoots).
Distal joints get proportionally more of both automatically, which is where
follow-through comes from without anybody keying it.

Legs taper far less than arms. A trailing hand is follow-through and the whole
reason the taper exists; a trailing *foot* is a foot in the floor, because the
ground is at a fixed height and does not wait for it.

This is not API taste. The first version of this file expressed weight as a
*low frequency*, which does not mean heavy — it means late. The Bulwark's slam
has a fourteen-frame startup and its silhouette had barely moved by frame five,
so there was nothing on screen to read while the opponent was supposed to be
deciding whether to block. **Weight must read as follow-through, never as
delay.** Presets: `CRISP`, `MARTIAL`, `STRIDE`, `HEAVY`, `FLOATY`, `LIMP`.

### Lengths come from the move table

An attack clip's length is `startup + active + recovery`, read live from the
Oven. Author against `clip.phases()` — the last startup frame, the first active
frame, the first recovery frame — rather than against typed-in numbers, and
retuning a move in the Oven moves its animation with it. A contact pose on the
wrong frame teaches the opponent the wrong timing, which is worse than no
animation.

## Playback

`view::play::pose_for` chooses and blends. Three tricks in it are worth knowing
about, because clips are authored against them:

**Locomotion is driven by ground covered, not by time.** A walk cycle on a fixed
cadence skates the moment the body moves at any other speed. The cycle is
indexed by a **stride phase** the simulation carries in the snapshot, so a
footfall happens every stride's worth of metres at any speed. The four
directional clips are sampled at the *same* phase before being blended, which is
what keeps a diagonal from producing two planted feet at once.

The phase is an **accumulator**, not a ratio, and that is load-bearing: a stride
is longer at a sprint than at a walk and shorter sideways than forwards, so
`distance / stride` jumps by whole cycles the moment the stride changes — which
reads as both legs teleporting, and did, before this was fixed. Integrating
`speed / stride` each tick cannot do that. `WALK_STRIDE`, `RUN_STRIDE`,
`CROUCH_STRIDE` and `STRAFE_STRIDE` live in `sim::tuning` because the simulation
is what integrates with them; `view::play` mirrors them as `f32` at compile time
and the recipes import those, so there is one of each number.

**Stun clips are indexed from the end.** `HitStun { left }` counts down, and
what matters is that the character is back on their feet on the exact frame
control returns. A stun longer than its clip holds the impact pose at the
*start*, where holding looks like being hurt.

**Turn clips are indexed by turn rate, not by time.** Facing follows the mouse,
so there is no turn event to start a clip on — only a body that is currently
rotating fast or slow. Frame zero of a turn clip must be neutral and its last
frame a full committed turn; the renderer picks the frame from how fast the body
is actually rotating and layers the difference over whatever the legs are doing.

### Cross-fading

`pose_for` switches clips; it does not fade between them. The fade lives in
`view::play::Crossfade`, which the renderer owns — the one piece of animation
state deliberately kept **outside** the snapshot, because a fade that hiccups
across a rollback's one to eight frames is imperceptible and a cut every time
you start walking is not. It fades on a coarse *shape* change (an attack
beginning, a body being hit, a sprint stopping) and also eases the travel
direction and the gait blend, which the simulation changes in a single frame.

## Looking at it

```text
cargo run -p anim --bin preview -- walk_forward         one clip
cargo run -p anim --bin preview -- --file dodge         a whole file
cargo run -p anim --bin preview -- walk_forward --feet  a per-frame foot table
```

A contact sheet has three panels: the clip from the side, from the front, and
every frame overlaid so the arcs of the hands and feet are visible as arcs. The
side view is the important one — it is what an opponent reads across the arena —
and the overlay is where foot skate shows up, as a planted ankle whose dots
drift instead of piling on one spot. `--feet` prints the numbers behind that:
while a foot is down, its world position should not change.

### Showing it to somebody

```text
cargo run -p anim --bin export -- docs/preview/anim.json
```

Writes the six class skeletons and every baked frame out as JSON, plus the same
bytes as a script. `docs/preview/index.html` reads it and plays any clip on any
build in a browser, with the phase strip, onion skin and a per-joint angle
readout — the same forward kinematics the game runs, off the same table. Open
it from a static server in that folder (`python3 -m http.server`), or publish
the two files anywhere. It is for the conversation that starts "does this walk
look like walking to you", which is not a conversation to have over a Rust
toolchain.

## The standards

`crates/anim/tests/clips.rs` holds every authored clip to these, and skips
anything nobody has written yet:

- keys in order and inside the clip
- no authored key outside a joint's range
- looping clips close on themselves
- grounded clips keep their feet out of the floor, and mostly on it
- **a planted foot does not slide** — measured across the flat phase of each
  stride, with the body's own travel added in
- **no joint moves faster than a body can move it**, measured *relative to the
  hips* (see below)
- an attack has visibly changed by the time its startup is a third gone
- the three phases of an attack look different from each other
- an idle keeps both feet down
- **a hand goes where it is sent** — within two centimetres, over a grid across
  the whole working envelope
- both readings of a joint's three angles are the same rotation, which is what
  lets the bake swap between them

`cargo test -p anim --test clips report_discontinuous_clips -- --ignored
--nocapture` lists everything over the motion ceilings with the multiple it is
over by, which is how a batch of new clips gets triaged in one go rather than
one assertion at a time. `report_floor_clearance` does the same for the floor.

### The motion ceiling, and why it is measured relative to the hips

A body that is travelling moves every joint on it. A dive roll or a jump
takeoff moves them all very fast, and that is the character going somewhere
rather than the pose jumping — so the test subtracts the hips' own movement and
looks at what is left.

The ceilings depend on where the joint sits, because a hand is at the end of a
two-metre lever and a hip is not:

| | metres per frame | |
| --- | --- | --- |
| root, spine, chest | 0.15 | nine metres a second; a jump's extension is the fastest thing in the game and still under it |
| head | 0.30 | half a metre up from the hips, and a hard tuck throws it |
| shoulder, hip | 0.24 | rides the chest, plus the third of a metre of spine under it |
| elbow, knee | 0.30 | |
| hand, foot | 0.36 | twenty-one metres a second, about what a sprinter's foot does at the top of its swing |

The opening frame of a one-shot is exempt: a dodge leaves at seventeen metres a
second, a takeoff at eighteen, and the game never shows that frame raw because
`Crossfade` eases into every one of them. Every frame of a **loop** is checked,
because a loop is seen exactly as baked.

An earlier version compared raw angle change instead. It forbade a forearm
rolling through a sword cut — a large number in the twist channel and no motion
at all on screen — and permitted a torso teleport, which is the failure that
actually matters.

## The hub

**F9.** The clip list, a timeline with draggable keys, the curve editor, joint
sliders bounded by the limits, the looseness knobs, and a save button that
regenerates `crates/anim/src/clips/<file>.rs` and re-bakes.

Edits re-run the solver immediately and drive a real fighter in the real arena,
so what you are looking at while you drag a slider is the thing that ships. The
prose describing a clip lives in its `notes` field rather than in a comment,
because the hub rewrites these files and a comment would not survive.

## Why generated Rust rather than a data file

No loader, no asset path, no runtime parsing, and a diff shows exactly what
changed when an animation is retuned. `crates/view/src/baked.rs` is one line per
frame for the same reason: a pose spread over twenty lines turns a two-frame
change into forty lines of noise.

## What is authored

All fifty-six. `cargo run -p anim --bin bake` lists any clip without a recipe
every time it runs, and a test fails if there is one, so this cannot go quietly
stale: an unauthored clip bakes as a held rest pose, which is a character
standing still in the middle of a match.

Two diagnostics are worth knowing about when a clip is not behaving:

- `report_discontinuous_clips` lists everything over the motion ceilings with
  the multiple it is over by. A clip that trips it has a **timing** problem, not
  a posing one — two good poses too far apart for the frames between them. The
  fix is an intermediate key, or one key moved to give the contact frame its
  approach, or a different ease. It is not a softer pose and it is not a relaxed
  ceiling.
- `report_clamped_joints` lists poses asking for more than a joint has, which is
  almost always an inverse-kinematics target further away than the limb is long.
  A few per cent is ordinary — a strained reach with a straight limb is a real
  thing a body does. Six or seven per cent means a grip or a foot target wants
  moving a few centimetres closer.

## Not yet

- **Mechanic animations.** Throwing the shield, and changing form. They need a
  clock in the simulation the way attacks have one, and they do not have one
  yet, so there is nothing to drive a clip from. Authoring them before that
  exists would be authoring content that never plays.

  Two classes are out of this list, and both got out the same way: their
  mechanic **became a move**, which is to say it grew the clock. The Blood
  mage's Black spike and the Reaver's Send shadow have a startup, an active
  window and a recovery, so their clips take their length from the move table
  like every other attack.

  The Reaver went one further and needed two clips that belong to nothing in the
  move table at all -- `shadow_dash` and `shadow_ready`, which are what her
  *second body* does with itself. They play on a second skeleton, and everything
  else that body does is one of her own clips replayed a few frames late. A
  clock that is not a move's, on a body that is not a fighter: the first of
  either, and the pattern to copy if another mechanic ever grows a body.
- **Weapons.** Hands have an orientation and a length to hang something off;
  nothing hangs off them.
- **glTF standins.** The pose function's signature does not change, only what it
  returns. The joint names here are deliberately the ones a rigged asset uses.
