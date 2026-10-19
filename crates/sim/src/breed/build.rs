//! The genome as a body and a move set.
//!
//! The body comes out in the same terms the species tables are written in --
//! [`crate::beast::Bone`]s hanging off their parents at rest offsets in
//! metres, [`crate::beast::Part`] boxes on those bones, and
//! [`crate::beast::Leg`]s -- so a built genome and a hand-written creature
//! can be laid side by side (`tests/breed.rs` does: the Ridgeback's hips are
//! 4.95 m up in its table, and its genome builds to within a fifth of that).
//! Nothing is allocated: every list is a fixed array with a count.
//!
//! The move set is what the plan *can* do: a jaw bites, horns gore, a level
//! body on walking legs charges, a raptorial fore pair scythes and guards, a
//! wing that lifts stoops. Each move's frames and damage are read off the
//! sliders -- heavier and bigger is slower and harder, a quick tempo trims
//! the startup -- and its appetite off the disposition.

use super::drive::{Drive, Wings};
use super::plan::{Caps, assess};
use super::{
    Back, Build, Crown, Genome, HeadKind, Hide, Jaw, Role, Stance, Station, Tail, TailTip, Tip, fx,
};
use crate::beast::{Bone, Leg, MAX_BONES, MAX_PARTS, NO_PARENT, Part, bone, part};
use crate::fixed::{Fx, cos_turns, sin_turns};
use crate::math::V3;
use crate::tuning::HUMAN_REACTION_FRAMES;

// ---------------------------------------------------------------------------
// Scale
// ---------------------------------------------------------------------------

/// Two to a power between nought and eight, from a table of eighths.
fn pow2(x: Fx) -> Fx {
    const EIGHTHS: [Fx; 9] = [
        fx(1000, 1000),
        fx(1091, 1000),
        fx(1189, 1000),
        fx(1297, 1000),
        fx(1414, 1000),
        fx(1542, 1000),
        fx(1682, 1000),
        fx(1834, 1000),
        fx(2000, 1000),
    ];
    let x = x.clamp(Fx::ZERO, Fx::from_int(8));
    let whole = x.to_int();
    let frac = x.sub(Fx::from_int(whole));
    let step = frac.mul(Fx::from_int(8));
    let i = step.to_int().clamp(0, 7) as usize;
    let t = step.sub(Fx::from_int(i as i32));
    let f = EIGHTHS[i].add(EIGHTHS[i + 1].sub(EIGHTHS[i]).mul(t));
    Fx::from_int(1 << whole).mul(f)
}

/// The body's length in metres, nose to rump, from the size slider: 0.4 m
/// at nought, 36 m at one, doubling every sixth and a half of the way. A
/// gnawer is a quarter of the way along, the Ridgeback three quarters, the
/// Siegeshell near the end.
pub fn length_m(size: Fx) -> Fx {
    fx(2, 5).mul(pow2(size.mul(fx(13, 2))))
}

// ---------------------------------------------------------------------------
// The body
// ---------------------------------------------------------------------------

/// The most legs a body has: two per station.
pub const MAX_LEGS: usize = super::STATIONS * 2;

/// A built body.
#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub bones: [Bone; MAX_BONES],
    pub bone_count: usize,
    pub parts: [Part; MAX_PARTS],
    pub part_count: usize,
    pub legs: [Leg; MAX_LEGS],
    pub leg_count: usize,
    /// Nose to rump, metres.
    pub length: Fx,
    /// How high the hips sit, metres.
    pub hips: Fx,
    /// Half the width of the torso.
    pub half_width: Fx,
    /// Tip to tip of the wings, metres, or zero.
    pub span: Fx,
    /// The pitch of the torso, in turns above level.
    pub pitch: Fx,
    /// Each part's axis: the unit direction its box runs along, in the
    /// body's frame. A part's box is authored in the frame this axis
    /// makes ([`frame`]): `shape.min`/`max` are along (axis, across,
    /// up), centred where the part sits. The game draws a part in its
    /// bone's rotated frame; this is that frame, written down.
    pub axes: [V3; MAX_PARTS],
}

/// Vector arithmetic the builder needs and `V3` does not carry.
fn dot(a: V3, b: V3) -> Fx {
    a.x.mul(b.x).add(a.y.mul(b.y)).add(a.z.mul(b.z))
}

fn cross(a: V3, b: V3) -> V3 {
    V3::new(
        a.y.mul(b.z).sub(a.z.mul(b.y)),
        a.z.mul(b.x).sub(a.x.mul(b.z)),
        a.x.mul(b.y).sub(a.y.mul(b.x)),
    )
}

fn vlen(a: V3) -> Fx {
    dot(a, a).sqrt()
}

/// `a` made unit length; `+x` for a zero vector.
pub fn unit(a: V3) -> V3 {
    let l = vlen(a);
    if l < fx(1, 1000) {
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
    } else {
        a.scale(Fx::ONE.div(l))
    }
}

/// The frame an axis makes: `(across, up)`, both unit and perpendicular to
/// it. `up` is the world's up turned perpendicular to the axis (or `+x`
/// turned, for a vertical axis), and `across` is `up × axis`, so for an
/// axis along `+x` the frame is (`-z`, `+y`): a box's (along, across, up)
/// extents are its (x, z, y) ones. The sheet derives the same frame.
pub fn frame(axis: V3) -> (V3, V3) {
    let u = unit(axis);
    let up = V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO);
    let mut w = up.sub(u.scale(dot(u, up)));
    if vlen(w) < fx(1, 100) {
        let x = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
        w = x.sub(u.scale(dot(u, x)));
    }
    let w = unit(w);
    let v = cross(w, u);
    (v, w)
}

impl Body {
    pub fn bones(&self) -> &[Bone] {
        &self.bones[..self.bone_count]
    }
    pub fn parts(&self) -> &[Part] {
        &self.parts[..self.part_count]
    }
    pub fn legs(&self) -> &[Leg] {
        &self.legs[..self.leg_count]
    }
    pub fn bone_named(&self, name: &str) -> Option<usize> {
        self.bones().iter().position(|b| b.name == name)
    }
    /// The rest position of a bone in the body's frame, metres.
    pub fn at(&self, bone: usize) -> V3 {
        let mut b = bone;
        let mut p = V3::ZERO;
        let mut guard = 0;
        while b != NO_PARENT && guard < MAX_BONES {
            p = p.add(self.bones[b].rest);
            b = self.bones[b].parent;
            guard += 1;
        }
        p
    }
    /// A part's centre in the body's frame, metres.
    pub fn centre(&self, i: usize) -> V3 {
        let p = &self.parts[i];
        let u = self.axes[i];
        let (v, w) = frame(u);
        let c = p.shape.min.add(p.shape.max).scale(fx(1, 2));
        self.at(p.shape.bone)
            .add(u.scale(c.x))
            .add(v.scale(c.y))
            .add(w.scale(c.z))
    }

    /// A part's eight corners in the body's frame, metres.
    pub fn corners(&self, i: usize) -> [V3; 8] {
        let p = &self.parts[i];
        let u = self.axes[i];
        let (v, w) = frame(u);
        let h = p.shape.max.sub(p.shape.min).scale(fx(1, 2));
        let c = self.centre(i);
        let mut out = [c; 8];
        for (k, corner) in out.iter_mut().enumerate() {
            let sa = if k & 1 == 0 { h.x } else { h.x.neg() };
            let sv = if k & 2 == 0 { h.y } else { h.y.neg() };
            let sw = if k & 4 == 0 { h.z } else { h.z.neg() };
            *corner = c.add(u.scale(sa)).add(v.scale(sv)).add(w.scale(sw));
        }
        out
    }

    /// The highest point of any part, metres.
    pub fn top(&self) -> Fx {
        let mut top = Fx::ZERO;
        for i in 0..self.part_count {
            for c in self.corners(i) {
                top = top.max(c.y);
            }
        }
        top
    }

    /// The lowest point of one part, metres.
    pub fn lowest(&self, i: usize) -> Fx {
        let mut low = Fx::from_int(1_000);
        for c in self.corners(i) {
            low = low.min(c.y);
        }
        low
    }
}

const EMPTY_BONE: Bone = bone("", NO_PARENT, V3::ZERO, 0);
const EMPTY_PART: Part = part("", 0, V3::ZERO, V3::ZERO, 0);
const EMPTY_LEG: Leg = Leg {
    upper: 0,
    foot: 0,
    hip: 0,
    knee: 0,
    front: false,
    side: 0,
};

struct Rig {
    body: Body,
}

