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
/// Everything, mechanic key and middle click included.
const EVERY_BUTTON: u16 = 0xfff;

/// Play 1200 frames of random input through SyncTest against a given world.
fn synctest(world: World, buttons: u16, label: &str) {
    synctest_with(world, label, 1200, |_, roll| {
        Input::aimed(roll() as u16 & buttons, roll() as u16)
    });
}

/// Play `frames` frames through SyncTest, each seat's input from `input`
/// (the frame, and a random number generator). The world as it ends.
fn synctest_with(
    mut world: World,
    label: &str,
    frames: u32,
    mut input: impl FnMut(u32, &mut dyn FnMut() -> u64) -> Input,
) -> World {
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

    let start = world.frame;
    for frame in 0..frames {
        for handle in 0..2 {
            session
                .add_local_input(handle, NetInput::from(input(frame, &mut next)))
                .expect("add input");
        }
        // A mismatch surfaces here as GgrsError::MismatchedChecksum.
        let requests = session
            .advance_frame()
            .unwrap_or_else(|e| panic!("{label}: desync at frame {frame}: {e}"));
        handle_requests(&mut world, requests);
    }

    assert_eq!(world.frame, start + frames);
    world
}

/// **The valley under rollback**: two fighters walk out of Hearth's gate
/// into the Mouth -- the world moves into the Mouth's coordinates on some
/// frame of it -- with every other button pressed at random, and SyncTest
/// rolls back across the move again and again. A move that was not a pure
/// function of the snapshot would desync on the frame it happens.
#[test]
fn ggrs_synctest_finds_no_desync_walking_from_one_place_into_the_next() {
    let mut w = World::versus_in(
        [Class::Elementalist, Class::ShadowReaver],
        sim::valley::START,
    );
    let zone = sim::valley::place(sim::arena::ArenaId::HEARTH)
        .unwrap()
        .seams[0]
        .zone;
    let mid = zone.middle();
    for p in w.players.iter_mut() {
        p.pos = sim::V3::new(mid.x.sub(sim::Fx::from_int(4)), sim::Fx::ZERO, mid.z);
    }
    let walking = |_: u32, roll: &mut dyn FnMut() -> u64| {
        let random = roll() as u16 & (Input::LEFT | Input::RIGHT | Input::SPACE | Input::MECHANIC);
        Input::looking_at(Input::W | random, 0, -1500)
    };
    let w = synctest_with(w, "the valley's gate", 900, walking);
    assert_eq!(
        w.arena,
        sim::arena::ArenaId::MOUTH,
        "never got out of the town"
    );
}

/// The same for a room: walking in starts the hunt, under rollback.
#[test]
fn ggrs_synctest_finds_no_desync_walking_into_a_room() {
    let mut w = World::arrive(
        [Class::Bulwark, Class::Champion],
        sim::valley::Journey::default(),
        sim::arena::ArenaId::PINEWOOD,
        Some(3),
        2,
    );
    let zone = sim::valley::place(sim::arena::ArenaId::PINEWOOD)
        .unwrap()
        .seams[3]
        .zone;
    let mid = zone.middle();
    let floor = w
        .terrain()
        .ground_under(sim::V3::new(mid.x, zone.max.y, mid.z));
    for p in w.players.iter_mut() {
        p.pos = sim::V3::new(mid.x, floor, mid.z);
    }
    let south = 3 * Input::QUARTER_TURN;
    let walking = |_: u32, roll: &mut dyn FnMut() -> u64| {
        let random = roll() as u16 & (Input::LEFT | Input::RIGHT);
        Input::looking_at(Input::W | random, south, -1500)
    };
    let w = synctest_with(w, "the Highlands' door", 600, walking);
    assert_eq!(
        w.arena,
        sim::arena::ArenaId::HIGHLANDS,
        "never got into the room"
    );
    assert!(w.hunting(), "walked in and nothing to hunt");
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
