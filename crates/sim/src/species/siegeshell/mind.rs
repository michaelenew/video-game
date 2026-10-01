//! What the Siegeshell wants, and what its parasites want.
//!
//! **Most of it is not a brain** (§5). The walk is a gait with a heading, and
//! every footfall is the gait. What is left to decide is which foot does
//! something other than walk -- **the legs' channel**, run here once a frame
//! ([`legs`]) -- and what the shell does about riders and the floor around it
//! -- **the body's channel**, the shared brain's `Doing`, with its own terms
//! in the scoring ([`appetite`]). A move on one channel may not land within
//! `ChannelGap` frames of a move on the other.
//!
//! **Steering** is no pursuit: a point on the valley's centre line ahead of it
//! ([`prowl_to`]), at a turn rate that is barely there.
//!
//! **Its parasites are gnawers**: the Gnawers' own mind ([`gnawers::Mind`]),
//! handed every question first, with [`Roost`] on top -- they keep to the
//! shell until something calls them down.

use super::fight::{self, leg};
use super::{Clip, Knob, SPECIES};
use crate::beast::{self, Pose};
use crate::critter::{Body, Critter, CritterMove, Critters};
use crate::fixed::Fx;
use crate::math::V3;
use crate::monster::{Attack, Doing, Herd, Mind, Monster};
use crate::pack::{Look, Pack, PackDecl, PackMind, Steer};
use crate::sign::Signs;
use crate::species::gnawers;
use crate::state::{Player, World};

// ---------------------------------------------------------------------------
// Steering, and how it looks walking
// ---------------------------------------------------------------------------

/// **Where it walks** (`FightDecl::prowl_to`): the valley's centre line, a
/// way ahead of it. It is not after anybody.
pub fn prowl_to(m: &Monster, _mind: &Mind) -> Option<V3> {
    Some(V3::new(
        m.pos.x.add(Knob::PathAhead.fx().max(Fx::ONE)),
        Fx::ZERO,
        Fx::ZERO,
    ))
}

/// **Its posture while it walks** (`FightDecl::clip`): the walk's sway by the
/// stride while it is walking, the idle when it has halted -- read from its
/// own pace rather than the shared speed, which the shared walk it is held
/// against has braked to nothing.
pub fn posture(m: &Monster) -> Option<Pose> {
    if !matches!(m.doing, Doing::Prowl) {
        return None;
    }
    let sp = m.sp();
    let pace = super::gait::pace(m, fight::phase(m));
    if pace.raw() <= 0 {
        return Some(beast::sample(
            sp,
            Clip::Idle as usize,
            Fx::from_raw(m.beat as i32),
        ));
    }
    let clip = if fight::phase(m) >= 2 {
        Clip::Hurry
    } else {
        Clip::Walk
    };
    Some(beast::sample(
        sp,
        clip as usize,
        Fx::from_raw(m.stride as i32),
    ))
}

// ---------------------------------------------------------------------------
// The body's channel
// ---------------------------------------------------------------------------

/// **Its own terms in the scoring** (`FightDecl::appetite`).
pub fn appetite(_m: &Monster, _kind: u8, _score: i32, _mind: &Mind) -> i32 {
    0
}

/// As a body move commits (`FightDecl::commit`).
pub fn commit(_m: &mut Monster, _kind: u8, _mind: &Mind) {}

// ---------------------------------------------------------------------------
// The legs' channel
// ---------------------------------------------------------------------------

/// **The legs, once a frame**: tick the move under way, or start one.
pub fn legs(_w: &mut World, _slot: usize) {}

/// Where a leg's pad is while the legs' channel has it.
pub fn foot_in_move(_m: &Monster, _c: leg::Channel) -> Option<V3> {
    None
}

/// What the legs' move draws on the floor.
pub fn signs(_w: &World, _m: &Monster, _out: &mut Signs) {}

// ---------------------------------------------------------------------------
// The parasites
// ---------------------------------------------------------------------------

pub const KINDS: [crate::critter::CritterKind; 1] = [gnawers::GNAWER_KIND];

pub static PACK: PackDecl = PackDecl {
    kinds: &KINDS,
    muster: &[],
    leader: None,
    mind: &Roost,
};

/// The parasites' mind: the gnawers', and the shell's roost on top.
pub struct Roost;

const GNAWER: gnawers::Mind = gnawers::Mind;

impl PackMind for Roost {
    fn appetite(&self, look: &Look, i: usize, m: CritterMove, a: &Attack) -> i32 {
        GNAWER.appetite(look, i, m, a)
    }

    fn frame(&self, pack: &mut Pack, critters: &mut Critters, herd: &Herd, frame: u32) {
        GNAWER.frame(pack, critters, herd, frame);
    }

    fn steer(&self, look: &Look, i: usize, want: Steer) -> Steer {
        GNAWER.steer(look, i, want)
    }

    fn landed(
        &self,
        pack: &mut Pack,
        critters: &mut Critters,
        i: usize,
        who: usize,
        victim: &mut Player,
        blocked: bool,
        parried: bool,
    ) {
        GNAWER.landed(pack, critters, i, who, victim, blocked, parried);
    }

    fn hurt(&self, pack: &mut Pack, critters: &mut Critters, i: usize, dealt: i32) {
        GNAWER.hurt(pack, critters, i, dealt);
    }

    fn died(&self, pack: &mut Pack, critters: &mut Critters, i: usize) {
        GNAWER.died(pack, critters, i);
    }

    fn body(&self, c: &Critter, plain: Body) -> Body {
        GNAWER.body(c, plain)
    }
}

/// The species, for the hooks that are handed less than a monster.
pub fn species() -> &'static crate::species::Species {
    &SPECIES
}
