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
// blow lands. And again the same day with no change to play: the clocks his
// liquid weapon is drawn from (`class::Quicksilver`) are in `state_checksum`,
// and with them left out every pin held bit for bit.
//
// Every hunt moved 2026-10-14 with no change to play: the snapshot holds four
// seats (two parked out of the world here) and a wider hit mask, and
// `state_checksum` hashes all of it. Before re-pinning, every one of these
// hunts was compared frame by frame against the code before the change --
// both fighters, the creature, the phase and the effects -- and was identical.
// The Blood mage's, merged with her rework, moved for play too: her effects
// overfilled twelve slots, and 128 evict nothing.
//
// Every hunt but the Champion's moved 2026-10-16, deliberately: wall slides
// and wall jumps (`state::cling_to_the_wall`). The hunters jump against the
// proving ground's walls, which now slow a fall and take space as a jump off
// them; with the wall fields left out of `state_checksum` the same five
// still moved, so it is play and not only the hash. The same five again
// that day for the stick against a wall: a slide now starts only pushing
// into the wall, so a hunter brushing one is no longer held to it. Counted
// over all five hunts, no hunter catches a ledge and two slide, for 4 and
// 10 frames: nothing hangs. The Dual mage's and the Reaver's again when the
// ledge became one catch rule (a walk off an edge is no longer caught): no
// hunter in any of these catches a ledge now.
// The Champion's moved 2026-10-20, deliberately: only a press throws a
// link, kept a few frames, so a weapon the plan holds throws one move rather
// than a string; and his tempo bar's clocks are in `Fury`, so in the hash.
// And again the same day: the Beat moved off the freeze to a third of the way
// along a window counted from when he may act, which runs on past the third.
//
// Every hunt but the Blood mage's moved 2026-10-20, deliberately: the wall
// jump's overhaul (the Champion's merged with his Beat, above). Counted over each hunt: no hunter wall jumps; the
// Bulwark, the Dual mage and the Reaver slide for 6, 14 and 7 frames (touching
// a wall is a slide now, and the slide is slower); the Champion's two and the
// Elementalist's do not slide at all, and moved only for the hash -- a jump
// pressed in the air is kept for a wall (`Player::wall_press`), which
// `state_checksum` holds while it is not nought.
// And the first Champion's, the Bulwark's, the Dual mage's and the Reaver's
// again the same day: only enough wall is a wall (`Terrain::wall_face`), so
// the edges they brushed are no longer slid on.
const PINNED: [(Class, u32, u64); 7] = [
    (Class::Champion, 0x2545_F491, 0x41fd1135bb87c036),
    (Class::Champion, 7, 0x8ff13d2a62ee205e),
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
    (Class::Bulwark, 101, 0xd8542982379b030b),
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
    // autos again, each hopping her the way she holds (0016); and again for
    // the second round -- forward wings, long holds, spending majors -- and
    // the third: the autos a ball, every click a tap or a hold (2026-10-13).
    (Class::DualMage, 2_222, 0xf97a9d9d8e728851),
    // Added 2026-10-01 with the class layer, so that what it does with the
    // shadow, the fire and the pools is pinned as the plan's moves are.
    // Moved 2026-10-09, deliberately: her clicks in the air and off the
    // floor are new moves (0011), and Executioner is on middle click.
    // Moved 2026-10-12 with no change to her fight: an effect is hashed by
    // its kind's number, and the Dual mage's retired Shade bolt sat before
    // her shadow's blades in the list (0015).
    (Class::ShadowReaver, 31, 0xd56bc982f1f72c16),
    // Moved 2026-10-09, deliberately: her buttons moved on to three clicks,
    // and the class layer presses her Bolt through the button it learns
    // rather than through left click. No other pin moved. See feel-log.md.
    // And again the same day: her takeoff window and held earth click went
    // into `state_checksum`, which had been leaving them out. And after the
    // second playtest: `Rise::floor`, and a stone carries whoever stands on it.
    // And 2026-10-13: slopes, steps and lips, at the rim (see the Bulwark's).
    // Moved 2026-10-18, deliberately: her QoL pass (a bigger, longer
    // pillar, pushes that act on the whole line, a sliding Fissure, burning
    // wind, a higher Updraft). Nobody else's moved.
    (Class::Elementalist, 59, 0x4a6ff2fbefa8c2e6),
    // Moved 2026-10-09, deliberately: her buttons moved on to three clicks
    // (the class layer presses her scythe and her Grasp through the buttons
    // it learns), and creatures bleed. See feel-log.md. Again the same day
    // for `Rise::floor`, the height her takeoff window remembers.
    // Moved 2026-10-12, deliberately: the blaster -- the class layer holds a
    // Transfusion to its third tier and drops a Hemoplague where it held a
    // Grasp, and every stain she makes is collected; and again for her dodge
    // sending her blood ahead. See feel-log.md. Moved 2026-10-14,
    // deliberately: the spike on the floor winds up in 16 frames rather
    // than 30, Command waits on the Champion's window, a spike aimed through
    // a stain's column goes to it, and a finisher pays back its grey.
    // Moved 2026-10-18, deliberately: her territory -- the class layer
    // sweeps the scythe in reach, plants a heart rather than raising stains,
    // and everything she throws lays her ground. See feel-log.md. Again the
    // same day: a heart is earned by the damage she deals. Moved 2026-10-19,
    // deliberately: the spike leaves no stain and erupts out of its blow,
    // eruptions spread, her ground hits harder, and the Well is gone. Again
    // the same day: which patches a heart holds is kept in the snapshot.
    // And again: her oldest stain makes way past a cap of 32, and her ground
    // holds 128 patches, merging what is laid over it.
    (Class::BloodMage, 83, 0x05305ad374ce6cea),
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
