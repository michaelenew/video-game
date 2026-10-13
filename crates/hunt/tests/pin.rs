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
const PINNED: [(Class, u32, u64); 7] = [
    (Class::Champion, 0x2545_F491, 0x53b27530caad3810),
    (Class::Champion, 7, 0x28db214ab3e65861),
    // Moved 2026-10-01, deliberately: the class layer throws his shield to
    // close on a window five to eleven metres off, leaps to it and Slams out
    // of the leap (`hunt::class`, `Hands::throw_in`), where it had handed
    // the Ridgeback plan's walk back unchanged. Nothing else in his hunt, and
    // no other pin, moved with it. See feel-log.md.
    // Moved 2026-10-10, deliberately: his three clicks (0012) -- a click in
    // a jump's first frames is a takeoff, and left click in the air Rebound.
    (Class::Bulwark, 101, 0x3fa044fb6cf577a5),
    // Moved 2026-10-01 by the class layer (`hunt::class`): the Dual mage's
    // hands are kept on the side that does not burn her, her finishers are
    // thrown into the windows the plan finds, and she goads her bars between
    // them. The Champion's and the Bulwark's hunts did not move -- the layer
    // hands their plans' input back unchanged here. See feel-log.md.
    // Moved 2026-10-09, deliberately: every move of hers is a spell now, and
    // the class layer weighs the spell a click throws. See feel-log.md.
    // Moved 2026-10-12, deliberately: her spellbook (0015) -- the litanies,
    // the curse and the purge, and the class layer turns to the light hand
    // when a curse is ripe.
    // Moved again 2026-10-12, deliberately: the bare clicks are her wing
    // autos again, each hopping her the way she holds (0016).
    (Class::DualMage, 2_222, 0x6f1cf7a8d3ab325f),
    // Added 2026-10-01 with the class layer, so that what it does with the
    // shadow, the fire and the pools is pinned as the plan's moves are.
    // Moved 2026-10-09, deliberately: her clicks in the air and off the
    // floor are new moves (0011), and Executioner is on middle click.
    // Moved 2026-10-12 with no change to her fight: an effect is hashed by
    // its kind's number, and the Dual mage's retired Shade bolt sat before
    // her shadow's blades in the list (0015).
    (Class::ShadowReaver, 31, 0x64bc037a1d8a1933),
    // Moved 2026-10-09, deliberately: her buttons moved on to three clicks,
    // and the class layer presses her Bolt through the button it learns
    // rather than through left click. No other pin moved. See feel-log.md.
    // And again the same day: her takeoff window and held earth click went
    // into `state_checksum`, which had been leaving them out. And after the
    // second playtest: `Rise::floor`, and a stone carries whoever stands on it.
    (Class::Elementalist, 59, 0x16759ffa51c07b35),
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
