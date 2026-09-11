//! The Oven: the live tuning store and its bake.
//!
//! These live in their own test binary on purpose. Cargo runs each target as a
//! separate process, so the mutating tests below cannot reach the store that
//! `combat.rs` and `feel.rs` are reading — a global that tests mutate is a
//! source of flakes that only show up under load, which is the worst kind.

use sim::class::{ALL_CLASSES, Class};
use sim::oven::{self, AirField, Knob, MoveField, Scalar, Unit};
use sim::{Input, World};

#[test]
fn the_registry_covers_every_stored_value() {
    // If these ever disagree, `emit` writes arrays of the wrong length and the
    // generated file stops compiling -- which is a good failure, but a worse
    // error message than this one.
    assert_eq!(
        oven::all_knobs().len(),
        oven::SCALAR_COUNT + oven::AIR_COUNT + oven::MOVE_COUNT
    );
    assert_eq!(Scalar::ALL.len(), oven::SCALAR_COUNT);
    assert_eq!(AirField::ALL.len() * 6, oven::AIR_COUNT);
    assert_eq!(MoveField::ALL.len() * 6 * oven::SLOTS, oven::MOVE_COUNT);
}

#[test]
fn every_knob_has_a_unique_id() {
    // Ids are what search matches and what labels each line of the baked file.
    // Two knobs sharing one would make a diff ambiguous about which number moved.
    let knobs = oven::all_knobs();
    let mut ids: Vec<String> = knobs.iter().map(|k| k.id()).collect();
    ids.sort();
    let before = ids.len();
    ids.dedup();
    assert_eq!(before, ids.len(), "duplicate knob id");
}

#[test]
fn every_knob_starts_inside_its_own_range() {
    // A value outside its slider range means the range is wrong, since the
    // committed value is by definition one someone chose.
    for knob in oven::all_knobs() {
        let (lo, hi) = knob.range();
        let v = knob.raw();
        assert!(
            v >= lo && v <= hi,
            "{} is {} but its range is {lo}..={hi}",
            knob.id(),
            v
        );
    }
}

#[test]
fn the_committed_file_is_what_the_oven_would_write() {
    // Catches a hand-edited `tuned.rs`, and catches a bake that wrote the file
    // but did not get committed. Either one leaves the repository saying
    // something the game does not do.
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/tuned.rs");
    let on_disk = std::fs::read_to_string(path).expect("tuned.rs is readable");
    let wanted = oven::emit();
    if on_disk != wanted {
        // Report the first difference rather than both files: a whole-file diff
        // of three hundred lines in an assertion message helps nobody.
        let first = on_disk
            .lines()
            .zip(wanted.lines())
            .position(|(a, b)| a != b)
            .unwrap_or(on_disk.lines().count().min(wanted.lines().count()));
        panic!(
            "tuned.rs is out of step with the Oven at line {} -- run \
             `cargo run -p sim --bin bake_tuning`\n  on disk: {:?}\n  wanted:  {:?}",
            first + 1,
            on_disk.lines().nth(first),
            wanted.lines().nth(first)
        );
    }
}

#[test]
fn nothing_is_dirty_before_anything_is_touched() {
    assert!(!oven::is_dirty());
    for knob in oven::all_knobs() {
        assert!(!knob.is_dirty(), "{} starts dirty", knob.id());
    }
}

#[test]
fn fixed_point_values_display_without_floating_point() {
    // The Oven lives inside `sim`, which is float-free as a determinism
    // guarantee. Showing a value to a person must not be the thing that breaks
    // it, so the conversion is integer arithmetic and this checks it is right.
    assert_eq!(Unit::Fixed.show(65536), "1");
    assert_eq!(Unit::Fixed.show(32768), "0.5");
    assert_eq!(Unit::Fixed.show(-65536), "-1");
    assert_eq!(Unit::Fixed.show(458752), "7");
    assert_eq!(Unit::Frames.show(14), "14");
    assert_eq!(Unit::Flag.show(0), "off");
}

/// Everything that writes to the store, in one test.
///
/// Tests inside a binary run in parallel, so a second mutating test would race
/// this one. One test that puts the store back when it is done is simpler than
/// a lock, and cannot be forgotten.
#[test]
fn editing_the_oven_changes_the_game_and_can_be_undone() {
    let knob = Knob::Move(Class::Bulwark, 0, MoveField::Startup);
    let original = knob.raw();

    // The value the simulation reads is the value the palette writes.
    knob.set_raw(original + 9);
    assert_eq!(
        sim::moves::get(Class::Bulwark, 0).startup as i32,
        original + 9
    );
    assert!(knob.is_dirty() && oven::is_dirty());

    // And it actually changes play: a slower poke takes longer to become active.
    let frames_to_active = |_: ()| {
        let mut w = World::new();
        for f in 0..60u32 {
            w.advance([Input::new(Input::LEFT), Input::default()]);
            if matches!(w.players[0].action, sim::state::Action::Active { .. }) {
                return f;
            }
        }
        u32::MAX
    };
    let slow = frames_to_active(());
    oven::reset_to_baked();
    let quick = frames_to_active(());
    assert!(
        slow > quick,
        "adding nine frames of startup did not delay the hitbox: {slow} against {quick}"
    );

    assert!(!oven::is_dirty(), "reset left the store dirty");
    assert_eq!(knob.raw(), original);
}

#[test]
fn the_tuning_is_part_of_the_desync_checksum() {
    // Tuning is a rule rather than state, so rollback never carries it. Two
    // peers tuned differently would otherwise diverge silently and look like a
    // netcode bug; this is what turns that into an immediate desync.
    let w = World::new();
    let before = w.checksum();

    let knob = Knob::Scalar(Scalar::MoveSpeed);
    let original = knob.raw();
    knob.set_raw(original + 1000);
    let after = w.checksum();
    knob.set_raw(original);

    assert_ne!(
        before, after,
        "changing the tuning left the checksum alone, so a mistuned peer would desync silently"
    );
    assert_eq!(before, w.checksum(), "checksum did not come back");
}

#[test]
fn families_group_the_way_a_person_would_look_for_them() {
    // Three hundred numbers need families to be navigable at all. Each move
    // gets its own, named for the move, because "Bulwark · Bash" is how someone
    // asks for it.
    let knobs = oven::all_knobs();
    let families: Vec<String> = knobs.iter().map(|k| k.family()).collect();
    for expected in ["Movement", "Air", "Defence", "Match", "Bulwark · Bash"] {
        assert!(
            families.iter().any(|f| f == expected),
            "no family called {expected}"
        );
    }
    for class in ALL_CLASSES {
        assert!(
            families
                .iter()
                .any(|f| f == &format!("Air · {}", class.name())),
            "{} has no air family",
            class.name()
        );
    }
}