impl Rig {
    fn new() -> Rig {
        Rig {
            body: Body {
                bones: [EMPTY_BONE; MAX_BONES],
                bone_count: 0,
                parts: [EMPTY_PART; MAX_PARTS],
                part_count: 0,
                legs: [EMPTY_LEG; MAX_LEGS],
                leg_count: 0,
                length: Fx::ZERO,
                hips: Fx::ZERO,
                half_width: Fx::ZERO,
                span: Fx::ZERO,
                pitch: Fx::ZERO,
                axes: [V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO); MAX_PARTS],
            },
        }
    }

    fn bone(&mut self, name: &'static str, parent: usize, rest: V3, side: i32) -> usize {
        let i = self.body.bone_count;
        if i < MAX_BONES {
            self.body.bones[i] = bone(name, parent, rest, side);
            self.body.bone_count += 1;
        }
        i.min(MAX_BONES - 1)
    }

    fn part(&mut self, name: &'static str, bone: usize, min: V3, max: V3) -> usize {
        let i = self.body.part_count;
        if i < MAX_PARTS {
            self.body.parts[i] = part(name, bone, min, max, 0);
            self.body.part_count += 1;
        }
        i.min(MAX_PARTS - 1)
    }

    /// A box centred on a point, half-sizes given.
    /// A box centred `at` (an offset from the bone, in the body's frame)
    /// running along `axis`, with half-extents `half` given as (along,
    /// up, across) -- the same order a box along `+x` would give them as
    /// (x, y, z). Stored in the axis's frame ([`frame`]) as the species
    /// tables store theirs in a bone's.
    fn along(&mut self, name: &'static str, bone: usize, at: V3, axis: V3, half: V3) -> usize {
        let u = unit(axis);
        let (v, w) = frame(u);
        let c = v3(dot(at, u), dot(at, v), dot(at, w));
        let h = v3(half.x, half.z, half.y);
        let i = self.part(name, bone, c.sub(h), c.add(h));
        self.body.axes[i] = u;
        i
    }

    /// Lift a part so its lowest corner rests on the floor.
    fn rest_on_floor(&mut self, i: usize) {
        let low = self.body.lowest(i);
        if low >= Fx::ZERO {
            return;
        }
        let u = self.body.axes[i];
        let (v, w) = frame(u);
        let up = V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO);
        let lift = v3(dot(up, u), dot(up, v), dot(up, w)).scale(low.neg());
        let p = &mut self.body.parts[i];
        p.shape.min = p.shape.min.add(lift);
        p.shape.max = p.shape.max.add(lift);
    }

    /// A box along the body's `+x`: `half` is (x, y, z).
    fn boxed(&mut self, name: &'static str, bone: usize, at: V3, half: V3) -> usize {
        self.along(name, bone, at, V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO), half)
    }

    /// A limb segment: a box from the bone's origin along `seg`, as thick
    /// as `girth`, reaching half a girth past either end so it meets what
    /// it joins.
    fn segment(&mut self, name: &'static str, bone: usize, seg: V3, girth: Fx) -> usize {
        let l = vlen(seg);
        self.along(
            name,
            bone,
            seg.scale(fx(1, 2)),
            seg,
            v3(l.mul(fx(1, 2)).add(girth.mul(fx(1, 2))), girth, girth),
        )
    }

    fn leg(&mut self, upper: usize, foot: usize, hip: usize, knee: usize, front: bool, side: i32) {
        let i = self.body.leg_count;
        if i < MAX_LEGS {
            self.body.legs[i] = Leg {
                upper,
                foot,
                hip,
                knee,
                front,
                side,
            };
            self.body.leg_count += 1;
        }
    }
}

/// Names for the limb bones and parts, by station and side, so nothing is
/// formatted at build time.
const LIMB_NAMES: [[[&str; 3]; 2]; super::STATIONS] = [
    [
        ["fore.l", "fore.l.lower", "fore.l.foot"],
        ["fore.r", "fore.r.lower", "fore.r.foot"],
    ],
    [
        ["midfore.l", "midfore.l.lower", "midfore.l.foot"],
        ["midfore.r", "midfore.r.lower", "midfore.r.foot"],
    ],
    [
        ["midhind.l", "midhind.l.lower", "midhind.l.foot"],
        ["midhind.r", "midhind.r.lower", "midhind.r.foot"],
    ],
    [
        ["hind.l", "hind.l.lower", "hind.l.foot"],
        ["hind.r", "hind.r.lower", "hind.r.foot"],
    ],
];

fn v3(x: Fx, y: Fx, z: Fx) -> V3 {
    V3::new(x, y, z)
}

// ---------------------------------------------------------------------------
// Proportions
// ---------------------------------------------------------------------------

/// The body's measurements in metres, before anything is placed: what the
/// posture pass balances and the builder draws. Positions are along the
/// torso's axis, forward positive, from the root.
#[derive(Clone, Copy, Debug)]
pub struct Proportions {
    pub len: Fx,
    pub torso: Fx,
    pub half_w: Fx,
    pub half_h: Fx,
    pub head_half: Fx,
    pub neck_len: Fx,
    pub tail_len: Fx,
    pub thorax: bool,
    pub abdomen: bool,
    pub shell: bool,
    /// Where the neck leaves: the front of the last segment.
    pub front: Fx,
    /// The rear of the last segment behind the root.
    pub rear: Fx,
    /// The furthest back a limb may attach: the haunch's rear.
    pub rear_attach: Fx,
    /// Each pair's length in metres (a wing's is longer than its share),
    /// where it attaches by default, and its half-thickness.
    pub limb: [Fx; super::STATIONS],
    pub attach: [Fx; super::STATIONS],
    pub girth: [Fx; super::STATIONS],
}

pub fn proportions(g: &Genome) -> Proportions {
    let s = &g.sliders;
    let len = length_m(s.size);
    let mut half_w = len.mul(fx(3, 50).add(s.mass.mul(fx(3, 50))));
    // A low body is wide -- a toad, a walking hill -- unless it stands on
    // spindles, whose width is in their legs: a spider's cephalothorax is
    // narrow and its legs make the span.
    let spindly = g.pairs().any(|(_, p)| p.role.grounded())
        && g.pairs()
            .filter(|(_, p)| p.role.grounded())
            .all(|(_, p)| p.build == Build::Spindle);
    if g.stance == Stance::Lowslung {
        half_w = half_w.mul(if spindly { fx(7, 5) } else { Fx::from_int(4) });
    }
    let half_h = half_w.mul(fx(9, 20));
    let torso = match g.stance {
        Stance::Level => len.mul(fx(3, 10)),
        Stance::Upright => len.mul(fx(1, 4)),
        Stance::Lowslung => len.mul(fx(9, 20)),
        Stance::Limbless => len.mul(fx(1, 16)),
    };
    let sixth = torso.mul(fx(1, 6));
    let front = if g.segments.thorax {
        torso.add(sixth)
    } else {
        torso.mul(fx(5, 6))
    };
    // An abdomen hangs from the waist at the haunch's rear and runs a
    // torso and a half behind it.
    let rear = if g.segments.abdomen {
        sixth.neg().sub(torso.mul(fx(3, 2)))
    } else {
        sixth.neg()
    };
    let head_half = len.mul(fx(1, 20)).add(half_w.mul(fx(1, 4)));
    let neck_len = match g.head {
        HeadKind::Forward => len.mul(fx(3, 20)),
        HeadKind::Raised => len.mul(fx(1, 5)),
        HeadKind::Low => len.mul(fx(3, 20)),
        HeadKind::Merged => len.mul(fx(1, 16)),
    };
    let tail_len = match g.segments.tail {
        Tail::None => Fx::ZERO,
        Tail::Level | Tail::Drooping => len.mul(fx(2, 5)),
        Tail::Raised => len.mul(fx(1, 4)),
    };
    let mut limb = [Fx::ZERO; super::STATIONS];
    let mut attach = [Fx::ZERO; super::STATIONS];
    let mut girth = [Fx::ZERO; super::STATIONS];
    for station in Station::ALL {
        let i = station.index();
        // Where a pair sits by default: under the front segment, along
        // the body, and just behind the root. On an upright body the
        // walking pairs cluster at the base of the raised part.
        attach[i] = match (g.stance, station) {
            (Stance::Upright, Station::Fore) => torso.mul(fx(1, 3)),
            (Stance::Upright, Station::MidFore) => sixth,
            (Stance::Upright, Station::MidHind) => torso.mul(fx(1, 12)),
            (Stance::Upright, Station::Hind) => torso.mul(fx(-1, 12)),
            (_, Station::Fore) => front.sub(sixth),
            (_, Station::MidFore) => torso.mul(fx(1, 2)),
            (_, Station::MidHind) => torso.mul(fx(1, 4)),
            (_, Station::Hind) => torso.mul(fx(-1, 12)),
        };
        let Some(p) = g.pair(station) else { continue };
        // A wing is longer than a leg of the same share: the Galewing's
        // reach nine metres on a body of five and a half.
        limb[i] = if p.role == Role::Wing {
            p.length.mul(len).mul(fx(3, 2))
        } else {
            p.length.mul(len)
        };
        // A limb's half-thickness: a fourteenth of its length times its
        // girth, plus an eighth of the body's half-width unless it is a
        // spindle, and never under a thirtieth of its length.
        let mut gi = limb[i].mul(p.girth).mul(fx(1, 14));
        if p.build != Build::Spindle {
            gi = gi.add(half_w.mul(fx(1, 8)));
        }
        girth[i] = gi.max(limb[i].mul(fx(1, 30)));
    }
    Proportions {
        len,
        torso,
        half_w,
        half_h,
        head_half,
        neck_len,
        tail_len,
        thorax: g.segments.thorax,
        abdomen: g.segments.abdomen,
        shell: g.minors.back == Back::Burden || g.minors.hide == Hide::Shell,
        front,
        rear,
        rear_attach: sixth.neg(),
        limb,
        attach,
        girth,
    }
}

