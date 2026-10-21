//! **A middle moves with its ends** (2026-10-21). The middle of a box was
//! written as half of each corner added together, which rounds away the odd
//! bit of both: moved by a change of the world's frame (`World::shift`), the
//! middle of a round column landed a sixty-thousandth from where the shift
//! put it, and the Caverns' Ridgebacks against it were pushed differently in
//! each frame (`tests/valley.rs`, `a_change_of_frame_changes_nothing_in_a_hunt`).
//! `sim::math::midpoint` is the one way to take a middle of two places.

use sim::fixed::Fx;
use sim::math::midpoint;
use std::path::{Path, PathBuf};

#[test]
fn a_midpoint_moves_exactly_with_its_ends() {
    let ends = [(3, 5), (-7, 11), (1, 1), (-65_537, 131_073), (99_999, -3)];
    let shifts = [1, -1, 3, 65_535, -1_234_567, 7_777_777];
    for (a, b) in ends {
        let (a, b) = (Fx::from_raw(a), Fx::from_raw(b));
        let m = midpoint(a, b);
        for d in shifts {
            let d = Fx::from_raw(d);
            assert_eq!(
                midpoint(a.add(d), b.add(d)),
                m.add(d),
                "the middle of {a:?} and {b:?}, moved by {d:?}"
            );
        }
    }
}

fn sim_sources() -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    let mut stack = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read sim/src") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let text = std::fs::read_to_string(&path).expect("read source");
                out.push((path, text));
            }
        }
    }
    out
}

#[test]
fn nothing_takes_a_middle_by_halving_each_end() {
    let mut found = Vec::new();
    for (path, text) in sim_sources() {
        for (n, line) in text.lines().enumerate() {
            if line.contains(".raw() / 2 + ") {
                found.push(format!("{}:{}  {}", path.display(), n + 1, line.trim()));
            }
        }
    }
    assert!(
        found.is_empty(),
        "a middle taken by halving each end, which a change of frame moves by a rounding: \
         use `sim::math::midpoint`.\n{}",
        found.join("\n")
    );
}
