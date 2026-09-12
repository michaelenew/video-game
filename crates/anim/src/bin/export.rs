//! Write the whole animation set out as JSON, for viewers that are not the game.
//!
//!     cargo run -p anim --bin export -- docs/preview/anim.json
//!
//! The game plays these clips by looking up baked joint angles and running
//! forward kinematics; so can anything else, given the same two tables. This
//! writes both -- the six class skeletons and every baked frame -- so a plain
//! web page can show the animations to somebody who has not got a Rust
//! toolchain in front of them, which is most people whose opinion is worth
//! having on whether a walk looks like walking.
//!
//! Angles go out in degrees rounded to two places and the hip offset in
//! millimetres, because the file is read by a person as often as by a program
//! and neither needs eight decimals of a radian.

use std::fmt::Write as _;
use view::clips::{ALL, Clip};
use view::skeleton::{JOINTS, Joint, skeleton_for};

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/preview/anim.json".into());
    let mut s = String::new();

    s.push_str("{\n\"joints\":[");
    for (i, j) in JOINTS.iter().enumerate() {
        let parent = j.parent().map_or(-1, |p| p.index() as i32);
        let comma = if i + 1 == JOINTS.len() { "" } else { "," };
        let _ = write!(
            s,
            "{{\"name\":\"{}\",\"parent\":{parent},\"left\":{}}}{comma}",
            j.name(),
            j.is_left()
        );
    }
    s.push_str("],\n\"classes\":[");
    for (ci, class) in sim::class::ALL_CLASSES.iter().enumerate() {
        let sk = skeleton_for(*class);
        let comma = if ci + 1 == sim::class::ALL_CLASSES.len() {
            ""
        } else {
            ","
        };
        let _ = write!(
            s,
            "\n{{\"name\":\"{}\",\"scale\":{:.4},\"hip\":{:.4},\"height\":{:.4},\"bones\":[",
            class.name(),
            sk.scale,
            sk.hip_height,
            sk.total_height
        );
        for (bi, j) in JOINTS.iter().enumerate() {
            let b = sk.bone(*j);
            let tail = if bi + 1 == JOINTS.len() { "" } else { "," };
            let _ = write!(
                s,
                "{{\"offset\":{},\"axis\":{},\"length\":{:.4},\"half\":{},\"boxAt\":{},\"swingSign\":{},\"limits\":[{},{},{}]}}{tail}",
                v3(b.offset),
                v3(b.axis),
                b.length,
                v3(b.half),
                v3(b.box_at),
                b.swing_sign,
                pair(b.limits.swing),
                pair(b.limits.spread),
                pair(b.limits.twist),
            );
        }
        let _ = write!(s, "]}}{comma}");
    }

    s.push_str("],\n\"clips\":[");
    for (i, clip) in ALL.iter().enumerate() {
        let frames = view::baked::TABLE[i];
        let comma = if i + 1 == ALL.len() { "" } else { "," };
        let phases = match clip.phases() {
            Some((a, b, c)) => format!("[{a},{b},{c}]"),
            None => "null".into(),
        };
        let _ = write!(
            s,
            "\n{{\"name\":\"{}\",\"family\":\"{}\",\"file\":\"{}\",\"looping\":{},\"phases\":{phases},\"what\":{},\"frames\":[",
            clip.name(),
            clip.family().name(),
            clip.file(),
            clip.looping(),
            quoted(clip.what()),
        );
        for (fi, pose) in frames.iter().enumerate() {
            if fi > 0 {
                s.push(',');
            }
            s.push('[');
            let r = pose.root_offset();
            // Millimetres for the hips, degrees for everything else.
            let _ = write!(
                s,
                "{},{},{}",
                round(r[0] * 1000.0, 1),
                round(r[1] * 1000.0, 1),
                round(r[2] * 1000.0, 1)
            );
            for j in JOINTS {
                let a = pose.angles(j);
                for v in a {
                    let _ = write!(s, ",{}", round(v.to_degrees(), 2));
                }
            }
            s.push(']');
        }
        let _ = write!(s, "]}}{comma}");
    }
    s.push_str("]\n}\n");

    if let Some(dir) = std::path::Path::new(&out).parent() {
        std::fs::create_dir_all(dir).expect("make the folder");
    }
    std::fs::write(&out, &s).expect("write the json");

    // And the same bytes as a script that hands them to a global. A page
    // served from a strict content policy can load a script from its own
    // origin when it cannot always fetch a file, and the viewer in this folder
    // has to work in both places.
    let js = std::path::Path::new(&out).with_file_name("anim-data.js");
    std::fs::write(&js, format!("window.ANIM = {s};\n")).expect("write the script");
    println!(
        "{out}  {} clips, {} classes, {:.0} KB",
        ALL.len(),
        sim::class::ALL_CLASSES.len(),
        s.len() as f32 / 1024.0
    );
}

/// Trailing zeroes are a third of the file, and nothing reads them.
fn round(v: f32, places: i32) -> String {
    let k = 10f32.powi(places);
    let r = (v * k).round() / k;
    let r = if r == 0.0 { 0.0 } else { r };
    let mut t = format!("{r}");
    if t.ends_with(".0") {
        t.truncate(t.len() - 2);
    }
    t
}

fn v3(v: [f32; 3]) -> String {
    format!(
        "[{},{},{}]",
        round(v[0], 4),
        round(v[1], 4),
        round(v[2], 4)
    )
}

fn pair(v: (f32, f32)) -> String {
    format!(
        "[{},{}]",
        round(v.0.to_degrees(), 1),
        round(v.1.to_degrees(), 1)
    )
}

fn quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[allow(dead_code)]
fn unused(_: Joint, _: Clip) {}
