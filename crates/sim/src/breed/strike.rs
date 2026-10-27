//! How a move looks: every move as a body in motion, and its hit read off
//! the parts that swing.
//!
//! A move has three poses -- **rest**, **cocked** and **struck** -- and the
//! frames between them come from its numbers ([`MoveSpec`]):
//!
//! - **Windup** (its startup): rest to cocked, fast at first and settling
//!   into the cocked pose, so the silhouette has changed most of the way by
//!   a third of the windup. The windup is the tell; a tell that arrives at
//!   the end of its windup is one nobody can read in time.
//! - **Strike** (its active frames): cocked to struck, accelerating. What
//!   is hit is what the striking parts pass through on these frames, so the
//!   hit is the move's own bones, never a sphere placed beside them.
//! - **Recovery**: held in the struck pose for the first third -- stuck,
//!   committed, the punish window -- then back to rest. The body stays
//!   where the strike carried it: a lunge ends a lunge away.
//!
//! Every move also says how it is answered (the bestiary's column, §1.2)
//! and how it is delivered: by a part swung through you, an area round a
//! point, a thing thrown, or nothing at all (a guard, a leap). Which lane
//! and which range a move threatens is not declared but measured, by
//! [`super::verdict`], off where its hit actually goes.

use super::build::{self, Body, MoveKind, MoveSpec, proportions};
use super::drive::{Drive, Wings};
use super::locomotion;
use super::posture::Posture;
use super::{Build, Genome, Role, STATIONS, Station, fx};
use crate::fixed::{Fx, sin_turns};
use crate::math::V3;

/// How a move is answered: the bestiary's vocabulary (§1.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Answer {
    /// Roll through it, on the frames it is out.
    Dodge,
    /// Over it: it is low.
    Jump,
    /// Step aside: it is aimed and locks.
    Sidestep,
    /// Off the line: it travels straight and far.
    LeaveLane,
    /// Behind something solid: it is thrown.
    Cover,
    /// Under it: it passes high.
    Crouch,
    /// It is slow and open: hit it first.
    HitFirst,
    /// Off its back: it is for riders.
    LeaveBack,
    /// Out of reach: where you stand is the answer.
    BackOff,
    /// Round to its side: it covers a front or a rear and nothing else.
    Flank,
    /// Stop swinging: it only hurts the one who hits it.
    Hold,
    /// Off the churning ground: it is coming up there.
    OffTheMark,
}

impl Answer {
    pub const fn name(self) -> &'static str {
        match self {
            Answer::Dodge => "dodge",
            Answer::Jump => "jump",
            Answer::Sidestep => "sidestep",
            Answer::LeaveLane => "leave the lane",
            Answer::Cover => "get behind a solid",
            Answer::Crouch => "crouch",
            Answer::HitFirst => "hit it first",
            Answer::LeaveBack => "leave its back",
            Answer::BackOff => "back off",
            Answer::Flank => "flank",
            Answer::Hold => "hold",
            Answer::OffTheMark => "step off the mark",
        }
    }

    /// Answered by where you stand rather than by reacting: the only kind
    /// a move faster than a reaction may have (§1.4).
    pub const fn positional(self) -> bool {
        matches!(
            self,
            Answer::BackOff | Answer::Flank | Answer::LeaveLane | Answer::LeaveBack | Answer::Hold
        )
    }
}

/// How a move's hit arrives.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Delivery {
    /// The striking parts themselves, swept through the strike.
    Swept,
    /// An area round where the striking parts come down, its radius a
    /// share of the body's length.
    Area(Fx),
    /// Something thrown from the striking parts, out to the move's reach.
    Thrown,
    /// No hit: a guard, a leap, a wallow.
    None,
}

/// Which parts carry a move: names that begin with one of these.
pub type Strikers = [&'static str; 4];

/// One move as motion.
#[derive(Clone, Copy, Debug)]
pub struct Motion {
    pub kind: MoveKind,
    /// **An artificial tell** (exploration 0033): a move whose windup the
    /// whole body cannot show -- one begun under the floor, one the body
    /// has no room to gather for -- is announced by a cry of its own, each
    /// of a body's cried moves a different one.
    pub cry: Option<Cry>,
    /// What the strike is brought to, kept so it can be aimed again once
    /// the move is lifted clear of the floor ([`kit`]).
    pub aim: Option<Aim>,
    pub cocked: Drive,
    pub struck: Drive,
    pub strikers: Strikers,
    pub delivery: Delivery,
    pub answer: Answer,
    /// How far the body rises at the middle of the strike, metres: the
    /// top of a leap's arc.
    pub arc: Fx,
    /// Where the windup starts from, over the body's rest: nothing for a
    /// move begun standing; lying under the floor for one begun there.
    pub from: Drive,
    /// For a thrown move, how far ahead it was aimed: the shot comes down
    /// to chest height there and on to the floor past it. Nought is its
    /// full reach.
    pub aim_at: Fx,
}

const NONE: Strikers = ["", "", "", ""];

/// The prefix a station's limb parts are named by.
const fn limb_prefix(i: usize) -> &'static str {
    match i {
        0 => "fore.",
        1 => "midfore.",
        2 => "midhind.",
        _ => "hind.",
    }
}

/// The prefix one side of a station's limb is named by.
const fn limb_side(i: usize, side: usize) -> &'static str {
    match (i, side) {
        (0, 0) => "fore.l",
        (0, _) => "fore.r",
        (1, 0) => "midfore.l",
        (1, _) => "midfore.r",
        (2, 0) => "midhind.l",
        (2, _) => "midhind.r",
        (_, 0) => "hind.l",
        _ => "hind.r",
    }
}

/// The first pair that is swung as `want` says, fore to hind.
fn station_where(g: &Genome, want: impl Fn(Station, Role, Build) -> bool) -> Option<usize> {
    g.pairs()
        .find(|(st, l)| want(*st, l.role, l.build))
        .map(|(st, _)| st.index())
}

fn fore(st: Station) -> bool {
    matches!(st, Station::Fore | Station::MidFore)
}

/// How high a jump has to rise to clear a low strike: a tail swept at the
/// shins, a shockwave along the floor.
pub const JUMP_CLEAR: Fx = fx(1, 1);

/// Frames an answer takes to work once a fighter starts it: a jump's rise
/// to [`JUMP_CLEAR`] at the game's own jump speed and gravity; nothing for
/// a dodge, whose invulnerability starts at once, or a crouch.
pub fn lead(answer: Answer) -> i32 {
    match answer {
        Answer::Jump => {
            // Solve h = v t - g t^2 / 2 for the first t.
            let v = crate::tuning::jump_speed();
            let g = crate::tuning::gravity().abs().max(fx(1, 10));
            let under = v
                .mul(v)
                .sub(g.mul(JUMP_CLEAR).mul(Fx::from_int(2)))
                .max(Fx::ZERO);
            let t = v.sub(under.sqrt()).div(g);
            t.mul(Fx::from_int(super::locomotion::FPS)).to_int() + 1
        }
        // Out of the widest mark at a run.
        Answer::OffTheMark => {
            let out = MARK_MOST.add(crate::tuning::body_radius());
            out.div(crate::tuning::move_speed().max(fx(1, 10)))
                .mul(Fx::from_int(super::locomotion::FPS))
                .to_int()
                + 1
        }
        _ => 0,
    }
}

/// The widest an eruption's mark is: a body bigger than twelve metres
/// comes up no wider, so stepping off it is always a few strides.
pub const MARK_MOST: Fx = fx(3, 1);

/// How tall an eruption's column is: no jump clears it.
pub const ERUPT_HEIGHT: Fx = fx(5, 2);

/// The radius of the ground that churns where a body will come up.
pub fn mark_radius(len: Fx) -> Fx {
    len.mul(fx(1, 4)).min(MARK_MOST).max(Fx::ONE)
}

/// How deep a body that burrows lies, over its rest: its top under the
/// floor.
pub fn under_drive(g: &Genome, q: &Posture) -> Drive {
    let p = proportions(g);
    let mut d = Drive::STILL;
    d.lift = q
        .root
        .add(p.half_h.mul(Fx::from_int(3)))
        .add(fx(1, 2))
        .neg();
    d.pitch = fx(-1, 40);
    if g.stance == super::Stance::Limbless {
        d.rise = fx(-1, 40);
    }
    d
}

/// How far an eruption comes up from where the body lies: half again its
/// length, and never less than four metres.
pub fn erupt_reach(len: Fx) -> Fx {
    len.mul(fx(3, 2)).max(Fx::from_int(4))
}

/// The height between a crouched fighter's head and a standing one's: a
/// strike that passes here is ducked, and hits anybody who does not.
pub fn crouch_line() -> Fx {
    let standing = crate::tuning::body_height();
    let crouched = standing.mul(crate::tuning::crouch_height_scale());
    standing.add(crouched).mul(fx(1, 2))
}

