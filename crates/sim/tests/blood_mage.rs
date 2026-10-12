//! The Blood mage's dodge with nothing under the crosshair: **the Sanguine
//! flow**, her blood-form slide along the floor (2026-10-12), behind the
//! `Dodge is a blink` flag -- on, since the blaster.
//!
//! **Its own test binary, like `oven.rs` and for the same reason.** The flag is
//! live, so testing it means writing to the Oven, and a global that tests
//! mutate is a source of flakes that only show up under load. Cargo runs each
//! target as a separate process, so nothing in here can reach the store
//! `combat.rs` and `feel.rs` are reading.
//!
//! The Grasp and its haul, the other half of what this file used to test, were
//! cut with the three-click kit: the haul lives on in Reaper's chain. See
//! `docs/design/exploration/0015_blood_mage_blaster.md`.

use sim::class::Class;
use sim::oven::{self, Scalar};
use sim::{Fx, Input, V3, World};
use std::sync::{Mutex, MutexGuard};

const SHIFT: u16 = Input::SHIFT;
const W: u16 = Input::W;
const LOOK_LEFT: u16 = 1 << 15;

/// Held by every test in here. There is one store, and two tests that disagree
/// about which flags are on cannot both be right at the same instant.
static STORE: Mutex<()> = Mutex::new(());

/// A panicking test poisons the lock, and the second failure would then be
/// reported instead of the first.
fn the_store() -> MutexGuard<'static, ()> {
    STORE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Set the flag for the duration of one test, and put the store back.
struct Flags {
    _lock: MutexGuard<'static, ()>,
}

impl Flags {
    fn new(blink: bool) -> Flags {
        let lock = the_store();
        oven::set_scalar(Scalar::DodgeBlinks, blink as i32);
        Flags { _lock: lock }
    }
}

impl Drop for Flags {
    fn drop(&mut self) {
        oven::reset_to_baked();
    }
}

fn mage() -> World {
    World::with_classes([Class::BloodMage, Class::Bulwark])
}

fn looking(w: &mut World, frames: u32, bits: u16, yaw: u16, pitch: i16) {
    for _ in 0..frames {
        w.advance([
            Input::looking_at(bits, yaw, pitch),
            Input::aimed(0, LOOK_LEFT),
        ]);
    }
}

/// Where she is standing, flat.
fn flat(w: &World) -> V3 {
    let p = w.players[0].pos;
    V3::new(p.x, Fx::ZERO, p.z)
}

/// Put her somewhere with room, out past the ends of the arena's platforms.
///
/// She spawns half a metre from the side of one, which is a fact about the
/// blockout and would otherwise be the thing every measurement in here was
/// really about.
fn in_the_open(w: &mut World) {
    w.players[0].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(8));
    w.players[1].pos = V3::new(Fx::from_int(10), Fx::ZERO, Fx::from_int(11));
}

/// The yaw that points along positive x.
const TOWARD_PLUS_X: u16 = 0;

// ---------------------------------------------------------------------------
// The blink dodge
// ---------------------------------------------------------------------------

/// Dodge forward and let it finish. Returns how far she went, flat.
fn dodge_distance(w: &mut World) -> f32 {
    let before = flat(w);
    looking(w, 2, SHIFT | W, TOWARD_PLUS_X, 0);
    looking(w, 40, 0, TOWARD_PLUS_X, 0);
    flat(w).sub(before).flat_len().to_f32_for_render()
}

#[test]
fn the_blink_goes_further_than_the_dodge_it_replaces() {
    // The only thing it changes. Same frames, same invulnerability, same
    // commitment -- a longer distance, and nothing else, which is also the
    // honest complaint about it: there is no thematic tie and it leans entirely
    // on the animation to make it feel like it belongs.
    let ordinary = {
        let _flags = Flags::new(false);
        let mut w = mage();
        in_the_open(&mut w);
        dodge_distance(&mut w)
    };
    let blinked = {
        let _flags = Flags::new(true);
        let mut w = mage();
        in_the_open(&mut w);
        dodge_distance(&mut w)
    };
    assert!(
        blinked > ordinary * 1.5,
        "the blink covered {blinked:.2} m against the dodge's {ordinary:.2} m"
    );
}

