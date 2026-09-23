# Working in this repository

Read [`docs/design/README.md`](docs/design/README.md) first — it is the map. The
design documents are the specification; the code is meant to match them, and
when it does not, one of the two is wrong and both get fixed.

Below are the rules that are load-bearing enough that breaking them has cost
real time more than once. Each has a test that enforces it, named in the rule.

## Aiming: there is one model, and you do not write a second one

**Everything that decides where an ability goes lives in
[`crates/sim/src/aim.rs`](crates/sim/src/aim.rs). Everything that intersects a
ray with a shape lives in `crates/sim/src/math.rs`. Nothing else in the
simulation may do either.** Enforced by `crates/sim/tests/one_aim.rs`.

The player's whole frame of reference is the crosshair, so the model is:

> One raycast, from the **camera** through the crosshair, ignoring anything
> behind the character model. It meets terrain, structures, and the ability's
> own max-range sphere. The first thing it reaches is what the player is
> pointing at, and the ability goes there.

**Bodies are not on that ray** — not other fighters, not the creature. It is
asking which *place* is under the crosshair, and a body is a thing standing in a
place, so the ray goes through it to the geometry behind. What a shot runs into
is a separate question, asked along the ability's own path by
`aim::first_along`. The creature is why: up close it fills the screen, the
reticle lands on its chest three metres up, and with it on the ray every
skillshot came out as a metre-long stub pointed at the sky.

Two kinds of skillshot start with that ray. Two more lines of effect do not —
they are pointed by something the player decided earlier. **Four in total, and
every one of them is a function in `aim.rs`:**

| Kind | Call | Rule |
| --- | --- | --- |
| Grounded | `aim::grounded_path` | Ground: cast exactly there. Max range: max range on the ground in the mouse's direction. If it travels, it travels from the character to that point. |
| Skillshot | `aim::skillshot_path` | Ground: that spot raised to the **middle of a fighter standing on it** (`aim::standing_middle`), because the floor is never the target — bodies are not on the ray, so a ground hit means "there". Anything else — wall, body, monster, range sphere — the point of intersection exactly. Straight line from the caster, and that line is its whole reach. |
| Swing | `aim::swing_path` | A body moving: no raycast, reach off the body. Yaw is `facing`; pitch follows the camera, **with a dead zone while standing** — level through the first 45° below the horizon, exact above it, and the leftover past it. The camera sits above the shoulder, so looking at somebody at your own height is looking slightly down at them. In the air there is no shared floor to read that way, so the pitch is followed exactly. A **one-armed** move leaves from that shoulder rather than the chest: `Move::hand`, declared in the table beside the shape, and `aim::across` is the only thing that turns it into a direction. |
| At the mechanic | `aim::mechanic_path` | Where the class mechanic is standing. The player aimed when they placed it. Guillotine lotus only. |

Five more functions live there and are **not** lines of effect. `aim::pointing_at`
answers *is the crosshair on that thing*, which the Reaver's forward dodge asks
about her shadow. It points nothing anywhere, but it is built from the eye and
the look direction, so it belongs with the rest of them — the alternative is an
angle worked out beside the ability, which is the mistake below wearing a hat.
`aim::clear_between` answers *is there any straight line from this body to that
one*, which the same dodge asks second: the dash crosses to wherever the shadow
is unless nothing reaches it. It is ray-against-shape work in service of a
decision about where something goes, so it belongs here rather than beside the
dodge for exactly the same reason. `aim::planted_ahead` answers *where does a
thing go that nobody aimed* — the slab the Elementalist's Landfall drives out of
the floor, a fixed distance along her flat facing, because she is arriving rather
than pointing. Its move still declares a line of effect (a swing, for the slam
itself); this is the second thing the same move puts in the world, and written
beside the ability it would be a facing, a distance and a floor query sitting
next to a move, which is the mistake below in its usual clothes.
`aim::shadow_faces` and `aim::copied_swing` answer *which way is forward for a
copy thrown from somewhere else*: the Reaver's shadow out on the field turns her
swing to the nearest body in reach, keeping her pitch. The copy is still a
swing; this is only its yaw.

Which one a move is comes from `Move::aim()`, **declared** in the move table so
every move has an answer, and printed in the `aimed` column of
`cargo run -p sim --bin frametable`. Inferring it from what a move leaves behind
is what let Fissure — a skillshot that races along the ground, by its own kit
entry — come out as a bubble seven metres in front of the body.

