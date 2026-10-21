//! **Getting up and getting down** (2026-10-21, from play): how a creature
//! with legs goes up a ledge or down off one -- as a move, which takes frames
//! and in which it throws nothing.
//!
//! The owner: "every monster needs a 'get up' and 'get down' animation and
//! move. I'd like the ridgebacks to still be able to pursue the player when
//! they are on the box ... the ridgebacks can get up things lower than their
//! shoulders ... The benefit to the player would be that the ridgeback cannot
//! go through other moves while getting up, so they get a window of
//! opportunity ... This get up / get down animation will need to be added for
//! morph creatures too, so it needs to be well thought through and can't be a
//! hand-craft just-so animation with bespoke code."
//!
//! So nothing here is authored per creature. Three things, each read off the
//! skeleton a species already has:
//!
//! - **[`Build`]**: how tall it is at the shoulder, where its front and hind
//!   legs hang, how far its body runs fore and aft -- and so what it can get
//!   up ([`reach`], a share of its shoulder height that is its knob) and how
//!   far it travels doing it ([`advance`]).
//! - **[`profile`]**: the motion itself, as five numbers of its progress and
//!   of the rise -- how far its middle has risen and gone forward, how high
//!   its front end is, and where its front and hind feet are. The same for
//!   every body: the front end goes first, the hind end after it, the feet
//!   lift clear of the lip in turn, and it settles into a crouch before it
//!   pushes. Down is the same sentence the other way: the forefeet reach down
//!   first.
//! - **[`layer`]**: that profile as a pose over a creature's legs -- the root
//!   pitched so the front end is where the profile says, each leg folded so
//!   its foot is where the profile says (the same crouch arithmetic a lame leg
//!   bends by, `math::crouch_turns`), the neck pitched back so the head stays
//!   on what it is after. Anything with `Leg`s gets it.
//!
//! A bred body is not a [`crate::species::Species`], but it has a front end,
//! a hind end and feet: [`crate::breed::climb`] reads the same [`profile`]
//! into its drive. One motion, two skeletons.
//!
//! **The window.** A creature getting up is in [`crate::monster::Doing::Climb`]:
//! not free, so it chooses nothing, and open, so the fight report counts it.
//! Its length is its knobs -- frames, and frames a metre -- so a tall ledge
//! is a long window and a step is a short one.

use crate::beast::{NO_PARENT, Pose};
use crate::fixed::{Fx, sin_turns};
use crate::math::{self, V3};
use crate::species::Species;

// ---------------------------------------------------------------------------
// The knobs
// ---------------------------------------------------------------------------

/// One field of a species' **climb row**: every species with a body has one,
/// appended after everything else in its store. "<Species> · climb".
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClimbField {
    /// The highest ledge it gets up, as a share of its shoulder height. Zero:
    /// it never climbs, and a ledge over its step is a wall.
    Reach,
    /// The deepest drop it gets down, as a share of its shoulder height. A
    /// drop deeper than this is an edge it will not go over.
    Drop,
    /// Frames to get up, before the height.
    Frames,
    /// And this many more for every metre of ledge.
    PerMetre,
    /// Getting down takes this share of what getting up the same height does.
    Down,
    /// How far its feet clear the lip, as a share of its shoulder height.
    Clear,
    /// How deep it settles before it pushes, as a share of its shoulder.
    Dip,
}

pub const CLIMB_FIELDS: usize = 7;

impl ClimbField {
    pub const ALL: &'static [ClimbField] = &[
        ClimbField::Reach,
        ClimbField::Drop,
        ClimbField::Frames,
        ClimbField::PerMetre,
        ClimbField::Down,
        ClimbField::Clear,
        ClimbField::Dip,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            ClimbField::Reach => "Gets up onto (x shoulder)",
            ClimbField::Drop => "Gets down off (x shoulder)",
            ClimbField::Frames => "Getting up, frames",
            ClimbField::PerMetre => "Getting up, frames a metre",
            ClimbField::Down => "Getting down (x getting up)",
            ClimbField::Clear => "Feet clear the lip by (x shoulder)",
            ClimbField::Dip => "Settles before it pushes (x shoulder)",
        }
    }

    pub const fn unit(self) -> crate::oven::Unit {
        match self {
            ClimbField::Frames | ClimbField::PerMetre => crate::oven::Unit::Frames,
            _ => crate::oven::Unit::Fixed,
        }
    }

    pub const fn range(self) -> (i32, i32) {
        const fn fx(n: i32, d: i32) -> i32 {
            Fx::ratio(n, d).raw()
        }
        match self {
            ClimbField::Reach | ClimbField::Drop => (0, fx(3, 1)),
            ClimbField::Frames => (1, 240),
            ClimbField::PerMetre => (0, 120),
            ClimbField::Down => (fx(1, 10), fx(2, 1)),
            ClimbField::Clear | ClimbField::Dip => (0, fx(1, 2)),
        }
    }
}

