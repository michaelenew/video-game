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
    pub cocked: Drive,
    pub struck: Drive,
    pub strikers: Strikers,
    pub delivery: Delivery,
    pub answer: Answer,
    /// How far the body rises at the middle of the strike, metres: the
    /// top of a leap's arc.
    pub arc: Fx,
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
        _ => 0,
    }
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
            strikers = ["horns", "head", "", ""];
            delivery = Delivery::Swept;
        }
        Charge => {
            c.neck_pitch = fx(-1, 10);
            c.tail_pitch = fx(1, 12);
            c.pitch = fx(-1, 80);
            for side in 0..2 {
                swing_foot(&mut c, STATIONS - 1, side, fx(-1, 5), Fx::ZERO);
            }
            // A forefoot pawing the ground, high.
            swing_foot(&mut c, 0, 0, fx(-1, 6), fx(1, 3));
            s.neck_pitch = fx(-1, 10);
            s.surge = len.mul(Fx::from_int(3));
            aim = Some(Aim::Head(Fx::ONE));
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
            c.pitch = fx(1, 30);
            c.lift = h.mul(fx(1, 10));
            swing_foot(&mut c, i, 0, fx(1, 5), fx(1, 2));
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
            swing_foot(&mut c, i, 1, fx(1, 3), fx(1, 3));
            swing_foot(&mut s, i, 0, fx(3, 5), Fx::ZERO);
            swing_foot(&mut s, i, 1, fx(1, 2), fx(1, 10));
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
            c.swell = fx(2, 5);
            c.lift = h.mul(fx(1, 5));
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
                d.pitch = d.pitch.add(d.neck_pitch.mul(fx(1, 3)));
            }
            d.surge = d.surge.add(d.neck_reach.mul(len).mul(fx(1, 6)));
        }
    }
    if let Some(a) = aim {
        aim_at(g, q, &mut s, &strikers, a);
        // A sweep is at its height before it swings -- the tail drops to
        // the shins in the windup, which is half its tell -- so its first
        // strike frames are as low as its last.
        if matches!(a, Aim::Tail(_)) {
            aim_at(g, q, &mut c, &strikers, a);
        }
    }
    Motion {
        kind,
        cocked: c,
        struck: s,
        strikers,
        delivery,
        answer: answer(kind),
        arc,
        aim_at: Fx::ZERO,
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
                let dy = y.sub(body.centre(i).y);
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
                    s.lift = s.lift.add(dy).max(q.root.mul(fx(-3, 5)));
                } else {
                    let want = s.neck_pitch.add(turn);
                    s.neck_pitch = want.max(fx(-1, 4)).min(fx(1, 8));
                    // A neck bent as far as it goes: the body sinks the rest.
                    if want < fx(-1, 4) {
                        s.lift = s.lift.add(dy.mul(fx(1, 2))).max(q.root.mul(fx(-3, 5)));
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
            Drive::lerp(&Drive::STILL, &m.cocked, ease_out(t))
        }
        Beat::Strike => {
            let t = Fx::ratio(frame - spec.startup + 1, spec.active.max(1));
            let mut d = Drive::lerp(&m.cocked, &m.struck, ease_in(t));
            d.lift = d.lift.add(m.arc.mul(sin_turns(t.mul(fx(1, 2)))));
            d
        }
        Beat::Stuck => m.struck,
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
    let mut out = [None; build::MAX_MOVES];
    let mut n = 0;
    for spec in set.iter() {
        out[n] = Some((*spec, motion(g, q, spec.kind)));
        n += 1;
    }
    (out, n)
}
