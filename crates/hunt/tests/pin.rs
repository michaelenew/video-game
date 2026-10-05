//! The scripted hunt against the Ridgeback, pinned bit for bit.
//!
//! The same promise `crates/sim/tests/ridgeback_pin.rs` keeps, played by the
//! hunter bot instead of by noise: it climbs, rides, breaks feet and topples
//! the animal, so every path through the creature's code is walked. Each frame
//! is hashed with `World::state_checksum` and folded into one number per seed.
//! See that file for what to do when it fails.

use sim::Class;
use sim::state::MAX_PLAYERS;

fn hashed(class: Class, seed: u32, frames: u32) -> u64 {
    let mut acc: u64 = 0xcbf2_9ce4_8422_2325;
    hunt::play_watched([class; MAX_PLAYERS], 1, frames, seed, |w| {
        acc = (acc ^ w.state_checksum()).wrapping_mul(0x0000_0100_0000_01b3);
    });
    acc
}

const PINNED: [(Class, u32, u64); 7] = [
    (Class::Champion, 0x2545_F491, 0x0efd64f4a51bd0e0),
    (Class::Champion, 7, 0x70a143ee72ab5797),
    // Moved 2026-10-01, deliberately: the class layer throws his shield to
    // close on a window five to eleven metres off, leaps to it and Slams out
    // of the leap (`hunt::class`, `Hands::throw_in`), where it had handed
    // the Ridgeback plan's walk back unchanged. Nothing else in his hunt, and
    // no other pin, moved with it. See feel-log.md.
    (Class::Bulwark, 101, 0x6dacc95234ce6050),
    // Moved 2026-10-01 by the class layer (`hunt::class`): the Dual mage's
    // hands are kept on the side that does not burn her, her finishers are
    // thrown into the windows the plan finds, and she goads her bars between
    // them. The Champion's and the Bulwark's hunts did not move -- the layer
    // hands their plans' input back unchanged here. See feel-log.md.
    (Class::DualMage, 2_222, 0x524327a652326b5c),
    // Added 2026-10-01 with the class layer, so that what it does with the
    // shadow, the fire and the pools is pinned as the plan's moves are.
    (Class::ShadowReaver, 31, 0xb2ab9338138ce5f0),
    (Class::Elementalist, 59, 0x33eb170574ec1780),
    (Class::BloodMage, 83, 0x72ec7ddafe8f9fd9),
];

#[test]
fn the_scripted_ridgeback_hunt_is_bit_identical() {
    let mut wrong = Vec::new();
    for (class, seed, want) in PINNED {
        let got = hashed(class, seed, 12_000);
        println!("    (Class::{class:?}, {seed}, 0x{got:016x}),");
        if got != want {
            wrong.push((class, seed));
        }
    }
    assert!(wrong.is_empty(), "these hunts changed: {wrong:?}");
}