impl Species {
    /// Where its climb row starts: after everything else it has.
    pub fn climb_base(&self) -> usize {
        self.objective_base() + self.fight.objectives.len() * crate::objective::OBJECTIVE_FIELDS
    }

    /// Where one field of its climb row sits.
    pub fn climb_index(&self, f: ClimbField) -> usize {
        self.climb_base() + f as usize
    }

    /// One field of its climb row: zero for a species with no body.
    pub fn climb_raw(&self, f: ClimbField) -> i32 {
        if !self.has_body() {
            return 0;
        }
        crate::oven::species_raw(self.id, self.climb_index(f))
    }

    pub fn climb_fx(&self, f: ClimbField) -> Fx {
        Fx::from_raw(self.climb_raw(f))
    }
}

// ---------------------------------------------------------------------------
// The body, read off its skeleton
// ---------------------------------------------------------------------------

/// **What the climb needs to know about a body**, at its tuned scale, in
/// metres along its own facing from where it stands (`Monster::pos`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Build {
    /// How high its front legs hang from, standing: its shoulder. The root's
    /// height for a body with no front legs.
    pub shoulder: Fx,
    /// Where its front legs and its hind legs hang, along its facing. The
    /// root for an end with no legs.
    pub front: Fx,
    pub rear: Fx,
    /// The furthest forward and back any of it reaches, standing.
    pub nose: Fx,
    pub tail: Fx,
}

/// A bone's place at rest, from the root's foot, summed up its chain.
fn rest_of(sp: &Species, bone: usize) -> V3 {
    let mut at = V3::ZERO;
    let mut b = bone;
    loop {
        at = at.add(sp.rest(b));
        let parent = sp.bones[b].parent;
        if parent == NO_PARENT {
            return at;
        }
        b = parent;
    }
}

/// [`Build`], for a species.
pub fn build(sp: &Species) -> Build {
    let root = sp.rest(0);
    let (mut front, mut rear) = (None::<V3>, None::<V3>);
    for leg in sp.legs {
        let at = rest_of(sp, leg.hip);
        let end = if leg.front { &mut front } else { &mut rear };
        // Both legs of a pair hang from the same place along the body; the
        // furthest forward of the front pair and the furthest back of the
        // hind one, for a body with more than two pairs.
        *end = Some(match *end {
            Some(e) if leg.front && e.x.raw() >= at.x.raw() => e,
            Some(e) if !leg.front && e.x.raw() <= at.x.raw() => e,
            _ => at,
        });
    }
    let shoulder = front.or(rear).map_or(root.y, |a| a.y).max(Fx::ZERO);
    let front_x = front.map_or(root.x, |a| a.x);
    let rear_x = rear.map_or(root.x, |a| a.x);
    // How far the body runs: every part's box at rest, by its bone's place.
    let (mut nose, mut tail) = (front_x.max(rear_x), front_x.min(rear_x));
    for p in 0..sp.parts.len() {
        let sh = sp.shape(p);
        let at = rest_of(sp, sh.bone);
        nose = nose.max(at.x.add(sh.max.x));
        tail = tail.min(at.x.add(sh.min.x));
    }
    Build {
        shoulder,
        front: front_x,
        rear: rear_x,
        nose,
        tail,
    }
}

/// **The highest ledge it gets up**, in metres: its reach times its shoulder.
pub fn reach(sp: &Species) -> Fx {
    sp.climb_fx(ClimbField::Reach).mul(build(sp).shoulder)
}

/// **The deepest drop it gets down**, in metres.
pub fn drop_reach(sp: &Species) -> Fx {
    sp.climb_fx(ClimbField::Drop).mul(build(sp).shoulder)
}