**The mistake this prevents, which has been made three times:** taking a ray
from the *chest* along the *look angle*. That ray is parallel to the crosshair's
and never converges with it, so the reticle sits on one thing and the ability
goes past it — by more the further away it is. If you find yourself writing
`camera::eye(...)`, `look_dir()`, or a ray-vs-shape call outside those two
files, stop: the thing you want already exists.

If none of the four fits a new ability, **change `aim.rs`** rather than working
around it. A change there is true of every ability at once, which is the point.
The full specification is [`docs/design/aiming.md`](docs/design/aiming.md).

## Tuning: every magnitude is a knob in the Oven

Feel numbers live in the Oven (`crates/sim/src/oven.rs`), are edited in the
running game with F7, and are baked to `crates/sim/src/tuned.rs`. A number
written straight into the code cannot be tuned or committed from there.
Enforced by `crates/sim/tests/knobs.rs`, which lists the handful of genuine
exemptions and demands a reason for each.

After changing the Oven's shape, run `cargo run -p sim --bin bake_tuning` —
`crates/sim/tests/oven.rs` fails if the committed file is not what the Oven
would write.

## The simulation is deterministic and float-free

`crates/sim` has no floating point at all, no dependencies, and no I/O: rollback
netcode re-simulates past frames and two machines must agree bit for bit. Fixed
point is `Fx`, 16.16. Enforced by `crates/sim/tests/no_floats.rs` and
`determinism.rs`.

Anything that affects gameplay belongs in the `World` snapshot — **including
animation clocks**, because a clip advancing on the renderer's own clock pops
every time a rollback happens.

## A frame costs what it is budgeted, and a frame never allocates

The simulation runs **more than once per picture**: a rollback re-simulates up to
eight frames and saves a snapshot for each, inside the same 16.7 ms that still
has to draw. So per-frame cost arrives multiplied, exactly when the connection
is already struggling. One rollback burst gets a quarter of the frame, one
`advance` a thirty-second of it, one frame of `view` an eighth. Enforced by
`crates/sim/tests/budget.rs`, `view/tests/budget.rs` and `net/tests/budget.rs`.

**Add to the `no floats, no deps, no I/O` list: no allocation.** A simulation
frame, and the whole path from snapshot to screen in `view`, must not touch the
heap — an allocator is a lock with a tail, and it is what turns one frame in a
thousand into a millisecond. That check and the 4 KiB cap on `World` are the
ones that matter: they give the same answer on every machine, whereas the three
timing budgets run at 5–40x the measured cost and will only catch a change of
*kind* — an accidental quadratic, an unbounded scan, a blocking call.

The full reasoning is [`docs/design/architecture.md`](docs/design/architecture.md)
§"The frame budget".

## Feel is a set of properties, not a vibe

`crates/sim/tests/feel.rs` holds the relationships that must survive tuning:
every attack punishable on block, risk scaling with reward, and so on. If one
fails, either it is a bug **or a design decision changed** — and then
`docs/design/feel-log.md` and the relevant kit document need updating too, not
just the assertion. Record what you tried in the feel log, including the things
you reverted.

## The browser build is the same program

`game` compiles for a window and for a canvas, and **the two differ in exactly
five things, each of which lives in a file that exists to hold it.** Where a
run's settings come from (argv, or the query string — `?dev` is `--dev`), where
a player's settings are kept (a file, or local storage) and where a panic can be
read are `crates/game/src/platform.rs`. Whether there is a peer is `online.rs`.
Whether there is a checkout to commit a bake to is `bake.rs` and `hub.rs`.
Enforced by `crates/game/tests/one_platform.rs`, which fails on `std::env`,
`std::fs`, `std::net`, `std::process`, `std::thread` or `web_sys` anywhere else
in the crate.

This rule is here for a failure that is *silent in one direction*, rather than
for one that has cost time already. A `std::fs::write` added to a system
compiles for wasm32 perfectly well — `std` is there, the call is there — and
returns an error nobody reads. The feature then works on the desk and quietly
does nothing on the web, and the first report of it comes from somebody who was
sent a link.

Build it with `./crates/web/build-game.sh`; the reasoning is
[`docs/design/web.md`](docs/design/web.md).

## The overlay draws what the hit test uses

`state::hitbox` is the one description of an attack's volume, and the debug
overlay, the browser sandbox and the creature's exchange all read it. An overlay
that can drift from the rule it illustrates is worse than no overlay.

## Checks before you push

```
cargo fmt --all
cargo clippy --workspace --all-targets
cargo test --workspace
```

These are the only checks there are: they run on your machine, and nothing
runs them for you. The one workflow in `.github/` publishes the browser build to
GitHub Pages and tests nothing, so a green Pages run means the page deployed,
not that the change is good.