/// How each move is answered.
pub const fn answer(kind: MoveKind) -> Answer {
    use MoveKind::*;
    match kind {
        Bite => Answer::Sidestep,
        Peck => Answer::Dodge,
        Swallow => Answer::Dodge,
        Tongue => Answer::LeaveLane,
        Plough => Answer::Sidestep,
        Gore => Answer::Sidestep,
        Charge => Answer::LeaveLane,
        TailSweep => Answer::Jump,
        TailSpear => Answer::Flank,
        Sting => Answer::Sidestep,
        TailTrip => Answer::Jump,
        Stomp => Answer::Jump,
        Rake => Answer::BackOff,
        Scythe => Answer::BackOff,
        Lunge => Answer::LeaveLane,
        Guard => Answer::Flank,
        Parry => Answer::Hold,
        LegStab => Answer::BackOff,
        BackKick => Answer::Flank,
        Pounce => Answer::Sidestep,
        Leap => Answer::Flank,
        Stoop => Answer::Dodge,
        TalonPass => Answer::Crouch,
        Downwash => Answer::BackOff,
        Buffet => Answer::BackOff,
        FeatherVolley => Answer::Cover,
        WingFlare => Answer::BackOff,
        RearSlam => Answer::HitFirst,
        BellyFlop => Answer::BackOff,
        SacSlam => Answer::Flank,
        RiseBite => Answer::Sidestep,
        BreachDive => Answer::Dodge,
        Lash => Answer::Jump,
        Spit => Answer::Cover,
        Shake => Answer::LeaveBack,
        Inflate => Answer::BackOff,
        Ambush => Answer::LeaveLane,
        Shoulder => Answer::BackOff,
        Rush => Answer::LeaveLane,
        Burrow => Answer::HitFirst,
        Erupt => Answer::OffTheMark,
        SpikeSpray => Answer::Cover,
        QuillFling => Answer::Sidestep,
        WebShot => Answer::Jump,
        Screech => Answer::HitFirst,
        Spew => Answer::Flank,
        Wallow => Answer::HitFirst,
        Shrug => Answer::LeaveBack,
        Shed => Answer::BackOff,
    }
}

