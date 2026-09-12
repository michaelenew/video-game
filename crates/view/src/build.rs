//! What a fighter is made of, as a parameter vector.
//!
//! Six boxes with fixed sizes made every class the same person in a different
//! colour. That is a readability problem before it is an aesthetic one: colour
//! is [already spoken for](../../art/src/palette.rs) as the channel that says
//! *which player*, which leaves **silhouette** as the channel that says *which
//! class* -- and a silhouette that is identical across the roster says nothing.
//!
//! So a body is a handful of numbers. Six classes are six vectors, and coop
//! monsters are more of them, which is the same trade the material system
//! makes: the parameters are the asset.
//!
//! ## Why this is worth more than a modelling package
//!
//! Because **silhouette becomes measurable.** Once a body is numbers, two
//! bodies can be rendered flat black in orthographic projection and compared,
//! and "can you tell these two apart at a glance" stops being a judgement
//! somebody has to re-form every time a proportion moves. It becomes a test,
//! the same way the palette's colour-blindness separation did.
//!
//! That is the actual argument for procedural bodies on a team with no artist.
//! Not that they are cheap -- though they are -- but that they can be
//! *checked*.

use crate::pose::{PART_COUNT, PARTS, Part};

/// A fighter's proportions.
///
/// Deliberately small. Eight numbers is a body somebody can reason about;
/// forty is a modelling package with none of the tools, and the whole point of
/// the parameter-space approach is that a person can hold the parameters in
/// their head. See the art doc on decisions being the scarce resource.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Build {
    /// Overall scale. Everything else is proportion, so this is the one knob
    /// that says "bigger" without saying "differently shaped".
    pub scale: f32,
    /// How wide the torso is against its height. Above one is a slab, below is
    /// a reed. The single most legible axis in a silhouette.
    pub breadth: f32,
    /// Shoulder width beyond the torso, in torso widths. What separates a
    /// heavy from a duellist at a hundred metres.
    pub shoulders: f32,
    /// Arm length as a fraction of standing height.
    pub reach: f32,
    /// Leg length as a fraction of standing height. Long legs read as fast,
    /// which is worth being literally true.
    pub stride: f32,
    /// Head size against the torso. Small heads read as large bodies -- the
    /// oldest trick in character design and free here.
    pub head: f32,
    /// How much the limbs thicken. A heavy class is not just wider, it is
    /// thicker everywhere, and forgetting that makes a broad fighter read as a
    /// normal fighter who has been stretched.
    pub limb: f32,
}

impl Build {
    /// The proportions the prototype used for everybody.
    pub const EVEN: Build = Build {
        scale: 1.0,
        breadth: 1.0,
        shoulders: 1.0,
        reach: 1.0,
        stride: 1.0,
        head: 1.0,
        limb: 1.0,
    };

    /// The size of one part, in metres.
    ///
    /// The base figures are the ones the prototype was tuned against, so
    /// `EVEN` reproduces it exactly -- which matters, because the camera, the
    /// poses and the hurtbox radius were all judged against that body.
    pub fn part_size(&self, part: Part) -> [f32; 3] {
        let s = self.scale;
        match part {
            Part::Torso => [
                0.62 * self.breadth * self.shoulders * s,
                0.80 * s,
                0.36 * self.breadth * s,
            ],
            Part::Head => [
                0.34 * self.head * s,
                0.34 * self.head * s,
                0.34 * self.head * s,
            ],
            Part::ArmL | Part::ArmR => [
                0.18 * self.limb * s,
                0.62 * self.reach * s,
                0.18 * self.limb * s,
            ],
            Part::LegL | Part::LegR => [
                0.22 * self.limb * s,
                0.76 * self.stride * s,
                0.22 * self.limb * s,
            ],
        }
    }

    /// How far out from the middle each arm hangs, in metres.
    ///
    /// Scales with the torso rather than being fixed, or a broad fighter's arms
    /// end up buried inside their own chest -- which is the first thing that
    /// goes wrong when proportions become adjustable and the attachment points
    /// do not.
    pub fn shoulder_offset(&self) -> f32 {
        0.42 * self.breadth * self.shoulders * self.scale
    }