// ---------------------------------------------------------------------------
// The body
// ---------------------------------------------------------------------------

/// Build the body, standing as its posture says.
pub fn body(g: &Genome) -> Body {
    let q = super::posture::posture(g);
    body_in(g, &q)
}

/// Build the body in a given posture.
pub fn body_in(g: &Genome, q: &super::posture::Posture) -> Body {
    build(g, q, &super::locomotion::rest(g))
}

/// The body in a posture, bent as a [`Drive`] says: the one way every
/// moment of a walk, a move or a flight is built.
pub fn body_driven(g: &Genome, q: &super::posture::Posture, d: &Drive) -> Body {
    build(g, q, d)
}

/// A moment of its walk: `phase` is where in the stride cycle, nought to
/// one, in the gait it walks in ([`super::locomotion::walking`]): feet planted
/// and swung in that gait's order, the body bobbing, a limbless body
/// throwing a wave down its length. A walk is never still.
pub fn body_posed(g: &Genome, q: &super::posture::Posture, phase: Fx) -> Body {
    let gait = super::locomotion::walking(g, q);
    build(
        g,
        q,
        &super::locomotion::drive(g, q, &gait, phase, fx(1, 2)),
    )
}

/// A moment of its flight: wings spread through the beat, legs trailing,
/// the body lifted off the floor by half its length. A creature whose
/// wings cannot lift it **flares** them on the ground instead -- spread
/// and standing. A creature with no wings walks. The pose that says
/// whether it flies, kept apart from how it stands, so a creature that
/// leans on its wings to walk still shows the wings it flies on.
pub fn body_flying(g: &Genome, q: &super::posture::Posture, phase: Fx) -> Body {
    build(g, q, &super::locomotion::flight(g, q, phase))
}

/// What a wing is made of, from the hide: three looks that each read as
/// a wing. Feathered keeps its chord to the tip and fans primaries behind
/// it; a membrane tapers to a point along a leading-edge finger; a furred
/// wing is a thick fringed plane.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WingLook {
    Feathered,
    Membrane,
    Furred,
}

impl WingLook {
    pub const fn of(hide: Hide) -> WingLook {
        match hide {
            Hide::Feathers => WingLook::Feathered,
            Hide::Fur => WingLook::Furred,
            Hide::Skin | Hide::Plates | Hide::Shell | Hide::Chitin | Hide::Slick => {
                WingLook::Membrane
            }
        }
    }

    /// The chord of each of the three segments, as shares of the body's
    /// length: the Galewing's root chord is three metres on a body of
    /// five and a half.
    const fn chords(self) -> [(i32, i32); 3] {
        match self {
            WingLook::Feathered => [(9, 20), (2, 5), (3, 10)],
            WingLook::Membrane => [(1, 2), (3, 10), (3, 20)],
            WingLook::Furred => [(2, 5), (7, 20), (1, 4)],
        }
    }

    /// Part names, by side, so the sheet can colour the look.
    const fn names(self, side: usize) -> [&'static str; 3] {
        match (self, side) {
            (WingLook::Feathered, 0) => {
                ["wing_feather.l", "wing_feather.l.mid", "wing_feather.l.tip"]
            }
            (WingLook::Feathered, _) => {
                ["wing_feather.r", "wing_feather.r.mid", "wing_feather.r.tip"]
            }
            (WingLook::Membrane, 0) => [
                "wing_membrane.l",
                "wing_membrane.l.mid",
                "wing_membrane.l.tip",
            ],
            (WingLook::Membrane, _) => [
                "wing_membrane.r",
                "wing_membrane.r.mid",
                "wing_membrane.r.tip",
            ],
            (WingLook::Furred, 0) => ["wing_fur.l", "wing_fur.l.mid", "wing_fur.l.tip"],
            (WingLook::Furred, _) => ["wing_fur.r", "wing_fur.r.mid", "wing_fur.r.tip"],
        }
    }
}