/// Every move kind's motion, for this body.
pub fn motion(g: &Genome, q: &Posture, kind: MoveKind) -> Motion {
    use MoveKind::*;
    let p = proportions(g);
    let h = q.root.max(fx(1, 10));
    let len = p.len;
    let mut c = Drive::STILL;
    let mut s = Drive::STILL;
    let mut strikers = NONE;
    // A strike aimed at a height on the floor ahead: its head is brought
    // to it, so the hit is where somebody standing there is.
    let mut aim: Option<Aim> = None;
    // How high the body arcs through the strike, at its middle: a leap.
    let mut arc = Fx::ZERO;
    let mut from = Drive::STILL;
    let delivery;
    // A limb that swings on the floor (it walks too) is driven by its
    // feet; one held up off it, by its arms.
    let swing_foot = |d: &mut Drive, i: usize, side: usize, forward: Fx, lift: Fx| {
        d.feet[i][side] = super::drive::Foot { forward, lift };
    };
    match kind {
        Bite => {
            // Drawn back and up, the jaw opening: then down onto you.
            c.neck_reach = fx(-2, 5);
            c.neck_pitch = fx(1, 10);
            c.pitch = fx(1, 80);
            c.jaw = Fx::ONE;
            c.lift = h.mul(fx(-1, 12));
            c.rise = fx(1, 30);
            if g.stance == super::Stance::Limbless {
                // A snake's coil: an S drawn back along the ground, the
                // head kept low (the wave's own lift taken off).
                c.rise = fx(-1, 40);
                c.neck_pitch = Fx::ZERO;
                c.wave = Some((Fx::ZERO, fx(1, 10)));
            }
            s.neck_reach = fx(1, 2);
            s.surge = len.mul(fx(1, 8));
            aim = Some(Aim::Head(fx(4, 5)));
            strikers = ["head", "jaw", "horns", ""];
            delivery = Delivery::Swept;
        }
        Peck => {
            // The head high and cocked back, the body still: a bird's.
            c.neck_reach = fx(-1, 3);
            c.neck_pitch = fx(1, 8);
            c.tail_pitch = fx(1, 30);
            s.neck_reach = fx(1, 2);
            s.surge = len.mul(fx(1, 10));
            aim = Some(Aim::Head(fx(1, 2)));
            strikers = ["head", "jaw", "crest", ""];
            delivery = Delivery::Swept;
        }
        Swallow => {
            // Low, swollen with a breath, the jaw wide and skimming the
            // ground: everything about it says *mouth*.
            c.jaw = Fx::ONE;
            c.neck_reach = fx(-1, 5);
            c.neck_pitch = fx(-1, 16);
            c.lift = h.mul(fx(-1, 5));
            c.swell = fx(1, 5);
            c.rise = fx(-1, 60);
            s.jaw = Fx::ONE;
            s.neck_reach = fx(3, 4);
            s.surge = len.mul(fx(1, 4));
            aim = Some(Aim::Head(fx(1, 2)));
            strikers = ["head", "jaw", "", ""];
            delivery = Delivery::Swept;
        }
        Tongue => {
            // Up on its legs, the head raised and still, pointing: aiming.
            c.jaw = fx(1, 3);
            c.neck_pitch = fx(1, 20);
            c.neck_reach = fx(1, 10);
            c.lift = h.mul(fx(1, 8));
            s.jaw = Fx::ONE;
            s.neck_reach = fx(1, 4);
            s.neck_pitch = fx(-1, 40);
            strikers = ["head", "jaw", "", ""];
            delivery = Delivery::Thrown;
        }
        Plough => {
            c.neck_pitch = fx(-1, 8);
            c.lift = h.mul(fx(-1, 10));
            c.neck_reach = fx(-1, 5);
            c.surge = len.mul(fx(-1, 12));
            s.neck_reach = fx(1, 4);
            s.surge = len.mul(fx(1, 2));
            aim = Some(Aim::Head(fx(1, 2)));
            strikers = ["head", "jaw", "shoulders", ""];
            delivery = Delivery::Swept;
        }
        Gore => {
            // Head down, horns forward, sunk on its legs: then up and
            // through, tossing.
            c.neck_pitch = fx(-1, 8);
            c.lift = h.mul(fx(-1, 10));
            c.neck_reach = fx(-1, 5);
            c.surge = len.mul(fx(-1, 16));
            c.neck_yaw = fx(1, 12);
            s.neck_pitch = fx(1, 10);
            s.neck_reach = fx(1, 4);
            s.surge = len.mul(fx(1, 6));
            // Tossed up through somebody standing, not over them: a tall
            // body's horns went through the air above a head (exploration
            // 0033).
            aim = Some(Aim::Head(fx(6, 5)));
            strikers = ["horns", "head", "", ""];
            delivery = Delivery::Swept;
        }
        Charge => {
            // Head down before it goes (exploration 0033): wound up with
            // the head high, a big body's head only came down once it was
            // past somebody standing close, and the charge went over them.
            c.neck_pitch = fx(-1, 6);
            c.tail_pitch = fx(1, 12);
            c.pitch = fx(-1, 80);
            for side in 0..2 {
                swing_foot(&mut c, STATIONS - 1, side, fx(-1, 5), Fx::ZERO);
            }
            // A forefoot pawing the ground, high.
            swing_foot(&mut c, 0, 0, fx(-1, 6), fx(1, 3));
            s.neck_pitch = fx(-1, 10);
            s.surge = len.mul(Fx::from_int(3));
            // Low enough to meet somebody, high enough that a big head's
            // jaw does not plough the floor.
            aim = Some(Aim::Head(Fx::ONE.add(p.head_half.mul(fx(8, 5)))));
            strikers = ["horns", "head", "shoulders", "fore."];
            delivery = Delivery::Swept;
        }
        TailSweep => {
            c.tail_yaw = fx(-1, 24);
            c.neck_yaw = fx(1, 12);
            c.lift = h.mul(fx(-1, 20));
            s.tail_yaw = fx(1, 12);
            // The body turns with the tail to wind it, then spins through:
            // a turn of the body toward `+z` swings the tail to `-z`, the
            // same way as the tail's own yaw.
            c.yaw = fx(-1, 16);
            s.yaw = fx(1, 8);
            s.neck_yaw = fx(-1, 24);
            aim = Some(Aim::Tail(Fx::ONE));
            strikers = ["tail", "", "", ""];
            delivery = Delivery::Swept;
        }
        TailSpear => {
            c.tail_pitch = fx(1, 16);
            c.neck_yaw = fx(1, 16);
            s.tail_pitch = fx(-1, 40);
            s.surge = len.mul(fx(-1, 10));
            aim = Some(Aim::Tail(Fx::ONE));
            strikers = ["tail tip", "tail, middle", "", ""];
            delivery = Delivery::Swept;
        }
        Sting => {
            if len > Fx::from_int(6) {
                // Too big to arc a sting over its back onto somebody
                // standing: it stabs down behind it instead.
                c.tail_pitch = fx(1, 10);
                c.neck_yaw = fx(1, 12);
                s.tail_pitch = fx(-1, 30);
                aim = Some(Aim::Tail(Fx::ONE));
            } else {
                c.tail_pitch = fx(1, 10);
                c.pitch = fx(1, 60);
                s.tail_pitch = fx(3, 20);
                s.pitch = fx(-1, 60);
            }
            strikers = ["tail tip", "", "", ""];
            delivery = Delivery::Swept;
        }
        TailTrip => {
            c.tail_yaw = fx(-1, 30);
            c.lift = h.mul(fx(-1, 10));
            s.tail_yaw = fx(1, 15);
            c.yaw = fx(-1, 24);
            s.yaw = fx(1, 12);
            aim = Some(Aim::Tail(fx(1, 2)));
            // The low end of it: what sweeps the shins, and what a jump
            // clears.
            strikers = ["tail tip", "tail, middle", "", ""];
            delivery = Delivery::Swept;
        }
        Stomp => {
            let i = station_where(g, |st, r, b| {
                fore(st) && r == Role::Both && b == Build::Column
            })
            .unwrap_or(0);
            // One foot up, the body on the other three: tipped nose-up and
            // lifted as it was, the other forefoot left the floor too and
            // the weight hung ahead of the hind feet (exploration 0033).
            c.pitch = fx(1, 120);
            swing_foot(&mut c, i, 0, fx(3, 10), fx(4, 5));
            s.pitch = fx(-1, 80);
            s.lift = h.mul(fx(-1, 20));
            swing_foot(&mut s, i, 0, fx(1, 4), Fx::ZERO);
            strikers = [limb_side(i, 0), "", "", ""];
            delivery = Delivery::Area(fx(1, 3));
        }
        Rake => {
            c.arms = [Fx::ONE.neg(), fx(-1, 2)];
            c.pitch = fx(1, 40);
            s.arms = [Fx::ONE, fx(1, 2)];
            s.surge = len.mul(fx(1, 10));
            let i = station_where(g, |_, r, b| r != Role::Walk && b == Build::Sprung).unwrap_or(0);
            strikers = [limb_prefix(i), "", "", ""];
            aim = Some(Aim::Body(Fx::ONE));
            delivery = Delivery::Swept;
        }
        Scythe => {
            c.arms = [Fx::ONE.neg(), Fx::ONE.neg()];
            c.pitch = fx(1, 30);
            c.neck_pitch = fx(1, 30);
            s.arms = [Fx::ONE, Fx::ONE];
            s.pitch = fx(-1, 40);
            s.surge = len.mul(fx(1, 8));
            let i = station_where(g, |_, _, b| b == Build::Raptorial).unwrap_or(0);
            strikers = [limb_prefix(i), "", "", ""];
            aim = Some(Aim::Body(Fx::ONE));
            delivery = Delivery::Swept;
        }
        Lunge => {
            // Rocked back on its hind legs, both blades drawn: then the
            // whole body thrown forward along a line.
            c.arms = [fx(-1, 2), fx(-1, 2)];
            c.lift = h.mul(fx(-1, 8));
            c.pitch = fx(1, 20);
            c.surge = len.mul(fx(-1, 6));
            s.arms = [Fx::ONE, Fx::ONE];
            s.pitch = fx(-1, 30);
            s.lift = h.mul(fx(1, 20));
            let i = station_where(g, |_, _, b| b == Build::Raptorial).unwrap_or(0);
            s.surge = proportions(g).limb[i].mul(Fx::from_int(2));
            strikers = [limb_prefix(i), "", "", ""];
            aim = Some(Aim::Body(Fx::ONE));
            delivery = Delivery::Swept;
        }
        Guard => {
            c.arms = [fx(-1, 3), fx(-1, 3)];
            c.lift = h.mul(fx(-1, 20));
            c.neck_reach = fx(-1, 5);
            s = c;
            delivery = Delivery::None;
        }
        Parry => {
            c.arms = [fx(-1, 3), fx(-1, 3)];
            c.neck_reach = fx(-1, 5);
            s.arms = [fx(3, 4), fx(-1, 2)];
            let i = station_where(g, |_, r, _| r == Role::Strike).unwrap_or(0);
            strikers = [limb_prefix(i), "", "", ""];
            aim = Some(Aim::Body(Fx::ONE));
            delivery = Delivery::Swept;
        }
        LegStab => {
            let i = station_where(g, |st, r, b| {
                fore(st) && r != Role::Walk && b == Build::Spindle
            })
            .unwrap_or(0);
            c.pitch = fx(1, 40);
            c.lift = h.mul(fx(1, 10));
            swing_foot(&mut c, i, 0, fx(1, 2), fx(1, 2));
            swing_foot(&mut s, i, 0, fx(3, 5), Fx::ZERO);
            // The other leg of the pair comes up with it only on a body
            // with more than one other pair to stand on: on two pairs, it
            // stabs with one and stands on three (exploration 0034).
            let others = (0..STATIONS)
                .filter(|&k| k != i && q.stations[k].grounded)
                .count();
            if others >= 2 {
                swing_foot(&mut c, i, 1, fx(1, 3), fx(1, 3));
                swing_foot(&mut s, i, 1, fx(1, 2), fx(1, 10));
            }
            s.pitch = fx(-1, 60);
            strikers = [limb_side(i, 0), "", "", ""];
            delivery = Delivery::Swept;
        }
        BackKick => {
            let i = station_where(g, |st, r, b| {
                !fore(st) && r != Role::Walk && b == Build::Column
            })
            .unwrap_or(STATIONS - 1);
            c.pitch = fx(-1, 30);
            c.neck_yaw = fx(1, 10);
            // The weight thrown forward onto the forefeet first, as a horse
            // kicking does: the body over them, the forefeet where they
            // stood (exploration 0033). With the hind feet up and the
            // weight where it was, it would sit down.
            let front = (0..STATIONS)
                .filter(|&k| k != i && q.stations[k].grounded)
                .map(|k| (k, q.stations[k].attach.add(q.stations[k].reach)))
                .max_by_key(|(_, x)| *x);
            if let Some((k, x)) = front {
                let shift = x.sub(q.com).add(p.torso.mul(fx(1, 10))).max(Fx::ZERO);
                let standing = p.limb[k].max(fx(1, 10));
                for d in [&mut c, &mut s] {
                    d.surge = d.surge.add(shift);
                    for side in 0..2 {
                        d.feet[k][side].forward = shift.neg().div(standing);
                    }
                }
            }
            for side in 0..2 {
                swing_foot(&mut c, i, side, fx(1, 4), fx(1, 4));
                swing_foot(&mut s, i, side, fx(-1, 2), fx(1, 5));
            }
            s.pitch = fx(-1, 20);
            strikers = [limb_prefix(i), "", "", ""];
            delivery = Delivery::Swept;
        }
        Pounce => {
            // Pressed flat, haunches gathered under, tail up, the head low
            // and on you: the crouch every cat makes.
            c.lift = h.mul(fx(-2, 5));
            c.pitch = fx(-1, 30);
            c.neck_pitch = fx(-1, 20);
            c.tail_pitch = fx(1, 20);
            c.arms = [fx(-1, 3), fx(-1, 3)];
            for i in 0..STATIONS {
                for side in 0..2 {
                    let k = if i >= 2 { fx(1, 4) } else { fx(-1, 10) };
                    swing_foot(&mut c, i, side, k, Fx::ZERO);
                }
            }
            arc = h.mul(fx(1, 2));
            s.pitch = fx(1, 40);
            s.jaw = Fx::ONE;
            s.arms = [Fx::ONE, Fx::ONE];
            // The front pair it stands on reaches for you: a cat's
            // forepaws, a bird's talons.
            let front = station_where(g, |_, r, _| r.grounded()).unwrap_or(0);
            for side in 0..2 {
                swing_foot(&mut s, front, side, fx(2, 5), fx(1, 3));
            }
            s.surge = len.mul(Fx::from_int(3));
            aim = Some(Aim::Body(fx(1, 2)));
            strikers = [limb_prefix(front), "head", "jaw", ""];
            delivery = Delivery::Swept;
        }
        Leap => {
            c.lift = h.mul(fx(-1, 5));
            c.tail_pitch = fx(1, 30);
            arc = h;
            s.surge = len.mul(Fx::from_int(2));
            delivery = Delivery::None;
        }
        Stoop => {
            c.wings = Wings::Aloft;
            c.lift = len;
            c.pitch = fx(1, 12);
            c.beat = fx(1, 8);
            s.wings = Wings::Aloft;
            s.lift = len.mul(fx(-1, 2));
            s.pitch = fx(-1, 12);
            s.beat = fx(-1, 10);
            s.surge = len.mul(Fx::from_int(4));
            aim = Some(Aim::Body(fx(1, 2)));
            strikers = ["hind.", "head", "jaw", "fore."];
            delivery = Delivery::Swept;
        }
        TalonPass => {
            c.wings = Wings::Aloft;
            c.lift = len.mul(fx(1, 3));
            c.beat = fx(1, 10);
            s.wings = Wings::Aloft;
            s.lift = len.mul(fx(-1, 4));
            s.beat = fx(-1, 12);
            for i in 0..STATIONS {
                for side in 0..2 {
                    swing_foot(&mut s, i, side, fx(3, 5), fx(1, 5));
                }
            }
            s.surge = len;
            // Between a crouch and a standing head: the crouch is the answer.
            aim = Some(Aim::Body(crouch_line()));
            strikers = ["hind.", "midhind.", "fore.", ""];
            delivery = Delivery::Swept;
        }
        Downwash => {
            c.wings = Wings::Aloft;
            c.lift = len.mul(fx(1, 2));
            c.beat = fx(1, 6);
            s.wings = Wings::Aloft;
            s.lift = len.mul(fx(1, 3));
            s.beat = fx(-1, 6);
            strikers = ["wing_", "", "", ""];
            delivery = Delivery::Area(fx(1, 1));
        }
        Buffet => {
            c.wings = Wings::Flared;
            c.beat = fx(1, 8);
            c.pitch = fx(1, 40);
            s.wings = Wings::Flared;
            s.beat = fx(-1, 6);
            s.surge = len.mul(fx(1, 10));
            // Swept at the height of a standing head: crouch under it.
            aim = Some(Aim::Wing(fx(6, 5)));
            strikers = ["wing_", "", "", ""];
            // The wings pass over a standing head; the gust under them is
            // what lands.
            delivery = Delivery::Area(fx(2, 5));
        }
        FeatherVolley => {
            c.wings = Wings::Aloft;
            c.lift = len.mul(fx(1, 2));
            c.beat = fx(1, 6);
            c.pitch = fx(1, 30);
            s.wings = Wings::Aloft;
            s.lift = len.mul(fx(1, 2));
            s.beat = fx(-1, 8);
            strikers = ["wing_", "", "", ""];
            delivery = Delivery::Thrown;
        }
        WingFlare => {
            c.wings = Wings::Flared;
            c.neck_pitch = fx(1, 16);
            c.swell = fx(1, 10);
            c.jaw = fx(1, 2);
            s.wings = Wings::Flared;
            s.beat = fx(1, 10);
            s.jaw = Fx::ONE;
            s.surge = len.mul(fx(1, 12));
            strikers = ["wing_", "head", "", ""];
            delivery = Delivery::Area(fx(1, 2));
        }
        RearSlam => {
            c.pitch = fx(1, 8);
            c.lift = h.mul(fx(1, 3));
            c.neck_pitch = fx(1, 20);
            for side in 0..2 {
                swing_foot(&mut c, 0, side, fx(1, 5), fx(1, 2));
            }
            s.pitch = fx(-1, 40);
            s.lift = h.mul(fx(-1, 10));
            s.neck_pitch = fx(-1, 20);
            for side in 0..2 {
                swing_foot(&mut s, 0, side, fx(1, 3), Fx::ZERO);
            }
            s.surge = len.mul(fx(1, 6));
            strikers = ["fore.", "shoulders", "head", ""];
            delivery = Delivery::Area(fx(1, 3));
        }
        BellyFlop => {
            c.lift = h.mul(fx(1, 2));
            c.pitch = fx(1, 30);
            c.swell = fx(1, 10);
            for i in 0..STATIONS {
                for side in 0..2 {
                    swing_foot(&mut c, i, side, Fx::ZERO, Fx::ZERO);
                }
            }
            s.lift = h.mul(fx(-4, 5));
            s.surge = len.mul(fx(1, 6));
            for i in 0..STATIONS {
                for side in 0..2 {
                    swing_foot(&mut s, i, side, Fx::ZERO, fx(2, 5));
                }
            }
            strikers = ["barrel", "haunch", "shoulders", "abdomen"];
            delivery = Delivery::Area(fx(1, 2));
        }
        SacSlam => {
            c.abdomen = fx(1, 6);
            c.pitch = fx(-1, 60);
            c.lift = h.mul(fx(1, 10));
            s.abdomen = fx(-1, 10);
            s.pitch = fx(-1, 40);
            s.lift = h.mul(fx(-1, 6));
            strikers = ["abdomen", "sacs", "", ""];
            delivery = Delivery::Area(fx(1, 3));
        }
        RiseBite => {
            // The front reared high above you, the head hooked down.
            c.rise = fx(1, 12);
            c.neck_pitch = fx(-1, 16);
            c.jaw = Fx::ONE;
            s.neck_reach = fx(1, 2);
            s.surge = len.mul(fx(1, 3));
            aim = Some(Aim::Head(fx(4, 5)));
            strikers = ["head", "jaw", "neck5", ""];
            delivery = Delivery::Swept;
        }
        BreachDive => {
            // Down into the ground, the floor heaving where it went.
            c.rise = fx(-1, 20);
            c.lift = h.mul(fx(-3, 2));
            c.wave = Some((Fx::ZERO, fx(1, 20)));
            s.lift = Fx::ZERO;
            arc = h.mul(Fx::from_int(2));
            s.rise = fx(1, 20);
            s.jaw = Fx::ONE;
            s.surge = len.mul(Fx::from_int(2));
            strikers = ["head", "jaw", "neck", ""];
            delivery = Delivery::Area(fx(1, 3));
        }
        Lash => {
            c.wave = Some((fx(1, 4), fx(1, 10)));
            c.neck_yaw = fx(1, 10);
            s.wave = Some((fx(3, 4), fx(1, 6)));
            c.yaw = fx(-1, 20);
            s.yaw = fx(1, 10);
            strikers = ["tail", "", "", ""];
            delivery = Delivery::Swept;
        }
        Spit => {
            // The head thrown back and the throat swollen: then forward.
            c.neck_reach = fx(-2, 5);
            c.neck_pitch = fx(1, 8);
            c.jaw = fx(1, 2);
            c.swell = fx(1, 6);
            if g.stance == super::Stance::Limbless {
                // Half-raised and swollen, drawn back: lower than a rearing
                // bite, so the two are told apart.
                c.rise = fx(1, 60);
                c.neck_pitch = Fx::ZERO;
                c.swell = fx(1, 4);
                c.surge = len.mul(fx(-1, 10));
            }
            s.neck_reach = fx(1, 4);
            s.jaw = Fx::ONE;
            s.neck_pitch = fx(-1, 40);
            strikers = ["head", "jaw", "", ""];
            delivery = Delivery::Thrown;
        }
        Shake => {
            c.lift = h.mul(fx(-1, 10));
            c.swell = fx(1, 20);
            s.lift = h.mul(fx(1, 10));
            s.pitch = fx(1, 40);
            s.neck_yaw = fx(1, 8);
            s.tail_yaw = fx(-1, 20);
            strikers = ["barrel", "shoulders", "haunch", ""];
            delivery = Delivery::Area(fx(1, 4));
        }
        Inflate => {
            // Swollen where it stands: lifted as well, its forefeet left
            // the floor (exploration 0033).
            c.swell = fx(2, 5);
            c.pitch = fx(1, 40);
            c.neck_reach = fx(-1, 3);
            s.swell = fx(3, 5);
            strikers = ["barrel", "shoulders", "haunch", ""];
            delivery = Delivery::Area(fx(1, 2));
        }
        Ambush => {
            // Flattened, still, the head down along the floor.
            c.lift = h.mul(fx(-1, 3));
            c.neck_pitch = fx(-1, 12);
            c.neck_reach = fx(-1, 5);
            c.tail_pitch = fx(-1, 30);
            s.lift = h.mul(fx(1, 4));
            s.jaw = Fx::ONE;
            s.neck_reach = fx(1, 3);
            s.surge = len.mul(Fx::from_int(2));
            aim = Some(Aim::Head(Fx::ONE));
            strikers = ["head", "jaw", "fore.", ""];
            delivery = Delivery::Swept;
        }
        Shoulder => {
            // Turned side-on, the head swung away, the weight sunk.
            c.lift = h.mul(fx(-1, 6));
            // Leaning in, the shoulder dropped toward you.
            c.pitch = fx(-1, 24);
            c.neck_yaw = fx(-1, 6);
            c.neck_reach = fx(-1, 6);
            c.tail_yaw = fx(1, 30);
            c.surge = len.mul(fx(-1, 16));
            s.surge = len.mul(fx(1, 5));
            s.neck_yaw = fx(-1, 8);
            strikers = ["shoulders", "thorax", "", ""];
            delivery = Delivery::Swept;
        }
        Rush => {
            // Head down, weight back on the haunches, then a run along a
            // line: through every range between it and you.
            c.neck_pitch = fx(-1, 12);
            c.lift = h.mul(fx(-1, 8));
            c.surge = len.mul(fx(-1, 12));
            c.tail_pitch = fx(1, 24);
            for side in 0..2 {
                swing_foot(&mut c, STATIONS - 1, side, fx(1, 5), Fx::ZERO);
            }
            if g.stance == super::Stance::Limbless {
                c.wave = Some((Fx::ZERO, fx(1, 10)));
                s.wave = Some((fx(1, 2), fx(1, 16)));
            }
            s.neck_pitch = fx(-1, 12);
            s.surge = rush_length(len);
            aim = Some(Aim::Head(Fx::ONE));
            strikers = ["head", "shoulders", "horns", limb_prefix(0)];
            delivery = Delivery::Swept;
        }
        Burrow => {
            // Nose down and digging, the forefeet scrabbling: then under.
            c.pitch = fx(-1, 12);
            c.lift = h.mul(fx(-1, 4));
            c.neck_pitch = fx(-1, 10);
            for side in 0..2 {
                swing_foot(&mut c, 0, side, fx(2, 5), fx(1, 4));
            }
            if g.stance == super::Stance::Limbless {
                c.rise = fx(-1, 30);
                c.wave = Some((Fx::ZERO, fx(1, 12)));
            }
            s = under_drive(g, q);
            delivery = Delivery::None;
        }
        Erupt => {
            // Starts under the floor and travels there, under where it was
            // aimed, through the windup -- the floor churning over it is
            // the tell -- then bursts up, front first.
            from = under_drive(g, q);
            c = from;
            let reach = erupt_reach(len);
            c.surge = reach;
            s.surge = reach;
            s.lift = h.mul(fx(1, 4));
            s.pitch = fx(1, 12);
            s.neck_pitch = fx(1, 12);
            s.jaw = Fx::ONE;
            if g.stance == super::Stance::Limbless {
                s.rise = fx(1, 12);
            }
            strikers = ["head", "jaw", "neck", "shoulders"];
            delivery = Delivery::Area(fx(1, 4));
        }
        SpikeSpray => {
            c.pitch = fx(-1, 40);
            c.tail_pitch = fx(1, 12);
            c.swell = fx(1, 10);
            s.tail_pitch = fx(-1, 30);
            s.swell = fx(1, 20);
            strikers = ["ridge", "tail", "", ""];
            delivery = Delivery::Thrown;
        }
        QuillFling => {
            // Hunched, the ridge bristling, the tail up, the head tucked:
            // a porcupine's back turned to you.
            c.pitch = fx(-1, 24);
            c.lift = h.mul(fx(-1, 6));
            c.neck_pitch = fx(-1, 12);
            c.tail_pitch = fx(1, 12);
            c.swell = fx(1, 6);
            s.pitch = fx(1, 40);
            s.swell = fx(1, 20);
            strikers = ["ridge", "", "", ""];
            delivery = Delivery::Thrown;
        }
        WebShot => {
            c.abdomen = fx(1, 8);
            c.pitch = fx(1, 40);
            s.abdomen = fx(-1, 20);
            strikers = ["abdomen", "sacs", "", ""];
            delivery = Delivery::Thrown;
        }
        Screech => {
            c.neck_pitch = fx(1, 10);
            c.jaw = fx(1, 2);
            c.swell = fx(1, 8);
            c.lift = h.mul(fx(1, 20));
            s.jaw = Fx::ONE;
            s.neck_reach = fx(1, 4);
            s.neck_pitch = fx(-1, 30);
            strikers = ["head", "", "", ""];
            delivery = Delivery::Area(fx(3, 2));
        }
        Spew => {
            // Hunched up, swelling, the head hung low and working.
            c.pitch = fx(-1, 20);
            c.surge = len.mul(fx(1, 12));
            c.swell = fx(1, 4);
            c.neck_pitch = fx(-1, 10);
            c.neck_reach = fx(-1, 4);
            s.neck_pitch = fx(-1, 16);
            s.jaw = Fx::ONE;
            s.neck_reach = fx(1, 3);
            strikers = ["head", "jaw", "", ""];
            delivery = Delivery::Thrown;
        }
        Wallow => {
            c.lift = h.mul(fx(-1, 3));
            c.pitch = fx(-1, 60);
            s.lift = h.mul(fx(-1, 2));
            s.tail_yaw = fx(1, 20);
            s.neck_yaw = fx(-1, 10);
            delivery = Delivery::None;
        }
        Shrug => {
            c.lift = h.mul(fx(-1, 5));
            c.neck_pitch = fx(-1, 10);
            c.pitch = fx(1, 40);
            s.lift = h.mul(fx(1, 6));
            s.pitch = fx(-1, 40);
            s.swell = fx(1, 20);
            strikers = ["shell", "barrel", "shoulders", ""];
            delivery = Delivery::Area(fx(1, 2));
        }
        Shed => {
            // Risen tall, the plates lifting off it.
            c.swell = fx(1, 5);
            c.lift = h.mul(fx(1, 10));
            c.neck_reach = fx(-1, 3);
            s.swell = fx(-1, 20);
            s.lift = h.mul(fx(1, 20));
            strikers = ["shell", "barrel", "ridge", ""];
            delivery = Delivery::Area(fx(1, 2));
        }
    }
    // A head with no neck is moved by the whole front of the body: a
    // neck's rise becomes the body's, its reach a rock forward or back.
    if g.head == super::HeadKind::Merged {
        for d in [&mut c, &mut s] {
            if g.stance == super::Stance::Limbless {
                d.rise = d.rise.add(d.neck_pitch.mul(fx(1, 4)));
            } else {
                // A little: tipped as far as a neck nods, a body on legs
                // lifted its forefeet off the floor (exploration 0033).
                d.pitch = d.pitch.add(d.neck_pitch.mul(fx(1, 8)));
            }
            d.surge = d.surge.add(d.neck_reach.mul(len).mul(fx(1, 6)));
        }
    }
    if let Some(a) = aim {
        aim_at(g, q, &mut s, &strikers, a);
        // A sweep is at its height before it swings -- the tail drops to
        // the shins in the windup, which is half its tell -- so its first
        // strike frames are as low as its last.
        // So is a charge's head: it covers three lengths in the strike and
        // passes somebody close in its first frames.
        if matches!(a, Aim::Tail(_)) || kind == Charge {
            aim_at(g, q, &mut c, &strikers, a);
        }
    }
    let (c, cry) = if from.lift < Fx::ZERO {
        // Begun under the floor, there is no body to see: it rumbles.
        (c, Some(Cry::Rumble))
    } else {
        gather(g, q, c, &s, &strikers)
    };
    Motion {
        kind,
        cry,
        aim,
        cocked: c,
        struck: s,
        strikers,
        delivery,
        answer: answer(kind),
        arc,
        aim_at: Fx::ZERO,
        from,
    }
}

