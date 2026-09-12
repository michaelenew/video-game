//! Bake the animation tables.
//!
//!     cargo run -p anim --bin bake
//!
//! Writes `crates/view/src/baked.rs`. **Edit the recipes here, not the
//! generated file.**
//!
//! Each recipe is a handful of poses and a looseness setting. Everything
//! between the keys — the lag, the overshoot, the settle — comes from the
//! solver. That is the point of the factory: the parts that make motion read
//! as alive are the parts that are miserable to key by hand.

use anim::bake::{Key, Looseness, Recipe, bake, emit, emit_moves};
use anim::derive::{Shape, Vocabulary};
use view::pose::{PartTransform, Pose};

/// Shorthand: torso, head, arm L, arm R, leg L, leg R.
const fn pose(parts: [([f32; 3], [f32; 3]); 6]) -> Pose {
    Pose {
        parts: [
            PartTransform {
                pos: parts[0].0,
                rot: parts[0].1,
            },
            PartTransform {
                pos: parts[1].0,
                rot: parts[1].1,
            },
            PartTransform {
                pos: parts[2].0,
                rot: parts[2].1,
            },
            PartTransform {
                pos: parts[3].0,
                rot: parts[3].1,
            },
            PartTransform {
                pos: parts[4].0,
                rot: parts[4].1,
            },
            PartTransform {
                pos: parts[5].0,
                rot: parts[5].1,
            },
        ],
    }
}

fn neutral() -> Pose {
    pose([
        ([0.0, 0.95, 0.0], [0.0, 0.0, 0.0]),
        ([0.0, 1.55, 0.0], [0.0, 0.0, 0.0]),
        ([-0.42, 1.05, 0.0], [0.0, 0.0, 0.12]),
        ([0.42, 1.05, 0.0], [0.0, 0.0, -0.12]),
        ([-0.18, 0.38, 0.0], [0.0, 0.0, 0.0]),
        ([0.18, 0.38, 0.0], [0.0, 0.0, 0.0]),
    ])
}

fn windup() -> Pose {
    pose([
        ([0.0, 0.90, -0.18], [0.0, -0.45, 0.0]),
        ([0.0, 1.50, -0.16], [0.0, -0.30, 0.0]),
        ([-0.40, 1.02, 0.10], [0.0, 0.0, 0.30]),
        ([0.46, 1.22, -0.40], [-1.20, 0.0, -0.30]),
        ([-0.20, 0.38, -0.10], [-0.20, 0.0, 0.0]),
        ([0.22, 0.38, 0.14], [0.30, 0.0, 0.0]),
    ])
}

fn strike() -> Pose {
    pose([
        ([0.0, 0.92, 0.26], [0.0, 0.40, 0.0]),
        ([0.0, 1.50, 0.24], [0.0, 0.26, 0.0]),
        ([-0.44, 0.98, -0.18], [0.0, 0.0, 0.36]),
        ([0.30, 1.10, 0.72], [1.35, 0.0, -0.16]),
        ([-0.20, 0.38, 0.20], [0.34, 0.0, 0.0]),
        ([0.22, 0.38, -0.16], [-0.26, 0.0, 0.0]),
    ])
}

/// Anticipation: a fast coil down and back before the arms go up.
///
/// This exists for a gameplay reason, not an aesthetic one. A heavy clip lags
/// by design, and that lag ate the first third of the overhead's startup --
/// the silhouette barely moved while the opponent was supposed to be reading
/// it and deciding whether to block. A sharp early coil puts a visible change
/// on frame two or three, so the startup is legible for its whole length.
fn overhead_coil() -> Pose {
    pose([
        ([0.0, 0.74, -0.20], [0.30, 0.0, 0.0]),
        ([0.0, 1.32, -0.22], [0.34, 0.0, 0.0]),
        ([-0.48, 0.86, -0.26], [0.55, 0.0, 0.34]),
        ([0.48, 0.86, -0.28], [0.60, 0.0, -0.34]),
        ([-0.22, 0.32, -0.04], [0.26, 0.0, 0.0]),
        ([0.24, 0.32, 0.06], [0.22, 0.0, 0.0]),
    ])
}

fn overhead_raise() -> Pose {
    pose([
        ([0.0, 1.02, -0.10], [-0.30, 0.0, 0.0]),
        ([0.0, 1.62, -0.12], [-0.35, 0.0, 0.0]),
        ([-0.44, 1.32, -0.10], [-1.70, 0.0, 0.30]),
        ([0.44, 1.34, -0.12], [-1.85, 0.0, -0.30]),
        ([-0.18, 0.40, -0.06], [-0.15, 0.0, 0.0]),
        ([0.20, 0.38, 0.06], [0.12, 0.0, 0.0]),
    ])
}