fn build(g: &Genome, q: &super::posture::Posture, d: &Drive) -> Body {
    let mut r = Rig::new();
    let flies = super::plan::assess(g).caps.flies;
    // In the air: off the floor by half its length, legs trailing.
    let aloft = d.wings == Wings::Aloft && flies;
    // Wings spread in flight or in a flare on the ground; folded otherwise.
    let spread = d.wings != Wings::Folded && g.has_wings();
    let look = WingLook::of(g.minors.hide);
    let p = proportions(g);
    let swell = Fx::ONE.add(d.swell).max(fx(1, 2));
    let (len, torso, half_w, half_h) = (p.len, p.torso, p.half_w.mul(swell), p.half_h.mul(swell));
    let pitch = q.pitch.add(d.pitch);
    let along = v3(cos_turns(pitch), sin_turns(pitch), Fx::ZERO);
    r.body.length = len;
    r.body.hips = q.root;
    r.body.half_width = half_w;
    r.body.pitch = pitch;

    let bob = d.lift;
    let bob = if aloft {
        bob.add(len.mul(fx(1, 2)))
    } else {
        bob
    };
    let root = r.bone("root", NO_PARENT, v3(d.surge, q.root.add(bob), Fx::ZERO), 0);

    // The torso and its segments.
    let (fore_bone, fore_at) = if g.stance == Stance::Limbless {
        // A limbless body in motion throws a wave down its length: each
        // segment yaws a little further round than the last, and the
        // front of it lifts.
        let seg = along.scale(torso);
        let mut parent = root;
        r.along(
            "segment",
            root,
            V3::ZERO,
            along,
            v3(torso.mul(fx(1, 2)), half_w, half_w),
        );
        for (k, name) in ["neck1", "neck2", "neck3", "neck4", "neck5"]
            .into_iter()
            .enumerate()
        {
            let seg_k = if d.wave.is_some() || d.rise != Fx::ZERO {
                let yaw = match d.wave {
                    Some((phase, sway)) => sin_turns(phase.add(Fx::ratio(k as i32, 5))).mul(sway),
                    None => Fx::ZERO,
                };
                let lifted = if d.wave.is_some() {
                    fx(1, 40)
                } else {
                    Fx::ZERO
                };
                // Never past upright: a reared front stands, it does not curl back
                // over itself.
                let rise = lifted
                    .add(d.rise)
                    .mul(Fx::from_int(k as i32 + 1))
                    .min(fx(1, 5))
                    .max(fx(-1, 8));
                v3(
                    torso.mul(cos_turns(yaw)).mul(cos_turns(rise)),
                    torso.mul(sin_turns(rise)),
                    torso.mul(sin_turns(yaw)),
                )
            } else {
                seg
            };
            parent = r.bone(name, parent, seg_k, 0);
            r.along(
                name,
                parent,
                V3::ZERO,
                seg_k,
                v3(torso.mul(fx(11, 20)), half_w, half_w),
            );
        }
        (parent, seg)
    } else {
        let third = along.scale(torso.mul(fx(1, 3)));
        let spine = r.bone("spine", root, third, 0);
        let chest = r.bone("chest", spine, third, 0);
        r.along(
            "haunch",
            root,
            V3::ZERO,
            along,
            v3(torso.mul(fx(1, 6)), half_h, half_w),
        );
        r.along(
            "barrel",
            spine,
            V3::ZERO,
            along,
            v3(torso.mul(fx(1, 6)), half_h, half_w),
        );
        r.along(
            "shoulders",
            chest,
            V3::ZERO,
            along,
            v3(torso.mul(fx(1, 6)), half_h, half_w),
        );
        (chest, third)
    };
    let thorax = if g.segments.thorax {
        let t = r.bone("thorax", fore_bone, fore_at, 0);
        r.along(
            "thorax",
            t,
            V3::ZERO,
            along,
            v3(
                torso.mul(fx(1, 6)),
                half_h.mul(fx(3, 2)),
                half_w.mul(fx(4, 5)),
            ),
        );
        t
    } else {
        fore_bone
    };
    if g.segments.abdomen {
        // The waist: the haunch's rear. The abdomen runs back from it,
        // overlapping the haunch so the two are one body, tilted up a
        // little when it is a sac (a spider's) and trailing level when it
        // is not (a mantis's).
        let waist = along.scale(torso.mul(fx(-1, 6)));
        let a = r.bone("abdomen", root, waist, 0);
        let sac = g.minors.back == Back::Sacs;
        let tilt = if sac { fx(1, 24) } else { Fx::ZERO }.add(d.abdomen);
        let dir = v3(
            cos_turns(tilt.sub(pitch)).neg(),
            sin_turns(tilt.sub(pitch)),
            Fx::ZERO,
        );
        let (half_len, half_up, half_across) = if sac {
            (
                torso.mul(fx(3, 4)),
                torso.mul(fx(2, 5)),
                torso.mul(fx(11, 20)),
            )
        } else {
            (
                torso.mul(fx(3, 4)),
                half_h.mul(fx(6, 5)),
                half_w.mul(fx(9, 10)),
            )
        };
        let centre = dir.scale(half_len.mul(fx(9, 10)));
        r.along(
            "abdomen",
            a,
            centre,
            dir,
            v3(half_len, half_up, half_across),
        );
        if sac {
            // The sacs ride the top of it.
            r.along(
                "sacs",
                a,
                centre.add(v3(Fx::ZERO, half_up.mul(fx(4, 5)), Fx::ZERO)),
                dir,
                v3(
                    half_len.mul(fx(3, 5)),
                    half_up.mul(fx(2, 5)),
                    half_across.mul(fx(3, 5)),
                ),
            );
        }
    }
    if q.drags {
        // The rear rests on the floor and trails: a rump from the haunch's
        // underside down to the ground behind it.
        // A sloping segment from the haunch's underside to the floor.
        let th = half_h.mul(fx(1, 2));
        let start = v3(p.rear, half_h.neg().mul(fx(3, 5)), Fx::ZERO);
        let end = v3(p.rear.sub(q.drag_len), q.root.neg().add(th), Fx::ZERO);
        let run = end.sub(start);
        let rump = r.along(
            "rump",
            root,
            start.add(run.scale(fx(1, 2))),
            run,
            v3(vlen(run).mul(fx(1, 2)).add(th), th, half_w.mul(fx(4, 5))),
        );
        r.rest_on_floor(rump);
    }
    if g.minors.back == Back::Ridge {
        r.along(
            "ridge",
            root,
            v3(
                torso.mul(fx(1, 3)),
                half_h.add(half_h.mul(fx(1, 3))),
                Fx::ZERO,
            ),
            along,
            v3(
                torso.mul(fx(1, 2)),
                half_h.mul(fx(1, 3)),
                half_w.mul(fx(1, 4)),
            ),
        );
    }
    if p.shell {
        r.along(
            "shell",
            root,
            v3(torso.mul(fx(1, 3)), half_h.mul(fx(3, 2)), Fx::ZERO),
            along,
            v3(torso.mul(fx(3, 4)), half_h, half_w.mul(fx(11, 10))),
        );
    }

    // The head, on a neck raised as the posture says.
    let head_half = p.head_half;
    let neck_len = p.neck_len.mul(Fx::ONE.add(d.neck_reach).max(fx(1, 4)));
    let neck_yaw = d.neck_yaw;
    let neck_pitch = pitch.add(q.neck_rise).add(d.neck_pitch);
    let neck_dir = v3(
        cos_turns(neck_pitch).mul(cos_turns(neck_yaw)),
        sin_turns(neck_pitch),
        cos_turns(neck_pitch).mul(sin_turns(neck_yaw)),
    );
    let head = if g.head == HeadKind::Merged {
        // The front of the body is the head: as wide and as deep as the
        // body, a head's length long.
        let h = r.bone("head", thorax, along.scale(torso.mul(fx(1, 2))), 0);
        let deep = if g.stance == Stance::Limbless {
            half_w
        } else {
            half_h
        };
        r.along("head", h, V3::ZERO, along, v3(head_half, deep, half_w));
        h
    } else {
        let n = r.bone("neck", thorax, neck_dir.scale(neck_len.mul(fx(1, 2))), 0);
        let n2 = r.bone("neck2", n, neck_dir.scale(neck_len.mul(fx(1, 2))), 0);
        let h = r.bone("head", n2, neck_dir.scale(head_half), 0);
        // The neck runs the whole way from the body to the head, in two
        // overlapping pieces.
        r.along(
            "neck",
            n,
            neck_dir.scale(neck_len.mul(fx(1, 4))),
            neck_dir,
            v3(
                neck_len.mul(fx(3, 10)),
                head_half.mul(fx(3, 5)),
                head_half.mul(fx(3, 5)),
            ),
        );
        r.along(
            "neck",
            n2,
            neck_dir.scale(neck_len.mul(fx(1, 4))),
            neck_dir,
            v3(
                neck_len.mul(fx(3, 10)),
                head_half.mul(fx(3, 5)),
                head_half.mul(fx(3, 5)),
            ),
        );
        r.along(
            "head",
            h,
            V3::ZERO,
            neck_dir,
            v3(head_half, head_half.mul(fx(4, 5)), head_half.mul(fx(4, 5))),
        );
        h
    };
    match g.minors.crown {
        Crown::Horns => {
            r.along(
                "horns",
                head,
                v3(head_half.mul(fx(1, 2)), head_half, Fx::ZERO),
                neck_dir,
                v3(head_half.mul(fx(1, 2)), head_half.mul(fx(1, 2)), head_half),
            );
        }
        Crown::Crest => {
            r.along(
                "crest",
                head,
                v3(Fx::ZERO, head_half, Fx::ZERO),
                neck_dir,
                v3(
                    head_half.mul(fx(3, 5)),
                    head_half.mul(fx(1, 2)),
                    head_half.mul(fx(1, 5)),
                ),
            );
        }
        Crown::Feelers | Crown::None => {}
    }
    if g.minors.jaw != Jaw::Grinder {
        // The jaw drops open about its hinge at the back of the head.
        let gape = d.jaw.mul(fx(1, 8));
        let jaw_pitch = neck_pitch.sub(gape);
        let jaw_dir = v3(
            cos_turns(jaw_pitch).mul(cos_turns(neck_yaw)),
            sin_turns(jaw_pitch),
            cos_turns(jaw_pitch).mul(sin_turns(neck_yaw)),
        );
        let hinge = neck_dir.scale(head_half.mul(fx(-1, 2)));
        r.along(
            "jaw",
            head,
            hinge.add(jaw_dir.scale(head_half)).add(v3(
                Fx::ZERO,
                head_half.mul(fx(-3, 5)),
                Fx::ZERO,
            )),
            jaw_dir,
            v3(
                head_half.mul(fx(3, 5)),
                head_half.mul(fx(1, 4)),
                head_half.mul(fx(3, 5)),
            ),
        );
    }

    // The tail.
    let tail_len = p.tail_len;
    if g.stance == Stance::Limbless {
        let seg = along.scale(torso.neg());
        let mut parent = root;
        for (k, name) in ["tail1", "tail2", "tail3", "tail4", "tail5"]
            .into_iter()
            .enumerate()
        {
            let seg = if let Some((phase, sway)) = d.wave {
                let yaw = sin_turns(phase.sub(Fx::ratio(k as i32 + 1, 5))).mul(sway);
                v3(
                    torso.neg().mul(cos_turns(yaw)),
                    Fx::ZERO,
                    torso.neg().mul(sin_turns(yaw)),
                )
            } else {
                seg
            };
            parent = r.bone(name, parent, seg, 0);
            r.along(
                name,
                parent,
                V3::ZERO,
                seg,
                v3(
                    torso.mul(fx(11, 20)),
                    half_w.mul(fx(4, 5)),
                    half_w.mul(fx(4, 5)),
                ),
            );
        }
    } else if tail_len > Fx::ZERO {
        let droop = match g.segments.tail {
            Tail::Drooping => fx(-1, 8),
            Tail::Raised => fx(1, 6),
            _ => Fx::ZERO,
        };
        // Each segment a little further round than the last, so the tail
        // bends rather than pivots.
        let seg_at = |k: i32| {
            let yaw = d.tail_yaw.mul(Fx::from_int(k + 1));
            let droop = droop.add(d.tail_pitch.mul(Fx::from_int(k + 1)));
            let l = tail_len.mul(fx(1, 3));
            v3(
                l.neg().mul(cos_turns(droop)).mul(cos_turns(yaw)),
                l.mul(sin_turns(droop)),
                l.neg().mul(cos_turns(droop)).mul(sin_turns(yaw)),
            )
        };
        let t1 = r.bone("tail1", root, seg_at(0), 0);
        let t2 = r.bone("tail2", t1, seg_at(1), 0);
        let t3 = r.bone("tail3", t2, seg_at(2), 0);
        let th = half_h.mul(fx(1, 2));
        // Each piece runs from its joint to the next, a little past, so
        // the tail is one curve and not three sticks.
        let piece = tail_len.mul(fx(1, 6)).add(th.mul(fx(1, 2)));
        r.along(
            "tail",
            t1,
            seg_at(1).scale(fx(1, 2)),
            seg_at(1),
            v3(piece, th, th),
        );
        r.along(
            "tail, middle",
            t2,
            seg_at(2).scale(fx(1, 2)),
            seg_at(2),
            v3(piece, th.mul(fx(4, 5)), th.mul(fx(4, 5))),
        );
        let tip_half = match g.minors.tail_tip {
            TailTip::Club => th,
            TailTip::Blade => th.mul(fx(1, 2)),
            TailTip::Sting | TailTip::Plain => th.mul(fx(3, 5)),
        };
        r.along(
            "tail tip",
            t3,
            seg_at(3).scale(fx(1, 2)),
            seg_at(3),
            v3(piece, tip_half, tip_half),
        );
    }

    // The limbs, each standing as its station's pose says.
    // Tip to tip with the wings open, whatever pose they are in now.
    let mut span = Fx::ZERO;
    for (st, pair) in g.pairs() {
        if pair.role == Role::Wing {
            span = span.max(p.limb[st.index()].add(half_w).mul(Fx::from_int(2)));
        }
    }
    let hang = half_h.mul(super::posture::ATTACH_DOWN);
    for station in Station::ALL {
        let i = station.index();
        let Some(pair) = g.pair(station) else {
            continue;
        };
        let pose = q.stations[i];
        let limb = p.limb[i];
        let girth = p.girth[i];
        let fore = matches!(station, Station::Fore | Station::MidFore);
        // The hip: on the flank, down at the belly line, where the posture
        // put the pair along the body.
        // A wing that is not walked on roots at the top of the flank, as a
        // bird's does; every other limb at the belly line.
        let root_at = if pair.role == Role::Wing && !pose.wing_walks {
            half_h.mul(fx(1, 2)).neg()
        } else {
            hang
        };
        let hip_at = v3(
            pose.attach.mul(cos_turns(pitch)),
            pose.attach.mul(sin_turns(pitch)).sub(root_at),
            Fx::ZERO,
        );
        let hip_y = q.root.add(hip_at.y);
        for (side_i, side) in [-1, 1].into_iter().enumerate() {
            let names = LIMB_NAMES[station.index()][side_i];
            let sd = Fx::from_int(side);
            let z = half_w.mul(sd);
            let hip = r.bone(names[0], root, v3(hip_at.x, hip_at.y, z), side);
            let half_limb = limb.mul(fx(1, 2));
            // How far the wing has grown, nought to one: its chord grows
            // with it, so a wing that was a leg a few generations ago is a
            // small wing, folded or spread, and a full one a full one.
            let grown = pair.length.max(fx(1, 5));
            // How far behind the shoulder a folded wing may lie: to about
            // the tip of the tail, or the rump without one.
            let reach_back = pose
                .attach
                .add(torso.mul(fx(1, 6)))
                .add(p.tail_len.mul(fx(4, 5)));
            if pair.role == Role::Wing && spread {
                // A wing spread: three segments out to the side, swept
                // back and rising a little, each a broad thin plane.
                let seg = limb.mul(fx(1, 3));
                let sweep = if aloft {
                    // In the air, a beat: the wing rises and falls.
                    let a = d.beat;
                    v3(
                        limb.mul(fx(-1, 10)),
                        seg.mul(sin_turns(a)).add(limb.mul(fx(1, 15))),
                        seg.mul(cos_turns(a)).mul(sd),
                    )
                } else {
                    // On the ground, a flare: spread and raised.
                    v3(limb.mul(fx(-1, 10)), limb.mul(fx(1, 8)), seg.mul(sd))
                };
                let thick = match look {
                    WingLook::Furred => len.mul(fx(1, 40)).max(girth.mul(fx(1, 2))),
                    _ => len.mul(fx(1, 100)).max(girth.mul(fx(1, 5))),
                };
                let arm = r.bone(names[1], hip, sweep, side);
                let hand = r.bone(names[2], arm, sweep, side);
                let chords = look.chords();
                let wnames = look.names(side_i);
                let bones = [hip, arm, hand];
                for k in 0..3 {
                    let chord = len.mul(fx(chords[k].0, chords[k].1)).mul(grown);
                    // One plane along the segment, a little past either
                    // end so the segments meet, its area aft of the bone.
                    let at =
                        sweep
                            .scale(fx(1, 2))
                            .add(v3(chord.mul(fx(-1, 4)), Fx::ZERO, Fx::ZERO));
                    r.along(
                        wnames[k],
                        bones[k],
                        at,
                        sweep,
                        v3(
                            vlen(sweep).mul(fx(1, 2)).add(thick),
                            thick,
                            chord.mul(fx(1, 2)),
                        ),
                    );
                    if look == WingLook::Membrane {
                        // The finger along the leading edge.
                        r.along(
                            wnames[k],
                            bones[k],
                            sweep.scale(fx(1, 2)),
                            sweep,
                            v3(
                                seg.mul(fx(1, 2)),
                                thick.mul(Fx::from_int(2)),
                                thick.mul(Fx::from_int(2)),
                            ),
                        );
                    }
                }
                if look == WingLook::Feathered {
                    // Primaries: a fan trailing behind the tip.
                    r.along(
                        wnames[2],
                        hand,
                        v3(
                            sweep.x.sub(len.mul(fx(3, 10))),
                            sweep.y,
                            sweep.z.mul(fx(4, 5)),
                        ),
                        sweep,
                        v3(seg.mul(fx(1, 4)), thick, len.mul(fx(1, 6))),
                    );
                }
                span = limb.add(half_w).mul(Fx::from_int(2));
                continue;
            }
            if pair.role == Role::Wing && !pose.wing_walks {
                // A wing folded and held: the humerus back along the
                // flank, the forearm forward to a wrist just below the
                // shoulder, coming down toward the floor as the body
                // leans more on it, the hand folded back to the flank.
                let (arm_a, arm_b) = wing_arm(limb);
                // Eased in: a wing is held high and tight until the body
                // really leans on it, then comes down to the floor.
                let reach_lean = super::unit(pose.wing_lean.div(super::posture::WING_PLANT));
                let reach_lean = reach_lean.mul(reach_lean);
                let held = limb.mul(fx(1, 20));
                let floor = hip_y.sub(girth);
                let down = held.add(floor.sub(held).max(Fx::ZERO).mul(reach_lean));
                let want = v3(limb.mul(fx(1, 10)), down.neg(), Fx::ZERO);
                let (elbow, wrist) = ik2(arm_a, arm_b, want, v3(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO));
                let lower = wrist.sub(elbow);
                let girth = walking_wing_girth(len, girth);
                let wnames = look.names(side_i);
                let e = r.bone(names[1], hip, elbow, side);
                let w = r.bone(names[2], e, lower, side);
                r.segment(wnames[0], hip, elbow, girth);
                r.segment(wnames[1], e, lower, girth);
                fold_planes(
                    &mut r, look, side_i, e, w, lower, wrist, limb, len, girth, grown, reach_back,
                );
                continue;
            }
            if !pose.grounded {
                // A striking limb held up and forward: folded blades.
                let (upper, lower) = match pair.build {
                    // A mantis's fold: the upper arm up and forward from
                    // the shoulder, the blade folded back down under it
                    // with its point ahead -- a Z held at the chest.
                    // Walking, the leading arm half-extends.
                    // Cocked, the blade is raised open over the shoulder;
                    // thrust, the whole arm is out straight ahead.
                    Build::Raptorial => {
                        let (u, l) = arm_pose(
                            d.arms[side_i],
                            [((2, 5), (9, 10)), ((7, 10), (7, 10)), ((19, 20), (3, 10))],
                            [((1, 2), (4, 5)), ((1, 4), (-9, 10)), ((19, 20), (-1, 5))],
                        );
                        (
                            v3(
                                half_limb.mul(u.0),
                                half_limb.mul(u.1),
                                half_w.mul(fx(-1, 4)).mul(sd),
                            ),
                            v3(half_limb.mul(l.0), half_limb.mul(l.1), Fx::ZERO),
                        )
                    }
                    _ => {
                        let (u, l) = arm_pose(
                            d.arms[side_i],
                            [((-1, 5), (9, 10)), ((1, 2), (-1, 2)), ((9, 10), (-1, 5))],
                            [((3, 5), (4, 5)), ((1, 2), (0, 1)), ((1, 1), (0, 1))],
                        );
                        (
                            v3(half_limb.mul(u.0), half_limb.mul(u.1), Fx::ZERO),
                            v3(half_limb.mul(l.0), half_limb.mul(l.1), Fx::ZERO),
                        )
                    }
                };
                let knee = r.bone(names[1], hip, upper, side);
                let foot = r.bone(names[2], knee, lower, side);
                r.segment(names[0], hip, upper, girth);
                r.segment(names[1], knee, lower, girth);
                let tip_half = match pair.tip {
                    Tip::Blade => v3(
                        girth.mul(fx(3, 1)),
                        girth.mul(fx(1, 2)),
                        girth.mul(fx(1, 2)),
                    ),
                    _ => v3(girth.mul(fx(3, 2)), girth, girth),
                };
                r.along(names[2], foot, V3::ZERO, lower, tip_half);
                continue;
            }
            // A standing limb: hip to foot is a straight line of length d,
            // with the foot `reach` ahead of the hip on the floor; the
            // joint sits off the midpoint of that line.
            let splay = super::posture::splayed(g.stance, pair.build);
            let standing =
                super::posture::standing_length(g.stance, pair.build, pose.wing_walks, limb);
            // Where the drive has the foot: ahead of or behind its
            // standing spot, and off the floor.
            let foot = d.feet[i][side_i];
            let (swing, lift) = (standing.mul(foot.forward), standing.mul(foot.lift));
            let (mut down, mut reach) = if aloft {
                // In the air the legs trail, folded under and back.
                (
                    standing.mul(fx(11, 20)).sub(lift),
                    standing.mul(fx(-9, 20)).add(swing),
                )
            } else {
                (hip_y.sub(lift), pose.reach.add(swing))
            };
            let mut d = down.mul(down).add(reach.mul(reach)).sqrt();
            if d > standing {
                // It cannot reach: it hangs toward the floor.
                let k = standing.div(d.max(fx(1, 100)));
                down = down.mul(k);
                reach = reach.mul(k);
                d = standing;
            }
            let (upper, lower) = match pair.build {
                _ if pose.wing_walks => {
                    // The humerus and forearm reach the floor, elbow out
                    // to the side, the wrist planted.
                    let (arm_a, arm_b) = wing_arm(limb);
                    let (elbow, wrist) = ik2(
                        arm_a,
                        arm_b,
                        v3(reach, down.neg(), Fx::ZERO),
                        v3(Fx::ZERO, Fx::ZERO, sd),
                    );
                    (elbow, wrist.sub(elbow))
                }
                Build::Spindle if !pose.wing_walks => {
                    // A spider's leg: out along its fan -- the fore pair
                    // reaching forward, the hind pair back -- the knee
                    // high above the body, the foot far out on the floor.
                    let yaw = match station {
                        Station::Fore => fx(1, 12),
                        Station::MidFore => fx(1, 36),
                        Station::MidHind => fx(-1, 36),
                        Station::Hind => fx(-1, 10),
                    };
                    let (fan_x, fan_z) = (sin_turns(yaw), cos_turns(yaw).mul(sd));
                    let want = limb.mul(fx(7, 10));
                    let out = want
                        .mul(want)
                        .sub(down.mul(down))
                        .sub(reach.mul(reach))
                        .max(Fx::ZERO)
                        .sqrt();
                    let foot = v3(reach.add(fan_x.mul(out)), down.neg(), fan_z.mul(out));
                    let d = vlen(foot).min(limb);
                    let h = half_limb
                        .mul(half_limb)
                        .sub(d.mul(d).mul(fx(1, 4)))
                        .max(Fx::ZERO)
                        .sqrt();
                    let (_, up) = frame(foot);
                    let knee = foot.scale(fx(1, 2)).add(up.scale(h));
                    (knee, foot.sub(knee))
                }
                _ if splay => {
                    // Out to the side, then down.
                    let out = half_limb
                        .mul(half_limb)
                        .sub(reach.mul(reach).mul(fx(1, 4)))
                        .max(Fx::ZERO)
                        .sqrt();
                    (
                        v3(reach.mul(fx(1, 2)), Fx::ZERO, out.mul(sd)),
                        v3(reach.mul(fx(1, 2)), down.neg(), Fx::ZERO),
                    )
                }
                _ => {
                    // Two halves, the joint off the line: elbow back on a
                    // fore limb, knee forward on a hind one.
                    let h = half_limb
                        .mul(half_limb)
                        .sub(d.mul(d).mul(fx(1, 4)))
                        .max(Fx::ZERO)
                        .sqrt();
                    let (ux, uy) = (
                        reach.div(d.max(fx(1, 100))),
                        down.neg().div(d.max(fx(1, 100))),
                    );
                    let (nx, ny) = (uy.neg(), ux);
                    let bend = if fore { h.neg() } else { h };
                    let mid = v3(reach.mul(fx(1, 2)), down.neg().mul(fx(1, 2)), Fx::ZERO);
                    let knee = v3(mid.x.add(nx.mul(bend)), mid.y.add(ny.mul(bend)), Fx::ZERO);
                    (
                        knee,
                        v3(reach.sub(knee.x), down.neg().sub(knee.y), Fx::ZERO),
                    )
                }
            };
            // A walking wing's arm is a wing's bone, not a leg: thin.
            let girth = if pose.wing_walks {
                walking_wing_girth(len, girth)
            } else {
                girth
            };
            let knee = r.bone(names[1], hip, upper, side);
            let foot_bone = r.bone(names[2], knee, lower, side);
            // A walking wing's bones are drawn as the wing they are.
            let (n0, n1) = if pose.wing_walks {
                let w = look.names(side_i);
                (w[0], w[1])
            } else {
                (names[0], names[1])
            };
            let up = r.segment(n0, hip, upper, girth);
            let lo = r.segment(n1, knee, lower, girth);
            // The foot sits on the floor: its box rises from the sole.
            let foot_half = v3(girth.mul(fx(3, 2)), girth.mul(fx(1, 2)), girth);
            r.boxed(
                names[2],
                foot_bone,
                v3(girth.mul(fx(1, 2)), girth.mul(fx(1, 2)), Fx::ZERO),
                foot_half,
            );
            if pose.wing_walks {
                fold_planes(
                    &mut r,
                    look,
                    side_i,
                    knee,
                    foot_bone,
                    lower,
                    upper.add(lower),
                    limb,
                    len,
                    girth,
                    grown,
                    reach_back,
                );
            }
            r.leg(up, lo, hip, knee, fore, side);
        }
    }
    r.body.span = span;
    if d.yaw != Fx::ZERO {
        turn(&mut r.body, d.yaw);
    }
    r.body
}