/// What a strike is brought down (or up) to: a height over the floor on
/// the ground ahead, reached by the head, by the tail, or by the whole body
/// sinking -- so the hit is where somebody standing there is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Aim {
    Head(Fx),
    Tail(Fx),
    Body(Fx),
    /// The wings' beat, for a sweep of spread wings.
    Wing(Fx),
}

/// Bring the struck pose's head, tail or striking parts to the height an
/// [`Aim`] names: a few rounds of build and correct, as `aim::look_onto`
/// settles a look, because the body is the only model of where its parts
/// are.
fn aim_at(g: &Genome, q: &Posture, s: &mut Drive, strikers: &Strikers, aim: Aim) {
    let p = proportions(g);
    let rest = locomotion::rest(g);
    let merged = g.head == super::HeadKind::Merged;
    let limbless = g.stance == super::Stance::Limbless;
    for _ in 0..8 {
        let body = build::body_driven(g, q, &rest.plus(s));
        match aim {
            Aim::Head(y) => {
                let Some(i) = body.parts().iter().position(|part| part.name == "head") else {
                    return;
                };
                // No lower than the head's own depth below its middle --
                // the jaw under it included -- allows: aimed by its middle
                // at a man's height, a big body's rush drove its jaw three
                // metres into the floor (exploration 0034).
                let centre = body.centre(i).y;
                let below = body
                    .parts()
                    .iter()
                    .enumerate()
                    .filter(|(_, part)| part.name == "head" || part.name == "jaw")
                    .map(|(k, _)| centre.sub(body.lowest(k)))
                    .max()
                    .unwrap_or(Fx::ZERO);
                let y = if merged {
                    y
                } else {
                    y.max(below.add(fx(1, 20)))
                };
                let dy = y.sub(centre);
                // Turns per metre of height, near enough: a radian is a
                // sixth of a turn, over the arm that swings it.
                let arm = if merged {
                    p.len.mul(fx(1, 2))
                } else {
                    p.neck_len.add(p.head_half)
                };
                let turn = dy
                    .mul(fx(159, 1000))
                    .div(arm.max(fx(1, 4)))
                    .max(fx(-1, 12))
                    .min(fx(1, 12));
                if merged && limbless {
                    s.rise = s.rise.add(turn.mul(fx(1, 4))).max(fx(-1, 20)).min(fx(1, 8));
                } else if merged {
                    // A head that is the front of the body reaches its
                    // mark with its lower jaw, a quarter of the way up it,
                    // and the body sinks no further than its belly:
                    // aimed by its middle, a toad's bite sank it through
                    // the floor (exploration 0034).
                    let dy = dy.add(p.half_h.mul(fx(3, 4)));
                    s.lift = s
                        .lift
                        .add(dy)
                        .max(q.root.mul(fx(-3, 5)))
                        .max(p.half_h.mul(fx(10, 9)).sub(q.root).min(Fx::ZERO))
                        .max(sink_room(g, q).neg());
                } else {
                    let want = s.neck_pitch.add(turn);
                    s.neck_pitch = want.max(fx(-1, 4)).min(fx(1, 8));
                    // A neck bent as far as it goes: the body sinks the rest.
                    if want < fx(-1, 4) {
                        s.lift = s
                            .lift
                            .add(dy.mul(fx(1, 2)))
                            .max(q.root.mul(fx(-3, 5)))
                            .max(sink_room(g, q).neg());
                    }
                }
            }
            Aim::Tail(y) => {
                let Some(low) = lowest(&body, strikers) else {
                    return;
                };
                // The tail bends by a share a segment, added down three:
                // its tip drops about twice the share times its length.
                let turn = y.sub(low).mul(fx(8, 100)).div(p.tail_len.max(fx(1, 4)));
                s.tail_pitch = s.tail_pitch.add(turn.max(fx(-1, 24)).min(fx(1, 24)));
                s.tail_pitch = s.tail_pitch.max(fx(-1, 10)).min(fx(1, 10));
            }
            Aim::Body(y) => {
                let Some(low) = lowest(&body, strikers) else {
                    return;
                };
                // In the air it may come all the way down; on its legs, no
                // further than a deep crouch.
                let air = if s.wings == Wings::Aloft {
                    p.len.mul(fx(1, 2))
                } else {
                    Fx::ZERO
                };
                let floor = q.root.mul(fx(-3, 5)).sub(air).min(s.lift);
                // Only ever down: a strike already low enough stays put.
                let dy = y.sub(low).min(Fx::ZERO);
                let want = s.lift.add(dy);
                s.lift = want.max(floor);
                // Sunk as far as its legs allow: it pitches its front down
                // the rest of the way, about its hips.
                if want < floor {
                    let turn = dy.mul(fx(159, 1000)).div(p.len.mul(fx(1, 2)).max(fx(1, 4)));
                    s.pitch = s.pitch.add(turn.max(fx(-1, 24))).max(fx(-1, 10));
                }
            }
            Aim::Wing(y) => {
                let Some(low) = lowest(&body, strikers) else {
                    return;
                };
                let wing = p.limb.iter().fold(Fx::ZERO, |a, b| a.max(*b));
                let turn = y.sub(low).mul(fx(159, 1000)).div(wing.max(fx(1, 4)));
                s.beat = s
                    .beat
                    .add(turn.max(fx(-1, 24)).min(fx(1, 24)))
                    .max(fx(-1, 5))
                    .min(fx(1, 5));
            }
        }
    }
}

