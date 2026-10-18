//! The bestiary contract (`docs/design/bestiary.md` §1), measured on a
//! built body: whether its moves can be read, learned and answered.
//!
//! Nothing here is declared by a move. A move's **lane** (front, rear,
//! side, above, all round, aboard) and its **range** (close, mid, long)
//! are read off where its hit actually goes, frame by frame; its **tell**
//! is how far its silhouette has moved, and how early. So a crossbred body
//! is judged by the same measures as one somebody drew, and a genome that
//! makes an unreadable move fails here before anybody fights it.
//!
//! What it measures, by the contract's numbers:
//!
//! - **§1.4, the windup is the move.** Every damaging move's tell is at
//!   least [`HUMAN_REACTION_FRAMES`] long, or its answer is where you
//!   stand. The silhouette changes **a lot** -- some part moves an eighth
//!   of the body's length -- and **early**: half of that change shows by a
//!   third of the windup. A move that is none of these is *unanswerable*,
//!   and the count of those must be zero.
//! - **§1.2, every move a different answer.** Two moves are the same
//!   question when they are answered the same way, in the same lane, at
//!   the same range. The kit's distinct questions are counted.
//! - **Readable apart.** Two moves whose cocked silhouettes are the same
//!   cannot be told apart, however different their answers: the pair is
//!   counted when no part differs by a tenth of the body.
//! - **§1.3, a threat at every range.** Close, mid and long, and aboard
//!   for a body that can be climbed.
//! - **§1.9, nothing safe.** Every damaging move holds still after it
//!   lands for long enough to be punished.
//!
//! [`HUMAN_REACTION_FRAMES`]: crate::tuning::HUMAN_REACTION_FRAMES

use super::build::{self, Body, MoveKind, MoveSpec, proportions};
use super::plan::assess;
use super::posture::Posture;
use super::strike::{self, Answer, Delivery, Motion};
use super::{Genome, fx};
use crate::fixed::Fx;
use crate::tuning::HUMAN_REACTION_FRAMES;

/// How tall somebody standing is, near enough: a hit whose lowest point
/// is above this passes over their head.
pub const STANDING: Fx = fx(9, 5);

/// Where a move's hit goes, against the body.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Lane {
    Front,
    Side,
    Rear,
    Above,
    Around,
    Aboard,
}

impl Lane {
    pub const fn name(self) -> &'static str {
        match self {
            Lane::Front => "front",
            Lane::Side => "side",
            Lane::Rear => "rear",
            Lane::Above => "above",
            Lane::Around => "around",
            Lane::Aboard => "aboard",
        }
    }
}

/// How far from the body a move reaches, in the distances classes fight
/// from: close is a sword's reach, mid a few strides, long a bow's.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Range {
    Close,
    Mid,
    Long,
}

impl Range {
    pub const fn name(self) -> &'static str {
        match self {
            Range::Close => "close",
            Range::Mid => "mid",
            Range::Long => "long",
        }
    }

    fn of(beyond: Fx) -> Range {
        if beyond <= Fx::from_int(3) {
            Range::Close
        } else if beyond <= Fx::from_int(10) {
            Range::Mid
        } else {
            Range::Long
        }
    }
}

/// One move, measured.
#[derive(Clone, Copy, Debug)]
pub struct Reading {
    pub kind: MoveKind,
    pub answer: Answer,
    pub damaging: bool,
    pub tell: i32,
    /// The most any part has moved by the end of the windup, metres.
    pub change: Fx,
    /// How much of that change shows a third of the way in, nought to one.
    pub early: Fx,
    pub lane: Lane,
    pub range: Range,
    /// How far outside the body at rest the hit reaches, metres, and how
    /// near to it it comes: it threatens every range between.
    pub beyond: Fx,
    pub near: Fx,
    /// How far the striking parts travel through the strike, metres.
    pub swing: Fx,
    /// Frames held still after the strike.
    pub stuck: i32,
    pub recovery: i32,
}

impl Reading {
    /// Fast enough to be read on sight.
    pub fn reactable(&self) -> bool {
        self.tell >= HUMAN_REACTION_FRAMES as i32
    }

    /// The silhouette moves a lot, and early.
    pub fn legible(&self, len: Fx) -> bool {
        self.change >= len.mul(fx(1, 8)) && self.early >= fx(1, 2)
    }

    /// Neither read on sight nor answered by where you stand.
    pub fn unanswerable(&self, len: Fx) -> bool {
        self.damaging && !self.answer.positional() && (!self.reactable() || !self.legible(len))
    }

    /// Its question: the answer, the lane, the range.
    pub fn question(&self) -> (Answer, Lane, Range) {
        (self.answer, self.lane, self.range)
    }
}