/// The whole body turned about the point it stands on by `yaw` turns,
/// toward `+z`: every bone's offset and every part's axis, so a turned
/// body is the same body facing elsewhere.
fn turn(body: &mut Body, yaw: Fx) {
    let (c, s) = (cos_turns(yaw), sin_turns(yaw));
    let spin = |v: V3| v3(v.x.mul(c).sub(v.z.mul(s)), v.y, v.x.mul(s).add(v.z.mul(c)));
    for b in body.bones[..body.bone_count].iter_mut() {
        b.rest = spin(b.rest);
    }
    for a in body.axes[..body.part_count].iter_mut() {
        *a = spin(*a);
    }
}

/// A striking arm's two bones at `a` (`-1` cocked, nought folded, one
/// thrust), each from three keys -- cocked, folded, thrust -- as
/// fractions of half the limb, eased between.
fn arm_pose(a: Fx, upper: [Key; 3], lower: [Key; 3]) -> ((Fx, Fx), (Fx, Fx)) {
    let at = |keys: [Key; 3]| {
        let k = |((a, b), (c, d)): Key| (fx(a, b), fx(c, d));
        let (from, to, t) = if a < Fx::ZERO {
            (k(keys[1]), k(keys[0]), a.neg().min(Fx::ONE))
        } else {
            (k(keys[1]), k(keys[2]), a.min(Fx::ONE))
        };
        (
            from.0.add(to.0.sub(from.0).mul(t)),
            from.1.add(to.1.sub(from.1).mul(t)),
        )
    };
    (at(upper), at(lower))
}

