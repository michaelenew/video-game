//! **Every seat is its own**: the per-fighter memories the simulation packs
//! into masks, words and bytes, each asked for every seat there is.
//!
//! Each of these was once an integer that happened to be wide enough on the
//! day -- a bit of seat while there were two fighters, a byte of stones while
//! there were six -- and each silently became two seats sharing one memory
//! the day the thing it counted grew past it: a Dual mage in seat two healed
//! seat nought, a Siegeshell counted three fighters as one, a disc that had
//! shoved one stone skipped another. Nothing failed; the fight was just
//! wrong. So the rule is tested the way it breaks: write something different
//! for every seat, and read every one of them back.

use sim::class::Class;
use sim::critter::Critter;
use sim::effects::{Effect, LOTUS_BLADES, VICTIMS, quarry_victim};
use sim::lore::{Lore, per_fighter};
use sim::math::V3;
use sim::monster::{self, MAX_MONSTERS, NO_MOUNT};
use sim::species::SpeciesId;
use sim::state::{Fighters, MAX_PLAYERS};

#[test]
fn a_curse_remembers_every_seat_that_lays_it() {
    for by in 0..MAX_PLAYERS {
        for n in 1..=7 {
            let c = sim::dual::curse_of(n, by);
            assert_eq!(sim::dual::stacks(c), n, "seat {by}, {n} stacks");
            assert_eq!(sim::dual::cursed_by(c), by, "seat {by}, {n} stacks");
        }
    }
    // Seats nought and one write the byte they always did, so a hunt
    // recorded with two fighters hashes as it did.
    assert_eq!(sim::dual::curse_of(3, 0), 3);
    assert_eq!(sim::dual::curse_of(3, 1), 0x83);
}

#[test]
fn a_per_fighter_field_is_one_per_seat_at_every_width() {
    for bits in [1u32, 2, 4, 8, 16] {
        let mut lore = Lore::of(SpeciesId::MANTIS);
        let base = 0;
        let mask = if bits == 32 {
            u32::MAX
        } else {
            (1 << bits) - 1
        };
        let value = |who: usize| (who as u32 * 7 + 1) & mask;
        for who in 0..MAX_PLAYERS {
            lore.set_field(base, bits, who, value(who));
        }
        for who in 0..MAX_PLAYERS {
            assert_eq!(
                lore.field(base, bits, who),
                value(who),
                "{bits}-bit field, seat {who}"
            );
        }
        // Nothing past the run was touched.
        assert_eq!(
            lore.word(base + per_fighter(bits)),
            0,
            "{bits}-bit run spilled"
        );
    }
}

#[test]
fn an_effect_remembers_every_part_against_every_body() {
    let mut e = Effect::pool(0, Class::BloodMage, 0, V3::ZERO, 0);
    let bodies: Vec<usize> = (0..MAX_PLAYERS)
        .chain((0..MAX_MONSTERS).map(quarry_victim))
        .collect();
    assert_eq!(VICTIMS, MAX_PLAYERS + 1);
    for part in 0..LOTUS_BLADES {
        for &v in &bodies {
            assert!(
                !e.already_hit(part, v),
                "part {part}, body {v} hit before it was"
            );
            assert!(e.take_hit(part, v));
            assert!(e.already_hit(part, v));
        }
    }
    // No body took another's bit: every one is still standing.
    for part in 0..LOTUS_BLADES {
        for &v in &bodies {
            assert!(e.already_hit(part, v));
        }
    }
}

#[test]
fn a_set_of_fighters_holds_every_seat() {
    let mut f = Fighters::EMPTY;
    for i in (0..MAX_PLAYERS).step_by(2) {
        f.set(i);
    }
    for i in 0..MAX_PLAYERS {
        assert_eq!(f.has(i), i % 2 == 0, "seat {i}");
    }
}

#[test]
fn a_critter_knows_whose_blade_cut_it_for_every_seat() {
    let mut c = Critter::EMPTY;
    for owner in 0..MAX_PLAYERS as u8 {
        c.sear(owner, owner % 8, true);
    }
    for owner in 0..MAX_PLAYERS as u8 {
        for slot in 0..8 {
            assert_eq!(
                c.seared_by(owner, slot),
                slot == owner % 8,
                "seat {owner}, slot {slot}"
            );
        }
    }
}

#[test]
fn a_mount_names_every_creature_slot_and_part() {
    for slot in 0..MAX_MONSTERS {
        for part in 0..sim::beast::MAX_PARTS {
            let m = monster::mount_of(slot, part);
            assert_ne!(m, NO_MOUNT);
            assert_eq!(monster::mount_slot(m), slot);
            assert_eq!(monster::mount_part(m), part);
            assert_eq!(
                sim::critter::ridden(m),
                None,
                "slot {slot} read as a critter"
            );
        }
    }
    for i in 0..sim::critter::MAX_CRITTERS {
        assert_eq!(sim::critter::ridden(sim::critter::mount_of(i)), Some(i));
    }
    assert_eq!(sim::critter::ridden(NO_MOUNT), None);
}