/// **Frames to get up (`rise` above nothing) or down (below)**.
pub fn frames(sp: &Species, rise: Fx) -> u16 {
    frames_at(
        rise,
        sp.climb_raw(ClimbField::Frames),
        sp.climb_raw(ClimbField::PerMetre),
        sp.climb_fx(ClimbField::Down),
    )
}

/// [`frames`], from the rates themselves: `frames` and `per_metre` to get
/// up, and `down` the share of that getting down takes. A bred body's come
/// from the Oven rather than a species' row ([`crate::breed::climb`]).
pub fn frames_at(rise: Fx, frames: i32, per_metre: i32, down: Fx) -> u16 {
    let up = Fx::from_int(frames).add(Fx::from_int(per_metre).mul(rise.abs()));
    let n = if rise.raw() < 0 { up.mul(down) } else { up };
    n.to_int().clamp(2, u16::MAX as i32) as u16
}

/// **Where it looks for a ledge or an edge**, along its facing: just past
/// the front of its legs -- the furthest disc it is fenced by at its feet
/// ([`discs`]), its radius, and a hand more. The lip of what it gets up or
/// down is about here.
pub fn probe(sp: &Species) -> Fx {
    let b = build(sp);
    let step = sp.fight_fx(crate::species::FightField::StepOver);
    let (discs, n) = discs(sp);
    let front = discs
        .iter()
        .take(n)
        .filter(|(_, low)| low.raw() <= step.raw())
        .map(|(x, _)| *x)
        .fold(b.front, Fx::max);
    front.add(footprint(sp)).add(Fx::ratio(1, 4))
}

/// **How far forward a body goes getting up or down**, with the lip `lip`
/// ahead of where it stands: until its hind feet are past the lip by a
/// quarter of its shoulder, so they are not left on the edge.
pub fn advance_of(b: &Build, lip: Fx) -> Fx {
    let quarter = Fx::from_raw(b.shoulder.raw() / 4);
    lip.sub(b.rear).add(quarter).max(quarter)
}

/// [`advance_of`], for a species, with the lip at its [`probe`].
pub fn advance(sp: &Species) -> Fx {
    advance_of(&build(sp), probe(sp))
}

/// **How wide it is against the solids**: its senses row's body radius, for a
/// species that has one; a quarter of its shoulder for one that does not.
pub fn footprint(sp: &Species) -> Fx {
    let r = sp.fight_fx(crate::species::FightField::BodyRadius);
    if r.raw() > 0 {
        r
    } else {
        Fx::from_raw(build(sp).shoulder.raw() / 4)
    }
}

/// The most discs a body is fenced by along its length.
pub const MAX_DISCS: usize = 8;

/// **The body against the solids, along its length**: discs of
/// [`footprint`] from its hind legs to its nose, each with how high its
/// underside is there -- a head held over a rock is not in the rock. In
/// metres along the facing, and over its feet. A single cylinder round its
/// middle let a Ridgeback's head and forelegs into anything it walked at.
pub fn discs(sp: &Species) -> ([(Fx, Fx); MAX_DISCS], usize) {
    let b = build(sp);
    let r = footprint(sp);
    let from = b.rear.min(b.front);
    let to = b.nose.sub(r).max(from);
    let span = to.sub(from);
    let n = if r.raw() > 0 {
        (span.div(r).to_int() + 1).clamp(1, MAX_DISCS as i32 - 1) as usize + 1
    } else {
        1
    };
    let mut out = [(Fx::ZERO, Fx::ZERO); MAX_DISCS];
    for (k, disc) in out.iter_mut().enumerate().take(n) {
        let x = if n > 1 {
            from.add(
                span.mul(Fx::from_int(k as i32))
                    .div(Fx::from_int(n as i32 - 1)),
            )
        } else {
            from
        };
        *disc = (x, underside(sp, x, r));
    }
    (out, n)
}

/// How high the lowest of it is within `r` of `x` along its length, standing.
fn underside(sp: &Species, x: Fx, r: Fx) -> Fx {
    let mut low: Option<Fx> = None;
    for p in 0..sp.parts.len() {
        let sh = sp.shape(p);
        let at = rest_of(sp, sh.bone);
        let (lo, hi) = (at.x.add(sh.min.x), at.x.add(sh.max.x));
        if hi.raw() < x.sub(r).raw() || lo.raw() > x.add(r).raw() {
            continue;
        }
        let y = at.y.add(sh.min.y);
        low = Some(low.map_or(y, |l: Fx| l.min(y)));
    }
    low.unwrap_or(Fx::ZERO).max(Fx::ZERO)
}