type Key = ((i32, i32), (i32, i32));

/// A wing's arm: the humerus and forearm, as shares of the wing. The hand
/// and its primaries are the other half: a wing's span is mostly hand.
pub fn wing_arm(limb: Fx) -> (Fx, Fx) {
    (limb.mul(fx(1, 5)), limb.mul(fx(3, 10)))
}

/// Two bones of lengths `a` and `b` reaching from the origin toward `t`:
/// the joint between them, bent toward `toward`, and where the end lands
/// (at `t`, or as near as the bones reach).
fn ik2(a: Fx, b: Fx, t: V3, toward: V3) -> (V3, V3) {
    // Nearer than the bones can fold to: the end goes out level, keeping
    // its height, rather than along the line -- a wrist on the floor stays
    // on the floor.
    let nearest = a.sub(b).abs().add(fx(1, 50));
    let t = if vlen(t) < nearest {
        let flat = v3(t.x, Fx::ZERO, t.z);
        let dir = if vlen(flat) < fx(1, 1000) {
            v3(Fx::ONE, Fx::ZERO, Fx::ZERO)
        } else {
            unit(flat)
        };
        let out = nearest.mul(nearest).sub(t.y.mul(t.y)).max(Fx::ZERO).sqrt();
        v3(dir.x.mul(out), t.y, dir.z.mul(out))
    } else {
        t
    };
    let u = unit(t);
    let d = vlen(t)
        .min(a.add(b).mul(fx(99, 100)))
        .max(a.sub(b).abs().add(fx(1, 100)));
    let x = a
        .mul(a)
        .sub(b.mul(b))
        .add(d.mul(d))
        .div(d.mul(Fx::from_int(2)));
    let h = a.mul(a).sub(x.mul(x)).max(Fx::ZERO).sqrt();
    let n = unit(toward.sub(u.scale(dot(u, toward))));
    (u.scale(x).add(n.scale(h)), u.scale(d))
}

/// A folded wing's planes: the hand folded from the wrist back to the
/// flank with its primaries along it, and the forearm's secondaries
/// below the forearm -- as large as the wing has grown, so a folded full
/// wing reads as a wing and a stub as a stub. The same whether the wrist
/// is held up or walked on; `wrist_at` is where the wrist is, from the
/// shoulder.
#[allow(clippy::too_many_arguments)]
fn fold_planes(
    r: &mut Rig,
    look: WingLook,
    side_i: usize,
    elbow: usize,
    wrist: usize,
    forearm: V3,
    wrist_at: V3,
    limb: Fx,
    len: Fx,
    girth: Fx,
    grown: Fx,
    reach_back: Fx,
) {
    // The primaries tuck to about the tail's tip and no further: the
    // rest of the hand lies folded under them.
    let hand = limb.mul(fx(1, 2)).min(reach_back.max(limb.mul(fx(1, 5))));
    // Back to the flank behind the shoulder, whatever height the wrist is.
    let fold = unit(v3(hand.mul(fx(-9, 10)), Fx::ZERO, Fx::ZERO).sub(wrist_at)).scale(hand);
    let chords = look.chords();
    let thick = match look {
        WingLook::Furred => len.mul(fx(1, 40)).max(girth.mul(fx(1, 2))),
        _ => len.mul(fx(1, 100)).max(girth.mul(fx(1, 4))),
    };
    let names = look.names(side_i);
    // Folded, the feathers stack: a folded wing shows a band about an
    // eighth of its open chord deep, along the bone.
    let c_hand = len
        .mul(fx(chords[1].0, chords[1].1))
        .mul(grown)
        .mul(fx(1, 8));
    r.along(
        names[2],
        wrist,
        fold.scale(fx(1, 2))
            .add(v3(Fx::ZERO, c_hand.mul(fx(-1, 4)), Fx::ZERO)),
        fold,
        v3(hand.mul(fx(1, 2)).add(thick), c_hand, thick),
    );
    let c_arm = len
        .mul(fx(chords[0].0, chords[0].1))
        .mul(grown)
        .mul(fx(1, 8));
    r.along(
        names[1],
        elbow,
        forearm
            .scale(fx(1, 2))
            .add(v3(Fx::ZERO, c_arm.mul(fx(-1, 4)), Fx::ZERO)),
        forearm,
        v3(vlen(forearm).mul(fx(1, 2)).add(thick), c_arm, thick),
    );
}

