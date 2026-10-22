//! **The Sleeper**: a dragon asleep in the mountains north of the Pinewood,
//! which is one of those mountains (the owner, 2026-10-22: "the dragon
//! itself should essentially be the mountain, and its dragon like features
//! should be just discernible only from certain angles").
//!
//! It is the land, not a creature: a height added to the mountain wherever
//! its body lies, before the land's noise and crags are laid over it, so it
//! wears the same rock, snow and broken crests as the range round it and
//! what is drawn is what a foot would stand on. From the road it is a
//! massif with a serrated ridge, seen head on; from the High Plane's east
//! edge, looking down, it is a body curled round on itself, wings folded,
//! chin on its forepaws and its tail drawn round to its nose; from the
//! Saddle, a long neck and a skull in profile against the sky.
//!
//! The body is a table of chains of joints -- each a point on the map, how
//! wide the body is there and how high it stands over the mountain -- and
//! each piece between two joints is a long mound, rounded across its top
//! like a back. Where pieces overlap the higher stands, so a leg folded
//! against a flank reads as a leg. Along the neck, the back and the tail a
//! row of plates stands up out of the spine. All of it is level data
//! (`tests/knobs.rs`): judged by looking at it.

use crate::fixed::Fx;

/// Where the Sleeper lies, in metres on the map: the joints below are
/// measured from here, east and north. Its chest is here; its snout points
/// south, at the valley.
const AT: (i32, i32) = (770, 360);

/// One joint: (east, north) of [`AT`], how far the body runs either side of
/// it, and how high over the mountain it stands, all metres.
type Joint = (i32, i32, i32, i32);

/// A chain of joints, and whether plates stand along it.
struct Chain {
    joints: &'static [Joint],
    plated: bool,
}

/// **The body.** The spine first, from the shoulders to the hips; then the
/// neck, laid down and forward to the head between the forepaws; the skull,
/// the brow and the horns swept back along the neck; the legs folded under;
/// the wings folded, their wrists the highest of it; the haunches; and the
/// tail drawn round the east flank to lie by the snout.
const BODY: &[Chain] = &[
    // The back, shoulders to hips.
    Chain {
        joints: &[
            (0, 40, 55, 70),
            (2, 100, 62, 82),
            (8, 160, 58, 76),
            (18, 205, 50, 66),
        ],
        plated: true,
    },
    // The neck, down to where the skull begins.
    Chain {
        joints: &[(0, 40, 34, 66), (-6, 0, 26, 46), (-10, -24, 22, 36)],
        plated: true,
    },
    // The head: skull, muzzle, snout.
    Chain {
        joints: &[(-10, -34, 26, 50), (-12, -75, 19, 36), (-13, -102, 13, 28)],
        plated: false,
    },
    // The brow, over the shut eyes.
    Chain {
        joints: &[(-10, -44, 17, 57)],
        plated: false,
    },
    // The horns, swept back along the neck.
    Chain {
        joints: &[(-22, -40, 10, 62), (-34, 2, 7, 58)],
        plated: false,
    },
    Chain {
        joints: &[(2, -40, 10, 62), (14, 2, 7, 58)],
        plated: false,
    },
    // The forelegs, elbows out, paws either side of the chin.
    Chain {
        joints: &[(-48, 55, 30, 55), (-62, 10, 22, 34), (-40, -55, 18, 24)],
        plated: false,
    },
    Chain {
        joints: &[(48, 55, 30, 55), (60, 5, 22, 34), (30, -60, 18, 24)],
        plated: false,
    },
    // The wings, folded: the wrist up over the shoulder, the trailing edge
    // down along the flank.
    Chain {
        joints: &[(-58, 80, 22, 98), (-70, 180, 28, 50)],
        plated: false,
    },
    Chain {
        joints: &[(62, 80, 22, 98), (70, 175, 28, 50)],
        plated: false,
    },
    // The haunches, and the hind feet tucked forward along the flanks.
    Chain {
        joints: &[(-50, 190, 42, 62)],
        plated: false,
    },
    Chain {
        joints: &[(55, 185, 42, 62)],
        plated: false,
    },
    Chain {
        joints: &[(-75, 150, 18, 26), (-68, 105, 16, 22)],
        plated: false,
    },
    Chain {
        joints: &[(78, 150, 18, 26), (70, 100, 16, 22)],
        plated: false,
    },
    // The tail, round the east flank and back to the snout.
    Chain {
        joints: &[
            (18, 205, 40, 56),
            (60, 265, 32, 44),
            (125, 265, 26, 34),
            (160, 200, 22, 26),
            (165, 120, 18, 20),
            (140, 40, 14, 15),
            (100, -30, 10, 11),
            (70, -70, 7, 8),
        ],
        plated: true,
    },
];

/// How far apart the plates stand along the spine, in metres.
const PLATE_EVERY: i32 = 18;

/// How far across a plate's foot is, either side, in metres.
const PLATE_WIDE: i32 = 10;

/// How tall a plate is, as a share of how high the body stands under it.
const PLATE_TALL: Fx = Fx::ratio(1, 5);

/// **How much of the Sleeper shows over the mountain**, by how high the
/// mountain has risen off its floor there: none of it until the mountain is
/// this high, so no valley or foothill is touched, all of it by
/// [`FULL_FROM`].
const SHOWS_FROM: Fx = Fx::from_int(40);
const FULL_FROM: Fx = Fx::from_int(100);

