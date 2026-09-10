//! GGRS SyncTest.
//!
//! SyncTest replays the game locally with forced rollbacks and compares GGRS's
//! own checksums across the re-simulations. It is stricter than the hand-rolled
//! harness in `rollback.rs` and it is the standard way to catch
//! non-determinism, so it is the test that should fail first if the simulation
//! ever stops being a pure function.

use ggrs::{PlayerType, SessionBuilder};
use net::{NetInput, SessionConfig, handle_requests};
use sim::World;

#[test]
fn ggrs_synctest_finds_no_desync() {
    let mut session = SessionBuilder::<SessionConfig>::new()
        .with_num_players(2)
        .with_check_distance(7)
        .add_player(PlayerType::Local, 0)
        .expect("add player 0")
        .add_player(PlayerType::Local, 1)
        .expect("add player 1")
        .start_synctest_session()
        .expect("synctest session");

    let mut world = World::new();
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
                .add_local_input(handle, NetInput((next() & 0x1ff) as u16))
                .expect("add input");
        }
        // A mismatch surfaces here as GgrsError::MismatchedChecksum.
        let requests = session
            .advance_frame()
            .unwrap_or_else(|e| panic!("desync at frame {frame}: {e}"));
        handle_requests(&mut world, requests);
    }

    assert_eq!(world.frame, 1200);
}
