//! Perception (bestiary P5): what a creature's glance is allowed to sample.
//!
//! The glance (`Monster::glance`) samples a fighter's position and velocity
//! every few frames. Three creatures need it to sample less: the Sandmaw hears
//! movement and does not see bodies, one of the Pair can be half-blinded and
//! neither cat sees through a wall, and the Veilstalker's whole fight is about
//! what the *hunter* can see. The change is small and central: **each species
//! has a perception filter**, a function in its own file
//! (`FightDecl::perceives`), and a fighter it does not perceive is not
//! sampled. A glance that perceives nobody keeps the old sample, and the old
//! sample keeps being led -- which is what makes breaking line of sight a real
//! thing to do.
//!
//! The filter is handed a [`Perceiver`]: the creature, its head, the fighter
//! as `monster::Quarry` sees them, and the scene. The pieces a filter is built
//! from are here -- a sight cone, a blind arc, a clear line, a feel radius --
//! each reading its numbers off the species' senses row
//! (`species::FightField`). The line of sight is asked through `aim`
//! ([`crate::aim::sight_clear`]), because whether a straight line reaches
//! something is the aiming model's question and nobody else's.
//!
//! Hearing is the other half: a species that `hears` is handed the loudest
//! noise that reached its head since its last glance (`monster::Heard`), and
//! takes its sample from that when it perceives no body. See
//! [`crate::noise`].

use crate::aim::Scene;
use crate::fixed::Fx;
use crate::math::{self, V3};
use crate::monster::{Monster, Quarry};
use crate::species::FightField;

/// Everything a perception filter may look at.
pub struct Perceiver<'a> {
    pub monster: &'a Monster,
    /// Which creature slot it is.
    pub slot: usize,
    /// Where it sees from: its head (`Monster::head`).
    pub head: V3,
    /// The fighter, as the glance would sample them.
    pub quarry: &'a Quarry,
    /// Which fighter.
    pub who: usize,
    pub scene: &'a Scene<'a>,
    /// The hunt's lore, for a species whose filter reads its own state (the
    /// Pair's scarred side).
    pub lore: &'a crate::lore::Lore,
}

/// A perception filter: does this creature perceive this fighter now?
pub type Perceive = fn(&Perceiver) -> bool;

/// **It sees everybody**: the Ridgeback's, and every species that says
/// nothing. Bit-identical to the glance before there were filters.
pub fn sees_all(_: &Perceiver) -> bool {
    true
}

/// **It perceives no bodies at all**: the Sandmaw underground, which knows only
/// what it hears and feels.
pub fn sees_nobody(_: &Perceiver) -> bool {
    false
}

/// The bearing of the fighter off the creature's facing, in turns: zero dead
/// ahead, a half straight behind, positive to its right.
pub fn bearing(p: &Perceiver) -> Fx {
    let to = p.quarry.pos.sub(p.monster.pos);
    let want = math::atan2_turns(to.z, to.x);
    math::wrap_turns(want.sub(p.monster.yaw))
}

/// Inside its sight cone: no further off its facing than the row's
/// `SightCone` (a half-angle, in turns). A half or more sees all round.
pub fn in_sight_cone(p: &Perceiver) -> bool {
    let cone = p.monster.sp().fight_fx(FightField::SightCone);
    bearing(p).abs().raw() <= cone.raw()
}

/// Inside a blind arc on one side -- `side` positive for the right, negative
/// for the left -- of the row's `BlindArc` either side of square off its
/// facing. The Pair's scar: a filter returns `!in_blind_arc(p, scar)`.
pub fn in_blind_arc(p: &Perceiver, side: i32) -> bool {
    if side == 0 {
        return false;
    }
    let half = p.monster.sp().fight_fx(FightField::BlindArc);
    let quarter = math::QUARTER_TURN;
    let centre = if side > 0 { quarter } else { quarter.neg() };
    math::wrap_turns(bearing(p).sub(centre)).abs().raw() <= half.raw()
}

/// Nothing solid between its head and the fighter's middle, and no smoke:
/// [`crate::aim::sight_clear`].
pub fn in_line_of_sight(p: &Perceiver) -> bool {
    let middle = crate::aim::standing_middle(p.quarry.pos, crate::tuning::body_height());
    crate::aim::sight_clear(p.head, middle, p.scene)
}

/// [`in_line_of_sight`], or a clear line to the top of the fighter's head: a
/// body behind something waist-high is still seen. The Pair, who see over a
/// platform's edge a standing fighter's head and shoulders behind it, and
/// not through a standing stone.
pub fn in_sight_over_cover(p: &Perceiver) -> bool {
    if in_line_of_sight(p) {
        return true;
    }
    let crown = p
        .quarry
        .pos
        .add(V3::new(Fx::ZERO, crate::tuning::body_height(), Fx::ZERO));
    crate::aim::sight_clear(p.head, crown, p.scene)
}

/// Felt: within the row's `FeelRadius` of its head in the floor plane, with
/// feet no more than `FeelHeight` off the floor under them. The Sandmaw's
/// undertow is thrown at what it feels.
pub fn felt(p: &Perceiver) -> bool {
    let sp = p.monster.sp();
    let near = math::wide_flat_dist(p.quarry.pos, p.head).raw()
        <= sp.fight_fx(FightField::FeelRadius).raw();
    let floor = p.scene.arena.ground_under(p.quarry.pos);
    let low = p.quarry.pos.y.sub(floor).raw() <= sp.fight_fx(FightField::FeelHeight).raw();
    near && low
}