    /// Where a part sits when the fighter is standing still, in metres.
    ///
    /// **This is what keeps limbs attached.** The poses are authored in metres
    /// against the even body, so carrying them onto a different one by scaling
    /// their absolute positions pulls arms off shoulders and leaves legs
    /// hanging -- which is exactly what the first attempt did, and it looked
    /// like a rigging bug rather than a maths one.
    ///
    /// With a rest position per build, the renderer can apply the pose as a
    /// *deviation* from rest instead: the joint stays where the body says it
    /// is and the animation still moves it the same distance.
    pub fn rest_position(&self, part: Part) -> [f32; 3] {
        let leg = self.part_size(Part::LegL);
        let torso = self.part_size(Part::Torso);
        let head = self.part_size(Part::Head);
        let arm = self.part_size(Part::ArmL);
        let hip = leg[1];
        let shoulder = hip + torso[1];
        match part {
            Part::Torso => [0.0, hip + torso[1] * 0.5, 0.0],
            Part::Head => [0.0, shoulder + head[1] * 0.5, 0.0],
            Part::ArmL => [-self.shoulder_offset(), shoulder - arm[1] * 0.5, 0.0],
            Part::ArmR => [self.shoulder_offset(), shoulder - arm[1] * 0.5, 0.0],
            Part::LegL => [-leg[0] * 0.8, hip * 0.5, 0.0],
            Part::LegR => [leg[0] * 0.8, hip * 0.5, 0.0],
        }
    }

    /// Standing height, for the camera and for anything that needs to know how
    /// tall a fighter is without measuring one.
    pub fn height(&self) -> f32 {
        (0.76 * self.stride + 0.80 + 0.34 * self.head) * self.scale
    }
}

/// A flat-on silhouette, as a grid of covered cells.
///
/// **This is the point of the whole module.** Two builds render to two grids,
/// and how much they overlap is a number.
///
/// Orthographic and flat-on, because that is the hardest case: perspective and
/// a three-quarter view both *add* cues, so two builds that separate here
/// separate everywhere. Resolution is deliberately coarse -- this is asking
/// whether two shapes read differently at a glance across an arena, not
/// whether they differ.
pub struct Silhouette {
    pub width: usize,
    pub height: usize,
    pub covered: Vec<bool>,
}

/// How wide and tall a silhouette grid is. Around fifty cells is roughly what a
/// fighter occupies on screen at fighting distance.
pub const GRID: usize = 48;

impl Silhouette {
    /// Render a build head-on, in its rest pose.
    ///
    /// Every build is measured in the *same* box rather than its own bounding
    /// box, because a silhouette test that normalised out size would declare a
    /// giant and a child identical -- and size is the most legible difference
    /// there is.
    pub fn of(build: &Build) -> Silhouette {
        // Two and a half metres across and tall covers the biggest body the
        // parameters can make with room to spare.
        const SPAN: f32 = 2.5;
        let mut covered = vec![false; GRID * GRID];

        let mut mark = |centre: [f32; 3], size: [f32; 3]| {
            for gy in 0..GRID {
                for gx in 0..GRID {
                    // Cell centres, in metres, with the origin at the fighter's
                    // feet and x running across.
                    let x = (gx as f32 + 0.5) / GRID as f32 * SPAN - SPAN * 0.5;
                    let y = (1.0 - (gy as f32 + 0.5) / GRID as f32) * SPAN;
                    if (x - centre[0]).abs() <= size[0] * 0.5
                        && (y - centre[1]).abs() <= size[1] * 0.5
                    {
                        covered[gy * GRID + gx] = true;
                    }
                }
            }
        };

        // The rest pose, laid out from the build's own proportions rather than
        // from the pose table -- a silhouette is about the body, and posing it
        // would measure the animation instead.
        let leg = build.part_size(Part::LegL);
        let torso = build.part_size(Part::Torso);
        let head = build.part_size(Part::Head);
        let arm = build.part_size(Part::ArmL);

        let hip = leg[1];
        let shoulder = hip + torso[1];
        mark([-leg[0] * 0.8, hip * 0.5, 0.0], leg);
        mark([leg[0] * 0.8, hip * 0.5, 0.0], leg);
        mark([0.0, hip + torso[1] * 0.5, 0.0], torso);
        mark([0.0, shoulder + head[1] * 0.5, 0.0], head);
        mark(
            [-build.shoulder_offset(), shoulder - arm[1] * 0.5, 0.0],
            arm,
        );
        mark([build.shoulder_offset(), shoulder - arm[1] * 0.5, 0.0], arm);

        Silhouette {
            width: GRID,
            height: GRID,
            covered,
        }
    }

    fn area(&self) -> usize {
        self.covered.iter().filter(|c| **c).count()
    }
}

