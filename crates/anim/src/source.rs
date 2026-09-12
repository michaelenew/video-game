//! Writing recipes back out as Rust.
//!
//! The hub edits clips while the game runs; this is what makes the edits
//! survive the process. It regenerates a whole recipe file from the in-memory
//! recipes, in the same shape a person would have written by hand -- named
//! methods, degrees, one key per block.
//!
//! Round-tripping through here has to be lossless, which is the reason poses
//! store *named* channels. Turning `[0.42, -0.13, 0.0]` back into
//! `.shoulder_r(24.1, -7.4, 0.0)` is a rename; turning a raw Euler triple back
//! into something a person can read is not possible at all. It is also why a
//! recipe's prose lives in a `notes` field rather than in a comment: a comment
//! would not survive a save.

use crate::bake::{Looseness, Recipe};
use crate::ease::Ease;
use view::pose::Pose;
use view::skeleton::{JOINTS, Joint};

/// A whole file's worth of recipes, ready to write to
/// `crates/anim/src/clips/<file>.rs`.
pub fn emit_file(file: &str, title: &str, recipes: &[Recipe]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "//! {title}\n\
         //!\n\
         //! GENERATED-FRIENDLY: the animation hub (F9) rewrites this file when it\n\
         //! saves, in the same shape you would write by hand. Editing it by hand is\n\
         //! fine and expected; keep the prose in each recipe's `notes` field, because\n\
         //! ordinary comments do not survive a save from the hub.\n\n\
         use crate::bake::{{Key, Looseness, Recipe}};\n\
         use crate::ease::Ease;\n\
         use view::clips::Clip;\n\
         use view::pose::Pose;\n\n\
         pub fn clips() -> Vec<Recipe> {{\n    vec![\n"
    ));
    let _ = file;
    for r in recipes {
        out.push_str(&emit_recipe(r, 8));
    }
    out.push_str("    ]\n}\n");
    out
}

pub fn emit_recipe(r: &Recipe, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let mut out = String::new();
    out.push_str(&format!("{pad}Recipe {{\n"));
    out.push_str(&format!("{pad}    clip: Clip::{},\n", variant(r)));
    out.push_str(&format!(
        "{pad}    looseness: {},\n",
        looseness_source(&r.looseness)
    ));
    out.push_str(&format!("{pad}    notes: {}.into(),\n", quote(&r.notes)));
    out.push_str(&format!("{pad}    keys: vec![\n"));
    for k in &r.keys {
        out.push_str(&format!(
            "{pad}        Key::eased(\n{pad}            {},\n{pad}            {},\n{pad}            {},\n{pad}        ),\n",
            k.frame,
            pose_source(&k.pose, indent + 12),
            ease_source(k.ease)
        ));
    }
    out.push_str(&format!("{pad}    ],\n{pad}}},\n"));
    out
}

fn variant(r: &Recipe) -> String {
    // `Clip` is an enum; its Debug is the variant name, which is exactly what
    // has to appear in the generated source.
    format!("{:?}", r.clip)
}

fn quote(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

pub fn ease_source(e: Ease) -> String {
    match e.preset_name() {
        Some(name) => format!("Ease::{}", name.to_uppercase()),
        None => format!(
            "Ease::new({:.3}, {:.3}, {:.3}, {:.3})",
            e.x1, e.y1, e.x2, e.y2
        ),
    }
}

pub fn looseness_source(l: &Looseness) -> String {
    match l.name() {
        "custom" => format!(
            "Looseness {{ root: {}, spine: {}, chest: {}, head: {}, arms: {}, legs: {} }}",
            feel(l.root.lag, l.root.ring),
            feel(l.spine.lag, l.spine.ring),
            feel(l.chest.lag, l.chest.ring),
            feel(l.head.lag, l.head.ring),
            feel(l.arms.lag, l.arms.ring),
            feel(l.legs.lag, l.legs.ring),
        ),
        name => format!("Looseness::{}", name.to_uppercase()),
    }
}

fn feel(lag: f32, ring: f32) -> String {
    format!("Feel::new({lag:.2}, {ring:.2})")
}

/// One pose as a chain of named calls, in degrees.
///
/// Only the joints that actually moved are emitted, so a pose that turns the
/// torso and swings one arm is four lines rather than sixteen -- which is the
/// difference between a file a person can read and a wall of zeroes.
pub fn pose_source(p: &Pose, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let mut lines: Vec<String> = Vec::new();
    let root = p.root_offset();
    if root.iter().any(|v| v.abs() > 1e-4) {
        lines.push(format!(
            ".hips({:.3}, {:.3}, {:.3})",
            root[0], root[1], root[2]
        ));
    }
    for j in JOINTS {
        let a = [p.degrees(j, 0), p.degrees(j, 1), p.degrees(j, 2)];
        if a.iter().all(|v| v.abs() < 0.05) {
            continue;
        }
        lines.push(call(j, a));
    }
    if lines.is_empty() {
        return "Pose::rest()".into();
    }
    let mut out = String::from("Pose::rest()");
    for line in lines {
        out.push_str(&format!("\n{pad}    {line}"));
    }
    out
}

fn call(j: Joint, a: [f32; 3]) -> String {
    let three = |name: &str| format!(".{name}({:.1}, {:.1}, {:.1})", a[0], a[1], a[2]);
    match j {
        Joint::Root => three("root"),
        Joint::Spine => three("spine"),
        Joint::Chest => three("chest"),
        Joint::Head => three("head"),
        Joint::ArmL => three("shoulder_l"),
        Joint::ArmR => three("shoulder_r"),
        Joint::HandL => three("wrist_l"),
        Joint::HandR => three("wrist_r"),
        Joint::ThighL => three("hip_l"),
        Joint::ThighR => three("hip_r"),
        Joint::FootL => three("ankle_l"),
        Joint::FootR => three("ankle_r"),
        // Hinges: one number unless the forearm is also rolled.
        Joint::ForearmL if a[2].abs() < 0.05 => format!(".elbow_l({:.1})", a[0]),
        Joint::ForearmR if a[2].abs() < 0.05 => format!(".elbow_r({:.1})", a[0]),
        Joint::ForearmL => format!(".forearm_l({:.1}, {:.1})", a[0], a[2]),
        Joint::ForearmR => format!(".forearm_r({:.1}, {:.1})", a[0], a[2]),
        Joint::ShinL => format!(".knee_l({:.1})", a[0]),
        Joint::ShinR => format!(".knee_r({:.1})", a[0]),
    }
}