// ---------------------------------------------------------------------------
// The motion
// ---------------------------------------------------------------------------

/// **Where a body is in getting up or down**, at one moment of it: what
/// [`profile`] answers and both skeletons are posed from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Phase {
    /// How far its middle has risen, as a share of the rise: what its
    /// standing height is carried by.
    pub body: Fx,
    /// How far it has gone forward, as a share of [`advance`].
    pub along: Fx,
    /// How high its front end is over where its middle is, in metres: what
    /// the body pitches by.
    pub front: Fx,
    /// Where its front and hind feet are, in metres over where it started.
    pub front_foot: Fx,
    pub rear_foot: Fx,
    /// How deep it has settled, in metres: the crouch before the push.
    pub dip: Fx,
}

/// A ramp from nothing to one between two moments, eased at both ends.
fn ramp(u: Fx, a: Fx, b: Fx) -> Fx {
    if u.raw() <= a.raw() {
        return Fx::ZERO;
    }
    if u.raw() >= b.raw() {
        return Fx::ONE;
    }
    math::smoothstep(u.sub(a).div(b.sub(a)))
}

/// A bump from nothing to one and back between two moments.
fn bump(u: Fx, a: Fx, b: Fx) -> Fx {
    if u.raw() <= a.raw() || u.raw() >= b.raw() {
        return Fx::ZERO;
    }
    math::hump(u.sub(a).div(b.sub(a)))
}

/// When each part of it moves, as shares of the whole: the forefeet, the
/// front end, the middle, the hind feet. Getting up the front end goes first
/// and the forefeet follow it over the lip; getting down the forefeet reach
/// down first and the front end follows them.
struct Beats {
    along: (Fx, Fx),
    front_foot: (Fx, Fx),
    front: (Fx, Fx),
    body: (Fx, Fx),
    rear_foot: (Fx, Fx),
}

const fn beats(up: bool) -> Beats {
    const fn r(n: i32) -> Fx {
        Fx::ratio(n, 100)
    }
    if up {
        Beats {
            along: (r(10), r(95)),
            front_foot: (r(20), r(45)),
            front: (r(10), r(50)),
            body: (r(45), r(90)),
            rear_foot: (r(50), r(85)),
        }
    } else {
        Beats {
            along: (r(10), r(95)),
            front_foot: (r(5), r(40)),
            front: (r(15), r(55)),
            body: (r(45), r(90)),
            rear_foot: (r(55), r(90)),
        }
    }
}

/// **The motion of getting up `rise` metres (down, below nothing)**, at
/// progress `u` from nought to one: the same for every body, scaled by its
/// shoulder for how high its feet clear the lip and how deep it settles.
pub fn profile(u: Fx, rise: Fx, shoulder: Fx, clear: Fx, dip: Fx) -> Phase {
    let up = rise.raw() >= 0;
    let b = beats(up);
    let u = u.clamp(Fx::ZERO, Fx::ONE);
    let body = ramp(u, b.body.0, b.body.1);
    let front_end = rise.mul(ramp(u, b.front.0, b.front.1));
    let lift = clear.mul(shoulder);
    let front_foot = rise
        .mul(ramp(u, b.front_foot.0, b.front_foot.1))
        .add(lift.mul(bump(u, b.front_foot.0, b.front_foot.1)));
    let rear_foot = rise
        .mul(ramp(u, b.rear_foot.0, b.rear_foot.1))
        .add(lift.mul(bump(u, b.rear_foot.0, b.rear_foot.1)));
    // Settled into its haunches before the push, and up off them through it:
    // only going up -- going down it is the forelegs that take the weight.
    let settle = if up {
        dip.mul(shoulder)
            .mul(bump(u, Fx::ZERO, math::half(b.body.0.add(b.body.1))))
    } else {
        Fx::ZERO
    };
    Phase {
        body,
        along: ramp(u, b.along.0, b.along.1),
        front: front_end.sub(rise.mul(body)),
        front_foot,
        rear_foot,
        dip: settle,
    }
}

/// [`profile`] for a species, with its own knobs.
pub fn phase_of(sp: &Species, u: Fx, rise: Fx) -> Phase {
    profile(
        u,
        rise,
        build(sp).shoulder,
        sp.climb_fx(ClimbField::Clear),
        sp.climb_fx(ClimbField::Dip),
    )
}

