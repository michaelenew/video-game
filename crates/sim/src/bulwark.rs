//! The Bulwark's weight: the shield as a battery.
//!
//! **Every blow taken on the shield is stored in it**, and the shield's own
//! attacks spend it. That is the whole answer to *why use a shield alone* --
//! blocking is not the absence of being hit, it is loading the weapon. See
//! `docs/design/bulwark-v2.md`.
//!
//! ## What lives here
//!
//! The rules weight follows: how a blow loads it, how it drains, and what it
//! does to a guard without being spent. Where it is stored is
//! [`crate::class::Shield`], on every state, so it travels with the object.
//! Loading is called from the one place a blocked hit is resolved --
//! `state::apply_hit` -- and from the Elementalist's poke, which is the one
//! blockable thing that does not go through it. The creature's blows do go
//! through it, so a blocked stomp loads the shield with no code of its own.

use crate::class::{Class, Mechanic, Shield, Structure};
use crate::fixed::Fx;
use crate::math::V3;
use crate::state::{Action, Player, SLOT_COMMITTED};
use crate::tuning as t;

/// The shield a fighter carries, if they are a Bulwark.
fn shield(p: &Player) -> Option<Shield> {
    match p.mechanic {
        Mechanic::Shield(s) => Some(s),
        _ => None,
    }
}

/// What the shield is holding. Zero for anybody without one.
pub fn weight(p: &Player) -> Fx {
    shield(p).map_or(Fx::ZERO, Shield::weight)
}

/// How full the shield is, from zero to one. What the pushback curve and the
/// drawing both read, so a knob that moves the cap moves both at once.
pub fn fullness(p: &Player) -> Fx {
    fullness_of(weight(p))
}

/// How full a shield holding `weight` is, from zero to one.
pub fn fullness_of(weight: Fx) -> Fx {
    let cap = t::weight_cap();
    if cap.raw() <= 0 {
        return Fx::ZERO;
    }
    weight.div(cap).clamp(Fx::ZERO, Fx::ONE)
}

/// A straight line from `empty` to `full`, by how full a shield is.
fn by_fullness(weight: Fx, empty: Fx, full: Fx) -> Fx {
    empty.add(full.sub(empty).mul(fullness_of(weight)))
}

/// A blow landed on the guard: store it.
///
/// A blocked blow stores its damage and a parried one stores `parry_load`
/// times that, capped. Called whether or not the guard was a parry, because
/// both are hits *taken on the shield* -- only the size of the deposit
/// differs.
pub fn load(p: &mut Player, damage: i32, parried: bool) {
    let Some(s) = shield(p) else {
        return;
    };
    let mut deposit = Fx::from_int(damage.max(0));
    if parried {
        deposit = deposit.mul(t::parry_load());
    }
    let weight = s.weight().add(deposit).min(t::weight_cap());
    p.mechanic = Mechanic::Shield(s.with_weight(weight));
}

/// One frame of the slow leak, so weight is about the current exchange rather
/// than the whole round. A full shield empties in `weight_drain_frames`, in
/// hand or in flight.
///
/// **Not planted.** A wall is sized by the weight it landed with and holds it
/// until it is recalled: a solid that shrank over ten seconds would be a wall
/// nobody could rely on, and would slide out from under whoever was standing
/// on it. The cost of parking weight in a wall is that there is no shield in
/// hand for anything else -- every Bulwark move needs it.
///
/// **Paused while Slam is out.** The weight is in the swing from the press to
/// the last active frame, so what the shake is worth -- and how wide it is
/// drawn -- is what the Bulwark had when he committed, not a number that leaks
/// while the overlay is on screen.
pub fn drain(p: &mut Player) {
    if slamming(p) {
        return;
    }
    let Some(s) = shield(p) else {
        return;
    };
    if s.weight().raw() <= 0 || matches!(s, Shield::Planted { .. }) {
        return;
    }
    let per_frame = t::weight_cap().div(Fx::from_int(t::weight_drain_frames().max(1)));
    let weight = s.weight().sub(per_frame).max(Fx::ZERO);
    p.mechanic = Mechanic::Shield(s.with_weight(weight));
}

/// What a blocked blow's knockback is multiplied by: one at empty, down to
/// `heavy_pushback` at the cap, in a straight line between.
///
/// Weight that is not spent still does something, and this is it. The more the
/// wall has taken, the less it moves -- which is what makes standing in a
/// string on purpose a plan rather than a way to be carried out of the fight.
pub fn pushback(p: &Player) -> Fx {
    let full = fullness(p);
    Fx::ONE.sub(Fx::ONE.sub(t::heavy_pushback()).mul(full))
}