#[test]
fn the_blink_is_flat() {
    // **What this class is short of is ground, not height.** A blink that also
    // went up would be the second jump the one class with no movement has not
    // earned, and it would quietly undo the point of the height nerf that made
    // this ability necessary.
    let _flags = Flags::new(true);
    let mut w = mage();
    in_the_open(&mut w);
    let mut highest = 0.0f32;
    looking(&mut w, 2, SHIFT | W, TOWARD_PLUS_X, 0);
    for _ in 0..40 {
        looking(&mut w, 1, 0, TOWARD_PLUS_X, 0);
        highest = highest.max(w.players[0].pos.y.to_f32_for_render());
    }
    assert!(
        highest < 0.1,
        "the blink lifted her {highest:.2} m off the floor"
    );
}

#[test]
fn a_blink_into_a_wall_arrives_at_the_wall() {
    // Not through it. `aim::first_along` against the arena and the structures,
    // which is the same question the Grasp's anchor asks -- one mechanism, so
    // the two experiments cannot disagree about where a wall is.
    let _flags = Flags::new(true);
    let mut w = mage();
    in_the_open(&mut w);
    w.players[0].pos = V3::new(Fx::from_int(12), Fx::ZERO, Fx::from_int(8));
    looking(&mut w, 2, SHIFT | W, TOWARD_PLUS_X, 0);
    looking(&mut w, 40, 0, TOWARD_PLUS_X, 0);
    let x = w.players[0].pos.x.to_f32_for_render();
    let wall = sim::arena::proving_ground::half().to_f32_for_render();
    assert!(
        x < wall,
        "she blinked to x = {x:.2}, through the wall at {wall:.2}"
    );
}

#[test]
fn the_blink_is_off_unless_the_flag_says_otherwise() {
    let _flags = Flags::new(false);
    let mut w = mage();
    in_the_open(&mut w);
    let plain = dodge_distance(&mut w);
    let reach = sim::tuning::blink_range().to_f32_for_render();
    assert!(
        plain < reach * 0.8,
        "with the blink off a dodge still covered {plain:.2} m of a {reach:.2} m blink"
    );
}

#[test]
fn neither_flag_reaches_another_class() {
    // It is hers. A flag in the Oven applies to the whole simulation, and the
    // place that reads it asks about the class first -- `state::blinks` -- so
    // this is the assertion that says so out loud.
    let _flags = Flags::new(true);
    for class in sim::class::ALL_CLASSES {
        if class == Class::BloodMage {
            continue;
        }
        let mut w = World::with_classes([class, Class::Bulwark]);
        w.players[0].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(8));
        w.players[1].pos = V3::new(Fx::from_int(10), Fx::ZERO, Fx::from_int(11));
        let went = dodge_distance(&mut w);
        let reach = sim::tuning::blink_range().to_f32_for_render();
        assert!(
            went < reach * 0.8,
            "the {} blinked {went:.2} m of a {reach:.2} m blink",
            class.name()
        );
    }
}

#[test]
fn the_flags_are_in_the_tuning_hash() {
    // Two peers running different combinations must desync loudly rather than
    // quietly. The Oven folds its whole contents into `World::checksum` for
    // exactly this, and a flag is no different from a distance.
    let plain = {
        let _flags = Flags::new(false);
        let mut w = mage();
        in_the_open(&mut w);
        w.advance([Input::default(); 2]);
        w.checksum()
    };
    let flowing = {
        let _flags = Flags::new(true);
        let mut w = mage();
        in_the_open(&mut w);
        w.advance([Input::default(); 2]);
        w.checksum()
    };
    assert_ne!(
        plain, flowing,
        "the flag is not in the checksum, so a mismatched pair of peers would \
         look like a netcode bug instead of a tuning one"
    );
}
