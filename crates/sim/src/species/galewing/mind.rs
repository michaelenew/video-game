//! What the Galewing wants: its own terms in the scoring, and what it keeps
//! as a move commits (`galewing.md` §5).
//!
//! ```text
//! U(m) =  the Ridgeback's terms (range, bearing, closing, hurt, variety)
//!       + lee(m)       Downwash: per target standing exposed near a drop
//!       + follow(m)    the volley after a Downwash; the buffet after a Screech
//!       + hurt(m)      wounded, the Stoop and the pass more
//!       ⟂ place(m)     air moves nothing on the ground, ground moves nothing aloft
//!       ⟂ arc(m)       aloft, nothing unless the target is inside the line-up arc
//!       ⟂ wind(m)      a move it cannot afford scores nothing
//!       ⟂ rider(m)     carrying a rider, nothing at all: the ride is its whole
//!                      attention (§5) -- the roll is the ride's, not a choice
//! ```

use crate::fixed::Fx;
use crate::math::{self, V3};
use crate::monster::{Mind, Monster};

use super::fight::{self, seen};
use super::{
    BUFFET, DOWNWASH, HOP, Knob, SCREECH, SPECIES, STOOP, TALON, VOLLEY, aerial, grounded,
};

/// Its own terms in the scoring, after the shared ones.
pub fn appetite(m: &Monster, kind: u8, score: i32, mind: &Mind) -> i32 {
    let lore = mind.lore;
    if fight::riding(m) {
        return 0;
    }
    let aloft = fight::aloft(m);
    if aerial(kind) {
        if !aloft || lore.word(fight::word::GLIDE) > 0 || fight::grounded_for_good(m) {
            return 0;
        }
        if fight::wind(lore).raw() < Fx::from_int(fight::cost(kind)).raw() {
            return 0;
        }
        // **Only the move this approach is for** (`fight::intend`).
        if fight::intent(lore) != Some(kind) {
            return 0;
        }
        // **It lines up**: only with the target inside the arc of its
        // heading. Where you stand against its circle is when it can come.
        let a = SPECIES.attack(kind);
        let lead = m.lead_point(a.startup);
        let to = V3::new(lead.x.sub(m.pos.x), Fx::ZERO, lead.z.sub(m.pos.z));
        if math::wide_flat_len(to).raw() > 0 {
            let bearing = math::atan2_turns(to.z, to.x);
            let off = math::wrap_turns(bearing.sub(m.yaw)).abs();
            if off.raw() > Knob::LineUpArc.fx().raw() {
                return 0;
            }
        }
        let mut s = score.max(0);
        if s == 0 {
            return 0;
        }
        // **The shelf is the volley's**: twelve metres down a cliff, with an
        // eighteen-metre wingspan, the only thing it throws there is a lane
        // of feathers along it (§3).
        if kind != VOLLEY {
            let floor = mind.ground.ground_under(V3::new(lead.x, Fx::ZERO, lead.z));
            let plateau = mind
                .ground
                .ground_under(V3::new(m.pos.x, Fx::ZERO, m.pos.z));
            let circle = fight::base_of(mind.ground);
            if floor.raw() < circle.min(plateau).sub(crate::tuning::fall_free()).raw() {
                return 0;
            }
        }
        match kind {
            DOWNWASH => {
                // **How exposed the targets are**: each standing where the
                // push would reach them, more for one near a drop or on the
                // tower; nothing if every one of them is sheltered.
                let mut exposed = 0;
                let mut edged = 0;
                for (i, q) in mind.quarry.iter().enumerate().take(4) {
                    if !q.alive || q.aboard {
                        continue;
                    }
                    let b = fight::seen_of(lore, i);
                    if b & seen::EXPOSED != 0 {
                        exposed += 1;
                        if b & seen::EDGE != 0 {
                            edged += 1;
                        }
                    }
                }
                if exposed == 0 {
                    return 0;
                }
                s += Knob::LeeAppetite.raw() * edged;
            }
            VOLLEY => {
                if m.brain.last_move == DOWNWASH && m.brain.repeat_left > 0 {
                    s += Knob::FollowAppetite.raw();
                }
            }
            STOOP | TALON => {
                if fight::below(m, Knob::DesperateHealth.fx()) {
                    s = Fx::from_int(s).mul(Knob::HurtAppetite.fx()).to_int();
                }
                // **Below the last threshold it stoops twice**: the Stoop
                // right after a Stoop.
                if kind == STOOP
                    && m.brain.last_move == STOOP
                    && fight::below(m, Knob::LastHealth.fx())
                {
                    s += Knob::FollowAppetite.raw();
                }
            }
            _ => {}
        }
        // **Patience**: up there it can wait for the shot it wants. A move
        // that fits worse than this is not thrown -- it comes round again.
        if s < Knob::Patience.raw() {
            return 0;
        }
        return s;
    }
    if grounded(kind) {
        if aloft {
            return 0;
        }
        // The hop is the grounded bird's: it has no wings to come down with.
        if kind == HOP && !fight::grounded_for_good(m) {
            return 0;
        }
        // On the perch, only the Screech: the tower top is a ledge.
        if fight::perched(m) && kind != SCREECH {
            return 0;
        }
        let mut s = score.max(0);
        if kind == BUFFET && m.brain.last_move == SCREECH && m.brain.repeat_left > 0 && s > 0 {
            s += Knob::FollowAppetite.raw();
        }
        return s;
    }
    score
}

/// As a move commits: a lobbed move's aim keeps the height of the ground it
/// lands on (the rig has no world to ask).
pub fn commit(m: &mut Monster, kind: u8, mind: &Mind) {
    if kind == STOOP || kind == HOP {
        let aim = m.aimed_at();
        let h = mind.ground.ground_under(V3::new(aim.x, Fx::ZERO, aim.z));
        fight::set_aim_height(m, h);
    }
}
