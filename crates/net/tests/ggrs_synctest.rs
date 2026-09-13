//! GGRS SyncTest.
//!
//! SyncTest replays the game locally with forced rollbacks and compares GGRS's
//! own checksums across the re-simulations. It is stricter than the hand-rolled
//! harness in `rollback.rs` and it is the standard way to catch
//! non-determinism, so it is the test that should fail first if the simulation
//! ever stops being a pure function.

use ggrs::{PlayerType, SessionBuilder};
use net::{NetInput, SessionConfig, handle_requests};
use sim::{Class, Input, World};

/// Which buttons a run is allowed to press.
///
/// The first nine bits: the clicks, the special, shift, WASD and space. It is
/// what the first version of this test used and it leaves out crouch and the
/// mechanic key.
const COMMON_BUTTONS: u16 = 0x1ff;
/// Everything, mechanic key included.
const EVERY_BUTTON: u16 = 0x7ff;

/// Play 1200 frames of random input through SyncTest against a given world.
fn synctest(mut world: World, buttons: u16, label: &str) {
    let mut session = SessionBuilder::<SessionConfig>::new()
        .with_num_players(2)
        .with_check_distance(7)
        .add_player(PlayerType::Local, 0)
        .expect("add player 0")
        .add_player(PlayerType::Local, 1)
        .expect("add player 1")
        .start_synctest_session()
        .expect("synctest session");

    let mut rng = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };

    for frame in 0..1200u32 {
        for handle in 0..2 {
            session
                .add_local_input(
                    handle,
                    NetInput::from(Input::aimed(next() as u16 & buttons, next() as u16)),
                )
                .expect("add input");
        }
        // A mismatch surfaces here as GgrsError::MismatchedChecksum.
        let requests = session
            .advance_frame()
            .unwrap_or_else(|e| panic!("{label}: desync at frame {frame}: {e}"));
        handle_requests(&mut world, requests);
    }

    assert_eq!(world.frame, 1200);
}

#[test]
fn ggrs_synctest_finds_no_desync() {
    synctest(World::new(), COMMON_BUTTONS, "versus");
}

#[test]
fn ggrs_synctest_finds_no_desync_with_a_creature_in_the_arena() {
    // The creature is the most determinism-hostile thing in the simulation:
    // it carries a generator, it decides, and riders keep a position in a
    // rotating frame that is rebuilt from the snapshot every tick. If any of
    // that ever stops being a pure function, this is where it shows up.
    synctest(
        World::hunt([Class::Champion, Class::Bulwark]),
        COMMON_BUTTONS,
        "hunt",
    );
}

#[test]
fn ggrs_synctest_finds_no_desync_with_things_in_flight() {
    // The Blood mage is the only class with effects that *move*, and they move
    // in the one way that makes rollback safe: their position is worked out
    // from the frame they were cast on and the frame it is now, never
    // integrated. A blade caught halfway through a re-simulation has to land in
    // exactly the same place it did the first time, and so does the bank of
    // health it is carrying home.
    // **Every** button, unlike the two above: her `E` is an ability rather than
    // a state change, and a run that never presses it never casts the one thing
    // in the game that puts a spike in the ground.
    synctest(
        World::with_classes([Class::BloodMage; 2]),
        EVERY_BUTTON,
        "blood",
    );
}