fn overhead_land() -> Pose {
    pose([
        ([0.0, 0.68, 0.22], [0.55, 0.0, 0.0]),
        ([0.0, 1.24, 0.30], [0.60, 0.0, 0.0]),
        ([-0.40, 0.62, 0.34], [1.30, 0.0, 0.25]),
        ([0.40, 0.62, 0.36], [1.35, 0.0, -0.25]),
        ([-0.22, 0.30, 0.10], [0.42, 0.0, 0.0]),
        ([0.22, 0.32, -0.12], [-0.20, 0.0, 0.0]),
    ])
}

fn guard() -> Pose {
    pose([
        ([0.0, 0.88, 0.0], [0.0, 0.70, 0.0]),
        ([0.0, 1.46, 0.06], [0.10, 0.40, 0.0]),
        ([-0.16, 1.14, 0.44], [-0.90, 0.0, 0.55]),
        ([0.40, 1.00, 0.10], [-0.30, 0.0, -0.25]),
        ([-0.24, 0.36, 0.06], [0.0, 0.0, 0.10]),
        ([0.24, 0.36, -0.10], [0.0, 0.0, -0.10]),
    ])
}

fn roll() -> Pose {
    pose([
        ([0.0, 0.52, 0.10], [-1.05, 0.0, 0.0]),
        ([0.0, 0.86, 0.34], [-0.90, 0.0, 0.0]),
        ([-0.36, 0.62, 0.26], [-1.30, 0.0, 0.45]),
        ([0.36, 0.62, 0.26], [-1.30, 0.0, -0.45]),
        ([-0.18, 0.30, -0.22], [-1.10, 0.0, 0.0]),
        ([0.18, 0.30, -0.28], [-1.30, 0.0, 0.0]),
    ])
}

fn recoil() -> Pose {
    pose([
        ([0.0, 0.86, -0.30], [-0.36, 0.0, 0.0]),
        ([0.0, 1.42, -0.40], [-0.55, 0.0, 0.0]),
        ([-0.52, 1.14, -0.24], [-0.70, 0.0, 0.60]),
        ([0.52, 1.14, -0.24], [-0.70, 0.0, -0.60]),
        ([-0.18, 0.38, 0.16], [0.34, 0.0, 0.0]),
        ([0.20, 0.38, -0.06], [-0.14, 0.0, 0.0]),
    ])
}

/// Reaching out to take hold. No arc, which is the point -- a grab is answered
/// by not being there rather than by reading its swing.
fn seize() -> Pose {
    pose([
        ([0.0, 0.92, 0.22], [0.0, 0.0, 0.0]),
        ([0.0, 1.50, 0.26], [0.12, 0.0, 0.0]),
        ([-0.40, 1.16, 0.58], [-1.25, 0.0, 0.30]),
        ([0.40, 1.16, 0.58], [-1.25, 0.0, -0.30]),
        ([-0.20, 0.38, 0.10], [0.18, 0.0, 0.0]),
        ([0.20, 0.38, -0.08], [-0.14, 0.0, 0.0]),
    ])
}

/// The three shapes a derived clip can take, as poses.
///
/// The one genuinely authored thing in the derivation: `anim::derive` decides
/// *when* each pose lands, from the move's own frame data, and this decides
/// what the poses are.
fn vocabulary(shape: Shape) -> Vocabulary {
    match shape {
        Shape::Level => Vocabulary {
            neutral: neutral(),
            coil: windup(),
            strike: strike(),
            anticipate: None,
        },
        Shape::Overhead => Vocabulary {
            neutral: neutral(),
            coil: overhead_raise(),
            strike: overhead_land(),
            // The sharp coil down, which is what makes a long startup legible
            // for its whole length rather than only its back half.
            anticipate: Some(overhead_coil()),
        },
        Shape::Seize => Vocabulary {
            neutral: neutral(),
            coil: windup(),
            strike: seize(),
            anticipate: None,
        },
    }
}

