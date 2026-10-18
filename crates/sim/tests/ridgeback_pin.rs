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

use sim::{Class, Input, World};

fn script(frames: u32, seed: u64) -> Vec<[Input; 2]> {
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
    let mut w = World::hunt([class; 2]);
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
///
/// **The Champion's was re-pinned 2026-10-12, later** for his rework: the
/// braced Rush and its shoulder, a held weapon charging, the full charges and
/// momentum. See `docs/design/exploration/0015_champion_momentum_and_the_charge.md`.
/// **And again 2026-10-13**: tap to combo and hold to finish, momentum spent by
/// every finisher, and the Clash where every blow lands.
///
/// **The Champion's and the Reaver's were re-pinned 2026-10-13, later**, for
/// slopes, steps and lips: their random walks reach the arena's rim -- alone,
/// an arena's rim is land -- at frames 2077 and 1292, where the steep-ground
/// wall is the footing now. The Bulwark's never reaches it. See the feel log.
///
/// **The Champion's again the same day, with no change to play**: the clocks
/// his liquid weapon is drawn from (`class::Quicksilver`) went into
/// `state_checksum` with the rest of his mechanic. With them left out of it,
/// every pin held bit for bit.
/// **Every class was re-pinned 2026-10-14, with no change to play**, for four
/// fighters: the snapshot holds four seats (two of them parked out of the
/// world here) and a hit mask twice as wide, and `state_checksum` hashes all
/// of it. Before re-pinning, both fighters' positions, velocities, health and
/// actions, the creature, the phase and every effect were compared frame by
/// frame against the code before the change, for all six classes over these
/// 2,400 frames, hunted and in versus: identical. **The Blood mage's moved
/// for play as well**: merged with her rework, her hunt fills twelve effect
/// slots, and where twelve evicted the oldest stain for a new one, 128 keep
/// both. Every fighter was identical up to that frame.
///
/// **Then the Dual mage's rework merged in** (2026-10-14): her own fight
/// moved, and the Reaver's and the Blood mage's with no change to play,
/// because her five retired effect kinds became four new ones and every kind
/// listed after hers is numbered one lower.
///
/// **The Blood mage's moved again the same day, deliberately**: her floor
/// spike winds up in 16 frames rather than 30 and stands wider, the Stake
/// narrower, Command waits on the Champion's window, a spike aimed through
/// a stain's column goes to it, and a finisher pays back the grey it burned.
///
/// **Every class but the Champion moved 2026-10-16, deliberately: wall slides
/// and wall jumps** (`state::cling_to_the_wall`). The random presses jump
/// against the proving ground's walls, which now slow the fall and take a
/// press of space as a jump off them. Not only the hash: with the wall
/// fields left out of `state_checksum` the same five still moved.
/// **The Bulwark's again the same day**: the same wall is offered again once
/// a body is `wall_jump_drop` below the last jump off it, rather than never,
/// and his random presses take one.
/// **And again the same day, for the stick against a wall and ledge grabs.** Replayed against the code before, frame by frame,
/// each hunt's first difference is a fighter in the air against a wall --
/// its speed along the face now eased toward what the stick asks
/// (`hold_to_the_wall`). The Dual mage's moved no body -- both fighters'
/// positions and speeds are identical on every frame -- only the new wall
/// state in the hash. The Champion's never met a wall. With the same-wall
/// drop merged in on top, the Bulwark's and the Blood mage's moved once more,
/// where the two wall changes meet.
/// And four again when the ledge became one catch rule: the random presses
/// had walked off edges and been caught, and a walk off is no longer caught.
/// Neither fighter catches a ledge in any of those four now.
/// **The Reaver's again the same day, deliberately: the step a body walks up
/// is a knee, 0.5 m, not 0.45 m.** Her random walk now walks up something it
/// used to stop at; with the step put back, every pin holds.
/// **The Blood mage's again 2026-10-18, deliberately: her territory**
/// (`docs/design/exploration/0021_blood_mage_territory.md`). Her random
/// presses now lay ground, swim in it, swing the scythe and plant hearts.
/// And again the same day: a heart is earned by the damage she deals, so
/// her random presses plant fewer.
const PINNED: [(Class, u64); 6] = [
    (Class::Champion, 0xde36cc574ef51ff7),
    (Class::Bulwark, 0xd12b055c26db8afb),
    (Class::ShadowReaver, 0x0b07d31d8c4ff345),
    (Class::BloodMage, 0x6c94d14d4f5ec81d),
    (Class::DualMage, 0x810a930f65195268),
    (Class::Elementalist, 0xa2d1095986bff1d9),
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
