//! **A grown body's armour** (2026-10-23; exploration 0029, "what the
//! algorithm has to learn"): the carapace the sculpted Ridgeback wears,
//! worked out from the species table instead of written by hand.
//!
//! The target is a soft skin with stone grown off it, and the stone's crisp
//! grooves, shingled overlaps and chunky rim are most of what reads as a
//! monster rather than a blob. A grown body is a field, so armour is
//! *thickness*: wherever a part is armoured, the skin is pushed out by the
//! carapace's depth there -- full over a scute, sunk in the groove between
//! two, rising toward each scute's back edge so it lies over the next, keeled
//! down its middle, stepping off at its rim.
//!
//! **Where** follows from what the table already says:
//!
//! - a part lying along the body is armoured over its top, further round
//!   where a leg hangs from it (the mantle over the shoulders, the haunch);
//! - a lower leg is plated down its front and outside, not its pad;
//! - a weak point parts the stone round it -- the trench the ridge stands
//!   in, the gap in the collars for the nape -- because a weak point is
//!   exactly where the armour is not.
//!
//! **The scutes** are cellular noise on the part's own coordinates, along it
//! and round it, with little jitter, so they fall in rows the way a
//! tortoise's and an ankylosaur's do, mirrored about the spine.

use bevy::prelude::*;

/// One part's armour: its frame, how far round it reaches, and its scutes.
#[derive(Clone, Copy, Debug)]
pub struct Shell {
    /// The long axis (0, 1 or 2) of the part's box, its half-size, and the
    /// across directions its angle is measured in (the part's frame).
    pub long: usize,
    pub half: Vec3,
    pub u_dir: Vec3,
    pub s_dir: Vec3,
    /// The angle round the part its armour is centred on, and how far
    /// either side it reaches (radians).
    pub centre: f32,
    pub reach: f32,
    /// The share of the part's length it covers, from its joint end.
    pub from: f32,
    pub to: f32,
    pub root: f32,
    /// How thick, how big a scute, and which way along the part is "behind"
    /// (+1 or -1): the edge that lies over the next scute.
    pub thick: f32,
    pub scute: f32,
    pub behind: f32,
    /// Whether the two sides mirror (a part on the spine) or this is one
    /// side's own (a leg).
    pub mirrored: bool,
    pub seed: u32,
}

/// What armour a part has, if any. `axis_world` is its long axis in the
/// world at rest, `up` and `fwd` the world's up and forward in its frame,
/// `outward` the way out from the body's middle in its frame (a leg's
/// outside), `carries_leg` whether a leg hangs from its bone, `lower_leg`
/// whether it is the lower half of one, `head` whether it is the head.
#[allow(clippy::too_many_arguments)]
pub fn shell_of(
    half: Vec3,
    long: usize,
    root: f32,
    up: Vec3,
    fwd: Vec3,
    outward: Vec3,
    axis_world: Vec3,
    carries_leg: bool,
    lower_leg: bool,
    head: bool,
    neck: bool,
    armour: f32,
    scute: f32,
    seed: u32,
) -> Option<Shell> {
    if armour <= 0.0 {
        return None;
    }
    let mut axis = Vec3::ZERO;
    axis[long] = 1.0;
    let vertical = axis_world.y.abs() > 0.7;
    let mut u_dir = up - axis * up.dot(axis);
    if u_dir.length_squared() < 0.3 {
        u_dir = fwd - axis * fwd.dot(axis);
    }
    let u_dir = u_dir.normalize_or(Vec3::Y);
    let s_dir = axis.cross(u_dir);
    let thin = {
        let (a1, a2) = ((long + 1) % 3, (long + 2) % 3);
        half[a1].min(half[a2])
    };
    // Thinner parts wear thinner stone in smaller scutes.
    let k = (thin / 0.8).clamp(0.45, 1.0);
    let (thick, scute) = (armour * k, scute * (thin / 1.0).clamp(0.5, 1.0));
    // "Behind" along the part: toward the tail for a part lying along the
    // body, downward for one that stands (plates shed downward).
    let behind_world = if vertical { -Vec3::Y } else { -Vec3::X };
    let behind = if axis_world.dot(behind_world) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    if vertical {
        if !lower_leg {
            return None;
        }
        // Down the front and the outside of a lower leg, off its pad.
        let out = Vec2::new(outward.dot(u_dir), outward.dot(s_dir));
        let centre = out.y.atan2(out.x) * 0.4;
        return Some(Shell {
            long,
            half,
            u_dir,
            s_dir,
            centre,
            reach: 1.25,
            from: 0.08,
            to: 0.72,
            root,
            thick: thick * 0.55,
            scute: scute * 0.9,
            behind,
            mirrored: false,
            seed,
        });
    }
    let (reach, from, to) = if head {
        (1.15, 0.0, 0.68)
    } else if neck {
        (1.6, 0.0, 1.0)
    } else if carries_leg {
        (1.85, 0.0, 1.0)
    } else {
        (1.3, 0.0, 1.0)
    };
    Some(Shell {
        long,
        half,
        u_dir,
        s_dir,
        centre: 0.0,
        reach,
        from,
        to,
        root,
        thick,
        scute,
        behind,
        mirrored: true,
        seed,
    })
}

