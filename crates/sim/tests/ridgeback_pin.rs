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
///
/// **The Shadow Reaver's was re-pinned the same day** for her three clicks:
/// her clicks in the air and off the floor are new moves. See
/// `docs/design/exploration/0011_shadow_reaver_on_three_clicks.md`.
///
/// **The Bulwark's was re-pinned 2026-10-10** for his: a click in a jump's
/// first frames is a takeoff, and left click in the air is Rebound. See
/// `docs/design/exploration/0012_bulwark_on_three_clicks.md`.
///
/// **The Elementalist's was re-pinned 2026-10-10**: the Updraft from the
/// floor throws her higher and along her run, and the Fire carpet's recovery
/// cancels. See the feel log.
///
/// **Every class was re-pinned 2026-10-12**: `Player::clear`, the camera's
/// room under the eye, went into `state_checksum` whenever it is not nought,
/// which is every jump; and the Reaver's kit moved. See
/// `docs/design/exploration/0014_shadow_reaver_marks_and_throws.md`.
///
/// **The Blood mage's was re-pinned the same day** for the blaster: her
/// clicks are new moves held in tiers, every stain is collected, and every
/// hit bleeds. See `docs/design/exploration/0015_blood_mage_blaster.md`.
/// **And again the same day**: her dodge sends her blood ahead, and Frenzy's
/// own blows spill nothing. See the feel log.
const PINNED: [(Class, u64); 6] = [
    (Class::Champion, 0x339bb2066878e1c8),
    (Class::Bulwark, 0x8465a86bccb8ef32),
    (Class::ShadowReaver, 0x125b4fc5c2559d9d),
    (Class::BloodMage, 0x236cf8434b16031f),
    (Class::DualMage, 0xd1dc8709fa4d36db),
    (Class::Elementalist, 0x19fbc249420a6a17),
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
