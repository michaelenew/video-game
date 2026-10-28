//! **The Oven moved in a match, under rollback.** SyncTest rolls every frame
//! back and plays it again, comparing checksums: here both players keep
//! moving knobs -- walk speed, gravity, a jump -- on their inputs, so frames
//! are run again on both sides of each edit, again and again. An edit that
//! was not a pure function of the snapshot and the inputs would be a
//! mismatch on the frame after it.
//!
//! In a file of its own, as `sim/tests/match_edits.rs` is: the Oven's store
//! is the process's, and a world run beside these on another thread would
//! run on their rules.

use ggrs::{PlayerType, SessionBuilder};
use net::{NetInput, SessionConfig, handle_requests};
use sim::input::Tune;
use sim::oven::{Knob, Scalar};
use sim::{Class, Input, World};

#[test]
fn knobs_moved_in_a_match_survive_every_rollback() {
    // The wire carries an edit exactly, negative values too.
    let tune = Tune::set(Knob::Scalar(Scalar::Gravity).cell(), -123_456);
    let input = Input::aimed(Input::W, 777).tuning(tune);
    assert_eq!(Input::from(NetInput::from(input)), input);

    let knobs = [
        Knob::Scalar(Scalar::MoveSpeed),
        Knob::Scalar(Scalar::Gravity),
        Knob::Scalar(Scalar::JumpSpeed),
    ];
    let baked: Vec<i32> = knobs.iter().map(|k| k.raw()).collect();
    let mut world = World::with_classes([Class::Champion, Class::ShadowReaver]);
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
    for frame in 0..1_200u32 {
        for handle in 0..2 {
            let r = next();
            let mut input = Input::aimed(r as u16 & 0x1ff, (r >> 16) as u16);
            // Now and then, a knob: one of three, somewhere between half
            // and double what is baked.
            if (r >> 40) % 23 == 0 {
                let k = ((r >> 48) % 3) as usize;
                let scale = 50 + ((r >> 52) % 151) as i64;
                let raw = (baked[k] as i64 * scale / 100) as i32;
                input = input.tuning(Tune::set(knobs[k].cell(), raw));
            }
            session
                .add_local_input(handle, NetInput::from(input))
                .expect("add input");
        }
        let requests = session
            .advance_frame()
            .unwrap_or_else(|e| panic!("desync at frame {frame}: {e}"));
        handle_requests(&mut world, requests);
    }
    assert_eq!(world.frame, 1_200);
    let moved = world.edits.iter().count();
    assert!(
        moved >= 2,
        "only {moved} knobs moved: the test proves little"
    );
}
