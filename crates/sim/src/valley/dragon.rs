//! **The Sleeper**: a great beast asleep in the mountains north of the
//! Pinewood, which is one of those mountains (the owner, 2026-10-22: "the
//! dragon itself should essentially be the mountain, and its dragon like
//! features should be just discernible only from certain angles"; and on
//! the first pass, "more of a tarrasque type of look than a European
//! dragon").
//!
//! It is the land, not a creature: a height added to the mountain wherever
//! its body lies, before the land's noise and crags are laid over it, so it
//! wears the same rock, snow and broken crests as the range round it and
//! what is drawn is what a foot would stand on.
//!
//! **Its pose is an animal's asleep**, not a statue's: a heavy, wingless
//! body lying a little over on its right side, so the high line of it is
//! its left flank and not its spine, and nothing about it is symmetrical;
//! the tail curled up round the low side to the front; the head turned back
//! and tucked under the upper forelimb, which lies over it from the
//! shoulder; the forelimbs small for the body, as a tarrasque's are; the
//! upper hind leg drawn up along the flank as a ridge, the lower one under
//! the body and not seen.
//!
//! The body is a table of chains of joints -- each a point on the map, how
//! wide the body is there and how high it stands over the mountain -- and
//! each piece between two joints is a long mound, highest along its crest
//! and easing out to nothing at its edge. Pieces are blended where they
//! meet, as a limb meets a body, rather than one standing out of the other
//! as a wall. A row of low spikes stands along the back and the tail. All of
//! it is level data (`tests/knobs.rs`): judged by looking at it.

use crate::fixed::Fx;

/// Where the Sleeper lies, in metres on the map: the joints below are
/// measured from here, east and north. Its shoulders are here.
const AT: (i32, i32) = (770, 360);

/// One joint: (east, north) of [`AT`], how far the body runs either side of
/// it, and how high over the mountain it stands, all metres.
type Joint = (i32, i32, i32, i32);

/// A chain of joints; whether spikes stand along it; and how far over it
/// lies, as a share of its width: its crest that far to the left of the
/// line its joints draw (looking from the first joint to the next), its
/// right-hand side the long, low one it lies on.
struct Chain {
    joints: &'static [Joint],
    spiked: bool,
    lean: Fx,
}

/// How far over the body lies on its right side.
const LYING: Fx = Fx::ratio(3, 10);

const fn chain(joints: &'static [Joint], spiked: bool, lean: Fx) -> Chain {
    Chain {
        joints,
        spiked,
        lean,
    }
}

/// **The body.** Shoulders to hips; the neck and the head, turned back
/// under the left forelimb; the forelimbs, from the shoulders; the haunch
/// and the drawn-up hind leg; the tail, curled round the right side.
const BODY: &[Chain] = &[
    // The back: heavy, a little bowed, shoulders to hips.
    chain(
        &[
            (0, 30, 60, 60),
            (-8, 95, 68, 68),
            (0, 160, 60, 60),
            (15, 200, 48, 48),
        ],
        true,
        LYING,
    ),
    // The neck, short and thick, turning back to the left.
    chain(
        &[(0, 35, 40, 50), (-20, -5, 32, 38), (-38, -22, 26, 28)],
        false,
        LYING,
    ),
    // The head, turned back along the left flank, under the forelimb: a
    // broad skull narrowing to a heavy muzzle, a low brow over the eye.
    chain(
        &[(-40, -24, 22, 26), (-58, -16, 17, 21), (-74, -4, 12, 15)],
        false,
        Fx::ratio(1, 5),
    ),
    chain(&[(-46, -30, 10, 29)], false, Fx::ZERO),
    // The left forelimb, uppermost: out of the shoulder, the elbow forward,
    // the forearm laid over the head.
    chain(
        &[(-42, 42, 16, 44), (-60, 4, 12, 34), (-58, -26, 9, 31)],
        false,
        Fx::ZERO,
    ),
    // The right forelimb, underneath: only its elbow and paw show.
    chain(
        &[(38, 38, 13, 24), (52, 4, 10, 17), (36, -22, 8, 12)],
        false,
        Fx::ZERO,
    ),
    // The left haunch, and the hind leg drawn up along the flank.
    chain(&[(-50, 190, 42, 62)], false, Fx::ZERO),
    chain(&[(-75, 150, 18, 26), (-68, 105, 16, 22)], false, Fx::ZERO),
    // The tail: thick at the root, round the right side and forward,
    // curling in on itself at the tip.
    chain(
        &[
            (15, 200, 42, 46),
            (60, 245, 34, 36),
            (115, 225, 28, 28),
            (140, 160, 23, 22),
            (130, 95, 18, 17),
            (100, 50, 14, 13),
            (78, 66, 10, 10),
            (86, 92, 7, 8),
        ],
        true,
        LYING,
    ),
];

/// How far apart the spikes stand along the back, in metres.
const SPIKE_EVERY: i32 = 22;

/// How far across a spike's foot is, either side, in metres.
const SPIKE_WIDE: i32 = 9;

/// How tall a spike is, as a share of how high the body stands under it.
const SPIKE_TALL: Fx = Fx::ratio(1, 10);