/// **How far a body pitches** to put its front end `front` metres over its
/// middle: the lever is how far apart its ends are, and it is never pitched
/// steeper than [`STEEPEST`] -- past that the forelegs reach rather than the
/// spine tilting. Both skeletons pitch by this.
pub fn pitch_of(b: &Build, front: Fx) -> Fx {
    let span = b.front.sub(b.rear).max(math::half(b.shoulder));
    math::atan2_turns(front, span).clamp(STEEPEST.neg(), STEEPEST)
}

/// The steepest a body is pitched getting up or down, in turns: sixty degrees.
pub const STEEPEST: Fx = Fx::ratio(1, 6);

/// The most legs a body is planted by: one fold each.
pub const MAX_LEGS: usize = 16;

/// Fold one leg by `c` turns: the hip one way and the knee twice the other,
/// the way its end folds -- the same rule a lame leg bends by.
fn fold(p: &mut Pose, leg: &crate::beast::Leg, c: Fx) {
    let (hip, knee) = if leg.front {
        (c.neg(), c.add(c))
    } else {
        (c, c.add(c).neg())
    };
    p.bone[leg.hip].x = p.bone[leg.hip].x.add(hip);
    p.bone[leg.knee].x = p.bone[leg.knee].x.add(knee);
}

/// **The pose of a creature getting up or down**, as a layer over its idle:
/// the root pitched so its front end is where the profile puts it, each leg
/// folded so its foot is where the profile puts it, the neck pitched back so
/// the head stays level. Additive, like the lame layer it sits beside. The
/// folds are a straight leg's (`math::crouch_turns`); [`plant`] makes them
/// true of the leg the body really has.
pub fn layer(sp: &Species, u: Fx, rise: Fx) -> Pose {
    layer_and_folds(sp, u, rise).0
}

fn layer_and_folds(sp: &Species, u: Fx, rise: Fx) -> (Pose, [Fx; MAX_LEGS]) {
    let mut p = Pose::rest(sp.bones.len());
    let mut folds = [Fx::ZERO; MAX_LEGS];
    let b = build(sp);
    let ph = phase_of(sp, u, rise);
    let scale = sp.scale();
    let pitch = pitch_of(&b, ph.front);
    p.bone[0].x = pitch;
    p.hips.y = ph.dip.neg().div(scale);
    // The head stays on what it is after: the neck takes the pitch back,
    // spread down it as the tracking is.
    if !sp.neck.is_empty() {
        let each = pitch.neg().div(Fx::from_int(sp.neck.len() as i32));
        for bone in sp.neck {
            p.bone[*bone].x = p.bone[*bone].x.add(each);
        }
    }
    let sin = sin_turns(pitch);
    // Where its middle is over where it started: the floor its feet are
    // measured from.
    let middle = rise.mul(ph.body);
    for (k, leg) in sp.legs.iter().enumerate().take(MAX_LEGS) {
        let lever = rest_of(sp, leg.hip).x.sub(sp.rest(0).x);
        // How far this leg's hip has come up, over where it started.
        let hip = middle.sub(ph.dip).add(lever.mul(sin));
        let foot = if leg.front {
            ph.front_foot
        } else {
            ph.rear_foot
        };
        // A hip lower over its foot than standing folds the leg by the
        // difference; one higher leaves the foot hanging -- reaching down off
        // the lip, or up over it.
        let drop = foot.sub(hip);
        let len = sp.rest(leg.knee).len().add(sp.shape(leg.foot).min.y.neg());
        let c = math::crouch_turns(drop, len);
        // The leg stays upright under a pitched body: it hangs from the hip,
        // not along the spine.
        p.bone[leg.hip].x = p.bone[leg.hip].x.add(pitch.neg());
        fold(&mut p, leg, c);
        folds[k] = c;
    }
    (p, folds)
}

/// Where a foot's sole is, posed: the middle of the bottom of its box.
fn sole(rig: &crate::beast::Rig, sp: &Species, foot: usize) -> Fx {
    let sh = sp.shape(foot);
    let mid = V3::new(
        math::half(sh.min.x.add(sh.max.x)),
        sh.min.y,
        math::half(sh.min.z.add(sh.max.z)),
    );
    rig.part_to_world(foot, mid).y
}

/// The most a leg folds getting up: a quarter of a turn at the hip.
const MOST_FOLD: Fx = Fx::ratio(1, 4);

