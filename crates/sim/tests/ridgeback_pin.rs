//! The Ridgeback, pinned bit for bit.
//!
//! When the creature code became species-generic (see
//! `docs/design/species.md`), the promise was that the Ridgeback came out of it
//! **bit-identical**: the same seeds, the same frames, the same numbers. This
//! is the check that keeps it so. A hunt driven by a fixed stream of inputs is
//! hashed every frame with `World::state_checksum` -- the snapshot, with the
//! Oven left out, because where a knob is stored is not the fight -- and the
//! running hash is compared against the one the code produced before the move.
//!
//! **If this fails after a change that was meant to change the Ridgeback**, it
//! is doing its job: re-pin it, and say in the commit and in
//! `docs/design/feel-log.md` what moved and why. If it fails after a change
//! that was not meant to touch the Ridgeback, the change is wrong.

use sim::state::MAX_PLAYERS;
use sim::{Class, Input, World};

fn script(frames: u32, seed: u64) -> Vec<[Input; MAX_PLAYERS]> {
    let mut rng = seed;
    let mut next = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    (0..frames)
        .map(|_| {
            [
                Input::aimed((next() & 0x1ff) as u16, next() as u16),
                Input::aimed((next() & 0x1ff) as u16, next() as u16),
            ]
        })
        .collect()
}

/// A running hash of every frame of a hunt, so a difference anywhere in it
/// shows rather than only one at the end.
fn hunt_hash(class: Class, frames: u32, seed: u64) -> u64 {
    let mut w = World::hunt([class; MAX_PLAYERS]);
    let mut acc: u64 = 0xcbf2_9ce4_8422_2325;
    for input in script(frames, seed) {
        w.advance(input);
        acc = (acc ^ w.state_checksum()).wrapping_mul(0x0000_0100_0000_01b3);
    }
    acc
}

/// What each class's hunt hashed to before the creature became a species.
///
/// **The Elementalist's was re-pinned on 2026-10-09**, when her buttons were
/// remapped on to three clicks: the same random presses now throw different
/// moves. See `docs/design/feel-log.md` for that date.
///
/// **The Blood mage's was re-pinned the same day** for her own three clicks,
/// and **the Elementalist's again**: her takeoff window and held earth click
/// went into `state_checksum`, which had been leaving them out.
///
/// **Both again after the second playtest** (2026-10-09, later): the takeoff
/// window remembers the height of what her feet were last on
/// (`Rise::floor`), which both classes keep, and the Elementalist's stones
/// carry a body that stands on them while they move. See the feel log.
///
/// **The Dual mage's was re-pinned the same day** for her rework: every move
/// a spell, and the hex. See `docs/design/exploration/0010_dual_mage_spells.md`.
const PINNED: [(Class, u64); 6] = [
    (Class::Champion, 0x92d2fe3e7aa9f640),
    (Class::Bulwark, 0x343a49eb275a9f8c),
    (Class::ShadowReaver, 0xe53fcf6df7af6f65),
    (Class::BloodMage, 0x1d800ee722377057),
    (Class::DualMage, 0xa09cde216173e69f),
    (Class::Elementalist, 0x70811b99d05c7254),
];

#[test]
fn the_ridgeback_hunt_is_bit_identical() {
    let mut wrong = Vec::new();
    for (class, want) in PINNED {
        let got = hunt_hash(class, 2400, 0x5eed_0000 ^ class as u64);
        println!("    (Class::{class:?}, 0x{got:016x}),");
        if got != want {
            wrong.push(class);
        }
    }
    assert!(wrong.is_empty(), "these hunts changed: {wrong:?}");
}
