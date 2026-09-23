//! The Blood mage's two movement experiments, and the four ways they combine.
//!
//! **Its own test binary, like `oven.rs` and for the same reason.** Both of
//! these are behind live flags, so testing them means writing to the Oven, and
//! a global that tests mutate is a source of flakes that only show up under
//! load. Cargo runs each target as a separate process, so nothing in here can
//! reach the store `combat.rs` and `feel.rs` are reading.
//!
//! The class has no movement at all today, and cannot stay that way: the rest
//! of the roster each has one thing it does with the ground, and the jump was
//! cut back across the cast on 2026-09-17 specifically so that those matter
//! more. Two answers are built and neither is chosen -- see
//! `docs/design/kits/blood-mage.md` for what each is trying to be and what is
//! wrong with it.

use sim::class::Class;
use sim::effects::EffectKind;
use sim::oven::{self, Scalar};
use sim::{Fx, Input, V3, World};
use std::sync::{Mutex, MutexGuard};

const Q: u16 = Input::SPECIAL;
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

/// Set the two flags for the duration of one test, and put the store back.
struct Flags {
    _lock: MutexGuard<'static, ()>,
}

impl Flags {
    fn new(haul: bool, blink: bool) -> Flags {
        let lock = the_store();
        oven::set_scalar(Scalar::GraspHauls, haul as i32);
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

/// Wind a Grasp all the way out along `yaw`/`pitch` and let go, then let the
/// arms fly and whatever they did play out.
fn grasp(w: &mut World, yaw: u16, pitch: i16) {
    let hold = sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL).channel;
    looking(w, hold as u32 + 1, Q, yaw, pitch);
    looking(w, 1, 0, yaw, pitch);
    // Long enough for the arms to converge, the effect to expire and the haul
    // it may have armed to finish.
    looking(w, 120, 0, yaw, pitch);
}

/// The yaw that points along positive or negative x.
const TOWARD_PLUS_X: u16 = 0;
const TOWARD_MINUS_X: u16 = 1 << 15;

// ---------------------------------------------------------------------------
// The Grasp haul
// ---------------------------------------------------------------------------

#[test]
fn a_grasp_that_closes_on_a_wall_pulls_her_to_it() {
    // **The whole idea.** Four arms converge on a point and whatever they catch
    // is hauled back to her; a point with nothing catchable at it but something
    // solid behind it has something to pull against and nothing to pull, so
    // what moves is her. A grappling hook made out of an ability she already
    // owns, which is the thematic tie the blink has not got.
    let _flags = Flags::new(true, false);
    let mut w = mage();
    in_the_open(&mut w);
    // Six metres off the far wall, winding out to ten, so the arms converge
    // inside it. **Further out than the reach is the whole point**: the Grasp
    // does not shorten itself against the scenery, so a wall inside the reach
    // is a wall the arms close behind, and the anchor is where the line met it.
    w.players[0].pos = V3::new(Fx::from_int(8), Fx::ZERO, Fx::from_int(8));
    let before = flat(&w);
    grasp(&mut w, TOWARD_PLUS_X, 0);
    let after = flat(&w);
    let moved = after.sub(before).flat_len().to_f32_for_render();
    assert!(
        moved > 3.0,
        "the arms closed on the wall and she stayed where she was: {moved:.2} m"
    );
    assert!(
        after.x.to_f32_for_render() > before.x.to_f32_for_render(),
        "she was pulled away from the wall she grasped"
    );
}

#[test]
fn she_arrives_beside_the_wall_rather_than_inside_it() {
    // The Grasp's reach is the length of the hold and it deliberately does not
    // shorten itself against the scenery -- point at a near wall, wind to full,
    // and the arms converge metres behind it. So the haul has to stop at the
    // *anchor* rather than at the convergence point, or a full wind-up at a
    // close wall would put her inside the arena's geometry.
    let _flags = Flags::new(true, false);
    let mut w = mage();
    in_the_open(&mut w);
    // A couple of metres off the wall, winding out to ten.
    w.players[0].pos = V3::new(Fx::from_int(11), Fx::ZERO, Fx::from_int(8));
    grasp(&mut w, TOWARD_PLUS_X, 0);
    let x = w.players[0].pos.x.to_f32_for_render();
    let wall = sim::arena::ARENA_HALF.to_f32_for_render();
    assert!(
        x < wall,
        "she ended at x = {x:.2}, past the wall at {wall:.2}"
    );
    assert!(
        x > 11.0,
        "she was not pulled toward the wall at all: x = {x:.2}"
    );
}

#[test]
fn a_grasp_that_catches_somebody_hauls_them_and_not_her() {
    // The arms do one job at a time. A catch is a catch and works exactly as it
    // always has; the haul is what happens when there was nothing to catch.
    let _flags = Flags::new(true, false);
    let mut w = mage();
    in_the_open(&mut w);
    let reach = sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL).reach;
    // Standing where the arms converge, out in the open along positive x.
    w.players[1].pos = V3::new(reach, Fx::ZERO, Fx::from_int(8));
    let her = flat(&w);
    let theirs = flat(&w);
    let theirs = V3::new(w.players[1].pos.x, Fx::ZERO, theirs.z);
    grasp(&mut w, TOWARD_PLUS_X, 0);
    let she_moved = flat(&w).sub(her).flat_len().to_f32_for_render();
    let they_moved = V3::new(w.players[1].pos.x, Fx::ZERO, w.players[1].pos.z)
        .sub(theirs)
        .flat_len()
        .to_f32_for_render();
    assert!(
        they_moved > she_moved,
        "the catch moved her {she_moved:.2} m and them {they_moved:.2} m -- the arms \
         pulled on the wall behind the person they had hold of"
    );
}

