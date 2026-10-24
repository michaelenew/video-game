//! **Contours: the shapes a creature's parts are grown in** (2026-10-23,
//! exploration 0027).
//!
//! A part of a creature is a box on a bone (`sim::species`). Grown as a
//! rounded box and smoothed, every part came out the same kind of lump; the
//! owner: *a ridgeback's legs should be like a rhino's or an elephant's --
//! thick, round, but with obvious muscle underneath and shaped to distribute
//! load; thin creatures show their ribs.* Different parts of a body are
//! different shapes for different reasons, so each is grown by a **kind** of
//! contour, and a part can be between kinds.
//!
//! Every kind is written in the one form every limb, tail and torso can be
//! (the generalized cylinder, Bloomenthal 1985; Infinigen's factored parts,
//! 2023): along the part's long axis, a **radius profile** `r(t)`; round it, a
//! **cross-section** `s(t, θ)`; over both, **detail** (muscle bellies, ribs,
//! a foot's pad). Because every kind is those three functions on the same
//! domain, a contour between two kinds is their weighted sum, which is
//! smooth and never lumpy -- the lesson of Infinigen, that noise belongs in
//! the factored space and never on the surface.
//!
//! What each kind is drawn from:
//!
//! - **Column**: a graviportal limb (the elephant, the rhino; Biewener 1989,
//!   Mallet 2022): thick, nearly straight-sided, the muscle proximal and
//!   showing as two longitudinal bellies that fade toward the foot, and a
//!   flared pad at the ground to spread the load.
//! - **Taper**: a cursorial limb (the horse, the deer): heavy at the top where
//!   the muscle is, thin and tendinous below, since a light distal limb is
//!   cheap to swing.
//! - **Barrel**: a trunk: fullest in the middle, a sag of belly under it, a
//!   ridge of spine over it, and ribs across the flank where the fat is thin
//!   (what body-condition scoring reads: ribs show below a middling condition).
//! - **Whip**: a tail or a tentacle: round, and thinning to a point.
//! - **Keel**: a narrow bird's body: fusiform, deeper than wide, keeled below.
//! - **Wing**: a membrane: flat, with a thickened leading edge.
//! - **Wedge**: a head: narrowing to the muzzle, a brow over the eyes, a
//!   flatter jaw.
//! - **Stalk** (2026-10-24, exploration 0030): an insect's thorax stood on
//!   end: a bulb at the joint where the legs meet, a slender stalk, and a
//!   flare at the far end under what it carries.
//!
//! Nothing here touches what the simulation collides against: a contour is
//! how the box is *drawn*, and the box stays the truth.

use bevy::prelude::*;
use std::f32::consts::PI;

/// The kinds of contour, in the order of a [`Contour`]'s weights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Column,
    Taper,
    Barrel,
    Whip,
    Keel,
    Wing,
    Wedge,
    Stalk,
}

pub const KINDS: usize = 8;

/// How a part is contoured: a point in the simplex of kinds, and two dials
/// shared by every kind. Blend two with [`Contour::between`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contour {
    /// How much of each [`Kind`], in its order. Normalised when used.
    pub kinds: [f32; KINDS],
    /// How much flesh lies over the structure: 0 shows every rib and belly,
    /// 1 hides them all under a smooth layer.
    pub fat: f32,
    /// How pronounced the muscle is where a kind has any: 0 none, 1 strong.
    pub muscle: f32,
    /// Where this part lies along a chain of parts contoured as one -- a
    /// tail's three segments thinning to one point -- as the share of the
    /// chain's length it starts and ends at. `(0, 1)` for a part on its own.
    pub span: (f32, f32),
}

impl Contour {
    pub const fn of(kind: Kind) -> Contour {
        let mut kinds = [0.0; KINDS];
        kinds[kind as usize] = 1.0;
        Contour {
            kinds,
            fat: 0.5,
            muscle: 0.5,
            span: (0.0, 1.0),
        }
    }