/// A clip for every move in the game, from its own frame data.
///
/// Names are `class_move`, lowercased, because they become `static` items and
/// a move called "Bash" exists on more than one class.
fn derived() -> (Vec<anim::bake::Recipe>, Vec<(String, usize, usize)>) {
    let mut recipes = Vec::new();
    let mut index = Vec::new();
    for (c, class) in sim::class::ALL_CLASSES.iter().enumerate() {
        for kind in 0..3u8 {
            let m = sim::moves::get(*class, kind);
            // Sanitised, because the name becomes a Rust `static`. The Blood
            // mage's "Reaper's Debt" is what found this: an apostrophe
            // survives happily all the way to a generated file that will not
            // parse.
            let ident = |s: &str| -> String {
                s.to_lowercase()
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                    .collect()
            };
            let name: &'static str = Box::leak(
                format!("move_{}_{}", ident(class.name()), ident(m.name)).into_boxed_str(),
            );
            let shape = Shape::of(m.hits_crouching, m.grabs);
            recipes.push(anim::from_frame_data(
                name,
                m.startup,
                m.active,
                m.recovery,
                shape,
                &vocabulary(shape),
            ));
            index.push((name.to_string(), c, kind as usize));
        }
    }
    (recipes, index)
}

fn main() {
    let recipes = [
        // A quick poke: wind up briefly, snap out, recover. Martial looseness
        // keeps it crisp -- a loose arm here would read as a flail.
        Recipe {
            name: "poke",
            length: 17,
            keys: vec![
                Key {
                    frame: 0,
                    pose: neutral(),
                },
                Key {
                    frame: 4,
                    pose: windup(),
                },
                Key {
                    frame: 7,
                    pose: strike(),
                },
                Key {
                    frame: 16,
                    pose: neutral(),
                },
            ],
            looseness: Looseness::MARTIAL,
        },
        // The committed overhead. Heavy looseness so the arms lag behind the
        // torso on the way up and overshoot on the way down.
        Recipe {
            name: "overhead",
            length: 42,
            keys: vec![
                Key {
                    frame: 0,
                    pose: neutral(),
                },
                Key {
                    frame: 3,
                    pose: overhead_coil(),
                },
                Key {
                    frame: 11,
                    pose: overhead_raise(),
                },
                Key {
                    frame: 18,
                    pose: overhead_land(),
                },
                Key {
                    frame: 41,
                    pose: neutral(),
                },
            ],
            looseness: Looseness::HEAVY,
        },
        // Settling into guard. Short, and stiff so it reads as deliberate.
        Recipe {
            name: "guard_in",
            length: 10,
            keys: vec![
                Key {
                    frame: 0,
                    pose: neutral(),
                },
                Key {
                    frame: 6,
                    pose: guard(),
                },
                Key {
                    frame: 9,
                    pose: guard(),
                },
            ],
            looseness: Looseness::MARTIAL,
        },
        // Dodge roll: snap down, come back up. The tail is the vulnerable part
        // and it should look like it.
        Recipe {
            name: "roll",
            length: 22,
            keys: vec![
                Key {
                    frame: 0,
                    pose: neutral(),
                },
                Key {
                    frame: 3,
                    pose: roll(),
                },
                Key {
                    frame: 12,
                    pose: roll(),
                },
                Key {
                    frame: 21,
                    pose: neutral(),
                },
            ],
            looseness: Looseness::HEAVY,
        },
        // Getting hit. Nothing here should look controlled.
        Recipe {
            name: "recoil",
            length: 26,
            keys: vec![
                Key {
                    frame: 0,
                    pose: recoil(),
                },
                Key {
                    frame: 6,
                    pose: recoil(),
                },
                Key {
                    frame: 25,
                    pose: neutral(),
                },
            ],
            looseness: Looseness::HEAVY,
        },
    ];

    let baked: Vec<_> = recipes.iter().map(bake).collect();

    // Every move in the game gets a clip timed to its own frame data. See
    // `anim::derive` for why that is not the same as picking one of two
    // hand-authored clips and hoping the lengths line up.
    let (move_recipes, index) = derived();
    let move_baked: Vec<_> = move_recipes.iter().map(bake).collect();
    let index_refs: Vec<(&str, usize, usize)> =
        index.iter().map(|(n, c, k)| (n.as_str(), *c, *k)).collect();

    let total: usize = baked
        .iter()
        .chain(move_baked.iter())
        .map(|b| b.frames.len())
        .sum();

    let path = "crates/view/src/baked.rs";
    let text = emit(&baked) + &emit_moves(&move_baked, &index_refs);
    std::fs::write(path, text).expect("write baked table");

    for b in &baked {
        println!("{:<34} {:>3} frames", b.name, b.frames.len());
    }
    println!();
    for b in &move_baked {
        println!("{:<34} {:>3} frames", b.name, b.frames.len());
    }
    println!("\n{path}  ({total} frames total)");
}