/// How far apart two pieces' heights may be and still be blended where they
/// meet, in metres: a limb runs into a flank over about this much.
const BLEND: Fx = Fx::from_int(14);

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
            let r = r + SPIKE_WIDE;
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

/// Where `p` is against the piece from `a` to `b` (raw 16.16, on the map's
/// plane): how far along it, 0 to 1; how far to the left of its line, looking
/// from `a` to `b` (negative to the right); and how far past either end.
fn along(p: (i64, i64), a: (i64, i64), b: (i64, i64)) -> (Fx, Fx, Fx) {
    let (vx, vz) = ((b.0 - a.0) as i128, (b.1 - a.1) as i128);
    let (wx, wz) = ((p.0 - a.0) as i128, (p.1 - a.1) as i128);
    let len2 = vx * vx + vz * vz;
    if len2 == 0 {
        let d = crate::math::isqrt_wide((wx * wx + wz * wz) as u128) as i32;
        return (Fx::ZERO, Fx::from_raw(d), Fx::ZERO);
    }
    let len = crate::math::isqrt_wide(len2 as u128) as i128;
    let one = Fx::ONE.raw() as i128;
    let dot = wx * vx + wz * vz;
    let t = dot.clamp(0, len2) * one / len2;
    let side = (vx * wz - vz * wx) / len;
    let on = dot / len;
    let past = (-on).max(on - len).max(0);
    (
        Fx::from_raw(t as i32),
        Fx::from_raw(side as i32),
        Fx::from_raw(past as i32),
    )
}

/// The larger of two heights, eased where they are within [`BLEND`] of
/// each other, so two pieces meet in a fillet and not a crease or a wall.
fn blend(a: Fx, b: Fx) -> Fx {
    let gap = BLEND.sub(a.sub(b).abs());
    if gap.raw() <= 0 {
        return a.max(b);
    }
    let h = gap.div(BLEND);
    a.max(b).add(h.mul(h).mul(BLEND).div(Fx::from_int(4)))
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
    let mut spike = Fx::ZERO;
    for chain in BODY {
        let joints = chain.joints;
        // A chain of one joint is a mound round that joint.
        let pieces = joints.len().max(2) - 1;
        let mut piece_best = Fx::ZERO;
        for k in 0..pieces {
            let (ax, az, ar, ah) = joints[k];
            let (bx, bz, br, bh) = joints[(k + 1).min(joints.len() - 1)];
            let (a, b) = ((m(ax), m(az)), (m(bx), m(bz)));
            let (t, side, past) = along(p, a, b);
            let r = crate::math::lerp(Fx::from_int(ar), Fx::from_int(br), t);
            let h = crate::math::lerp(Fx::from_int(ah), Fx::from_int(bh), t);
            // Across the piece from its crest -- `lean` of its width to the
            // left of its line -- to its edge either side, and round its
            // ends.
            let crest = r.mul(chain.lean);
            let run = if side.raw() >= crest.raw() {
                r.sub(crest)
            } else {
                r.add(crest)
            };
            let across = side.sub(crest).abs().div(run.max(Fx::ONE));
            let beyond = past.div(r.max(Fx::ONE));
            if across.raw() < Fx::ONE.raw() && beyond.raw() < Fx::ONE.raw() {
                let u2 = across.mul(across).add(beyond.mul(beyond));
                if u2.raw() < Fx::ONE.raw() {
                    // Highest along its crest, easing out to nothing at its
                    // edge: a smoothstep of what is left of the width.
                    let q = Fx::ONE.sub(u2);
                    let q = q.mul(q).mul(Fx::from_int(3).sub(q.add(q)));
                    piece_best = piece_best.max(h.mul(q));
                }
            }
            if !chain.spiked {
                continue;
            }
            // The spike nearest along this piece, and how far off it.
            let len = crate::math::wide_len(crate::math::V3::new(
                Fx::from_raw((b.0 - a.0) as i32),
                Fx::ZERO,
                Fx::from_raw((b.1 - a.1) as i32),
            ));
            let every = Fx::from_int(SPIKE_EVERY);
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
            let wide = m(SPIKE_WIDE) as i128;
            let d2 = dx * dx + dz * dz;
            if d2 < wide * wide {
                let d = Fx::from_raw(crate::math::isqrt_wide(d2 as u128) as i32);
                let h = crate::math::lerp(Fx::from_int(ah), Fx::from_int(bh), at);
                let tall = h.mul(SPIKE_TALL);
                let cone = Fx::ONE.sub(d.div(Fx::from_int(SPIKE_WIDE)));
                spike = spike.max(tall.mul(cone));
            }
        }
        // Pieces of one chain are one limb; chains meet as limbs meet.
        body = if body.raw() > 0 && piece_best.raw() > 0 {
            blend(body, piece_best)
        } else {
            body.max(piece_best)
        };
    }
    let shows = rise
        .sub(SHOWS_FROM)
        .div(FULL_FROM.sub(SHOWS_FROM))
        .min(Fx::ONE);
    body.add(spike).mul(shows)
}