    pub const fn span(self, from: f32, to: f32) -> Contour {
        Contour {
            span: (from, to),
            ..self
        }
    }

    pub const fn fat(self, fat: f32) -> Contour {
        Contour { fat, ..self }
    }

    pub const fn muscle(self, muscle: f32) -> Contour {
        Contour { muscle, ..self }
    }

    /// `t` of the way from `self` toward one kind, in a table.
    pub const fn between_const(self, kind: Kind, t: f32) -> Contour {
        let mut kinds = self.kinds;
        let mut k = 0;
        while k < KINDS {
            let target = if k == kind as usize { 1.0 } else { 0.0 };
            kinds[k] = kinds[k] + (target - kinds[k]) * t;
            k += 1;
        }
        Contour { kinds, ..self }
    }

    /// `t` of the way from `self` to `other`.
    pub fn between(self, other: Contour, t: f32) -> Contour {
        let mut kinds = self.kinds;
        for (k, o) in kinds.iter_mut().zip(other.kinds) {
            *k += (o - *k) * t;
        }
        Contour {
            kinds,
            fat: self.fat + (other.fat - self.fat) * t,
            muscle: self.muscle + (other.muscle - self.muscle) * t,
            span: self.span,
        }
    }

    fn weights(&self) -> [f32; KINDS] {
        let sum: f32 = self.kinds.iter().map(|w| w.max(0.0)).sum();
        if sum <= 0.0 {
            let mut w = [0.0; KINDS];
            w[Kind::Barrel as usize] = 1.0;
            return w;
        }
        self.kinds.map(|w| w.max(0.0) / sum)
    }

    /// The surface's radius at `t` along the part (0 at the joint end, 1 at
    /// the far end) and `theta` round it (0 toward the part's `up`, growing
    /// toward its `side`), as a multiple of the box's half-width there: 1 is
    /// the box's own face. `(radius, section)`: the profile and the
    /// cross-section, multiplied; kept apart so the ends can be rounded off
    /// the profile alone.
    pub fn surface(&self, t: f32, theta: f32) -> (f32, f32) {
        // Where along the chain this point is.
        let t = self.span.0 + (self.span.1 - self.span.0) * t;
        let w = self.weights();
        let (mut r, mut s) = (0.0, 0.0);
        for (k, &wk) in w.iter().enumerate() {
            if wk <= 0.0 {
                continue;
            }
            let kind = KIND_LIST[k];
            r += wk * radius(kind, t, self);
            s += wk * section(kind, t, theta, self);
        }
        (r, s)
    }

    /// How round the cross-section is made, 0 keeping the box's own aspect
    /// and 1 a circle: a tail is round whatever its box, a wing never is.
    pub fn roundness(&self) -> f32 {
        let w = self.weights();
        let mut r = 0.0;
        for (k, &wk) in w.iter().enumerate() {
            r += wk
                * match KIND_LIST[k] {
                    Kind::Column => 0.75,
                    Kind::Taper => 0.6,
                    Kind::Barrel => 0.3,
                    Kind::Whip => 1.0,
                    Kind::Keel => 0.2,
                    Kind::Wing => 0.0,
                    Kind::Wedge => 0.35,
                    Kind::Stalk => 0.9,
                };
        }
        r
    }
}

const KIND_LIST: [Kind; KINDS] = [
    Kind::Column,
    Kind::Taper,
    Kind::Barrel,
    Kind::Whip,
    Kind::Keel,
    Kind::Wing,
    Kind::Wedge,
    Kind::Stalk,
];

