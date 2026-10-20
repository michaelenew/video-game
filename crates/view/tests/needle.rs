//! The follow camera on top of the Needle: two things a replay of a climb
//! found (2026-10-20), as sentences.

use sim::arena::{ArenaId, spire};
use sim::fixed::Fx;
use sim::{Class, Input, V3, World};
use view::camera::{RigConfig, Zones};
use view::{CameraRig, Framing, Surroundings};

const SEATS: [Class; sim::state::MAX_PLAYERS] = [Class::Champion; sim::state::MAX_PLAYERS];

/// The roost: the Needle's dome as a place of its own, its floor the dome's
/// top -- arrived at the Needle and stood on the ruins, which is how the
/// world comes to be in it, with its Galewing awake.
fn roost() -> World {
    let j = sim::valley::Journey {
        beaten: u32::MAX,
        ..sim::valley::Journey::default()
    };
    let mut w = World::arrive(SEATS, j, ArenaId::SPIRE, Some(0), 1);
    let cm = sim::arena::cm;
    w.players[0].pos = V3::new(cm(spire::AT.0 + 1000), cm(spire::TOP + 50), cm(spire::AT.1));
    for _ in 0..30 {
        w.advance([Input::default(); sim::state::MAX_PLAYERS]);
    }
    assert_eq!(
        w.arena,
        sim::valley::ranges::RANGES[5].arena,
        "on the roost"
    );
    assert!(w.monsters.iter().any(|m| m.is_some()), "with its Galewing");
    w
}

/// The rig, settled on a fighter held at `at`, looking `yaw` at `pitch`.
fn settled(w: &World, at: [f32; 3], yaw: f32, pitch: f32) -> Framing {
    let mut rig = CameraRig::new(RigConfig::default());
    let around = Surroundings {
        beasts: &w.monsters,
        arena: w.terrain(),
        ..Default::default()
    };
    for _ in 0..240 {
        rig.update_around(0.016, at, yaw, pitch, around);
    }
    rig.update_around(0.016, at, yaw, pitch, around)
}

fn arm(f: &Framing, at: [f32; 3]) -> f32 {
    let focus = [at[0], at[1] + RigConfig::default().look_height, at[2]];
    (0..3)
        .map(|i| (f.eye[i] - focus[i]).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// **A round solid is round.** The dome is a stack of rings ninety metres
/// across, each a disc in a square box, and the arm was tested against the
/// box: a fighter on the ledge in the corner of the top ring's box, outside
/// the dome itself, had the camera pulled into the back of their head
/// whenever the arm swung back over the box -- "wouldn't zoom out at all
/// when facing a certain direction".
#[test]
fn the_corner_of_a_round_solids_box_is_air() {
    let w = roost();
    // The ledge the replay stood on: two metres under the dome's top, out
    // past its rim -- and inside the corner of its top ring's box.
    let at = [37.2f32, -2.0, 44.3];
    let r = (at[0] * at[0] + at[2] * at[2]).sqrt();
    let rim = spire::RIM as f32 / 100.0;
    assert!(r > rim + 1.0 && at[0] < rim && at[2] < rim);
    let sphere = Zones::tuned().sphere;
    // Looking out over the town, so the arm swings back over the box's
    // corner and not the dome: what the replay could not get out of.
    for yaw_deg in [20.0f32, 35.0, 50.0, 65.0] {
        let f = settled(&w, at, yaw_deg.to_radians(), -0.35);
        assert!(
            arm(&f, at) > sphere * 0.6,
            "looking {yaw_deg} degrees off the ledge the arm is {:.2} m of {sphere:.2}",
            arm(&f, at)
        );
    }
}

/// **The floor is the terrain's, not zero.** A place's floor is at zero in its
/// own coordinates only, and the roost's is the Needle's dome: a fighter who
/// fell off it was left behind by a camera resting on nought -- on the
/// Galewing's "top under the eye", which said zero with no part of the bird
/// there -- while they fell three hundred metres to the town.
#[test]
fn a_fall_off_the_dome_is_followed() {
    let w = roost();
    let t = w.terrain();
    let half = spire::RIM as f32 / 100.0;
    // Out past the rim, a hundred metres under the dome's top.
    let (x, z) = (half * 1.3, half * 1.3);
    let ground = t.relief_at(Fx::from_int(x as i32), Fx::from_int(z as i32));
    assert!(
        ground.to_f32_for_render() < -200.0,
        "the floor out past the rim is the town's, far under the dome"
    );
    let at = [x, -100.0, z];
    for pitch in [-1.2f32, -0.3, 0.0] {
        let f = settled(&w, at, 0.0, pitch);
        assert!(
            arm(&f, at) < 12.0,
            "falling past the dome at pitch {pitch} the eye is {:.1} m away, at {:?}",
            arm(&f, at),
            f.eye
        );
    }
}