/// The box the body lies in, from [`AT`], metres: (west, south, east, north).
const fn extent() -> (i32, i32, i32, i32) {
    let mut out = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    let mut c = 0;
    while c < BODY.len() {
        let joints = BODY[c].joints;
        let mut j = 0;
        while j < joints.len() {
            let (x, z, r, _) = joints[j];
            let r = r + PLATE_WIDE;
            if x - r < out.0 {
                out.0 = x - r;
            }
            if z - r < out.1 {
                out.1 = z - r;
            }
            if x + r > out.2 {
                out.2 = x + r;
            }
            if z + r > out.3 {
                out.3 = z + r;
            }
            j += 1;
        }
        c += 1;
    }
    out
}

const EXTENT: (i32, i32, i32, i32) = extent();

fn m(v: i32) -> i64 {
    Fx::from_int(v).raw() as i64
}

/// The nearest point to `p` of the piece from `a` to `b` (raw 16.16, on
/// the map's plane): how far along it, 0 to 1, and how far off it, squared,
/// in raw units.
fn along(p: (i64, i64), a: (i64, i64), b: (i64, i64)) -> (Fx, i128) {
    let (vx, vz) = ((b.0 - a.0) as i128, (b.1 - a.1) as i128);
    let (wx, wz) = ((p.0 - a.0) as i128, (p.1 - a.1) as i128);
    let len2 = vx * vx + vz * vz;
    let one = Fx::ONE.raw() as i128;
    let t = if len2 == 0 {
        0
    } else {
        (wx * vx + wz * vz).clamp(0, len2) * one / len2
    };
    let (dx, dz) = (wx - vx * t / one, wz - vz * t / one);
    (Fx::from_raw(t as i32), dx * dx + dz * dz)
}

/// **How far the Sleeper raises the land** at a map point (x, z), on a
/// mountain that has risen `rise` off its floor there: zero off its body.
pub fn lift(x: Fx, z: Fx, rise: Fx) -> Fx {
    if rise.raw() <= SHOWS_FROM.raw() {
        return Fx::ZERO;
    }
    let p = (x.raw() as i64 - m(AT.0), z.raw() as i64 - m(AT.1));
    let (w, s, e, n) = EXTENT;
    if p.0 < m(w) || p.0 > m(e) || p.1 < m(s) || p.1 > m(n) {
        return Fx::ZERO;
    }
    let one = Fx::ONE.raw() as i128;
    let mut body = Fx::ZERO;
    let mut plate = Fx::ZERO;
    for chain in BODY {
        let joints = chain.joints;
        // A chain of one joint is a mound round that joint.
        let pieces = joints.len().max(2) - 1;
        for k in 0..pieces {
            let (ax, az, ar, ah) = joints[k];
            let (bx, bz, br, bh) = joints[(k + 1).min(joints.len() - 1)];
            let (a, b) = ((m(ax), m(az)), (m(bx), m(bz)));
            let (t, d2) = along(p, a, b);
            let r = crate::math::lerp(Fx::from_int(ar), Fx::from_int(br), t);
            let h = crate::math::lerp(Fx::from_int(ah), Fx::from_int(bh), t);
            let r2 = r.raw() as i128 * r.raw() as i128;
            if d2 < r2 {
                // Rounded across its top, like a back: highest along its
                // line, falling away to nothing at its edge.
                let u2 = Fx::from_raw((d2 * one / r2) as i32);
                body = body.max(h.mul(Fx::ONE.sub(u2)));
            }
            if !chain.plated {
                continue;
            }
            // The plate nearest along this piece, and how far off it.
            let len = crate::math::wide_len(crate::math::V3::new(
                Fx::from_raw((b.0 - a.0) as i32),
                Fx::ZERO,
                Fx::from_raw((b.1 - a.1) as i32),
            ));
            let every = Fx::from_int(PLATE_EVERY);
            let steps = len.mul(t).add(every.div(Fx::from_int(2))).div(every);
            let at = Fx::from_int(steps.raw() >> 16)
                .mul(every)
                .div(len.max(Fx::ONE))
                .min(Fx::ONE);
            let (cx, cz) = (
                a.0 + ((b.0 - a.0) as i128 * at.raw() as i128 / one) as i64,
                a.1 + ((b.1 - a.1) as i128 * at.raw() as i128 / one) as i64,
            );
            let (dx, dz) = ((p.0 - cx) as i128, (p.1 - cz) as i128);
            let wide = m(PLATE_WIDE) as i128;
            let d2 = dx * dx + dz * dz;
            if d2 < wide * wide {
                let d = Fx::from_raw(crate::math::isqrt_wide(d2 as u128) as i32);
                let h = crate::math::lerp(Fx::from_int(ah), Fx::from_int(bh), at);
                let tall = h.mul(PLATE_TALL);
                let cone = Fx::ONE.sub(d.div(Fx::from_int(PLATE_WIDE)));
                plate = plate.max(tall.mul(cone));
            }
        }
    }
    let shows = rise
        .sub(SHOWS_FROM)
        .div(FULL_FROM.sub(SHOWS_FROM))
        .min(Fx::ONE);
    body.add(plate).mul(shows)
}
