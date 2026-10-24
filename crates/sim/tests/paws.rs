//! **Every leg folds twice, and the paw stays flat** (the owner, 2026-10-24:
//! "basically all animals have two joints in the legs").
//!
//! Every creature that walks on legs has a bone below each leg's second
//! joint -- a wrist, an ankle, an insect's tarsus -- marked
//! `sim::beast::Bone::paw`, and the rig takes the pitch of everything above
//! it back off so it lies level however the leg swings. Checked here on every
//! baked frame of every species: a clip that keys no turn of its own into a
//! paw leaves it level. Roll is left out of it -- a creature on its side
//! carries its paws over with it, which is what a toppled animal does -- so
//! the frames are those whose body is not rolled.

use sim::Fx;
use sim::V3;
use sim::beast::{Pose, Rig};

/// Degrees a paw may tilt off level while the body is upright: what yawing
/// a leg out to the side adds, since only pitch is taken back off.
const SLACK: f64 = 6.0;

fn degrees(raw: i32) -> f64 {
    raw as f64 / 65536.0 * 360.0
}

#[test]
fn every_legged_creature_has_a_second_joint_in_every_leg() {
    for sp in sim::species::all() {
        for leg in sp.legs {
            // The leg's bones: from the knee up to the body (a bone on a
            // side is a limb's), and whatever hangs below the knee.
            let mut bones = 0;
            let mut b = leg.knee;
            while b != sim::beast::NO_PARENT && sp.bones[b].side != 0 {
                bones += 1;
                b = sp.bones[b].parent;
            }
            bones += (0..sp.bones.len())
                .filter(|&c| sp.bones[c].parent == leg.knee)
                .count();
            assert!(
                bones >= 3,
                "{}: the leg through {} has {bones} bones, one joint short",
                sp.name,
                sp.bones[leg.knee].name
            );
        }
    }
}

#[test]
fn a_paw_stays_level_while_the_body_is_upright() {
    let mut wrong = Vec::new();
    let mut checked = 0;
    for sp in sim::species::all() {
        let paws: Vec<usize> = (0..sp.bones.len()).filter(|&b| sp.bones[b].paw).collect();
        if paws.is_empty() {
            continue;
        }
        for r in 0..sp.rows {
            let row = (sp.row)(r);
            let pose = Pose::from_channels(row);
            // Upright: nothing on the path from any paw to the root rolled.
            let rolled = (0..sp.bones.len()).any(|b| degrees(row[5 + 3 * b]).abs() > 1.0);
            if rolled || paws.iter().any(|&p| row[3 + 3 * p] != 0) {
                continue;
            }
            checked += 1;
            let rig = Rig::build(sp, V3::ZERO, Fx::ZERO, &pose);
            for &p in &paws {
                let fwd = rig.bone[p].rot.apply(V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
                let tilt = (fwd.y.raw() as f64 / 65536.0)
                    .clamp(-1.0, 1.0)
                    .asin()
                    .to_degrees();
                if tilt.abs() > SLACK {
                    wrong.push(format!(
                        "{} row {r}: {} at {tilt:.1}",
                        sp.name, sp.bones[p].name
                    ));
                }
            }
        }
    }
    eprintln!("{checked} upright frames checked");
    assert!(checked > 1000, "only {checked} upright frames to check");
    assert!(wrong.is_empty(), "paws off level:\n{}", wrong.join("\n"));
}