/// How much thicker the body is at `q` (in the part's box frame, from its
/// middle) for its armour, in metres; and how near the seam between two
/// scutes it is (metres to the groove), and its scute's own number (0 to 1).
pub fn thickness(sh: &Shell, q: Vec3) -> (f32, f32, f32) {
    let mut axis = Vec3::ZERO;
    axis[sh.long] = 1.0;
    let along = q[sh.long];
    let t = (along / sh.half[sh.long] * -sh.root).clamp(-1.0, 1.0) * 0.5 + 0.5;
    let across = q - axis * along;
    let (u, s) = (across.dot(sh.u_dir), across.dot(sh.s_dir));
    let r = (u * u + s * s).sqrt();
    let theta = s.atan2(u);
    let off = wrap(theta - sh.centre);
    let a = if sh.mirrored { off.abs() } else { off };
    let r_mean = {
        let (a1, a2) = ((sh.long + 1) % 3, (sh.long + 2) % 3);
        (sh.half[a1] + sh.half[a2]) * 0.5
    };
    // How far inside the armour's edge, in metres: round the part, and
    // along it.
    let len = sh.half[sh.long] * 2.0;
    let inside = ((sh.reach - off.abs()) * r.max(r_mean * 0.5))
        .min((t - sh.from) * len)
        .min((sh.to - t) * len);
    if inside <= -0.04 {
        return (0.0, 9.0, 0.0);
    }
    // Scutes in rows: round the body measured in metres at its typical
    // radius, the outer rows a little smaller; longer along than across.
    let mut round = a * r_mean;
    round *= 1.0 + 0.35 * (a - 0.4).clamp(0.0, 2.0);
    let (x, y) = (along * 0.62 / sh.scute, round / sh.scute);
    let (f1, f2, cx, _cy, id) = cellular2(x, y, 0.45, sh.seed);
    let groove = (f2 - f1) * sh.scute * 0.5;
    let own = hash01(id ^ 0x9e37);
    let mut h = sh.thick * (0.85 + 0.3 * own);
    // Domed: each scute a cushion of stone, highest at its middle, so a
    // plate reads by its own shape and not only by the crack round it.
    h += sh.thick * 0.45 * (1.0 - (f1 / 0.62).powi(2)).max(0.0);
    // Shingled: rising toward the scute's back edge.
    let lean = (along - cx * sh.scute / 0.62) * sh.behind;
    h += sh.thick * 0.55 * (lean / 0.45).clamp(-0.45, 1.0);
    // The groove: a channel between scutes, chamfered either side -- wide
    // enough that the mesher, a cell every six centimetres or so, sees a
    // channel rather than a crack it tears into fins (2026-10-24).
    h *= 0.15 + 0.85 * smoothstep(0.025, 0.10, groove);
    // The rim: chunky, eased in over its last hand's width.
    h *= smoothstep(-0.05, 0.13, inside) * (0.75 + 0.25 * smoothstep(0.0, 0.16, inside));
    (h.max(0.0), groove, own)
}

fn wrap(a: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    (a + std::f32::consts::PI).rem_euclid(tau) - std::f32::consts::PI
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn hash(x: i32, y: i32, seed: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(73_856_093)
        ^ (y as u32).wrapping_mul(19_349_663)
        ^ seed.wrapping_mul(2_654_435_761);
    h = (h ^ (h >> 16)).wrapping_mul(0x045d_9f3b);
    h = (h ^ (h >> 16)).wrapping_mul(0x045d_9f3b);
    h ^ (h >> 16)
}

/// A number in [0, 1) for an integer.
pub fn hash01(x: u32) -> f32 {
    (hash(x as i32, 7, 3) & 0x00ff_ffff) as f32 / 16_777_216.0
}

/// Two-dimensional Worley noise: the distances to the nearest and the
/// second-nearest jittered lattice point, where the nearest is, and its id.
pub fn cellular2(x: f32, y: f32, jitter: f32, seed: u32) -> (f32, f32, f32, f32, u32) {
    let (bx, by) = (x.floor() as i32, y.floor() as i32);
    let (mut f1, mut f2) = (9.0f32, 9.0f32);
    let (mut cx, mut cy, mut id) = (0.0, 0.0, 0u32);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (gx, gy) = (bx + dx, by + dy);
            let h = hash(gx, gy, seed);
            let jx = (h & 0xffff) as f32 / 65_536.0 * jitter + (1.0 - jitter) * 0.5;
            let jy = (h >> 16) as f32 / 65_536.0 * jitter + (1.0 - jitter) * 0.5;
            let (px, py) = (gx as f32 + jx, gy as f32 + jy);
            let d = ((px - x).powi(2) + (py - y).powi(2)).sqrt();
            if d < f1 {
                f2 = f1;
                f1 = d;
                cx = px;
                cy = py;
                id = h;
            } else if d < f2 {
                f2 = d;
            }
        }
    }
    (f1, f2, cx, cy, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scutes_have_grooves_between_them() {
        // Along a line, the groove distance falls to near nothing somewhere
        // and rises well clear of it elsewhere.
        let mut lo = f32::MAX;
        let mut hi = 0.0f32;
        for k in 0..400 {
            let (f1, f2, ..) = cellular2(k as f32 * 0.02, 0.3, 0.45, 1);
            lo = lo.min(f2 - f1);
            hi = hi.max(f2 - f1);
        }
        assert!(lo < 0.05 && hi > 0.3, "{lo} {hi}");
    }
}