// ---------------------------------------------------------------------------
// Slam: the withdrawal
// ---------------------------------------------------------------------------

/// Is this fighter's current move the Bulwark's Slam, from wind-up through its
/// active frames -- the stretch during which the weight is *in the swing*?
pub fn slamming(p: &Player) -> bool {
    p.class == Class::Bulwark
        && matches!(
            p.action,
            Action::Startup { kind, .. } | Action::Active { kind, .. } if kind == SLOT_COMMITTED
        )
}

/// One frame of Slam's bookkeeping, before the body moves.
///
/// Keeps the fastest the Bulwark has fallen during the wind-up, read off the
/// velocity the last frame left -- so on the frame the feet arrive the fall
/// that got them there is still counted. Anything else clears it: a fall
/// before the press is not a slam out of a fall.
pub fn track_fall(p: &mut Player) {
    if slamming(p) {
        p.slam_fall = p.slam_fall.max(p.vel.y.neg());
    } else {
        p.slam_fall = Fx::ZERO;
    }
}

/// Slam's damage: the blow, plus a share of every unit of weight it spends,
/// plus what the fall into it was worth. Anything that is not a Slam is
/// returned as it came.
pub fn slam_damage(p: &Player, kind: u8, base: i32) -> i32 {
    if p.class != Class::Bulwark || kind != SLOT_COMMITTED {
        return base;
    }
    base + weight(p).mul(t::slam_weight_damage()).to_int()
        + p.slam_fall.to_int().max(0) * t::slam_fall_damage()
}

/// Slam's shake radius: the move's own, widened in proportion to how full the
/// shield it spends is.
pub fn slam_radius(p: &Player, kind: u8, base: Fx) -> Fx {
    if p.class != Class::Bulwark || kind != SLOT_COMMITTED {
        return base;
    }
    base.add(fullness(p).mul(t::slam_weight_radius()))
}

/// Does this Slam's shake stagger what it catches? Only near the cap: the
/// area stagger is the thing a full shield buys, and nothing less buys it.
pub fn slam_staggers(p: &Player) -> bool {
    slamming(p) && fullness(p).raw() >= t::slam_stagger_share().raw() && weight(p).raw() > 0
}

/// Slam's active frames are over: whatever it carried is spent, landed or not.
/// Committing to the big hit is the cost, the same way a whiffed Slam's
/// recovery is.
pub fn spend_on_slam(p: &mut Player, kind: u8) {
    if p.class == Class::Bulwark && kind == SLOT_COMMITTED {
        if let Some(s) = shield(p) {
            p.mechanic = Mechanic::Shield(s.with_weight(Fx::ZERO));
        }
    }
}

// ---------------------------------------------------------------------------
// The throw and the wall: M3
// ---------------------------------------------------------------------------

/// How fast a shield holding `weight` flies, out and back. A loaded one is a
/// boulder: slower to arrive, and easier to read coming.
pub fn flight_speed(weight: Fx) -> Fx {
    t::shield_speed().mul(by_fullness(weight, Fx::ONE, t::throw_speed_full()))
}

/// What the thrown shield deals: its own damage, plus a share of its weight.
pub fn throw_damage(weight: Fx) -> i32 {
    t::shield_damage() + weight.mul(t::throw_weight_damage()).to_int()
}

/// Does a thrown shield holding `weight` knock down what it hits?
pub fn knocks_down(weight: Fx) -> bool {
    weight.raw() > 0 && fullness_of(weight).raw() >= t::knockdown_share().raw()
}

/// The planted shield as a solid: a stone standing at full height the moment
/// it lands, sized between `wall_size_empty` and `wall_size_full` by the
/// weight it landed with.
///
/// **One more source for the stones' field, not a second collision path.**
/// `stones::gather` puts it in its owner's first slot -- a Bulwark has no
/// stones of his own, so the slots are always free -- and from there it
/// stops bodies, stops shots and stands under the crosshair exactly as a
/// stone does, through the same functions. It is rebuilt from the shield
/// every frame and never written back, so nothing can kick it, carry it or
/// break it: Cataclysm's `stones::destroy` refuses it. Recalled, it is gone,
/// because the shield it was built from is no longer planted.
pub fn wall(at: V3, weight: Fx) -> Structure {
    Structure {
        // Out of the floor already: a shield is planted, not grown.
        age: 1,
        rise: 1,
        scale: by_fullness(weight, t::wall_size_empty(), t::wall_size_full()),
        ..Structure::raised(at)
    }
}