/// **How a leg is moved to put its foot somewhere**: folded (a leg under
/// the body, hip one way and knee the other), spread sideways at the hip (a
/// leg splayed out to the side: a spider's, a tower of the Siegeshell's), or
/// swung at the hip alone (a leg reaching out ahead, a spider's forelegs).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Move {
    Fold,
    Spread,
    Swing,
}

const MOVES: [Move; 3] = [Move::Fold, Move::Spread, Move::Swing];

fn bend(p: &mut Pose, leg: &crate::beast::Leg, how: Move, c: Fx) {
    match how {
        Move::Fold => fold(p, leg, c),
        Move::Spread => {
            p.bone[leg.hip].z = p.bone[leg.hip].z.add(c.mul(Fx::from_int(leg.side)));
        }
        Move::Swing => p.bone[leg.hip].x = p.bone[leg.hip].x.add(c),
    }
}

/// **The climb, planted on the legs the body really has**: `base` is how it
/// stands this frame without the climb (its idle, its lame legs, its
/// species' own say), standing at `origin` facing `yaw`. The climb's layer
/// goes over it; then each foot is measured where it is, and its leg moved
/// -- by whichever of folding, spreading or swinging moves that foot most --
/// toward where the profile says, over where that foot stands. Three steps of
/// it, the later ones with the first one's measure of how the foot answers. So a
/// toad sitting on its folded hind feet, a spider on its splayed ones and a
/// tower of a leg all come out right, which a straight leg's arithmetic
/// alone does not. Seven builds of the skeleton, and only while it climbs.
pub fn plant(
    sp: &'static Species,
    base: &crate::beast::Pose,
    origin: V3,
    yaw: Fx,
    u: Fx,
    rise: Fx,
) -> Pose {
    use crate::beast::Rig;
    let ph = phase_of(sp, u, rise);
    let (layer, folds) = layer_and_folds(sp, u, rise);
    let mut out = base.over(&layer, Fx::ONE);
    let legs = sp.legs.len().min(MAX_LEGS);
    // Where each sole stands without the climb, and where the climb wants
    // it: the floor it started on is the middle's rise below the origin.
    let standing = Rig::build(sp, origin, yaw, base);
    let start = origin.y.sub(rise.mul(ph.body));
    let mut want = [Fx::ZERO; MAX_LEGS];
    for (k, leg) in sp.legs.iter().enumerate().take(legs) {
        want[k] = start
            .add(if leg.front {
                ph.front_foot
            } else {
                ph.rear_foot
            })
            .add(sole(&standing, sp, leg.foot).sub(origin.y));
    }
    // How each foot answers each way of moving its leg, by a small nudge.
    let now = Rig::build(sp, origin, yaw, &out);
    let step = Fx::ratio(1, 64);
    let mut how = [(Move::Fold, Fx::ZERO); MAX_LEGS];
    for m in MOVES {
        let mut nudged = out;
        for leg in sp.legs.iter().take(legs) {
            bend(&mut nudged, leg, m, step);
        }
        let rig = Rig::build(sp, origin, yaw, &nudged);
        for (k, leg) in sp.legs.iter().enumerate().take(legs) {
            let slope = sole(&rig, sp, leg.foot).sub(sole(&now, sp, leg.foot));
            if slope.abs().raw() > how[k].1.abs().raw() {
                how[k] = (m, slope);
            }
        }
    }
    let mut moved = [Fx::ZERO; MAX_LEGS];
    let mut rig = now;
    for pass in 0..3 {
        if pass > 0 {
            rig = Rig::build(sp, origin, yaw, &out);
        }
        for (k, leg) in sp.legs.iter().enumerate().take(legs) {
            let (m, slope) = how[k];
            if slope.raw() == 0 {
                continue;
            }
            let gap = want[k].sub(sole(&rig, sp, leg.foot));
            let total = moved[k].add(gap.mul(step).div(slope));
            // A fold never straightens a leg past standing; nothing moves a
            // leg more than a quarter of a turn.
            let total = match m {
                Move::Fold => total.clamp(folds[k].neg(), MOST_FOLD.sub(folds[k])),
                _ => total.clamp(MOST_FOLD.neg(), MOST_FOLD),
            };
            bend(&mut out, leg, m, total.sub(moved[k]));
            moved[k] = total;
        }
    }
    out
}
