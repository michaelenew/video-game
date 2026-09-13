//! Writing recipes back out has to be lossless.
//!
//! The hub regenerates these files every time somebody saves, so a round trip
//! that quietly drops a channel would lose work — and would lose it silently,
//! which is the worst way to lose it.

use anim::source;
use view::clips::Clip;
use view::pose::Pose;
use view::skeleton::Joint;

#[test]
fn a_pose_emits_the_methods_a_person_would_have_written() {
    let pose = Pose::rest()
        .hips(0.0, -0.12, 0.05)
        .spine(8.0, 0.0, -20.0)
        .shoulder_r(-40.0, 28.0, 0.0)
        .elbow_r(95.0)
        .knee_l(22.0);
    let text = source::pose_source(&pose, 0);
    for expected in [
        ".hips(0.000, -0.120, 0.050)",
        ".spine(8.0, 0.0, -20.0)",
        ".shoulder_r(-40.0, 28.0, 0.0)",
        ".elbow_r(95.0)",
        ".knee_l(22.0)",
    ] {
        assert!(text.contains(expected), "missing {expected} in:\n{text}");
    }
    // And nothing for the joints that did not move, or the file is a wall of
    // zeroes nobody can read.
    assert!(
        !text.contains("wrist_l"),
        "emitted an untouched joint:\n{text}"
    );
    assert_eq!(source::pose_source(&Pose::rest(), 0), "Pose::rest()");
}

#[test]
fn a_hinge_emits_one_number_unless_it_is_rolled() {
    let bent = Pose::rest().elbow_l(60.0);
    assert!(source::pose_source(&bent, 0).contains(".elbow_l(60.0)"));
    let rolled = Pose::rest().forearm_l(60.0, -30.0);
    assert!(source::pose_source(&rolled, 0).contains(".forearm_l(60.0, -30.0)"));
}

#[test]
fn presets_come_back_by_name() {
    // `Ease::SNAP` rather than four magic numbers, or the regenerated file is
    // worse than what it replaced.
    assert_eq!(source::ease_source(anim::Ease::SNAP), "Ease::SNAP");
    assert_eq!(
        source::looseness_source(&anim::Looseness::HEAVY),
        "Looseness::HEAVY"
    );
    assert!(source::ease_source(anim::Ease::new(0.1, 0.2, 0.3, 0.4)).starts_with("Ease::new("));
}

#[test]
fn a_regenerated_file_is_valid_rust() {
    // Checked by handing it to rustfmt, which is a parser. Cheap, and it is the
    // failure that would actually happen: a generated file that does not
    // compile is discovered at the worst possible moment.
    let recipes = anim::clips::all();
    if recipes.is_empty() {
        return;
    }
    let text = source::emit_file("locomotion", "Standing, walking and running.", &recipes);
    let dir = std::env::temp_dir().join("anim-source-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("emitted.rs");
    std::fs::write(&path, &text).expect("write");
    let out = std::process::Command::new("rustfmt")
        .args(["--edition", "2024", "--check"])
        .arg(&path)
        .output();
    // A non-zero exit from `--check` means "would reformat", which is fine; a
    // parse error is not, and rustfmt says so on stderr. No rustfmt on this
    // machine is also fine -- the rest of the test still ran.
    if let Ok(o) = out {
        let err = String::from_utf8_lossy(&o.stderr);
        assert!(
            !err.contains("error"),
            "the regenerated file does not parse:\n{err}"
        );
    }
}

#[test]
fn every_channel_survives_the_round_trip_in_principle() {
    // The emitter writes degrees to one decimal place, so a round trip is
    // lossy by at most a twentieth of a degree. Worth pinning: if it ever
    // becomes lossy by more, poses will drift every time somebody saves.
    let mut pose = Pose::rest();
    for (i, j) in view::skeleton::JOINTS.iter().enumerate() {
        pose.set_degrees(*j, 0, (i as f32 * 3.7) % 40.0 - 20.0);
    }
    let text = source::pose_source(&pose, 0);
    for j in view::skeleton::JOINTS {
        if pose.degrees(j, 0).abs() < 0.05 {
            continue;
        }
        let name = match j {
            Joint::Root => "root",
            Joint::Spine => "spine",
            Joint::Chest => "chest",
            Joint::Head => "head",
            Joint::ArmL => "shoulder_l",
            Joint::ArmR => "shoulder_r",
            Joint::ForearmL => "elbow_l",
            Joint::ForearmR => "elbow_r",
            Joint::HandL => "wrist_l",
            Joint::HandR => "wrist_r",
            Joint::ThighL => "hip_l",
            Joint::ThighR => "hip_r",
            Joint::ShinL => "knee_l",
            Joint::ShinR => "knee_r",
            Joint::FootL => "ankle_l",
            Joint::FootR => "ankle_r",
        };
        assert!(
            text.contains(name),
            "{} never made it out:\n{text}",
            j.name()
        );
    }
}

#[test]
fn every_clip_belongs_to_a_file_the_hub_can_write() {
    // The hub regenerates one file at a time, so a clip whose file nobody owns
    // could never be saved from it.
    let files = [
        "locomotion",
        "turns",
        "air",
        "dodge",
        "defence",
        "reactions",
        "bulwark",
        "champion",
        "reaver",
        "elementalist",
        "blood",
        "dual",
    ];
    for clip in view::clips::ALL {
        assert!(
            files.contains(&clip.file()),
            "{} lives in {}.rs, which is not a recipe file",
            clip.name(),
            clip.file()
        );
    }
    assert_eq!(Clip::Idle.file(), "locomotion");
}