/// How thick a wing's bone is when it walks on it: a sixtieth of the body,
/// or a third of what the leg would be, whichever is more.
pub fn walking_wing_girth(len: Fx, girth: Fx) -> Fx {
    len.mul(fx(1, 60)).max(girth.mul(fx(1, 3)))
}

// ---------------------------------------------------------------------------
// Moves
// ---------------------------------------------------------------------------

/// A kind of move a body plan can produce. Each is the shape of one or more
/// of the cast's moves (`archetypes` says which).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum MoveKind {
    // the jaw
    Bite,
    Peck,
    Swallow,
    Tongue,
    Plough,
    // the crown
    Gore,
    Charge,
    // the tail
    TailSweep,
    TailSpear,
    Sting,
    TailTrip,
    // the limbs
    Stomp,
    Rake,
    Scythe,
    Lunge,
    Guard,
    Parry,
    LegStab,
    BackKick,
    Pounce,
    Leap,
    // the wings
    Stoop,
    TalonPass,
    Downwash,
    Buffet,
    FeatherVolley,
    WingFlare,
    // the body
    RearSlam,
    BellyFlop,
    SacSlam,
    RiseBite,
    BreachDive,
    Lash,
    Spit,
    Shake,
    Inflate,
    Ambush,
    Shoulder,
    /// A run at you along a line: what any body that walks has when
    /// nothing else in its kit reaches past a few strides (bestiary §1.3).
    Rush,
    /// Down into the ground: the way under, for a body that burrows.
    Burrow,
    /// Up out of it, under where it was aimed: the floor churns there
    /// through the whole windup, and that is the tell.
    Erupt,
    // the back and hide
    SpikeSpray,
    QuillFling,
    WebShot,
    Screech,
    Spew,
    Wallow,
    Shrug,
    Shed,
}

/// Quick, heavy, or neither: which end of the tempo slider likes it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pace {
    Quick,
    Heavy,
    Even,
}

impl MoveKind {
    /// Startup frames, damage and pace for a middling body (size and mass
    /// a half, tempo a half). The built numbers scale from these.
    const fn base(self) -> (i32, i32, Pace) {
        use MoveKind::*;
        match self {
            Bite => (18, 90, Pace::Quick),
            Peck => (14, 70, Pace::Quick),
            Swallow => (40, 60, Pace::Heavy),
            Tongue => (24, 50, Pace::Even),
            Plough => (50, 160, Pace::Heavy),
            Gore => (22, 120, Pace::Even),
            Charge => (36, 180, Pace::Heavy),
            TailSweep => (30, 130, Pace::Even),
            TailSpear => (20, 110, Pace::Quick),
            Sting => (16, 80, Pace::Quick),
            TailTrip => (14, 50, Pace::Quick),
            Stomp => (28, 150, Pace::Heavy),
            Rake => (12, 70, Pace::Quick),
            Scythe => (16, 120, Pace::Quick),
            Lunge => (22, 110, Pace::Even),
            Guard => (6, 0, Pace::Even),
            Parry => (4, 90, Pace::Quick),
            LegStab => (20, 100, Pace::Even),
            BackKick => (18, 140, Pace::Even),
            Pounce => (24, 100, Pace::Quick),
            Leap => (16, 0, Pace::Quick),
            Stoop => (40, 170, Pace::Heavy),
            TalonPass => (20, 90, Pace::Quick),
            Downwash => (30, 0, Pace::Even),
            Buffet => (26, 80, Pace::Even),
            FeatherVolley => (30, 60, Pace::Even),
            WingFlare => (10, 30, Pace::Quick),
            RearSlam => (45, 220, Pace::Heavy),
            BellyFlop => (50, 240, Pace::Heavy),
            SacSlam => (44, 200, Pace::Heavy),
            RiseBite => (36, 170, Pace::Heavy),
            BreachDive => (48, 200, Pace::Heavy),
            Lash => (26, 110, Pace::Even),
            Spit => (24, 60, Pace::Even),
            Shake => (20, 0, Pace::Even),
            Inflate => (30, 40, Pace::Heavy),
            Ambush => (28, 150, Pace::Even),
            Shoulder => (18, 60, Pace::Even),
            Rush => (32, 110, Pace::Heavy),
            Burrow => (36, 0, Pace::Heavy),
            Erupt => (44, 170, Pace::Heavy),
            SpikeSpray => (34, 60, Pace::Even),
            QuillFling => (20, 50, Pace::Quick),
            WebShot => (32, 20, Pace::Even),
            Screech => (24, 0, Pace::Even),
            Spew => (34, 60, Pace::Heavy),
            Wallow => (60, 0, Pace::Heavy),
            Shrug => (40, 120, Pace::Heavy),
            Shed => (50, 140, Pace::Heavy),
        }
    }

    pub const fn name(self) -> &'static str {
        use MoveKind::*;
        match self {
            Bite => "bite",
            Peck => "peck",
            Swallow => "swallow",
            Tongue => "tongue",
            Plough => "plough",
            Gore => "gore",
            Charge => "charge",
            TailSweep => "tail sweep",
            TailSpear => "tail spear",
            Sting => "sting",
            TailTrip => "tail trip",
            Stomp => "stomp",
            Rake => "rake",
            Scythe => "scythe",
            Lunge => "lunge",
            Guard => "guard",
            Parry => "parry",
            LegStab => "leg stab",
            BackKick => "back kick",
            Pounce => "pounce",
            Leap => "leap",
            Stoop => "stoop",
            TalonPass => "talon pass",
            Downwash => "downwash",
            Buffet => "buffet",
            FeatherVolley => "feather volley",
            WingFlare => "wing flare",
            RearSlam => "rear and slam",
            BellyFlop => "belly flop",
            SacSlam => "sac slam",
            RiseBite => "rise-bite",
            BreachDive => "breach-dive",
            Lash => "lash",
            Spit => "spit",
            Shake => "shake",
            Inflate => "inflate",
            Ambush => "ambush",
            Shoulder => "shoulder",
            Rush => "rush",
            Burrow => "burrow",
            Erupt => "erupt",
            SpikeSpray => "spike spray",
            QuillFling => "quill fling",
            WebShot => "web shot",
            Screech => "screech",
            Spew => "spew",
            Wallow => "wallow",
            Shrug => "shrug",
            Shed => "shed",
        }
    }
}

/// One move, with its numbers worked out for this body.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MoveSpec {
    pub kind: MoveKind,
    pub startup: i32,
    pub active: i32,
    pub recovery: i32,
    pub damage: i32,
    /// How far it reaches from the body, metres.
    pub reach: Fx,
    /// The brain's appetite for it, nought to one.
    pub weight: Fx,
}

/// The most moves a plan produces.
pub const MAX_MOVES: usize = 20;

#[derive(Clone, Copy, Debug)]
pub struct MoveSet {
    pub items: [Option<MoveSpec>; MAX_MOVES],
    pub count: usize,
}

impl MoveSet {
    pub fn iter(&self) -> impl Iterator<Item = &MoveSpec> {
        self.items[..self.count].iter().flatten()
    }

    pub fn has(&self, kind: MoveKind) -> bool {
        self.iter().any(|m| m.kind == kind)
    }

    pub fn get(&self, kind: MoveKind) -> Option<&MoveSpec> {
        self.iter().find(|m| m.kind == kind)
    }

    /// Add a move, if it is not there and there is room.
    pub fn push(&mut self, spec: MoveSpec) {
        if !self.has(spec.kind) && self.count < MAX_MOVES {
            self.items[self.count] = Some(spec);
            self.count += 1;
        }
    }
}

struct Deriver<'a> {
    g: &'a Genome,
    caps: Caps,
    set: MoveSet,
}

impl Deriver<'_> {
    fn add(&mut self, kind: MoveKind, reach: Fx) {
        if self.set.has(kind) || self.set.count >= MAX_MOVES {
            return;
        }
        self.set.items[self.set.count] = Some(spec_for(self.g, kind, reach));
        self.set.count += 1;
    }
}