/// How different two silhouettes are, from 0 (identical) to 1 (no overlap).
///
/// One minus intersection over union, which is the standard way to compare two
/// shapes and has the property that matters here: it punishes *both* a shape
/// that is the same size and in the wrong place and one that is the right shape
/// and the wrong size.
///
/// Rough calibration, from the roster: below about 0.15 two builds are the same
/// body with a tweak, and above 0.3 they read as different kinds of fighter.
pub fn distinctness(a: &Silhouette, b: &Silhouette) -> f32 {
    let mut both = 0usize;
    let mut either = 0usize;
    for i in 0..a.covered.len().min(b.covered.len()) {
        if a.covered[i] && b.covered[i] {
            both += 1;
        }
        if a.covered[i] || b.covered[i] {
            either += 1;
        }
    }
    if either == 0 {
        return 0.0;
    }
    1.0 - both as f32 / either as f32
}

/// Total covered area, for tests that want to talk about size directly.
pub fn area(s: &Silhouette) -> usize {
    s.area()
}

/// Sanity: every part is a real box.
pub fn is_sane(build: &Build) -> bool {
    PARTS.iter().take(PART_COUNT).all(|p| {
        let s = build.part_size(*p);
        s.iter().all(|v| v.is_finite() && *v > 0.02 && *v < 3.0)
    })
}

/// One build per class.
///
/// Derived from what each class *is* in the design docs rather than invented:
/// the Bulwark holds a line, the Reaver is a duellist who is not there when you
/// swing, the Champion swaps between a sword, a hammer and a spear. Reading the
/// kits and turning the sentences into proportions is the whole method.
///
/// The numbers were then **searched**, the same way the palette's lightnesses
/// were, because six shapes that each sound right individually are not
/// necessarily six shapes you can tell apart. Written by hand from the kits,
/// the worst pair -- Champion against Elementalist -- sat at 0.228, which is
/// distinguishable if you are looking for it and not much more. A hill-climb
/// over the whole roster, maximising the *worst* pair inside bounds that keep
/// each class recognisably itself, took that to **0.392**.
///
/// Bounds rather than a free search, because an unconstrained optimiser has no
/// taste -- the same lesson the palette search taught. It picks between
/// plausible bodies; it does not get to decide the Bulwark is willowy.
///
/// `cargo run --release -p view --example shapehunt` re-runs it, and
/// `--example builds` prints where the roster currently stands.
pub const CLASS_BUILDS: [Build; 6] = [
    // Bulwark: the communal defender. Broad, short, thick, planted.
    Build {
        scale: 1.02,
        breadth: 1.30,
        shoulders: 1.12,
        reach: 0.92,
        stride: 0.84,
        head: 0.88,
        limb: 1.30,
    },
    // Champion: the hired duellist. The baseline athlete, because the class
    // that swaps between three weapons should not also be a strange shape.
    Build {
        scale: 1.04,
        breadth: 1.00,
        shoulders: 1.04,
        reach: 0.98,
        stride: 1.02,
        head: 1.04,
        limb: 1.00,
    },
    // Shadow Reaver: long and thin, and the longest reach on the roster --
    // which is literally true of the kit.
    Build {
        scale: 1.04,
        breadth: 0.76,
        shoulders: 0.92,
        reach: 1.22,
        stride: 1.18,
        head: 0.90,
        limb: 0.78,
    },
    // Elementalist: plants structures and stands behind them. Upright, heavy
    // headed, short reach -- her range comes from what she builds.
    Build {
        scale: 0.94,
        breadth: 1.06,
        shoulders: 0.88,
        reach: 0.85,
        stride: 0.90,
        head: 1.22,
        limb: 0.94,
    },
    // Blood mage: pays in health. Gaunt, small, long armed.
    Build {
        scale: 0.86,
        breadth: 0.74,
        shoulders: 0.86,
        reach: 1.18,
        stride: 0.94,
        head: 1.10,
        limb: 0.76,
    },
    // Dual mage: two poles held in balance, which is a wide stance and even
    // proportions carried tall.
    Build {
        scale: 1.12,
        breadth: 0.96,
        shoulders: 1.24,
        reach: 0.96,
        stride: 1.12,
        head: 0.84,
        limb: 1.08,
    },
];

/// The build for a class, by its index in `sim::class::ALL_CLASSES`.
pub fn for_class(index: usize) -> Build {
    CLASS_BUILDS[index % CLASS_BUILDS.len()]
}