/// A cry a body gives as a move's tell, where its whole body cannot.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Cry {
    Bellow,
    Roar,
    Hiss,
    Screech,
    Chitter,
    Rattle,
    Rumble,
    Click,
}

impl Cry {
    pub const ALL: [Cry; 8] = [
        Cry::Bellow,
        Cry::Roar,
        Cry::Hiss,
        Cry::Screech,
        Cry::Chitter,
        Cry::Rattle,
        Cry::Rumble,
        Cry::Click,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Cry::Bellow => "bellow",
            Cry::Roar => "roar",
            Cry::Hiss => "hiss",
            Cry::Screech => "screech",
            Cry::Chitter => "chitter",
            Cry::Rattle => "rattle",
            Cry::Rumble => "rumble",
            Cry::Click => "click",
        }
    }

    /// The voice a body has first: what its mouth and hide make.
    fn voice(g: &Genome) -> Cry {
        match (g.minors.jaw, g.minors.hide) {
            (super::Jaw::Beak, _) => Cry::Screech,
            (super::Jaw::Mandibles, _) | (_, super::Hide::Chitin) => Cry::Chitter,
            (_, super::Hide::Plates | super::Hide::Shell) => Cry::Rattle,
            (super::Jaw::Maw, _) => Cry::Bellow,
            (_, super::Hide::Slick) => Cry::Hiss,
            _ => Cry::Roar,
        }
    }
}

/// How far the trunk must move in a windup, as a share of the body's
/// length, for the whole body to be the tell: a sixteenth.
pub const BODY_TELL: Fx = fx(1, 16);