/// One move's numbers for this body: slower and harder the bigger and
/// heavier it is, quicker for a quick tempo, never faster than a reaction
/// unless where you stand answers it, and always with a window to punish.
pub fn spec_for(g: &Genome, kind: MoveKind, reach: Fx) -> MoveSpec {
    let s = &g.sliders;
    let (startup, damage, pace) = kind.base();
    // Bigger and heavier is slower; a quick tempo trims it.
    let slow = Fx::ONE
        .add(s.mass.sub(fx(1, 2)).mul(fx(3, 5)))
        .add(s.size.sub(fx(1, 2)).mul(fx(1, 2)))
        .mul(fx(13, 10).sub(s.tempo.mul(fx(3, 5))));
    let mut startup = Fx::from_int(startup).mul(slow).to_int().max(1);
    // A move answered by reacting to it is never faster than a
    // reaction (bestiary §1.4); only one answered by where you stand
    // may be, and a small quick body's bite is slowed to it.
    // An answer that takes time to do -- a jump has to rise clear --
    // needs that much more tell.
    let answer = super::strike::answer(kind);
    if damage > 0 && !answer.positional() {
        startup = startup.max(HUMAN_REACTION_FRAMES as i32 + super::strike::lead(answer));
    }
    let active = (startup / 3).max(2);
    // Every move holds still long enough after it to be punished
    // (§1.9): a third of its recovery, never under eight frames.
    let recovery = Fx::from_int(startup).mul(fx(3, 2)).to_int().max(24);
    // Bigger and heavier hits harder.
    let hard = fx(1, 4)
        .add(s.size.mul(s.size).mul(Fx::from_int(2)))
        .mul(fx(3, 5).add(s.mass.mul(fx(4, 5))));
    let damage = Fx::from_int(damage).mul(hard).to_int();
    let mut weight = fx(1, 2).add(s.aggression.mul(fx(1, 2)));
    weight = match pace {
        Pace::Quick => weight.mul(fx(1, 2).add(s.tempo)),
        Pace::Heavy => weight.mul(fx(3, 2).sub(s.tempo)),
        Pace::Even => weight,
    };
    if matches!(kind, MoveKind::Guard | MoveKind::Parry) {
        weight = s.guard;
    }
    MoveSpec {
        kind,
        startup,
        active,
        recovery,
        damage,
        reach,
        weight: super::unit(weight),
    }
}

/// The move set a plan produces.
pub fn moves(g: &Genome) -> MoveSet {
    let a = assess(g);
    let len = length_m(g.sliders.size);
    let mut d = Deriver {
        g,
        caps: a.caps,
        set: MoveSet {
            items: [None; MAX_MOVES],
            count: 0,
        },
    };
    let s = &g.sliders;
    let head_reach = len.mul(fx(1, 4));

    // The jaw.
    match g.minors.jaw {
        Jaw::Grinder => {
            if s.size >= fx(4, 5) {
                d.add(MoveKind::Plough, head_reach);
            }
        }
        Jaw::Teeth | Jaw::Fangs | Jaw::Mandibles => d.add(MoveKind::Bite, head_reach),
        Jaw::Beak => d.add(MoveKind::Peck, head_reach),
        Jaw::Maw => {
            d.add(MoveKind::Bite, head_reach);
            if d.caps.swallows {
                d.add(MoveKind::Swallow, head_reach);
            }
            if g.stance == Stance::Lowslung {
                d.add(MoveKind::Tongue, len);
            }
        }
    }

    // The crown and the charge.
    if g.minors.crown == Crown::Horns {
        d.add(MoveKind::Gore, head_reach);
    }
    if d.caps.charges && (g.minors.crown == Crown::Horns || s.mass >= fx(3, 5)) {
        d.add(MoveKind::Charge, len.mul(Fx::from_int(3)));
    }

    // The tail.
    if g.segments.tail != Tail::None && g.stance != Stance::Limbless {
        let tail_reach = len.mul(fx(2, 5));
        match g.minors.tail_tip {
            TailTip::Club => d.add(MoveKind::TailSweep, tail_reach),
            TailTip::Blade => d.add(MoveKind::TailSpear, tail_reach),
            TailTip::Sting => d.add(MoveKind::Sting, tail_reach),
            TailTip::Plain => {
                if g.stance == Stance::Level && s.size < fx(3, 5) {
                    d.add(MoveKind::TailTrip, tail_reach);
                }
            }
        }
    }

    // The limbs.
    for (station, p) in g.pairs() {
        let reach = p.length.mul(len);
        let fore = matches!(station, Station::Fore | Station::MidFore);
        match (p.role, p.build) {
            (Role::Wing, _) => {
                if d.caps.flies {
                    d.add(MoveKind::Stoop, len.mul(Fx::from_int(4)));
                    d.add(MoveKind::Downwash, reach);
                    d.add(MoveKind::Buffet, reach);
                    if g.minors.hide == Hide::Feathers {
                        d.add(MoveKind::FeatherVolley, len.mul(Fx::from_int(3)));
                    }
                    if g.pairs()
                        .any(|(_, q)| q.role.grounded() && q.tip == Tip::Talon)
                    {
                        d.add(MoveKind::TalonPass, len);
                    }
                } else {
                    d.add(MoveKind::WingFlare, reach);
                }
            }
            (Role::Walk, _) => {}
            (_, Build::Column) => {
                if fore {
                    d.add(MoveKind::Stomp, reach);
                } else {
                    d.add(MoveKind::BackKick, reach);
                }
            }
            (_, Build::Sprung) => d.add(MoveKind::Rake, reach),
            (_, Build::Raptorial) => {
                d.add(MoveKind::Scythe, reach);
                d.add(MoveKind::Lunge, reach.mul(Fx::from_int(3)));
                d.add(MoveKind::Parry, reach);
            }
            (_, Build::Spindle) => d.add(MoveKind::LegStab, reach),
            (_, Build::Stub) => {}
            (_, Build::Wing) => d.add(MoveKind::WingFlare, reach),
            // A digger's claws rake as they dig.
            (_, Build::Spade) => d.add(MoveKind::Rake, reach),
        }
    }
    if d.caps.pounces {
        d.add(MoveKind::Pounce, len.mul(Fx::from_int(3)));
        d.add(MoveKind::Leap, len.mul(Fx::from_int(2)));
    }
    if g.minors.jaw == Jaw::Fangs && d.caps.pounces {
        d.add(MoveKind::Ambush, len.mul(Fx::from_int(2)));
    }
    if g.minors.hide == Hide::Chitin && g.stance == Stance::Upright && !g.has_wings() {
        d.add(MoveKind::WingFlare, len.mul(fx(1, 2)));
    }

    // The body.
    if g.stance == Stance::Level && s.size >= fx(1, 2) && s.mass >= fx(3, 5) {
        d.add(MoveKind::RearSlam, len.mul(fx(1, 2)));
    }
    if g.stance == Stance::Lowslung && s.size >= fx(1, 2) {
        if g.segments.abdomen && g.minors.back == Back::Sacs {
            d.add(MoveKind::SacSlam, len.mul(fx(1, 2)));
        } else {
            d.add(MoveKind::BellyFlop, len.mul(fx(1, 2)));
        }
    }
    if g.stance == Stance::Limbless {
        d.add(MoveKind::RiseBite, len.mul(fx(1, 2)));
        d.add(MoveKind::BreachDive, len.mul(Fx::from_int(2)));
        d.add(MoveKind::Spit, len);
        if g.segments.tail != Tail::None {
            d.add(MoveKind::Lash, len.mul(fx(1, 2)));
        }
    }
    if d.caps.burrows {
        d.add(MoveKind::Burrow, Fx::ZERO);
        d.add(MoveKind::Erupt, len.mul(fx(3, 2)).max(Fx::from_int(4)));
    }
    if d.caps.mountable {
        d.add(MoveKind::Shake, Fx::ZERO);
    }
    // Anything with a body can put its shoulder into you -- if its
    // shoulder is where you are: a giant's passes over your head.
    if g.stance != Stance::Limbless && s.size < fx(1, 2) {
        d.add(MoveKind::Shoulder, len.mul(fx(1, 4)));
    }

    // The back and the hide.
    match g.minors.back {
        Back::Ridge => {
            let sprung = g.pairs().any(|(_, p)| p.build == Build::Sprung);
            if sprung {
                d.add(MoveKind::QuillFling, len.mul(Fx::from_int(3)));
            } else if g.segments.tail != Tail::None {
                d.add(MoveKind::SpikeSpray, len.mul(Fx::from_int(2)));
            }
        }
        Back::Sacs => {
            if g.segments.abdomen {
                d.add(MoveKind::WebShot, len.mul(Fx::from_int(3)));
                d.add(MoveKind::Screech, len.mul(Fx::from_int(2)));
            } else {
                d.add(MoveKind::Inflate, len.mul(fx(1, 2)));
            }
        }
        Back::Burden => d.add(MoveKind::Shrug, len.mul(fx(1, 2))),
        Back::Plain => {}
    }
    if g.minors.hide == Hide::Shell {
        d.add(MoveKind::Shed, len.mul(fx(1, 2)));
    }
    if g.minors.hide == Hide::Slick {
        d.add(MoveKind::Spew, len.mul(Fx::from_int(2)));
        d.add(MoveKind::Wallow, Fx::ZERO);
    }
    if g.minors.jaw == Jaw::Beak && d.caps.flies {
        d.add(MoveKind::Screech, len.mul(Fx::from_int(2)));
    }
    if s.guard >= fx(2, 5) {
        d.add(MoveKind::Guard, Fx::ZERO);
    }
    d.set
}