/// A bump centred at `c`, `w` wide, 1 at its middle, 0 outside.
fn bell(t: f32, c: f32, w: f32) -> f32 {
    let x = ((t - c) / w).clamp(-1.0, 1.0);
    0.5 + 0.5 * (x * PI).cos()
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The radius profile of one kind along its part.
fn radius(kind: Kind, t: f32, c: &Contour) -> f32 {
    let m = c.muscle;
    match kind {
        // Nearly straight-sided; a little fuller where the thigh's mass is,
        // a slight waist at the knee, and the pad flaring at the ground.
        Kind::Column => {
            1.0 + 0.10 * m * bell(t, 0.3, 0.35) - 0.05 * bell(t, 0.7, 0.12)
                + 0.14 * smooth(0.82, 1.0, t)
        }
        // Heavy at the top, thin below: the proximal bulb of muscle, then the
        // cannon.
        Kind::Taper => 1.1 - 0.6 * smooth(0.15, 0.8, t) + 0.18 * m * bell(t, 0.22, 0.28),
        // Fullest in the middle.
        Kind::Barrel => 1.0 + 0.06 * (PI * t).sin(),
        // Thinning to a point.
        Kind::Whip => (1.0 - 0.82 * t).max(0.02) * (1.0 - 0.3 * smooth(0.0, 0.08, 1.0 - t)),
        // Fusiform: narrow at both ends.
        Kind::Keel => 0.25 + 0.85 * (PI * t).sin().powf(0.6),
        // A wing keeps its planform.
        Kind::Wing => 1.0 - 0.1 * t,
        // Narrowing to the muzzle.
        Kind::Wedge => 1.0 - 0.32 * smooth(0.15, 1.0, t),
        // A bulb, a stalk, a flare.
        Kind::Stalk => 0.3 + 0.7 * bell(t, 0.15, 0.32) + 0.3 * bell(t, 0.86, 0.16),
    }
}

/// The cross-section of one kind at `t`, round `theta` (0 up, π/2 to the
/// side, π down), as a multiple of the profile there.
fn section(kind: Kind, t: f32, theta: f32, c: &Contour) -> f32 {
    let (m, lean) = (c.muscle, 1.0 - c.fat);
    let up = theta.cos();
    let side = theta.sin().abs();
    let down = (-theta.cos()).max(0.0);
    match kind {
        // Two longitudinal bellies, in front and behind, with a groove
        // between them, fading toward the foot.
        Kind::Column => {
            let along = bell(t, 0.3, 0.45);
            let front = (theta.cos()).max(0.0).powf(1.5);
            let back = down.powf(1.5);
            let groove = (2.0 * theta).sin().abs().powf(4.0);
            1.0 + m * along * (0.10 * front + 0.08 * back - 0.05 * groove)
        }
        // A rounder top, and below the knee a cannon flatter across than
        // fore-and-aft, as a tendon-wrapped bone is.
        Kind::Taper => {
            let below = smooth(0.5, 0.9, t);
            1.0 + 0.06 * m * bell(t, 0.25, 0.3) * side - 0.1 * below * side
        }
        // A ridge of spine over, a sag of belly under, and ribs across the
        // flank where the fat is thin: hoops a little proud of the hide,
        // between the shoulder and the haunch.
        Kind::Barrel => {
            let spine = 0.035 * up.max(0.0).powf(8.0);
            let belly = 0.07 * down.powf(2.0) * (PI * t).sin();
            let flank = side.powf(2.0) * (1.0 - up.max(0.0).powf(2.0));
            let ribs = lean
                * 0.035
                * (2.0 * PI * RIBS * t).sin().max(0.0).powf(0.7)
                * bell(t, 0.5, 0.42)
                * flank;
            1.0 + spine + belly + ribs
        }
        Kind::Whip => 1.0,
        // Deeper than wide, keeled below.
        Kind::Keel => 1.0 + 0.3 * down.powf(2.0) - 0.08 * side,
        // A flat plate with a thickened leading edge (toward `up`, which for
        // a wing laid flat is forward).
        Kind::Wing => 1.0 + 0.6 * up.max(0.0).powf(4.0),
        // A brow over the eyes, a flatter jaw.
        Kind::Wedge => {
            1.0 + 0.06 * up.max(0.0).powf(3.0) * bell(t, 0.3, 0.25) - 0.05 * down.powf(2.0)
        }
        // Flattened front to back where it flares, a shield's edge.
        Kind::Stalk => 1.0 + 0.25 * side * bell(t, 0.86, 0.16),
    }
}

/// How many ribs cross a barrel.
const RIBS: f32 = 9.0;

/// The signed distance from `p` to a contoured part: the part's box has half
/// extents `half` in its own frame, its long axis is `long` (0, 1 or 2) with
/// the joint at its `root` end (+1 toward the axis' positive end), and `p`
/// is already in the box's frame with the box's middle at the origin. The
/// distance is exact in the plane across the axis and bounded along it,
/// which is what surface nets need.
/// `up` and `fwd` are the world's up and the creature's forward in the box's
/// frame: what the cross-section's `theta` is measured from. For a part
/// whose long axis is up (a leg), forward takes up's place, so its bellies
/// sit front and back.
pub fn distance(
    c: &Contour,
    p: Vec3,
    half: Vec3,
    long: usize,
    root: f32,
    up: Vec3,
    fwd: Vec3,
) -> f32 {
    let (a1, a2) = ((long + 1) % 3, (long + 2) % 3);
    // Along: 0 at the joint end, 1 at the far end.
    let t = (p[long] / half[long] * -root).clamp(-1.0, 1.0) * 0.5 + 0.5;
    // Round: the box's own ellipse, pulled toward a circle by the kind.
    let round = c.roundness();
    let mean = (half[a1] + half[a2]) * 0.5;
    let (h1, h2) = (
        half[a1] + (mean - half[a1]) * round,
        half[a2] + (mean - half[a2]) * round,
    );
    // Up across the part: the world's up projected off the long axis, or,
    // where the part stands up, the creature's forward.
    let mut axis = Vec3::ZERO;
    axis[long] = 1.0;
    let mut u_dir = up - axis * up.dot(axis);
    if u_dir.length_squared() < 0.3 {
        u_dir = fwd - axis * fwd.dot(axis);
    }
    let u_dir = u_dir.normalize_or(Vec3::ZERO);
    let s_dir = axis.cross(u_dir);
    let across = Vec3::new(
        if a1 == 0 || a2 == 0 { p.x } else { 0.0 },
        if a1 == 1 || a2 == 1 { p.y } else { 0.0 },
        if a1 == 2 || a2 == 2 { p.z } else { 0.0 },
    );
    let (u, s) = (across.dot(u_dir), across.dot(s_dir));
    // The box's ellipse along those directions.
    let hu = (u_dir[a1].powi(2) * h1 * h1 + u_dir[a2].powi(2) * h2 * h2)
        .sqrt()
        .max(1e-4);
    let hs = (s_dir[a1].powi(2) * h1 * h1 + s_dir[a2].powi(2) * h2 * h2)
        .sqrt()
        .max(1e-4);
    let theta = s.atan2(u);
    let (r, sec) = c.surface(t, theta);
    let want = r * sec;
    // Where the surface is along this ray, in the box's own ellipse.
    let rho = ((u / hu).powi(2) + (s / hs).powi(2)).sqrt();
    let scale = (hu * hs).sqrt();
    let across = (rho - want) * scale;
    // Along the axis, past the ends: a flat cap, rounded by the profile
    // itself where the kind narrows.
    let over = (p[long].abs() - half[long]).max(0.0);
    let inside_along = (p[long].abs() - half[long]).min(0.0);
    if over > 0.0 {
        (over * over + across.max(0.0).powi(2)).sqrt() + across.min(0.0).max(-over)
    } else {
        across.max(inside_along)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_contour_between_two_kinds_is_between_their_surfaces() {
        let a = Contour::of(Kind::Column);
        let b = Contour::of(Kind::Whip);
        let m = a.between(b, 0.5);
        for &t in &[0.1, 0.5, 0.9] {
            let (ra, _) = a.surface(t, 1.0);
            let (rb, _) = b.surface(t, 1.0);
            let (rm, _) = m.surface(t, 1.0);
            assert!((rm - (ra + rb) * 0.5).abs() < 1e-5);
        }
    }

    #[test]
    fn a_column_flares_at_the_foot_and_a_taper_thins() {
        let col = Contour::of(Kind::Column);
        let tap = Contour::of(Kind::Taper);
        assert!(col.surface(1.0, 0.0).0 > col.surface(0.6, 0.0).0);
        assert!(tap.surface(0.95, 0.0).0 < tap.surface(0.1, 0.0).0 * 0.6);
    }

    #[test]
    fn ribs_show_on_a_lean_barrel_and_not_a_fat_one() {
        let lean = Contour::of(Kind::Barrel).fat(0.0);
        let fat = Contour::of(Kind::Barrel).fat(1.0);
        let spread = |c: &Contour| {
            let v: Vec<f32> = (0..40)
                .map(|i| c.surface(0.3 + 0.4 * i as f32 / 40.0, PI / 2.0).1)
                .collect();
            v.iter().cloned().fold(f32::MIN, f32::max) - v.iter().cloned().fold(f32::MAX, f32::min)
        };
        assert!(
            spread(&lean) > 0.02 && spread(&fat) < 0.01,
            "{} {}",
            spread(&lean),
            spread(&fat)
        );
    }

    #[test]
    fn the_distance_is_signed_the_right_way_round() {
        let c = Contour::of(Kind::Column);
        let half = Vec3::new(0.3, 1.0, 0.3);
        assert!(distance(&c, Vec3::ZERO, half, 1, 1.0, Vec3::Y, Vec3::X) < 0.0);
        assert!(distance(&c, Vec3::new(0.6, 0.0, 0.0), half, 1, 1.0, Vec3::Y, Vec3::X) > 0.0);
        assert!(distance(&c, Vec3::new(0.0, 1.5, 0.0), half, 1, 1.0, Vec3::Y, Vec3::X) > 0.0);
    }

    /// A sheet of every kind's side profile and middle cross-section, to look
    /// at without the game (`target/contours.pgm`): derive, look, tweak.
    #[test]
    fn sheet() {
        let (w, h) = (64usize, 48usize);
        let cols = KINDS;
        let mut img = vec![255u8; w * cols * h * 2];
        for (k, kind) in KIND_LIST.iter().enumerate() {
            let c = Contour::of(*kind).fat(0.1).muscle(1.0);
            let half = Vec3::new(1.0, 0.35, 0.35);
            for j in 0..h {
                for i in 0..w {
                    // Side view: x along, y up.
                    let p = Vec3::new(
                        (i as f32 / w as f32 - 0.5) * 2.6,
                        (0.5 - j as f32 / h as f32) * 1.4,
                        0.0,
                    );
                    if distance(&c, p, half, 0, -1.0, Vec3::Y, Vec3::X) < 0.0 {
                        img[j * w * cols + k * w + i] = 40;
                    }
                    // Cross-section at the middle: z side, y up.
                    let q = Vec3::new(
                        0.0,
                        (0.5 - j as f32 / h as f32) * 1.4,
                        (i as f32 / w as f32 - 0.5) * 1.4,
                    );
                    if distance(&c, q, half, 0, -1.0, Vec3::Y, Vec3::X) < 0.0 {
                        img[(h + j) * w * cols + k * w + i] = 40;
                    }
                }
            }
        }
        // Written as text (P2) through the platform seam, the one place the
        // crate may reach a filesystem; `convert` reads it.
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/contours.pgm");
        let mut out = format!("P2\n{} {}\n255\n", w * cols, h * 2);
        for (i, v) in img.iter().enumerate() {
            out.push_str(&v.to_string());
            out.push(if (i + 1) % (w * cols) == 0 { '\n' } else { ' ' });
        }
        crate::platform::write_to(&path, &out, "the contour sheet");
    }
}
