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
> behind the character model. It meets terrain, other players, monsters,
> structures, and the ability's own max-range sphere. The first thing it reaches
> is what the player is pointing at, and the ability goes there.

Two kinds of skillshot start with that ray. Two more lines of effect do not —
they are pointed by something the player decided earlier. **Four in total, and
every one of them is a function in `aim.rs`:**

| Kind | Call | Rule |
| --- | --- | --- |
| Grounded | `aim::grounded_path` | Ground: cast exactly there. Max range: max range on the ground in the mouse's direction. If it travels, it travels from the character to that point. |
| Skillshot | `aim::skillshot_path` | Ground: that spot raised to the caster's ability-origin height, so it flies level over it. Anything else — wall, body, monster, range sphere — the point of intersection exactly. Straight line from the caster, and that line is its whole reach. |
| Swing | `aim::swing_path` | A body moving: no raycast, reach off the body. Yaw is `facing`; pitch follows the camera, **with a dead zone while standing** — level through the first 45° below the horizon, exact above it, and the leftover past it. The camera sits above the shoulder, so looking at somebody at your own height is looking slightly down at them. In the air there is no shared floor to read that way, so the pitch is followed exactly. A **one-armed** move leaves from that shoulder rather than the chest: `Move::hand`, declared in the table beside the shape, and `aim::across` is the only thing that turns it into a direction. |
| At the mechanic | `aim::mechanic_path` | Where the class mechanic is standing. The player aimed when they placed it. Guillotine lotus only. |

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

## Feel is a set of properties, not a vibe

`crates/sim/tests/feel.rs` holds the relationships that must survive tuning:
every attack punishable on block, risk scaling with reward, and so on. If one
fails, either it is a bug **or a design decision changed** — and then
`docs/design/feel-log.md` and the relevant kit document need updating too, not
just the assertion. Record what you tried in the feel log, including the things
you reverted.

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

There is no CI in this repository, so these are the only checks there are.
