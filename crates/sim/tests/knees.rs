//! **A foreleg's knee bends one way** (the owner, 2026-10-23: "the forelegs
//! bend both ways at the knee right now").
//!
//! The joint halfway down a creature's foreleg is its elbow: the walk, the
//! gallop and every layer the simulation adds over a clip -- a sagging leg,
//! a lame one, a body getting up a ledge -- fold it with the foot swinging
//! forward and the point of the joint going back. Five of the Ridgeback's
//! clips once keyed it the other way, so from one move to the next the leg
//! hinged forwards and backwards like nothing that has bones. A baked clip
//! is checked here, frame by frame; the layers are checked by their own
//! signs (`monster::Monster::lame_pose`, `climb::fold`).

use sim::species::ridgeback::SPECIES;

/// How far past straight the other way a knee may go, in degrees: none, but
/// for the spring the bake's settling adds.
const SLACK: f64 = 2.0;

#[test]
fn no_baked_clip_bends_a_foreleg_backward() {
    let sp = &SPECIES;
    let mut wrong = Vec::new();
    for (clip, &(first, count)) in sp.span.iter().enumerate() {
        for r in first as usize..(first + count) as usize {
            let row = (sp.row)(r);
            for leg in sp.legs.iter().filter(|l| l.front) {
                let raw = row[3 + 3 * leg.knee];
                let degrees = raw as f64 / 65536.0 * 360.0;
                if degrees < -SLACK {
                    wrong.push(format!(
                        "clip {clip} row {r}: {} at {degrees:.1}",
                        sp.bones[leg.knee].name
                    ));
                }
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "forelegs bent the wrong way at the knee:\n{}",
        wrong.join("\n")
    );
}