/// The middle of the striking parts.
fn centre_of(body: &Body, strikers: &Strikers) -> Option<V3> {
    let mut sum = V3::ZERO;
    let mut n = 0;
    for (i, part) in body.parts().iter().enumerate() {
        if struck_by(part.name, strikers) {
            sum = sum.add(body.centre(i));
            n += 1;
        }
    }
    (n > 0).then(|| sum.scale(Fx::ONE.div(Fx::from_int(n))))
}

/// **The whole body winds up** (exploration 0033, the owner: *"attacks
/// should include a whole body indicator for the most part to keep them
/// readable"*). Where a move's windup moves only a limb, a head or a tail,
/// the body gathers the other way from where the strike goes -- drawn back
/// before a lunge, sunk before a blow upward, leant away before a sweep --
/// until its trunk has moved [`BODY_TELL`] of its length, within what its
/// legs allow. A move it still cannot show, it cries.
fn gather(
    g: &Genome,
    q: &Posture,
    mut c: Drive,
    s: &Drive,
    strikers: &Strikers,
) -> (Drive, Option<Cry>) {
    let p = proportions(g);
    let len = p.len;
    let want = len.mul(BODY_TELL);
    let rest = locomotion::rest(g);
    let base = build::body_driven(g, q, &rest);
    let cocked = |c: &Drive| build::body_driven(g, q, &rest.plus(c));
    // Shown: the trunk moved, and the silhouette by as much as the
    // verdict asks of a windup (an eighth of the body). A big body with a
    // small head met the first and not the second, and its bite was read
    // by nobody (exploration 0034).
    let shown = |c: &Drive| {
        let at = cocked(c);
        super::verdict::trunk_apart(&base, &at) >= want
            && super::verdict::apart(&base, &at) >= len.mul(fx(1, 8))
    };
    if shown(&c) {
        return (c, None);
    }
    // Which way the strike goes, from the cocked pose to the struck one.
    let struck = build::body_driven(g, q, &rest.plus(s));
    let dir = match (
        centre_of(&cocked(&c), strikers),
        centre_of(&struck, strikers),
    ) {
        (Some(a), Some(b)) => b.sub(a),
        // A blow all round, or none: the body sinks to it.
        _ => V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO),
    };
    let size = dir.len();
    let dir = if size > fx(1, 100) {
        dir.scale(Fx::ONE.div(size))
    } else {
        V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO)
    };
    let h = q.root.max(fx(1, 10));
    let (c0, mut k) = (c, fx(1, 2));
    for _ in 0..6 {
        let mut t = c0;
        // Away along the strike's line: back, down, aside; and the trunk
        // turned and tipped with it, which a player sees first.
        // The body rocks over its feet, which stay planted: a weight
        // shift, not a slide.
        let rock = dir.x.mul(len).mul(fx(1, 20)).mul(k);
        t.surge = t.surge.sub(rock);
        for (i, pose) in q.stations.iter().enumerate() {
            if !pose.grounded {
                continue;
            }
            let standing = p.limb[i].max(fx(1, 10));
            for side in 0..2 {
                if t.feet[i][side].lift == Fx::ZERO {
                    t.feet[i][side].forward = t.feet[i][side].forward.add(rock.div(standing));
                }
            }
        }
        t.lift = t
            .lift
            .sub(dir.y.abs().mul(h).mul(fx(1, 8)).mul(k))
            .max(h.mul(fx(-2, 5)));
        // Tipped only for a blow mostly up or down: a body tipped nose-up
        // before a charge carried its head over the target.
        if dir.y.abs() > fx(1, 2) {
            t.pitch = t.pitch.sub(dir.y.mul(fx(1, 60)).mul(k));
        }
        t.yaw = t.yaw.sub(dir.z.mul(fx(1, 30)).mul(k));
        if shown(&t) {
            return (t, None);
        }
        c = t;
        k = k.add(fx(1, 2));
    }
    (c, Some(Cry::voice(g)))
}

/// **Nothing through the floor but what strikes** (exploration 0033): a
/// move whose crouch, sink or reach takes a belly, a haunch, a thigh or a
/// jaw under the ground at any frame of its windup or strike is lifted --
/// its cocked and struck poses together, so every frame between rises with
/// them -- until only its striking parts and its feet are on it, and is
/// aimed again. A move begun under the floor is left there.
fn settle(g: &Genome, q: &Posture, spec: &MoveSpec, m: &mut Motion) {
    if m.from.lift < Fx::ZERO || g.stance == super::Stance::Limbless {
        return;
    }
    let clear = fx(1, 50);
    // What the body already has on (or in) the floor standing: a move asks
    // no part to be higher than it stands.
    let rest = body_at(g, q, m, spec, 0);
    for _ in 0..12 {
        // What goes under in the windup, which is held and stood on, and
        // what goes under in the strike, which is quick: lifted apart, so a
        // bite whose neck dips at the strike no longer lifted the windup
        // too, every foot off the floor (exploration 0034).
        let (mut short, mut short_strike) = (Fx::ZERO, Fx::ZERO);
        let last = spec.startup + spec.active;
        for k in 1..=12 {
            let frame = last * k / 12;
            let short = if frame < spec.startup {
                &mut short
            } else {
                &mut short_strike
            };
            let body = body_at(g, q, m, spec, frame);
            for (i, part) in body.parts().iter().enumerate() {
                let bone = body.bones[part.shape.bone].name;
                if part.shape.hollow
                    || struck_by(part.name, &m.strikers)
                    || bone.ends_with(".foot")
                    || bone.ends_with(".lower")
                    || part.name.starts_with("tail")
                {
                    continue;
                }
                let floor = clear.min(rest.lowest(i));
                *short = (*short).max(floor.sub(body.lowest(i)));
            }
        }
        // The windup no higher than its planted legs reach; what is left
        // comes out of its tilt, the body held level instead.
        // A windup on spread wings, or one that leaves the floor anyway, is
        // borne by more than its legs.
        let borne = m.cocked.wings != Wings::Folded || m.arc > Fx::ZERO;
        let room = if borne {
            Fx::from_int(1000)
        } else {
            leg_room(g, q, &m.cocked)
        };
        if room < Fx::ZERO {
            m.cocked.lift = m.cocked.lift.add(room);
            continue;
        }
        if short <= Fx::ZERO && short_strike <= Fx::ZERO {
            return;
        }
        // As high as the legs reach, and no higher: tried, and what the
        // built feet say is too much taken back off.
        let mut up = short.max(Fx::ZERO).min(room);
        if up > Fx::ZERO && !borne {
            let mut tried = m.cocked;
            tried.lift = tried.lift.add(up);
            let over = leg_room(g, q, &tried);
            if over < Fx::ZERO {
                up = up.add(over).max(Fx::ZERO);
            }
        }
        if short > up {
            // Its legs can lift it no higher: the windup is made smaller,
            // the same gathering a quarter less, until it clears.
            let wings = m.cocked.wings;
            m.cocked = m.cocked.scaled(fx(3, 4));
            m.cocked.wings = wings;
        }
        m.cocked.lift = m.cocked.lift.add(up);
        m.struck.lift = m.struck.lift.add(up.max(short_strike));
        // Aimed again by what it reaches with -- a head, a tail -- not by
        // sinking the body, which is what was just undone.
        if let Some(a) = m.aim.filter(|a| !matches!(a, Aim::Body(_))) {
            aim_at(g, q, &mut m.struck, &m.strikers, a);
            if matches!(a, Aim::Tail(_)) || m.kind == MoveKind::Charge {
                aim_at(g, q, &mut m.cocked, &m.strikers, a);
            }
        }
    }
}

/// How far the body can sink on its legs: the least, over the pairs it
/// stands on, of how far its hip is above two fifths of its leg -- a deep
/// crouch, and no deeper, or the thigh is on the floor (exploration 0034).
fn sink_room(g: &Genome, q: &Posture) -> Fx {
    let p = proportions(g);
    let mut room = q.root;
    if q.bellied {
        return Fx::ZERO;
    }
    for (i, pose) in q.stations.iter().enumerate() {
        let Some(pair) = g.limbs[i] else { continue };
        if !pose.grounded {
            continue;
        }
        let standing =
            super::posture::standing_length(g.stance, pair.build, pose.wing_walks, p.limb[i]);
        let hip = q
            .root
            .add(pose.attach.mul(sin_turns(q.pitch)))
            .sub(p.half_h.mul(super::posture::ATTACH_DOWN));
        room = room.min(hip.sub(standing.mul(fx(2, 5))));
    }
    room.max(Fx::ZERO)
}

/// How much higher this drive could carry the body before a foot it
/// stands on leaves the floor: the least, over the planted feet, of a
/// leg's reach less its hip's height.
fn leg_room(g: &Genome, q: &Posture, d: &Drive) -> Fx {
    let p = proportions(g);
    let mut room = Fx::from_int(1000);
    // A body on its belly stands on nothing: its legs are paddles.
    if q.bellied {
        return room;
    }
    for (i, pose) in q.stations.iter().enumerate() {
        let Some(pair) = g.limbs[i] else { continue };
        if !pose.grounded || d.feet[i].iter().all(|f| f.lift > Fx::ZERO) {
            continue;
        }
        let standing =
            super::posture::standing_length(g.stance, pair.build, pose.wing_walks, p.limb[i]);
        // A swelling body rises by its growth and its hips hang lower by
        // three quarters of it (`build`): a quarter net.
        let swell = if g.stance == super::Stance::Limbless {
            Fx::ZERO
        } else {
            p.half_h
                .mul(d.swell.max(Fx::ZERO))
                .mul(Fx::ONE.sub(super::posture::ATTACH_DOWN))
        };
        let hip = q
            .root
            .add(d.lift)
            .add(swell)
            .add(pose.attach.mul(sin_turns(q.pitch.add(d.pitch))))
            .sub(p.half_h.mul(super::posture::ATTACH_DOWN));
        room = room.min(standing.mul(fx(97, 100)).sub(hip));
    }
    // And measured: a planted foot the built body holds off the floor is
    // room already overspent.
    let body = build::body_driven(g, q, &locomotion::rest(g).plus(d));
    let still = build::body_driven(g, q, &locomotion::rest(g));
    for (i, part) in body.parts().iter().enumerate() {
        let bone = body.bones[part.shape.bone].name;
        if !bone.ends_with(".foot") {
            continue;
        }
        let planted = (0..STATIONS).any(|k| {
            (0..2).any(|side| {
                q.stations[k].grounded
                    && bone == build::LIMB_NAMES[k][side][2]
                    && d.feet[k][side].lift == Fx::ZERO
            })
        });
        if planted {
            let off = body
                .lowest(i)
                .sub(still.lowest(i).max(Fx::ZERO))
                .sub(fx(1, 20));
            if off > Fx::ZERO {
                room = room.min(off.neg());
            }
        }
    }
    room
}