/// A whole kit, measured.
#[derive(Clone, Copy, Debug)]
pub struct Verdict {
    pub readings: [Option<Reading>; build::MAX_MOVES],
    pub count: usize,
    pub damaging: usize,
    pub unanswerable: usize,
    /// Distinct questions among the damaging moves.
    pub questions: usize,
    /// Distinct answers among the damaging moves.
    pub answers: usize,
    /// Pairs of damaging moves whose cocked silhouettes cannot be told
    /// apart, and the first few of them.
    pub lookalikes: usize,
    pub lookalike: [Option<(MoveKind, MoveKind)>; 6],
    pub close: bool,
    pub mid: bool,
    pub long: bool,
    /// For a body that can be climbed: something that hits aboard.
    pub aboard: Option<bool>,
    /// Damaging moves that do not hold still long enough to be punished.
    pub safe: usize,
}

impl Verdict {
    pub fn iter(&self) -> impl Iterator<Item = &Reading> {
        self.readings[..self.count].iter().flatten()
    }
}

/// How far apart two bodies' silhouettes are: the most any part moved,
/// matching parts by name and order. A part only one of them has (a wing
/// spread, which is drawn in more pieces) counts as a quarter of the body.
pub fn apart(a: &Body, b: &Body) -> Fx {
    let mut most = Fx::ZERO;
    let quarter = a.length.mul(fx(1, 4));
    for (i, pa) in a.parts().iter().enumerate() {
        let nth = a.parts()[..i].iter().filter(|p| p.name == pa.name).count();
        let found = b
            .parts()
            .iter()
            .enumerate()
            .filter(|(_, p)| p.name == pa.name)
            .nth(nth);
        let d = match found {
            Some((j, _)) => a.centre(i).sub(b.centre(j)).len(),
            None => quarter,
        };
        most = most.max(d);
    }
    if a.part_count != b.part_count {
        most = most.max(quarter);
    }
    most
}

/// The body's extent at rest: nose, rump, half its width, its top.
struct Extent {
    nose: Fx,
    rump: Fx,
    half: Fx,
    top: Fx,
}

/// Measured off the trunk and the legs it stands on, not the neck, head,
/// tail and wings: those are what reach, and somebody standing beside a
/// tail is standing beside the body, not inside it. A limbless body is all
/// trunk.
fn extent(body: &Body, limbless: bool) -> Extent {
    const REACHING: [&str; 7] = ["tail", "head", "jaw", "horns", "crest", "neck", "wing_"];
    let mut e = Extent {
        nose: Fx::from_int(-1000),
        rump: Fx::from_int(1000),
        half: Fx::ZERO,
        top: Fx::ZERO,
    };
    for i in 0..body.part_count {
        let name = body.parts[i].name;
        if !limbless && REACHING.iter().any(|r| name.starts_with(r)) {
            continue;
        }
        for c in body.corners(i) {
            e.nose = e.nose.max(c.x);
            e.rump = e.rump.min(c.x);
            e.half = e.half.max(c.z.abs());
            e.top = e.top.max(c.y);
        }
    }
    e
}

