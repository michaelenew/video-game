//! **Riding a cow** (hornback.md §2, §10 item 2).
//!
//! A cow's back is a surface (`CritterKind::mountable`): a fighter who lands
//! on it rides it, by the generic critter ride in `state` (the position on
//! the back is authoritative, and the grip is tested against what the back
//! does under the feet). What is the herd's own is here:
//!
//! - **The ride clock**: frames a fighter has been on each cow's back, a
//!   sixteen-bit half of a lore word per body.
//! - **The buck**: at `RidePatience` the cow throws its first buck, and
//!   `SecondBuck` later its second, harder by `SecondHarder`. A buck is the
//!   rump kicking up fast and dropping away slower ([`surface`]): at the top
//!   the back falls out from under the feet, and that is what the grip test
//!   reads. The first is sized between the grip and the braced grip, the
//!   second above both: **crouch to hold the first; jump before the second**.
//! - **A ridden cow carries you home**: it goes on doing what the herd does,
//!   and home is behind the bull.

use super::mind::cow;
use super::{BUCK, COW, Knob, knob, knob_fx};
use crate::critter::{Critter, MAX_CRITTERS, is};
use crate::fixed::Fx;
use crate::pack::Pack;
use crate::state::{Player, World};

/// The critter a fighter is riding, if any.
pub fn rider_of(p: &Player) -> Option<usize> {
    crate::critter::ridden(p.mount)
}

/// Frames a fighter has been on critter `i`'s back.
pub fn clock(w: &World, i: usize) -> i32 {
    let word = w.lore.word(super::rules::word::RIDE + i / 2);
    (word >> (16 * (i as u32 % 2)) & 0xFFFF) as i32
}

fn set_clock(w: &mut World, i: usize, v: i32) {
    let at = super::rules::word::RIDE + i / 2;
    let shift = 16 * (i as u32 % 2);
    let word = w.lore.word(at) & !(0xFFFF << shift);
    w.lore
        .set_word(at, word | ((v.clamp(0, 0xFFFF) as u32) << shift));
}

/// The species' ride frame: the clocks and the bucks.
pub fn frame(w: &mut World, pack: &mut Pack) {
    let _ = pack;
    let sp = &super::SPECIES;
    for i in 0..MAX_CRITTERS {
        let c = w.critters[i];
        if c.kind != COW || !c.alive() {
            set_clock(w, i, 0);
            continue;
        }
        let ridden = w
            .players
            .iter()
            .any(|p| p.health > 0 && rider_of(p) == Some(i));
        if !ridden {
            set_clock(w, i, 0);
            continue;
        }
        let t = clock(w, i) + 1;
        set_clock(w, i, t);
        let first = knob(Knob::RidePatience).max(1);
        let second = first + knob(Knob::SecondBuck).max(1);
        if t == first || t == second {
            let body = &mut w.critters[i];
            if body.state == is::ACTIVE && body.act == super::STAMPEDE {
                // Running a lane, it bucks when the run is over.
                set_clock(w, i, t - 1);
                continue;
            }
            body.state = is::STARTUP;
            body.act = BUCK;
            body.timer = sp.attack(BUCK).startup.max(1);
            body.set(crate::critter::flag::HIT_USED, false);
            if t == second {
                body.role |= cow::HARD;
            } else {
                body.role &= !cow::HARD;
            }
        }
    }
}

/// **How the back is moving**, for the generic ride and the renderer
/// (`PackMind::surface`): level and still, but in a buck's active frames,
/// where the rump snaps up over `BuckSnap` frames and drops back over
/// `BuckSettle`. The top of that, where the back drops away, is the
/// acceleration that throws: about `BuckHeave × (1/snap + 1/settle) × 3600`
/// m/s² -- 0.27 m over two and six frames is 650, between the grip and the
/// braced grip; the second, twice that, is above both.
pub fn surface(c: &Critter) -> (Fx, Fx) {
    if c.kind != COW || c.act != BUCK || c.state != is::ACTIVE {
        return (Fx::ZERO, Fx::ZERO);
    }
    let sp = &super::SPECIES;
    let active = sp.attack(BUCK).active as i32;
    let t = active - c.timer as i32;
    let snap = knob(Knob::BuckSnap).max(1);
    let settle = knob(Knob::BuckSettle).max(1);
    let share = if t < 0 {
        Fx::ZERO
    } else if t <= snap {
        Fx::ratio(t, snap)
    } else if t <= snap + settle {
        Fx::ONE.sub(Fx::ratio(t - snap, settle))
    } else {
        Fx::ZERO
    };
    let harder = if c.role & cow::HARD != 0 {
        knob_fx(Knob::SecondHarder)
    } else {
        Fx::ONE
    };
    let heave = knob_fx(Knob::BuckHeave).mul(harder).mul(share);
    // Rump up is nose down.
    let pitch = knob_fx(Knob::BuckPitch).mul(harder).mul(share).neg();
    (pitch, heave)
}