/// Where a body's weight is along it and across, and the span of its feet
/// on the floor: each part weighed by its box (`None` if no foot is down).
fn weight_and_feet(body: &Body) -> Option<(Fx, Fx, Fx, Fx)> {
    let (mut mass, mut cx) = (Fx::ZERO, Fx::ZERO);
    let (mut lo, mut hi) = (Fx::from_int(1000), Fx::from_int(-1000));
    for (i, part) in body.parts().iter().enumerate() {
        if part.shape.hollow {
            continue;
        }
        let h = part.shape.max.sub(part.shape.min);
        let v = h.x.mul(h.y).mul(h.z).abs();
        cx = cx.add(body.centre(i).x.mul(v));
        mass = mass.add(v);
        let bone = body.bones[part.shape.bone].name;
        if (bone.ends_with(".foot") || bone.ends_with(".lower")) && body.lowest(i) < fx(3, 20) {
            for c in body.corners(i) {
                if c.y < fx(3, 10) {
                    lo = lo.min(c.x);
                    hi = hi.max(c.x);
                }
            }
        }
    }
    (mass > Fx::ZERO && lo <= hi).then(|| (cx.div(mass), lo, hi, mass))
}

/// **A windup is stood on** (exploration 0033): where a held windup puts
/// the weight past the feet on the floor -- arms out ahead, a head drawn
/// up -- the body rocks back over its planted feet until it is over them,
/// as an animal shifts its weight before it strikes.
fn steady(g: &Genome, q: &Posture, spec: &MoveSpec, m: &mut Motion) {
    if m.from.lift < Fx::ZERO || g.stance == super::Stance::Limbless || q.bellied {
        return;
    }
    let p = proportions(g);
    // A tenth of the torso in from the feet's edge: the weight is over
    // them through the windup's middle too, not only at its end.
    let margin = p.torso.mul(fx(1, 10));
    for _ in 0..4 {
        let body = body_at(g, q, m, spec, spec.startup - 1);
        let Some((com, lo, hi, _)) = weight_and_feet(&body) else {
            return;
        };
        let rock = if com > hi.sub(margin) {
            hi.sub(margin).sub(com)
        } else if com < lo.add(margin) {
            lo.add(margin).sub(com)
        } else {
            return;
        };
        m.cocked.surge = m.cocked.surge.add(rock);
        for (i, pose) in q.stations.iter().enumerate() {
            if !pose.grounded {
                continue;
            }
            let standing = p.limb[i].max(fx(1, 10));
            for side in 0..2 {
                if m.cocked.feet[i][side].lift == Fx::ZERO {
                    m.cocked.feet[i][side].forward =
                        m.cocked.feet[i][side].forward.sub(rock.div(standing));
                }
            }
        }
    }
}

/// The lowest point of the striking parts.
fn lowest(body: &Body, strikers: &Strikers) -> Option<Fx> {
    let mut low: Option<Fx> = None;
    for (i, part) in body.parts().iter().enumerate() {
        if struck_by(part.name, strikers) {
            let l = body.lowest(i);
            low = Some(low.map_or(l, |m: Fx| m.min(l)));
        }
    }
    low
}

/// How far a rush runs: three body lengths, and never less than twelve
/// metres, so it is a long-range threat whatever the body.
pub fn rush_length(len: Fx) -> Fx {
    len.mul(Fx::from_int(3)).max(Fx::from_int(12))
}

/// The most a move turns to face where it was aimed, during its windup.
/// Past that, the body turns on its feet first.
pub const MOST_TURN: Fx = fx(1, 8);

/// This motion turned to face `bearing` (turns, toward `+z`), as far as a
/// windup can turn: the cocked pose faces it, and the strike keeps it. The
/// aim locks when the move starts -- whatever the target does after, the
/// hit goes where the move was pointed (bestiary §1.5).
pub fn toward(m: &Motion, bearing: Fx) -> Motion {
    aimed(m, bearing, Fx::ZERO)
}

/// [`toward`], and a thrown move aimed to come down `distance` metres
/// ahead (nought: its full reach).
pub fn aimed(m: &Motion, bearing: Fx, distance: Fx) -> Motion {
    let b = bearing.max(MOST_TURN.neg()).min(MOST_TURN);
    let mut out = *m;
    out.aim_at = distance;
    // An eruption comes up where it was aimed, as far as it reaches.
    if m.kind == MoveKind::Erupt && distance > Fx::ZERO {
        let reach = m.struck.surge;
        out.cocked.surge = distance.min(reach);
        out.struck.surge = distance.min(reach);
    }
    out.cocked.yaw = out.cocked.yaw.add(b);
    out.struck.yaw = out.struck.yaw.add(b);
    out
}

/// A move whose area is a shockwave along the floor, answered by being off
/// it.
pub const fn shockwave(kind: MoveKind) -> bool {
    matches!(
        kind,
        MoveKind::Stomp
            | MoveKind::RearSlam
            | MoveKind::BellyFlop
            | MoveKind::SacSlam
            | MoveKind::BreachDive
    )
}

/// Where in a move a frame is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Beat {
    Windup,
    Strike,
    /// Held where the strike left it: the punish window.
    Stuck,
    Recover,
    Done,
}

/// The whole length of a move, in frames.
pub fn frames(spec: &MoveSpec) -> i32 {
    spec.startup + spec.active + spec.recovery
}

/// Which beat a frame is in.
pub fn beat(spec: &MoveSpec, frame: i32) -> Beat {
    let stuck = spec.recovery / 3;
    if frame < spec.startup {
        Beat::Windup
    } else if frame < spec.startup + spec.active {
        Beat::Strike
    } else if frame < spec.startup + spec.active + stuck {
        Beat::Stuck
    } else if frame < frames(spec) {
        Beat::Recover
    } else {
        Beat::Done
    }
}

/// Fast at first, settling: most of the change in the first part.
fn ease_out(t: Fx) -> Fx {
    let u = Fx::ONE.sub(super::unit(t));
    Fx::ONE.sub(u.mul(u).mul(u))
}

/// Slow at first, accelerating into the end.
fn ease_in(t: Fx) -> Fx {
    let t = super::unit(t);
    t.mul(t)
}

fn smooth(t: Fx) -> Fx {
    let t = super::unit(t);
    t.mul(t).mul(Fx::from_int(3).sub(t.mul(Fx::from_int(2))))
}

/// The move's drive at a frame, over the body's rest.
pub fn drive_at(g: &Genome, m: &Motion, spec: &MoveSpec, frame: i32) -> Drive {
    let rest = locomotion::rest(g);
    let over = match beat(spec, frame) {
        Beat::Windup => {
            let t = Fx::ratio(frame, spec.startup.max(1));
            Drive::lerp(&m.from, &m.cocked, ease_out(t))
        }
        Beat::Strike => {
            let t = Fx::ratio(frame - spec.startup + 1, spec.active.max(1));
            let mut d = Drive::lerp(&m.cocked, &m.struck, ease_in(t));
            d.lift = d.lift.add(m.arc.mul(sin_turns(t.mul(fx(1, 2)))));
            d
        }
        Beat::Stuck => m.struck,
        // A burrow's recovery is spent under the floor: it does not come
        // back up the way it went down (the actor is under by then).
        Beat::Recover | Beat::Done if m.kind == MoveKind::Burrow => m.struck,
        Beat::Recover | Beat::Done => {
            let stuck = spec.recovery / 3;
            let into = frame - spec.startup - spec.active - stuck;
            let t = Fx::ratio(into, (spec.recovery - stuck).max(1));
            // The body stays where the strike carried it, facing the way
            // it ended.
            let mut back = Drive::lerp(&m.struck, &Drive::STILL, smooth(t));
            back.surge = m.struck.surge;
            back.yaw = m.struck.yaw;
            back
        }
    };
    // Feet a move drives replace the rest's; the rest's stance otherwise.
    let mut d = rest;
    for i in 0..STATIONS {
        for side in 0..2 {
            let moved = m.cocked.feet[i][side] != super::drive::Foot::default()
                || m.struck.feet[i][side] != super::drive::Foot::default();
            if moved {
                d.feet[i][side] = super::drive::Foot::default();
            }
        }
    }
    d.plus(&over)
}

/// The body at a frame of a move.
pub fn body_at(g: &Genome, q: &Posture, m: &Motion, spec: &MoveSpec, frame: i32) -> Body {
    build::body_driven(g, q, &drive_at(g, m, spec, frame))
}

/// One sphere of a hit.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sphere {
    pub at: V3,
    pub radius: Fx,
    /// Nought for a ball round `at`; otherwise an upright disc this tall
    /// standing on `at` -- a shockwave along the floor, which a jump
    /// clears and a ball of the same width would not let it.
    pub height: Fx,
}

impl Sphere {
    pub fn bottom(&self) -> Fx {
        if self.height > Fx::ZERO {
            self.at.y
        } else {
            self.at.y.sub(self.radius)
        }
    }

