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

const PINNED: [(Class, u32, u64); 4] = [
    (Class::Champion, 0x2545_F491, 0x0efd64f4a51bd0e0),
    (Class::Champion, 7, 0x70a143ee72ab5797),
    (Class::Bulwark, 101, 0x1b707320b9c4af07),
    (Class::DualMage, 2_222, 0x9a10dc2ea84d2f17),
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
