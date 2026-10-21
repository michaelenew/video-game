//! **A bred body getting up and getting down** (2026-10-21): the creatures'
//! one motion ([`crate::climb::profile`]) read into a [`Drive`], so a morph
//! goes up a ledge the way a Ridgeback does -- front end first, forefeet over
//! the lip, haunches after -- with nothing written for it. A bred body is not
//! a [`crate::species::Species`]; what the climb needs of it is the same
//! three things, read off its posture instead of a skeleton table: how tall it
//! stands, where its fore and hind pairs hang, and how long each leg stands.
//!
//! The body is drawn from where it stood until it is up: the drive raises its
//! root, pitches it, carries it forward and puts each foot where the profile
//! says, over the floor it started on. Whoever draws it moves it on by
//! [`advance`] when the climb ends ([`crate::climb::advance_of`]).

use super::drive::Drive;
use super::posture::{Posture, standing_length};
use super::{Genome, STATIONS};
use crate::climb::{self, Build, Phase};
use crate::fixed::Fx;

/// **The climb's measure of a bred body**: its shoulder (how high its root
/// rides), and where its foremost and hindmost standing pairs hang along it.
/// Its nose and tail are where its torso ends.
pub fn build(g: &Genome, q: &Posture) -> Build {
    let p = super::build::proportions(g);
    let grounded = || (0..STATIONS).filter(|&i| q.stations[i].grounded);
    let front = grounded()
        .map(|i| q.stations[i].attach)
        .fold(None, |a: Option<Fx>, x| Some(a.map_or(x, |a| a.max(x))));
    let rear = grounded()
        .map(|i| q.stations[i].attach)
        .fold(None, |a: Option<Fx>, x| Some(a.map_or(x, |a| a.min(x))));
    Build {
        shoulder: q.root.max(Fx::ZERO),
        front: front.unwrap_or(Fx::ZERO),
        rear: rear.unwrap_or(Fx::ZERO),
        nose: p.front.add(p.neck_len).add(p.head_half),
        tail: p.rear.neg().sub(p.tail_len),
    }
}

/// **Where its lip is**, ahead of where it stands: a quarter of its shoulder
/// past its forefeet -- where a bred body walking at a ledge meets it.
pub fn lip(b: &Build) -> Fx {
    b.front.add(Fx::from_raw(b.shoulder.raw() / 4))
}

/// **How far forward it goes getting up or down**: [`climb::advance_of`],
/// from its lip.
pub fn advance(b: &Build) -> Fx {
    climb::advance_of(b, lip(b))
}

/// Which of its standing pairs go up with the front end: those nearer its
/// foremost pair than its hindmost.
fn fore(b: &Build, attach: Fx) -> bool {
    attach.sub(b.rear).raw() >= b.front.sub(attach).raw()
}

/// **The drive of a bred body getting up `rise` metres (down, below
/// nothing)**, at progress `u`, over `base` -- whatever it was doing with its
/// neck and tail, its standing drive for one at rest -- at `rates`.
pub fn drive(g: &Genome, q: &Posture, base: Drive, u: Fx, rise: Fx, rates: &Rates) -> Drive {
    let b = build(g, q);
    let ph: Phase = climb::profile(u, rise, b.shoulder, rates.clear, rates.dip);
    let pitch = climb::pitch_of(&b, ph.front);
    let mut d = base;
    let raised = rise.mul(ph.body).sub(ph.dip);
    d.lift = d.lift.add(raised);
    d.pitch = d.pitch.add(pitch);
    // The head stays on what it is after, as a creature's does.
    d.neck_pitch = d.neck_pitch.sub(pitch);
    d.surge = d.surge.add(advance(&b).mul(ph.along));
    let p = super::build::proportions(g);
    for i in 0..STATIONS {
        let st = q.stations[i];
        let Some(pair) = g.limbs[i] else { continue };
        if !st.grounded {
            continue;
        }
        let standing = standing_length(g.stance, pair.build, st.wing_walks, p.limb[i]);
        if standing.raw() <= 0 {
            continue;
        }
        // Where the profile has this end's feet, over the floor it started
        // on, as a share of the leg -- the drive's unit -- and measured from
        // under the body, which the drive has raised already.
        let foot = if fore(&b, st.attach) {
            ph.front_foot
        } else {
            ph.rear_foot
        };
        let share = foot.sub(raised).div(standing);
        for side in 0..2 {
            d.feet[i][side].lift = share;
            // Feet go with the body: they stand under the hips, not where
            // the walk had them.
            d.feet[i][side].forward = Fx::ZERO;
        }
    }
    d
}

/// **How it climbs**, from the Oven's morph row: the highest it gets up and
/// the deepest it gets down as shares of its shoulder, the step it walks up
/// without a climb, frames to get up and a metre more, and the shares its
/// feet clear the lip by and it settles by. Every morph climbs alike, scaled
/// by its own body -- what a bred creature is, is what it grew into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rates {
    pub reach: Fx,
    pub drop: Fx,
    pub step: Fx,
    pub frames: i32,
    pub per_metre: i32,
    pub down: Fx,
    pub clear: Fx,
    pub dip: Fx,
}

impl Rates {
    pub fn tuned() -> Rates {
        use crate::tuning as t;
        Rates {
            reach: t::morph_climb_reach(),
            drop: t::morph_climb_drop(),
            step: t::morph_climb_step(),
            frames: t::morph_climb_frames(),
            per_metre: t::morph_climb_per_metre(),
            down: t::morph_climb_down(),
            clear: t::morph_climb_clear(),
            dip: t::morph_climb_dip(),
        }
    }
}

/// **What a bred body does about a change of floor of `rise`** under where
/// it is walking: `None` walks it (a step, or no change); `Some(true)` gets
/// up or down it; `Some(false)` is a ledge or a drop it will not take.
pub fn meets(b: &Build, rates: &Rates, rise: Fx) -> Option<bool> {
    let step = b.shoulder.mul(rates.step);
    if rise.abs().raw() <= step.raw() {
        return None;
    }
    let most = if rise.raw() > 0 {
        rates.reach
    } else {
        rates.drop
    };
    Some(rise.abs().raw() <= b.shoulder.mul(most).raw())
}

/// The frames a bred body takes to get up (or down) `rise`.
pub fn frames(rates: &Rates, rise: Fx) -> u16 {
    climb::frames_at(rise, rates.frames, rates.per_metre, rates.down)
}