/// One move, measured on this body.
pub fn read(g: &Genome, q: &Posture, rest: &Body, spec: &MoveSpec, m: &Motion) -> Reading {
    let len = proportions(g).len;
    let at = |f: i32| strike::body_at(g, q, m, spec, f);
    let cocked = at(spec.startup - 1);
    let change = apart(rest, &cocked);
    let third = at((spec.startup / 3).max(0));
    let early = if change > fx(1, 100) {
        super::unit(apart(rest, &third).div(change))
    } else {
        Fx::ZERO
    };
    let damaging = spec.damage > 0 && m.delivery != Delivery::None;
    let e = extent(rest, g.stance == super::Stance::Limbless);
    // Where the hit goes: the sphere reaching furthest outside the body
    // that comes low enough to touch somebody standing. A hit that never
    // does is above, however far it reaches.
    let (mut beyond, mut lane) = (Fx::ZERO, Lane::Front);
    let mut far_side = (Fx::ZERO, Lane::Front);
    let mut far_high = Fx::ZERO;
    let mut near = Fx::from_int(1000);
    for f in spec.startup..spec.startup + spec.active {
        let hit = strike::hit_at(g, q, m, spec, f);
        for s in hit.iter() {
            let forward = s.at.x.sub(e.nose);
            let behind = e.rump.sub(s.at.x);
            let side = s.at.z.abs().sub(e.half);
            let out = forward.max(behind).max(side).add(s.radius);
            let inside = forward < Fx::ZERO && behind < Fx::ZERO;
            let l = if s.radius >= len.mul(fx(1, 2)) {
                Lane::Around
            } else if inside && s.at.y.sub(s.radius) > e.top.mul(fx(9, 10)) {
                Lane::Aboard
            } else if inside || side >= forward.max(behind) {
                Lane::Side
            } else if forward >= behind {
                Lane::Front
            } else {
                Lane::Rear
            };
            let low = s.at.y.sub(s.radius) < STANDING;
            if low {
                near = near.min(forward.max(behind).max(side).sub(s.radius).max(Fx::ZERO));
            }
            if !low {
                far_high = far_high.max(out);
            } else if out > far_side.0 {
                far_side = (out, l);
            }
        }
    }
    if far_side.0 > Fx::ZERO {
        beyond = far_side.0;
        lane = far_side.1;
    } else if far_high > Fx::ZERO {
        beyond = far_high;
        lane = Lane::Above;
    }
    // A hit aboard or all round can only be measured as close; a throw is
    // as far as it flies.
    if m.delivery == Delivery::Thrown {
        beyond = beyond.max(spec.reach);
        near = Fx::ZERO;
    }
    let near = near.min(beyond);
    let range = Range::of(beyond);
    // How far the striking parts travel through the strike.
    let first = strike::body_at(g, q, m, spec, spec.startup);
    let last = strike::body_at(g, q, m, spec, spec.startup + spec.active - 1);
    let swing = apart(&cocked, &last).max(apart(&first, &last));
    Reading {
        kind: m.kind,
        answer: m.answer,
        damaging,
        tell: spec.startup,
        change,
        early,
        lane,
        range,
        beyond,
        near,
        swing,
        stuck: spec.recovery / 3,
        recovery: spec.recovery,
    }
}

/// The whole kit, measured.
pub fn judge(g: &Genome, q: &Posture) -> Verdict {
    let rest = build::body_in(g, q);
    let len = proportions(g).len;
    let (kit, n) = strike::kit(g, q);
    let mut v = Verdict {
        readings: [None; build::MAX_MOVES],
        count: n,
        damaging: 0,
        unanswerable: 0,
        questions: 0,
        answers: 0,
        lookalikes: 0,
        lookalike: [None; 6],
        close: false,
        mid: false,
        long: false,
        aboard: if assess(g).caps.mountable {
            Some(false)
        } else {
            None
        },
        safe: 0,
    };
    let mut cocked: [Option<Body>; build::MAX_MOVES] = [None; build::MAX_MOVES];
    for (k, entry) in kit[..n].iter().enumerate() {
        let Some((spec, m)) = entry else { continue };
        let r = read(g, q, &rest, spec, m);
        v.readings[k] = Some(r);
        if !r.damaging {
            continue;
        }
        cocked[k] = Some(strike::body_at(g, q, m, spec, spec.startup - 1));
        v.damaging += 1;
        if r.unanswerable(len) {
            v.unanswerable += 1;
        }
        match r.lane {
            Lane::Aboard => {
                if let Some(a) = v.aboard.as_mut() {
                    *a = true;
                }
            }
            // Every range between how near it comes and how far it goes.
            _ => {
                let three = Fx::from_int(3);
                let ten = Fx::from_int(10);
                v.close |= r.near <= three;
                v.mid |= r.near <= ten && r.beyond > three;
                v.long |= r.beyond > ten;
            }
        }
        if r.stuck < 8 || r.recovery < HUMAN_REACTION_FRAMES as i32 {
            v.safe += 1;
        }
    }
    // Distinct questions and answers, and lookalike pairs.
    let damaging: [Option<Reading>; build::MAX_MOVES] = {
        let mut out = [None; build::MAX_MOVES];
        for (k, r) in v.readings[..n].iter().enumerate() {
            out[k] = r.filter(|r| r.damaging);
        }
        out
    };
    for a in 0..n {
        let Some(ra) = damaging[a] else { continue };
        let first_q = (0..a).all(|b| damaging[b].is_none_or(|rb| rb.question() != ra.question()));
        let first_a = (0..a).all(|b| damaging[b].is_none_or(|rb| rb.answer != ra.answer));
        if first_q {
            v.questions += 1;
        }
        if first_a {
            v.answers += 1;
        }
        for b in 0..a {
            if let (Some(ca), Some(cb)) = (&cocked[a], &cocked[b])
                && apart(ca, cb) < len.mul(fx(1, 10))
            {
                if v.lookalikes < v.lookalike.len() {
                    v.lookalike[v.lookalikes] = damaging[b].map(|rb| (rb.kind, ra.kind));
                }
                v.lookalikes += 1;
            }
        }
    }
    v
}