#[test]
fn the_haul_is_off_unless_the_flag_says_otherwise() {
    // The point of it being a flag. Two answers are built and neither is
    // chosen, so both have to be genuinely absent when they are off -- a
    // half-present experiment is worse than no experiment, because the thing
    // being judged is not the thing that shipped.
    let _flags = Flags::new(false, false);
    let mut w = mage();
    in_the_open(&mut w);
    let before = flat(&w);
    grasp(&mut w, TOWARD_PLUS_X, 0);
    let moved = flat(&w).sub(before).flat_len().to_f32_for_render();
    assert!(
        moved < 0.5,
        "the Grasp moved her {moved:.2} m with the haul switched off"
    );
}

#[test]
fn hauling_costs_her_the_whole_price_of_a_whiff() {
    // Which is why it is allowed to be this strong. The health went on the
    // press and none of it comes back, because nothing was cut -- so using the
    // Grasp as a grappling hook costs a missed cast every time, out of the
    // resource the class is made of.
    let _flags = Flags::new(true, false);
    let mut w = mage();
    in_the_open(&mut w);
    let before = w.players[0].health;
    let cost = sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL).cost;
    grasp(&mut w, TOWARD_PLUS_X, 0);
    let spent = before - w.players[0].health;
    assert_eq!(
        spent, cost,
        "a Grasp used as movement cost {spent} of a {cost} cast"
    );
}

#[test]
fn a_hit_makes_the_arms_let_go() {
    // The rule every commitment in the game follows: one you can be hit out of
    // and keep is a commitment with invulnerability attached. The Reaver's dash
    // breaks the same way, from the same place -- `apply_hit`, on the same
    // `interrupts` flag.
    //
    // Thrown at her by the other fighter rather than written into her state,
    // because what has to be true is that a *hit* does it, and the path from a
    // swing to `Player::haul` runs through three or four decisions that a
    // directly-set field would skip over.
    let _flags = Flags::new(true, false);
    let run = |opponent: u16| {
        let mut w = mage();
        w.players[0].pos = V3::new(Fx::from_int(8), Fx::ZERO, Fx::from_int(8));
        // Stood in her way, a stride along the pull.
        w.players[1].pos = V3::new(Fx::from_int(10), Fx::ZERO, Fx::from_int(8));
        let hold = sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL).channel;
        for i in 0..200u32 {
            let mine = if i < hold as u32 + 1 { Q } else { 0 };
            w.advance([
                Input::looking_at(mine, TOWARD_PLUS_X, 0),
                Input::aimed(opponent, LOOK_LEFT),
            ]);
        }
        w.players[0].pos.x.to_f32_for_render()
    };
    let clear = run(0);
    let punched = run(Input::LEFT);
    assert!(
        clear > 9.0,
        "fixture: the pull did not get anywhere even unopposed ({clear:.2} m)"
    );
    assert!(
        punched < clear - 0.5,
        "she was hit mid-pull and arrived anyway: {punched:.2} m against {clear:.2} m"
    );
}

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
        let _flags = Flags::new(false, false);
        let mut w = mage();
        in_the_open(&mut w);
        dodge_distance(&mut w)
    };
    let blinked = {
        let _flags = Flags::new(false, true);
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
    let _flags = Flags::new(false, true);
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
    let _flags = Flags::new(false, true);
    let mut w = mage();
    in_the_open(&mut w);
    w.players[0].pos = V3::new(Fx::from_int(12), Fx::ZERO, Fx::from_int(8));
    looking(&mut w, 2, SHIFT | W, TOWARD_PLUS_X, 0);
    looking(&mut w, 40, 0, TOWARD_PLUS_X, 0);
    let x = w.players[0].pos.x.to_f32_for_render();
    let wall = sim::arena::ARENA_HALF.to_f32_for_render();
    assert!(
        x < wall,
        "she blinked to x = {x:.2}, through the wall at {wall:.2}"
    );
}