    pub fn top(&self) -> Fx {
        if self.height > Fx::ZERO {
            self.at.y.add(self.height)
        } else {
            self.at.y.add(self.radius)
        }
    }
}

/// How tall a shockwave along the floor is.
pub const SHOCK_HEIGHT: Fx = fx(1, 2);

/// The most spheres one frame's hit is made of.
pub const MAX_SPHERES: usize = 48;

/// What one frame of a move hits.
#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub spheres: [Sphere; MAX_SPHERES],
    pub count: usize,
}

impl Hit {
    pub const EMPTY: Hit = Hit {
        spheres: [Sphere {
            at: V3::ZERO,
            radius: Fx::ZERO,
            height: Fx::ZERO,
        }; MAX_SPHERES],
        count: 0,
    };

    fn push(&mut self, at: V3, radius: Fx) {
        if self.count < MAX_SPHERES {
            self.spheres[self.count] = Sphere {
                at,
                radius,
                height: Fx::ZERO,
            };
            self.count += 1;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Sphere> {
        self.spheres[..self.count].iter()
    }
}

fn struck_by(name: &str, strikers: &Strikers) -> bool {
    strikers
        .iter()
        .any(|s| !s.is_empty() && name.starts_with(s))
}

/// The striking parts of a body, each as spheres strung down its box.
fn striking_spheres(body: &Body, strikers: &Strikers, out: &mut Hit) {
    for (i, part) in body.parts().iter().enumerate() {
        if !struck_by(part.name, strikers) {
            continue;
        }
        let half = part.shape.max.sub(part.shape.min).scale(fx(1, 2));
        let thick = half.y.max(half.z).max(fx(1, 20));
        let c = body.centre(i);
        let u = body.axes[i];
        // As many spheres down its length as it is long for its thickness.
        let n = half.x.div(thick).to_int().clamp(0, 2);
        for k in -n..=n {
            let along = if n == 0 {
                Fx::ZERO
            } else {
                half.x.sub(thick).max(Fx::ZERO).mul(Fx::ratio(k, n))
            };
            out.push(c.add(u.scale(along)), thick);
        }
    }
}

/// What a move hits at a frame: nothing outside its strike frames, and on
/// them the striking parts where they are -- the parts themselves for a
/// swept hit, an area round where they come down, or a line of shot out
/// to the move's reach.
pub fn hit_at(g: &Genome, q: &Posture, m: &Motion, spec: &MoveSpec, frame: i32) -> Hit {
    if beat(spec, frame) != Beat::Strike || m.delivery == Delivery::None {
        return Hit::EMPTY;
    }
    let body = body_at(g, q, m, spec, frame);
    hit_on(g, &body, m, spec, frame)
}

/// What a move hits at a frame, on a body already built for that frame.
pub fn hit_on(g: &Genome, body: &Body, m: &Motion, spec: &MoveSpec, frame: i32) -> Hit {
    let mut hit = Hit::EMPTY;
    if beat(spec, frame) != Beat::Strike || m.delivery == Delivery::None {
        return hit;
    }
    let len = proportions(g).len;
    match m.delivery {
        Delivery::Swept => striking_spheres(body, &m.strikers, &mut hit),
        // An eruption: a column the width of its mark, on the mark, every
        // strike frame -- what comes up comes up there.
        Delivery::Area(_) if m.kind == MoveKind::Erupt => {
            let d = drive_at(g, m, spec, frame);
            let at = super::actor::spin(V3::new(d.surge, Fx::ZERO, Fx::ZERO), d.yaw);
            hit.push(at, mark_radius(len));
            hit.spheres[hit.count - 1].height = ERUPT_HEIGHT;
        }
        Delivery::Area(share) => {
            // On the last strike frame, where the parts came down.
            if frame == spec.startup + spec.active - 1 {
                let mut parts = Hit::EMPTY;
                striking_spheres(body, &m.strikers, &mut parts);
                let (mut sum, mut low) = (V3::ZERO, Fx::from_int(1000));
                for s in parts.iter() {
                    sum = sum.add(s.at);
                    low = low.min(s.at.y);
                }
                let n = parts.count.max(1) as i32;
                let c = sum.scale(Fx::ratio(1, n));
                let floor = if matches!(m.kind, MoveKind::Shake | MoveKind::Shrug) {
                    c.y
                } else {
                    Fx::ZERO.max(low.min(c.y)).min(len.mul(fx(1, 4)))
                };
                if shockwave(m.kind) {
                    // A ring along the floor from where it came down.
                    hit.push(V3::new(c.x, Fx::ZERO, c.z), len.mul(share));
                    hit.spheres[hit.count - 1].height = SHOCK_HEIGHT;
                } else {
                    hit.push(V3::new(c.x, floor, c.z), len.mul(share));
                }
            } else {
                striking_spheres(body, &m.strikers, &mut hit);
            }
        }
        Delivery::Thrown => {
            // A line from the striking parts out to the reach, spread
            // through the strike frames: a shot's flight is its strike.
            let mut parts = Hit::EMPTY;
            striking_spheres(body, &m.strikers, &mut parts);
            let from = parts.iter().map(|s| s.at).fold(V3::ZERO, |a, b| a.add(b));
            let from = from.scale(Fx::ratio(1, parts.count.max(1) as i32));
            // Along the way the move faces, from this frame's start of
            // its flight to its end, strung close enough that nothing
            // passes between: a shot is tested along its path, not where
            // it happens to be each frame.
            let dir = super::actor::spin(
                V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
                drive_at(g, m, spec, frame).yaw,
            );
            let r = len.mul(fx(1, 12)).max(fx(3, 10));
            let t0 = Fx::ratio(frame - spec.startup, spec.active.max(1));
            let t1 = Fx::ratio(frame - spec.startup + 1, spec.active.max(1));
            let (d0, d1) = (spec.reach.mul(t0), spec.reach.mul(t1));
            let start = V3::new(from.x, Fx::ZERO, from.z);
            // Aimed at chest height on the ground ahead: about a metre.
            let to_y = Fx::ONE;
            let aim = if m.aim_at > Fx::ZERO {
                m.aim_at.min(spec.reach)
            } else {
                spec.reach
            };
            let mut dist = d0;
            loop {
                // Down to chest height where it was aimed, and on down to
                // the floor past it.
                let t = dist.div(aim.max(fx(1, 10)));
                let y = from.y.add(to_y.sub(from.y).mul(t)).max(Fx::ZERO);
                hit.push(
                    start
                        .add(dir.scale(dist))
                        .add(V3::new(Fx::ZERO, y, Fx::ZERO)),
                    r,
                );
                if dist >= d1 || hit.count >= MAX_SPHERES {
                    break;
                }
                dist = dist.add(r).min(d1);
            }
        }
        Delivery::None => {}
    }
    hit
}

/// Every move this body has, with its motion -- and, when nothing in it
/// threatens a range a fighter can stand at (bestiary §1.3), the move that
/// covers it: a rush for mid and long, which runs through both, and a
/// shoulder or a stomp up close.
pub fn kit(g: &Genome, q: &Posture) -> ([Option<(MoveSpec, Motion)>; build::MAX_MOVES], usize) {
    let mut set = build::moves(g);
    let len = proportions(g).len;
    let rest = build::body_in(g, q);
    let (close, mid, long) = super::verdict::coverage(g, q, &rest, &set);
    if !(mid && long) {
        set.push(build::spec_for(g, MoveKind::Rush, rush_length(len)));
    }
    if !close {
        let kind = if g.sliders.size < fx(1, 2) || g.stance == super::Stance::Limbless {
            MoveKind::Shoulder
        } else {
            MoveKind::Stomp
        };
        set.push(build::spec_for(g, kind, len.mul(fx(1, 3))));
    }
    let mut out: [Option<(MoveSpec, Motion)>; build::MAX_MOVES] = [None; build::MAX_MOVES];
    let mut n = 0;
    for spec in set.iter() {
        let mut m = motion(g, q, spec.kind);
        steady(g, q, spec, &mut m);
        settle(g, q, spec, &mut m);
        // Measured as the verdict measures it, after everything that moved
        // it: a move that must be read on sight, and that the whole body
        // still does not show, cries. One answered by where you stand needs
        // no cry (exploration 0034).
        if m.cry.is_none()
            && spec.damage > 0
            && super::verdict::read(g, q, &rest, spec, &m).unanswerable(len)
        {
            m.cry = Some(Cry::voice(g));
        }
        out[n] = Some((*spec, m));
        n += 1;
    }
    // Two windups the eye cannot tell apart: the later one cries, the edge
    // case a body's gathering leaves (exploration 0033).
    let len_ = proportions(g).len;
    for a in 0..n {
        for b in 0..a {
            let (Some((sa, ma)), Some((sb, mb))) = (out[a], out[b]) else {
                continue;
            };
            if sa.damage == 0 || sb.damage == 0 || ma.cry.is_some() || mb.cry.is_some() {
                continue;
            }
            let ca = body_at(g, q, &ma, &sa, sa.startup - 1);
            let cb = body_at(g, q, &mb, &sb, sb.startup - 1);
            if super::verdict::apart(&ca, &cb) < len_.mul(fx(1, 10)) {
                if let Some((_, m)) = out[a].as_mut() {
                    m.cry = Some(Cry::voice(g));
                }
            }
        }
    }
    // Each cried move a cry of its own: a body's second is the next voice
    // after its first, so a player can tell which is coming by ear.
    let mut used = [false; 8];
    for (_, m) in out[..n].iter_mut().flatten() {
        let Some(cry) = m.cry else { continue };
        let start = Cry::ALL.iter().position(|c| *c == cry).unwrap_or(0);
        let k = (0..8)
            .map(|d| (start + d) % 8)
            .find(|&k| !used[k])
            .unwrap_or(start);
        used[k] = true;
        m.cry = Some(Cry::ALL[k]);
    }
    (out, n)
}
