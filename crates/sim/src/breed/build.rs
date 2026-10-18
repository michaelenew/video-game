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

use super::plan::{Caps, assess};
use super::{
    Back, Build, Crown, Genome, HeadKind, Hide, Jaw, Role, Stance, Station, Tail, TailTip, Tip, fx,
};
use crate::beast::{Bone, Leg, MAX_BONES, MAX_PARTS, NO_PARENT, Part, bone, part};
use crate::fixed::{Fx, cos_turns, sin_turns};
use crate::math::V3;

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
    /// The highest point of any part, metres.
    pub fn top(&self) -> Fx {
        let mut top = Fx::ZERO;
        for p in self.parts() {
            let y = self.at(p.shape.bone).y.add(p.shape.max.y);
            top = top.max(y);
        }
        top
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
    fn boxed(&mut self, name: &'static str, bone: usize, at: V3, half: V3) -> usize {
        self.part(name, bone, at.sub(half), at.add(half))
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
    if g.stance == Stance::Lowslung {
        half_w = half_w.mul(Fx::from_int(4));
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
    let rear = if g.segments.abdomen {
        torso.mul(fx(-9, 4))
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
    build(g, q, None)
}

/// A moment of its walk: `phase` is where in the stride cycle, nought to
/// one. Diagonal pairs swing in antiphase whatever the leg count, a foot
/// lifts through the middle of its swing, the head turns a little toward
/// the viewer, the tail sways, wings that fly beat and wings that do not
/// fold back along the body, a limbless body throws a wave down its
/// length. The same genome at phase nought is not the rest pose: a walk
/// is never still.
pub fn body_posed(g: &Genome, q: &super::posture::Posture, phase: Fx) -> Body {
    build(g, q, Some(phase))
}

/// The walk's shape, as shares of a limb's standing length.
const STRIDE: Fx = fx(1, 4);
const LIFT: Fx = fx(1, 5);

fn build(g: &Genome, q: &super::posture::Posture, gait: Option<Fx>) -> Body {
    let mut r = Rig::new();
    // sin and cos of the stride cycle, and a second harmonic for the bob.
    let phase = gait.unwrap_or(Fx::ZERO);
    let flies = super::plan::assess(g).caps.flies;
    let p = proportions(g);
    let (len, torso, half_w, half_h) = (p.len, p.torso, p.half_w, p.half_h);
    let pitch = q.pitch;
    let along = v3(cos_turns(pitch), sin_turns(pitch), Fx::ZERO);
    r.body.length = len;
    r.body.hips = q.root;
    r.body.half_width = half_w;
    r.body.pitch = pitch;

    // Walking, the body bobs twice a stride.
    let bob = if gait.is_some() {
        len.mul(fx(1, 100))
            .mul(sin_turns(phase.mul(Fx::from_int(2))))
    } else {
        Fx::ZERO
    };
    let root = r.bone(
        "root",
        NO_PARENT,
        v3(Fx::ZERO, q.root.add(bob), Fx::ZERO),
        0,
    );

    // The torso and its segments.
    let (fore_bone, fore_at) = if g.stance == Stance::Limbless {
        // A limbless body in motion throws a wave down its length: each
        // segment yaws a little further round than the last, and the
        // front of it lifts.
        let seg = along.scale(torso);
        let mut parent = root;
        r.boxed(
            "segment",
            root,
            V3::ZERO,
            v3(torso.mul(fx(1, 2)), half_w, half_w),
        );
        for (k, name) in ["neck1", "neck2", "neck3", "neck4", "neck5"]
            .into_iter()
            .enumerate()
        {
            let seg_k = if gait.is_some() {
                let yaw = sin_turns(phase.add(Fx::ratio(k as i32, 5))).mul(fx(1, 12));
                let rise = fx(1, 40).mul(Fx::from_int(k as i32 + 1));
                v3(
                    torso.mul(cos_turns(yaw)).mul(cos_turns(rise)),
                    torso.mul(sin_turns(rise)),
                    torso.mul(sin_turns(yaw)),
                )
            } else {
                seg
            };
            parent = r.bone(name, parent, seg_k, 0);
            r.boxed(
                name,
                parent,
                V3::ZERO,
                v3(torso.mul(fx(1, 2)), half_w, half_w),
            );
        }
        (parent, seg)
    } else {
        let third = along.scale(torso.mul(fx(1, 3)));
        let spine = r.bone("spine", root, third, 0);
        let chest = r.bone("chest", spine, third, 0);
        r.boxed(
            "haunch",
            root,
            V3::ZERO,
            v3(torso.mul(fx(1, 6)), half_h, half_w),
        );
        r.boxed(
            "barrel",
            spine,
            V3::ZERO,
            v3(torso.mul(fx(1, 6)), half_h, half_w),
        );
        r.boxed(
            "shoulders",
            chest,
            V3::ZERO,
            v3(torso.mul(fx(1, 6)), half_h, half_w),
        );
        (chest, third)
    };
    let thorax = if g.segments.thorax {
        let t = r.bone("thorax", fore_bone, fore_at, 0);
        r.boxed(
            "thorax",
            t,
            V3::ZERO,
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
        let back = along.scale(torso.neg());
        let a = r.bone("abdomen", root, back, 0);
        r.boxed(
            "abdomen",
            a,
            v3(torso.mul(fx(-1, 2)), half_h.mul(fx(1, 2)), Fx::ZERO),
            v3(
                torso.mul(fx(3, 4)),
                half_h.mul(fx(3, 2)),
                half_w.mul(fx(6, 5)),
            ),
        );
        if g.minors.back == Back::Sacs {
            r.boxed(
                "sacs",
                a,
                v3(torso.mul(fx(-1, 2)), half_h.mul(fx(2, 1)), Fx::ZERO),
                v3(
                    torso.mul(fx(1, 2)),
                    half_h.mul(fx(1, 2)),
                    half_w.mul(fx(1, 2)),
                ),
            );
        }
    }
    if q.drags {
        // The rear rests on the floor and trails: a rump from the haunch's
        // underside down to the ground behind it.
        let bottom = q.root.sub(half_h).max(Fx::ZERO);
        r.part(
            "rump",
            root,
            v3(
                p.rear.sub(q.drag_len),
                q.root.neg(),
                half_w.neg().mul(fx(4, 5)),
            ),
            v3(
                p.rear.add(torso.mul(fx(1, 12))),
                bottom.sub(q.root).add(half_h.mul(fx(1, 2))),
                half_w.mul(fx(4, 5)),
            ),
        );
    }
    if g.minors.back == Back::Ridge {
        r.boxed(
            "ridge",
            root,
            v3(
                torso.mul(fx(1, 3)),
                half_h.add(half_h.mul(fx(1, 3))),
                Fx::ZERO,
            ),
            v3(
                torso.mul(fx(1, 2)),
                half_h.mul(fx(1, 3)),
                half_w.mul(fx(1, 4)),
            ),
        );
    }
    if p.shell {
        r.boxed(
            "shell",
            root,
            v3(torso.mul(fx(1, 3)), half_h.mul(fx(3, 2)), Fx::ZERO),
            v3(torso.mul(fx(3, 4)), half_h, half_w.mul(fx(11, 10))),
        );
    }

    // The head, on a neck raised as the posture says.
    let head_half = p.head_half;
    let neck_len = p.neck_len;
    // Walking, the head turns a little toward the viewer's side and
    // dips with the stride.
    let (neck_yaw, neck_dip) = if gait.is_some() {
        (
            fx(1, 16),
            sin_turns(phase.mul(Fx::from_int(2))).mul(fx(1, 50)),
        )
    } else {
        (Fx::ZERO, Fx::ZERO)
    };
    let neck_pitch = pitch.add(q.neck_rise).add(neck_dip);
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
        r.boxed("head", h, V3::ZERO, v3(head_half, deep, half_w));
        h
    } else {
        let n = r.bone("neck", thorax, neck_dir.scale(neck_len.mul(fx(1, 2))), 0);
        let n2 = r.bone("neck2", n, neck_dir.scale(neck_len.mul(fx(1, 2))), 0);
        let h = r.bone("head", n2, neck_dir.scale(head_half), 0);
        r.boxed(
            "neck",
            n2,
            V3::ZERO,
            v3(
                neck_len.mul(fx(1, 4)),
                head_half.mul(fx(3, 5)),
                head_half.mul(fx(3, 5)),
            ),
        );
        r.boxed(
            "head",
            h,
            V3::ZERO,
            v3(head_half, head_half.mul(fx(4, 5)), head_half.mul(fx(4, 5))),
        );
        h
    };
    match g.minors.crown {
        Crown::Horns => {
            r.boxed(
                "horns",
                head,
                v3(head_half.mul(fx(1, 2)), head_half, Fx::ZERO),
                v3(head_half.mul(fx(1, 2)), head_half.mul(fx(1, 2)), head_half),
            );
        }
        Crown::Crest => {
            r.boxed(
                "crest",
                head,
                v3(Fx::ZERO, head_half, Fx::ZERO),
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
        r.boxed(
            "jaw",
            head,
            v3(head_half.mul(fx(1, 2)), head_half.mul(fx(-3, 5)), Fx::ZERO),
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
            let seg = if gait.is_some() {
                let yaw = sin_turns(phase.sub(Fx::ratio(k as i32 + 1, 5))).mul(fx(1, 12));
                v3(
                    torso.neg().mul(cos_turns(yaw)),
                    Fx::ZERO,
                    torso.neg().mul(sin_turns(yaw)),
                )
            } else {
                seg
            };
            parent = r.bone(name, parent, seg, 0);
            r.boxed(
                name,
                parent,
                V3::ZERO,
                v3(
                    torso.mul(fx(1, 2)),
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
        // Walking, the tail sways: each segment a little further round.
        let seg_at = |k: i32| {
            let yaw = if gait.is_some() {
                sin_turns(phase.add(Fx::ratio(k, 4)))
                    .mul(fx(1, 16))
                    .mul(Fx::from_int(k + 1))
            } else {
                Fx::ZERO
            };
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
        r.boxed("tail", t1, V3::ZERO, v3(tail_len.mul(fx(1, 6)), th, th));
        r.boxed(
            "tail, middle",
            t2,
            V3::ZERO,
            v3(tail_len.mul(fx(1, 6)), th.mul(fx(4, 5)), th.mul(fx(4, 5))),
        );
        let tip_half = match g.minors.tail_tip {
            TailTip::Club => th,
            TailTip::Blade => th.mul(fx(1, 2)),
            TailTip::Sting | TailTip::Plain => th.mul(fx(3, 5)),
        };
        r.boxed(
            "tail tip",
            t3,
            V3::ZERO,
            v3(tail_len.mul(fx(1, 6)), tip_half, tip_half),
        );
    }

    // The limbs, each standing as its station's pose says.
    let mut span = Fx::ZERO;
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
        let hip_at = v3(
            pose.attach.mul(cos_turns(pitch)),
            pose.attach.mul(sin_turns(pitch)).sub(hang),
            Fx::ZERO,
        );
        let hip_y = q.root.add(hip_at.y);
        for (side_i, side) in [-1, 1].into_iter().enumerate() {
            let names = LIMB_NAMES[station.index()][side_i];
            let sd = Fx::from_int(side);
            let z = half_w.mul(sd);
            let hip = r.bone(names[0], root, v3(hip_at.x, hip_at.y, z), side);
            let half_limb = limb.mul(fx(1, 2));
            if pair.role == Role::Wing && !pose.wing_walks {
                // A wing: three segments out to the side, swept back and
                // rising a little, each a broad thin plane whose chord
                // narrows toward the tip.
                let seg = limb.mul(fx(1, 3));
                let sweep = match gait {
                    // Flying, a beat: the wing rises and falls through the
                    // stride. Not flying, folded: swept hard back along
                    // the body, barely out from it.
                    Some(_) if flies => {
                        // Shallow: a box drawn round a steep plane is a wall.
                        let a = sin_turns(phase).mul(fx(1, 20));
                        v3(
                            limb.mul(fx(-1, 10)),
                            seg.mul(sin_turns(a)).add(limb.mul(fx(1, 15))),
                            seg.mul(cos_turns(a)).mul(sd),
                        )
                    }
                    Some(_) => v3(
                        seg.mul(fx(-9, 10)),
                        seg.mul(fx(1, 8)),
                        seg.mul(fx(1, 4)).mul(sd),
                    ),
                    None => v3(limb.mul(fx(-1, 10)), limb.mul(fx(1, 15)), seg.mul(sd)),
                };
                let thick = len.mul(fx(3, 200)).max(girth.mul(fx(1, 4)));
                let arm = r.bone(names[1], hip, sweep, side);
                let hand = r.bone(names[2], arm, sweep, side);
                let chords = [
                    torso.mul(fx(1, 2)),
                    torso.mul(fx(2, 5)),
                    torso.mul(fx(1, 4)),
                ];
                for (k, (bone_i, name)) in [(hip, names[0]), (arm, names[1]), (hand, names[2])]
                    .into_iter()
                    .enumerate()
                {
                    r.boxed(
                        name,
                        bone_i,
                        sweep.scale(fx(1, 2)),
                        // The plane spans its segment: as high as the
                        // segment rises, as far out as it goes.
                        v3(
                            chords[k].mul(fx(1, 2)),
                            thick
                                .max(sweep.y.abs().mul(fx(1, 2)))
                                .min(chords[k].mul(fx(1, 4))),
                            sweep.z.abs().mul(fx(1, 2)).max(thick),
                        ),
                    );
                }
                span = limb.add(half_w).mul(Fx::from_int(2));
                continue;
            }
            if !pose.grounded {
                // A striking limb held up and forward: folded blades.
                let (upper, lower) = match pair.build {
                    Build::Raptorial => (
                        v3(half_limb.mul(fx(1, 2)), half_limb.mul(fx(1, 2)), Fx::ZERO),
                        v3(half_limb, Fx::ZERO, Fx::ZERO),
                    ),
                    _ => (
                        v3(half_limb.mul(fx(1, 2)), half_limb.mul(fx(-1, 2)), Fx::ZERO),
                        v3(half_limb.mul(fx(1, 2)), Fx::ZERO, Fx::ZERO),
                    ),
                };
                let knee = r.bone(names[1], hip, upper, side);
                let foot = r.bone(names[2], knee, lower, side);
                r.boxed(
                    names[0],
                    hip,
                    upper.scale(fx(1, 2)),
                    limb_half(upper, girth),
                );
                r.boxed(
                    names[1],
                    knee,
                    lower.scale(fx(1, 2)),
                    limb_half(lower, girth),
                );
                let tip_half = match pair.tip {
                    Tip::Blade => v3(
                        girth.mul(fx(3, 1)),
                        girth.mul(fx(1, 2)),
                        girth.mul(fx(1, 2)),
                    ),
                    _ => v3(girth.mul(fx(3, 2)), girth, girth),
                };
                r.boxed(names[2], foot, V3::ZERO, tip_half);
                continue;
            }
            // A standing limb: hip to foot is a straight line of length d,
            // with the foot `reach` ahead of the hip on the floor; the
            // joint sits off the midpoint of that line.
            let splay = super::posture::splayed(g.stance, pair.build);
            let standing =
                super::posture::standing_length(g.stance, pair.build, pose.wing_walks, limb);
            // Walking, the foot swings fore and aft of its resting place
            // and lifts through the middle of its swing; diagonal pairs
            // move together, whatever the number of legs.
            let (swing, lift) = match gait {
                Some(_) => {
                    let offset = if (station.index() + side_i) % 2 == 0 {
                        Fx::ZERO
                    } else {
                        fx(1, 2)
                    };
                    let theta = phase.add(offset);
                    (
                        standing.mul(STRIDE).mul(cos_turns(theta)),
                        standing.mul(LIFT).mul(sin_turns(theta).max(Fx::ZERO)),
                    )
                }
                None => (Fx::ZERO, Fx::ZERO),
            };
            let mut down = hip_y.sub(lift);
            let mut reach = pose.reach.add(swing);
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
                    // Two wing segments reach the floor, elbow out.
                    let seg = limb.mul(fx(1, 3));
                    let h = seg
                        .mul(seg)
                        .sub(d.mul(d).mul(fx(1, 4)))
                        .max(Fx::ZERO)
                        .sqrt();
                    (
                        v3(reach.mul(fx(1, 2)), down.neg().mul(fx(1, 2)), h.mul(sd)),
                        v3(
                            reach.mul(fx(1, 2)),
                            down.neg().mul(fx(1, 2)),
                            h.neg().mul(sd),
                        ),
                    )
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
            let knee = r.bone(names[1], hip, upper, side);
            let foot_bone = r.bone(names[2], knee, lower, side);
            let up = r.boxed(
                names[0],
                hip,
                upper.scale(fx(1, 2)),
                limb_half(upper, girth),
            );
            let lo = r.boxed(
                names[1],
                knee,
                lower.scale(fx(1, 2)),
                limb_half(lower, girth),
            );
            // The foot sits on the floor: its box rises from the sole.
            let foot_half = v3(girth.mul(fx(3, 2)), girth.mul(fx(1, 2)), girth);
            r.boxed(
                names[2],
                foot_bone,
                v3(girth.mul(fx(1, 2)), girth.mul(fx(1, 2)), Fx::ZERO),
                foot_half,
            );
            if pose.wing_walks {
                // The hand folds up and back from the wrist, a long thin
                // plane along the body.
                let seg = limb.mul(fx(1, 3));
                let fold = v3(seg.mul(fx(-3, 5)), seg.mul(fx(4, 5)), Fx::ZERO);
                let thick = len.mul(fx(3, 200)).max(girth.mul(fx(1, 4)));
                let hand_name = if side < 0 {
                    "wing.l.hand"
                } else {
                    "wing.r.hand"
                };
                r.boxed(
                    hand_name,
                    foot_bone,
                    fold.scale(fx(1, 2)),
                    v3(seg.mul(fx(3, 10)), seg.mul(fx(2, 5)), thick),
                );
            }
            r.leg(up, lo, hip, knee, fore, side);
        }
    }
    r.body.span = span;
    r.body
}

/// A limb segment's half-size: along its own direction half its length,
/// across it its girth.
fn limb_half(seg: V3, girth: Fx) -> V3 {
    let along = |c: Fx| c.abs().mul(fx(1, 2)).max(girth);
    v3(along(seg.x), along(seg.y), along(seg.z))
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
        let s = &self.g.sliders;
        let (startup, damage, pace) = kind.base();
        // Bigger and heavier is slower; a quick tempo trims it.
        let slow = Fx::ONE
            .add(s.mass.sub(fx(1, 2)).mul(fx(3, 5)))
            .add(s.size.sub(fx(1, 2)).mul(fx(1, 2)))
            .mul(fx(13, 10).sub(s.tempo.mul(fx(3, 5))));
        let startup = Fx::from_int(startup).mul(slow).to_int().max(1);
        let active = (startup / 3).max(2);
        let recovery = Fx::from_int(startup).mul(fx(3, 2)).to_int().max(4);
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
        self.set.items[self.set.count] = Some(MoveSpec {
            kind,
            startup,
            active,
            recovery,
            damage,
            reach,
            weight: super::unit(weight),
        });
        self.set.count += 1;
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
    if d.caps.mountable {
        d.add(MoveKind::Shake, Fx::ZERO);
    }
    // Anything with a body can put its shoulder into you.
    if g.stance != Stance::Limbless {
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