#[test]
fn the_blink_is_off_unless_the_flag_says_otherwise() {
    let _flags = Flags::new(true, false);
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
fn the_two_flags_are_independent() {
    // The reason they are two flags and not one setting: what is wanted is a
    // judgement about each and about the pair, and all four combinations have
    // to be reachable without a rebuild.
    for (haul, blink) in [(false, false), (true, false), (false, true), (true, true)] {
        let _flags = Flags::new(haul, blink);
        assert_eq!(sim::tuning::grasp_hauls(), haul);
        assert_eq!(sim::tuning::dodge_blinks(), blink);
        // And the game still runs in each of them, which is the other thing a
        // combination has to be.
        let mut w = mage();
        in_the_open(&mut w);
        looking(&mut w, 2, SHIFT | W, TOWARD_PLUS_X, 0);
        looking(&mut w, 30, Q, TOWARD_MINUS_X, 0);
        looking(&mut w, 60, 0, TOWARD_MINUS_X, 0);
        assert!(
            w.players[0].health > 0,
            "she killed herself with haul={haul} blink={blink}"
        );
    }
}

#[test]
fn neither_flag_reaches_another_class() {
    // Both of these are hers. A flag in the Oven applies to the whole
    // simulation, and the two places that read these ask about the class
    // first -- `state::blinks` and the Grasp being the only thing that arms a
    // haul -- so this is the assertion that says so out loud.
    let _flags = Flags::new(true, true);
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
        assert_eq!(w.players[0].haul, 0, "the {} is being hauled", class.name());
    }
}

#[test]
fn the_flags_are_in_the_tuning_hash() {
    // Two peers running different combinations must desync loudly rather than
    // quietly. The Oven folds its whole contents into `World::checksum` for
    // exactly this, and a flag is no different from a distance.
    let plain = {
        let _flags = Flags::new(false, false);
        let mut w = mage();
        in_the_open(&mut w);
        w.advance([Input::default(); 2]);
        w.checksum()
    };
    let hauling = {
        let _flags = Flags::new(true, false);
        let mut w = mage();
        in_the_open(&mut w);
        w.advance([Input::default(); 2]);
        w.checksum()
    };
    assert_ne!(
        plain, hauling,
        "the haul flag is not in the checksum, so a mismatched pair of peers would \
         look like a netcode bug instead of a tuning one"
    );
}

#[test]
fn the_grasp_still_erupts_and_still_hurts_with_the_haul_on() {
    // The thing a flag is most likely to break: an ability that grew a second
    // job and stopped doing its first one.
    let _flags = Flags::new(true, false);
    let mut w = mage();
    in_the_open(&mut w);
    let reach = sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL).reach;
    w.players[1].pos = V3::new(reach, Fx::ZERO, Fx::from_int(8));
    let before = w.players[1].health;
    let hold = sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL).channel;
    looking(&mut w, hold as u32 + 1, Q, TOWARD_PLUS_X, 0);
    looking(&mut w, 1, 0, TOWARD_PLUS_X, 0);
    let mut armed = false;
    for _ in 0..60 {
        looking(&mut w, 1, 0, TOWARD_PLUS_X, 0);
        armed |= w
            .effects
            .iter()
            .flatten()
            .any(|e| e.kind == EffectKind::Grasp);
    }
    assert!(armed, "no Grasp was ever in the world");
    assert!(
        w.players[1].health < before,
        "the arms did no damage at all with the haul switched on"
    );
}
