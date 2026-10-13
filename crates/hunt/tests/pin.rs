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

// Every hunt moved 2026-10-12, deliberately: `Player::clear`, the camera's
// room under the eye (exploration 0014), is in `state_checksum` whenever it is
// not nought, which is every jump -- and off the ground the eye the hunter
// aims from is let back out of the head looking up. The Reaver's moved again
// for her third pass and her class layer (she cashes with the Reave, sends the
// shadow past the work, keeps it out and lotuses it between openings).
//
// The Champion's moved again later that day, for his rework (exploration
// 0015): the braced Rush, the hold and its full charges, momentum, Break and
// the Clash -- and his hands press a weapon the plan holds on every other
// frame, because held it is a charge. And again 2026-10-13: tap to combo and
// hold to finish, momentum spent by every finisher, the Clash where every
// blow lands.
const PINNED: [(Class, u32, u64); 7] = [
    (Class::Champion, 0x2545_F491, 0xe9ca1f6562765a63),
    (Class::Champion, 7, 0x49ce7f2989520e77),
    // Moved 2026-10-01, deliberately: the class layer throws his shield to
    // close on a window five to eleven metres off, leaps to it and Slams out
    // of the leap (`hunt::class`, `Hands::throw_in`), where it had handed
    // the Ridgeback plan's walk back unchanged. Nothing else in his hunt, and
    // no other pin, moved with it. See feel-log.md.
    // Moved 2026-10-10, deliberately: his three clicks (0012) -- a click in
    // a jump's first frames is a takeoff, and left click in the air Rebound.
    // Moved 2026-10-13, deliberately, with the Elementalist's: slopes, steps
    // and lips -- the steep-ground wall is the footing, and the arena's rim
    // is land, which the scripted hunt walks into. See feel-log.md.
    (Class::Bulwark, 101, 0x9df65d759401d3ac),
    // Moved 2026-10-01 by the class layer (`hunt::class`): the Dual mage's
    // hands are kept on the side that does not burn her, her finishers are
    // thrown into the windows the plan finds, and she goads her bars between
    // them. The Champion's and the Bulwark's hunts did not move -- the layer
    // hands their plans' input back unchanged here. See feel-log.md.
    // Moved 2026-10-09, deliberately: every move of hers is a spell now, and
    // the class layer weighs the spell a click throws. See feel-log.md.
    (Class::DualMage, 2_222, 0x054b32fa5a4155db),
    // Added 2026-10-01 with the class layer, so that what it does with the
    // shadow, the fire and the pools is pinned as the plan's moves are.
    // Moved 2026-10-09, deliberately: her clicks in the air and off the
    // floor are new moves (0011), and Executioner is on middle click.
    (Class::ShadowReaver, 31, 0x1d584066a6e51c49),
    // Moved 2026-10-09, deliberately: her buttons moved on to three clicks,
    // and the class layer presses her Bolt through the button it learns
    // rather than through left click. No other pin moved. See feel-log.md.
    // And again the same day: her takeoff window and held earth click went
    // into `state_checksum`, which had been leaving them out. And after the
    // second playtest: `Rise::floor`, and a stone carries whoever stands on it.
    // And 2026-10-13: slopes, steps and lips, at the rim (see the Bulwark's).
    (Class::Elementalist, 59, 0x3dd2441975228513),
    // Moved 2026-10-09, deliberately: her buttons moved on to three clicks
    // (the class layer presses her scythe and her Grasp through the buttons
    // it learns), and creatures bleed. See feel-log.md. Again the same day
    // for `Rise::floor`, the height her takeoff window remembers.
    (Class::BloodMage, 83, 0x24b139d8bfe531dd),
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
