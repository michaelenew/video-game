//! Stone as a volume, not a surface.
//!
//! A rock is not a picture on a wall. It is a **solid**, and every face you can
//! see is a cut through it. That is not a philosophical point -- it is the
//! reason surface-based stone looks wrong in ways nobody can name. The pattern
//! repeats, because a tile repeats. It does not turn corners, because the two
//! faces of a corner are two separate pictures. And it has no *history*: real
//! stone records how it formed and what has happened to it since, and those are
//! three-dimensional facts.
//!
//! So this models the volume. Feed it a point in space and it says what the
//! rock is like *there*. A texture is then that volume sampled where an object
//! actually sits -- the intersection of the object with the stone -- which
//! makes every wall in the arena a different piece of rock without a single
//! extra parameter, and makes a pattern run round a corner because it was never
//! on the surface in the first place.
//!
//! This is **solid texturing**, which is old (Peachey and Perlin, both 1985)
//! and was always the right answer for rock. What is here that is not in the
//! classic recipe is that the parameters are *geological* rather than
//! arithmetic: you ask for a coarse-grained igneous rock with two joint sets
//! and heavy weathering, not for four octaves at a frequency of 0.55.
//!
//! ## The three fabrics, as mechanisms
//!
//! Rocks are classified by how they formed, and how they formed is exactly what
//! their texture *is*. Each of these is a different generative process, so each
//! is a different piece of arithmetic rather than a different set of constants.
//!
//! **Igneous** -- frozen from a melt. Crystals nucleate and grow until they run
//! into each other, so the grains **interlock** with no gaps and meet at angular
//! boundaries. Grain size is set by cooling rate: slow and deep gives large
//! crystals, fast and shallow gives a mass too fine to see. Several minerals
//! crystallise at once, which is why granite is speckled rather than plain --
//! it is a population of grains, each a different mineral. A two-stage cooling
//! history leaves large early crystals floating in a fine later groundmass.
//!
//! **Sedimentary** -- settled out of water and buried. Each depositional
//! episode leaves a **bed** with its own grain size and composition, so the
//! rock is layered, and the layers are near-parallel because gravity is. Later
//! movement buckles them.
//!
//! **Metamorphic** -- cooked and squeezed. Differential stress recrystallises
//! platy minerals *aligned perpendicular to the squeeze*, which is **foliation**
//! -- and with enough of it, light and dark minerals separate into alternating
//! bands. The grains are flattened into the foliation plane, which is the same
//! statement as "the minerals are aligned" and is what makes schist glitter
//! along one direction and not others.
//!
//! Two things then happen to all three. **Joints**: rock breaks along roughly
//! planar surfaces, and they come in *sets* -- families of near-parallel planes,
//! two or three sets at angles, cutting the mass into blocks. **Weathering**:
//! water gets into the joints and works outward from them, so alteration is
//! strongest where the rock is most broken.
//!
//! ## Why joints are planes and not cells
//!
//! The obvious way to break rock into blocks is a cellular basis, and it is
//! wrong in a way worth stating. Cells give you *a* partition of space -- blocks
//! of random shape in random orientations. Real jointing gives you **sets**: all
//! the joints in a family are parallel, and the blocks are therefore
//! rectangular-ish and share orientations across the whole mass. That
//! shared orientation is most of what makes a rock face read as rock rather
//! than as crazy paving, and a cellular function cannot produce it at any
//! setting.
//!
//! So joints here are explicit families of planes, displaced by noise so they
//! are not flat, and the fracture field is the distance to the nearest plane of
//! any family.

use crate::color::{Lch, Rgb};
use crate::noise::{gradient_noise, worley};

/// How a rock formed, which is the same question as what its grain looks like.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fabric {
    /// Frozen from a melt: interlocking angular grains, no layering.
    Igneous,
    /// Settled and buried: near-parallel beds, each its own composition.
    Sedimentary,
    /// Cooked and squeezed: aligned grains and segregation banding.
    Metamorphic,
}

/// One mineral in the rock.
#[derive(Clone, Copy, Debug)]
pub struct Mineral {
    pub colour: Lch,
    /// How much of the rock is this, relative to the others. Not normalised --
    /// the sampler does that -- so abundances can be written as the numbers a
    /// description actually gives, like "60% feldspar".
    pub abundance: f32,
    /// 0 is chalk, 1 is quartz. Drives how polished the grain ends up and how
    /// far it stands proud once the softer grains around it have worn away.
    pub hardness: f32,
}

impl Mineral {
    pub const fn new(colour: Lch, abundance: f32, hardness: f32) -> Mineral {
        Mineral {
            colour,
            abundance,
            hardness,
        }
    }
}

/// Layering: bedding in a sedimentary rock, foliation in a metamorphic one.
#[derive(Clone, Copy, Debug)]
pub struct Bedding {
    /// Which way is "up" through the layers. Need not be vertical -- tilted
    /// beds are the normal case in anything that has been through an orogeny,
    /// and a wall cut across tilted bedding is far more interesting than one
    /// cut along it.
    pub normal: [f32; 3],
    /// Metres between layers.
    pub spacing: f32,
    /// How far the layers buckle, in metres. Zero is a diagram; real beds are
    /// never flat.
    pub fold: f32,
    /// Size of the folds, in metres. Large and gentle, or small and crumpled.
    pub fold_scale: f32,
    /// How strongly a layer's composition differs from its neighbours', 0 to 1.
    pub contrast: f32,
}

/// A family of near-parallel fractures.
#[derive(Clone, Copy, Debug)]
pub struct JointSet {
    /// Normal of the planes in this family.
    pub normal: [f32; 3],
    /// Metres between planes.
    pub spacing: f32,
    /// How far each plane wanders from flat, in metres.
    pub roughness: f32,
    /// How wide the open part of a joint is, in metres. This is what shows as
    /// a dark line on a cut face.
    pub width: f32,
}

/// A stone, entire.
#[derive(Clone, Debug)]
pub struct Stone {
    pub fabric: Fabric,
    /// Up to four mineral populations. Four because granite is three and a
    /// weathering product makes four, and nothing has yet wanted five.
    pub minerals: [Mineral; 4],
    pub mineral_count: u8,
    /// Typical grain size in metres. The single most legible property of a
    /// rock and the one a person actually has an opinion about.
    pub grain: f32,
    /// Fraction of the rock made of large early crystals, 0 to 1.
    pub phenocryst: f32,
    /// How big those are, in metres.
    pub phenocryst_grain: f32,
    pub bedding: Bedding,
    pub joints: [JointSet; 3],
    pub joint_count: u8,
    /// How far alteration has got, 0 to 1.
    pub weathering: f32,
    /// What it turns into. Oxidation, mostly, so ochres and rusts.
    pub weathered: Lch,
    pub seed: u32,
}

/// What the rock is like at a point.
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub colour: Rgb,
    /// 0 soft, 1 hard. The renderer turns this into roughness.
    pub hardness: f32,
    /// How far below the nominal surface this point sits, in metres. Joints cut
    /// in; hard grains stand proud.
    pub depth: f32,
    /// Distance to the nearest joint, in metres, capped at the widest spacing.
    pub fracture: f32,
    /// How altered this point is, 0 to 1.
    pub weathered: f32,
}

/// Blend two colours in OkLCh, the short way round the hue circle.
fn mix_lch(a: Lch, b: Lch, t: f32) -> Lch {
    let t = t.clamp(0.0, 1.0);
    let mut dh = b.h - a.h;
    if dh > 0.5 {
        dh -= 1.0;
    } else if dh < -0.5 {
        dh += 1.0;
    }
    Lch::new(
        a.l + (b.l - a.l) * t,
        a.c + (b.c - a.c) * t,
        (a.h + dh * t).rem_euclid(1.0),
    )
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalise(v: [f32; 3]) -> [f32; 3] {
    let l = dot(v, v).sqrt();
    if l > 1e-6 {
        [v[0] / l, v[1] / l, v[2] / l]
    } else {
        [0.0, 1.0, 0.0]
    }
}

/// Summed octaves of gradient noise, in the range -1 to 1.
fn turbulence(p: [f32; 3], seed: u32, octaves: u8) -> f32 {
    let mut f = 1.0;
    let mut amp = 1.0;
    let mut total = 0.0;
    let mut norm = 0.0;
    for _ in 0..octaves.max(1) {
        total += gradient_noise([p[0] * f, p[1] * f, p[2] * f], seed) * amp;
        norm += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    total / norm.max(1e-6)
}

impl Stone {
    /// How many minerals are actually in use.
    fn minerals(&self) -> &[Mineral] {
        &self.minerals[..(self.mineral_count.max(1) as usize).min(4)]
    }

    /// Where a point sits through the layering, in layer-spacings.
    ///
    /// This is the classic marble trick and it is the right one: a periodic
    /// function along an axis, **displaced by turbulence**, so the layers
    /// buckle instead of running dead flat. Perlin's original marble was
    /// `sin(y + turbulence(p))` and nothing has improved on the shape of it.
    ///
    /// Returns the continuous coordinate rather than the layer index, because
    /// the caller wants both: the whole part says which bed, the fraction says
    /// where in it -- which is what graded bedding needs.
    fn bedding_coord(&self, p: [f32; 3]) -> f32 {
        let b = &self.bedding;
        if b.spacing <= 0.0 {
            return 0.0;
        }
        let n = normalise(b.normal);
        let along = dot(p, n);
        let scale = 1.0 / b.fold_scale.max(1e-3);
        let fold = turbulence(
            [p[0] * scale, p[1] * scale, p[2] * scale],
            self.seed ^ 0x5EED,
            3,
        );
        let coord = (along + fold * b.fold) / b.spacing;

        // Bands vary in thickness, and without this they do not. Evenly spaced
        // layers read as a zebra or as wood grain -- something *manufactured* --
        // because nothing in geology deposits or segregates on a metronome. One
        // bed is a thick lazy season and the next is a thin one.
        //
        // Applied to the coordinate rather than to the spacing, so it stretches
        // and compresses the sequence continuously instead of introducing a
        // discontinuity wherever the spacing changes.
        coord + (coord * 0.37).sin() * 0.45 + (coord * 0.11).sin() * 0.9
    }

    /// Which grain a point belongs to, and how far it is from the grain
    /// boundary.
    ///
    /// Anisotropic for a metamorphic rock, and that anisotropy **is** the
    /// foliation: "platy minerals aligned by stress" and "grains flattened into
    /// a plane" are the same sentence. Squashing the cell lookup along the
    /// bedding normal produces exactly that, and it is one line rather than a
    /// separate mechanism.
    fn grain_cell(&self, p: [f32; 3], size: f32) -> (u32, f32) {
        let s = 1.0 / size.max(1e-4);
        let q = if self.fabric == Fabric::Metamorphic {
            // Flatten along the foliation. Three-to-one is a schist; the
            // effect is strong and cheap and it is the whole reason a
            // metamorphic rock reads differently from an igneous one at the
            // same grain size.
            let n = normalise(self.bedding.normal);
            let along = dot(p, n);
            [
                (p[0] - n[0] * along) * s + n[0] * along * s * 3.0,
                (p[1] - n[1] * along) * s + n[1] * along * s * 3.0,
                (p[2] - n[2] * along) * s + n[2] * along * s * 3.0,
            ]
        } else {
            [p[0] * s, p[1] * s, p[2] * s]
        };

        let (f1, f2) = worley(q, self.seed ^ 0xC0FFEE);
        // Which cell, by hashing the lattice point nearest the sample. Cheap
        // and stable: two samples inside one grain agree, which is what makes a
        // grain one colour instead of a gradient.
        let id = crate::noise::cell_id(q, self.seed ^ 0xC0FFEE);
        (id, (f2 - f1).min(1.0))
    }

    /// Pick a mineral for a grain, from a set of weights.
    ///
    /// **Weights, not a bias.** The first version rotated the draw -- it added
    /// an offset to the hash before picking -- which reassigns *which* grain
    /// gets which mineral and leaves the proportions exactly as they were. Every
    /// sedimentary bed came out with the same composition as its neighbours in
    /// a different arrangement, which is invisible, and the beds simply did not
    /// appear. A bed differs from the one above it by being *made of different
    /// stuff*, so the thing that has to vary is the abundances.
    fn mineral_of(&self, id: u32, weights: [f32; 4]) -> Mineral {
        let minerals = self.minerals();
        let total: f32 = minerals
            .iter()
            .zip(weights.iter())
            .map(|(m, w)| (m.abundance * w).max(0.0))
            .sum();
        if total <= 0.0 {
            return minerals[0];
        }
        // The hash is the grain's own identity, so a grain keeps its mineral no
        // matter how many times it is sampled.
        let pick = (id % 1_000_003) as f32 / 1_000_003.0 * total;
        let mut acc = 0.0;
        for (i, m) in minerals.iter().enumerate() {
            acc += (m.abundance * weights[i]).max(0.0);
            if pick <= acc {
                return *m;
            }
        }
        minerals[minerals.len() - 1]
    }

    /// The average colour and hardness of the rock, ignoring which grain is
    /// where.
    ///
    /// What a rock looks like from far enough away that the grains are smaller
    /// than a pixel -- which is most of the time, and the reason this exists.
    /// See [`Stone::at_scale`].
    fn mean(&self, weights: [f32; 4]) -> (Lch, f32) {
        let minerals = self.minerals();
        let mut total = 0.0;
        let (mut l, mut c, mut sx, mut sy, mut hard) = (0.0, 0.0, 0.0f32, 0.0f32, 0.0);
        for (i, m) in minerals.iter().enumerate() {
            let w = (m.abundance * weights[i]).max(0.0);
            total += w;
            l += m.colour.l * w;
            c += m.colour.c * w;
            // Hues average round the circle, not along the number line -- the
            // mean of a hue just under one and one just over zero is red, not
            // cyan.
            let a = m.colour.h * std::f32::consts::TAU;
            sx += a.cos() * w;
            sy += a.sin() * w;
            hard += m.hardness * w;
        }
        if total <= 0.0 {
            return (minerals[0].colour, minerals[0].hardness);
        }
        let h = sy.atan2(sx) / std::f32::consts::TAU;
        (
            Lch::new(l / total, c / total, h.rem_euclid(1.0)),
            hard / total,
        )
    }

    /// Distance to the nearest joint, in metres.
    ///
    /// Each family is a stack of parallel planes, so the distance to the
    /// nearest plane in a family is a sawtooth along that family's normal --
    /// which is one `rem_euclid` rather than any search. Displaced by noise so
    /// the planes are surfaces rather than diagrams, then the minimum over
    /// families.
    fn fracture(&self, p: [f32; 3]) -> f32 {
        let count = (self.joint_count as usize).min(3);
        let mut nearest = f32::MAX;
        for (i, set) in self.joints.iter().take(count).enumerate() {
            if set.spacing <= 0.0 {
                continue;
            }
            let n = normalise(set.normal);
            let wobble = turbulence(
                [p[0] * 0.7, p[1] * 0.7, p[2] * 0.7],
                self.seed ^ (0x1000 + i as u32),
                2,
            ) * set.roughness;
            let along = dot(p, n) + wobble;
            let within = along.rem_euclid(set.spacing);
            nearest = nearest.min(within.min(set.spacing - within));
        }
        if nearest == f32::MAX { 1.0 } else { nearest }
    }

    /// What each mineral's share is at a point.
    ///
    /// This is where the three fabrics stop being the same function with
    /// different constants. Igneous rock has one composition everywhere;
    /// sedimentary rock has a composition *per bed*; metamorphic rock has one
    /// that slides continuously along the foliation, because segregation
    /// banding is a gradient of separation rather than a stack of discrete
    /// layers.
    fn weights_at(&self, bed: f32) -> [f32; 4] {
        let n = (self.mineral_count.max(1) as usize).min(4);
        match self.fabric {
            Fabric::Igneous => [1.0; 4],
            Fabric::Sedimentary => {
                // Each bed is mostly one thing. Which one comes from the bed's
                // own index through the golden ratio, which is the cheapest way
                // to get a sequence that never settles into a pattern -- beds
                // that alternated A B A B would read as stripes on wallpaper
                // rather than as a sequence of depositional episodes.
                let layer = bed.floor();
                let dominant = ((layer * 0.618_034).rem_euclid(1.0) * n as f32) as usize % n;
                let mut w = [1.0f32; 4];
                w[dominant] = 1.0 + self.bedding.contrast * 6.0;
                // The neighbouring mineral gets a little, so a bed is a mixture
                // rather than a single colour.
                w[(dominant + 1) % n] = 1.0 + self.bedding.contrast * 1.2;
                w
            }
            Fabric::Metamorphic => {
                // Two poles, light and dark, separating along the foliation. A
                // sine rather than a square wave because the separation is a
                // gradient: that is what makes gneiss look folded rather than
                // striped.
                let swing = (bed * std::f32::consts::TAU).sin() * self.bedding.contrast;
                let mut w = [1.0f32; 4];
                w[0] = (1.0 + swing * 3.0).max(0.02);
                if n > 1 {
                    w[1] = (1.0 - swing * 3.0).max(0.02);
                }
                w
            }
        }
    }

    /// Everything about the rock at a point, as if sampled infinitely finely.
    ///
    /// Use [`Stone::at_scale`] anywhere the result becomes a pixel.
    pub fn at(&self, p: [f32; 3]) -> Sample {
        self.at_scale(p, 0.0)
    }

    /// The same, told how big a pixel is.
    ///
    /// **Grain finer than the thing sampling it does not come out fine -- it
    /// comes out as noise**, and on a wall texture that is a grey fizz that
    /// crawls when the camera moves. It is the same sampling limit that caught
    /// the cloth weave and the skin pores, arriving in three dimensions.
    ///
    /// The wrong fix is to make the grain coarser, which is a lie about the
    /// rock. The right one is to hand back **what the rock averages to** at
    /// that scale, which is exactly what a distant granite looks like: not
    /// speckled, just grey. So `texel` is the size of one pixel in metres, and
    /// as the grain drops below a few of them the individual crystals fade into
    /// their own mean.
    ///
    /// The large-scale structure -- joints, bedding, banding, weathering --
    /// survives, because it is metres across and always resolvable. That split
    /// is the whole reason a solid model is worth having at low resolution:
    /// the part that has to be unique is the part that is still visible.
    pub fn at_scale(&self, p: [f32; 3], texel: f32) -> Sample {
        let bed = self.bedding_coord(p);
        let weights = self.weights_at(bed);

        // Grain size varies bed to bed in a sedimentary rock, which is most of
        // what distinguishes one bed from another at a glance.
        let grain = match self.fabric {
            Fabric::Sedimentary => {
                let layer = bed.floor();
                let coarse = (layer * 0.754_877).rem_euclid(1.0);
                self.grain * (0.5 + coarse * 1.5)
            }
            _ => self.grain,
        };

        // How much of the individual grain survives at this sampling rate. Full
        // above four texels per grain, gone below one.
        let resolved = if texel <= 0.0 {
            1.0
        } else {
            ((grain / texel - 1.0) / 3.0).clamp(0.0, 1.0)
        };

        let (mean_colour, mean_hardness) = self.mean(weights);
        let (id, edge) = self.grain_cell(p, grain);
        let mut mineral = self.mineral_of(id, weights);

        // Large early crystals in a finer groundmass. A second, sparser cell
        // layer: where its own cell interior is deep enough, the phenocryst
        // wins and the groundmass is what is left between them.
        //
        // Phenocrysts survive a coarse sampling that the groundmass does not,
        // which is correct -- they are the one part of a porphyritic rock you
        // can still see from across a room.
        let pheno_grain = self.phenocryst_grain.max(grain * 2.0);
        let pheno_resolved = if texel <= 0.0 {
            1.0
        } else {
            ((pheno_grain / texel - 1.0) / 3.0).clamp(0.0, 1.0)
        };
        if self.phenocryst > 0.0 && pheno_resolved > 0.0 {
            let (pid, pedge) = self.grain_cell(p, pheno_grain);
            if pedge > 1.0 - self.phenocryst {
                mineral = self.mineral_of(pid ^ 0x9E37, weights);
            }
        }

        // --- fractures --------------------------------------------------------
        let frac = self.fracture(p);
        let widest = self
            .joints
            .iter()
            .take((self.joint_count as usize).min(3))
            .map(|j| j.width)
            .fold(0.0f32, f32::max);
        // **A crack narrower than a pixel must not disappear. It has to get
        // wider and fainter.**
        //
        // This is the other half of band-limiting and it is the half that is
        // easy to get wrong, because the first half suggests the opposite.
        // Grain finer than a texel fades into its own mean, which preserves the
        // *average colour* -- that is right for a field that fills an area.
        // Doing the same to a joint deletes it, because a joint is a line: its
        // mean over a texel is almost entirely rock.
        //
        // At arena scale this was not a subtlety. An arena wall is thirty
        // metres of a two-hundred-and-fifty-six pixel texture, so a texel is
        // twelve centimetres and a one-centimetre joint is a twelfth of one.
        // Every crack in the rock vanished, which is most of what makes stone
        // read as stone.
        //
        // So the joint is drawn at least a texel wide and darkened in
        // proportion to how much of that texel it really occupies. The total
        // light it removes is the same at any resolution -- which is what a
        // correct mipmap of a line does, and what makes a distant wall still
        // show its jointing as faint lines rather than as nothing.
        let true_width = widest.max(1e-4);
        let drawn_width = true_width.max(texel * 1.2);
        let coverage = (true_width / drawn_width).min(1.0);
        let in_joint = if widest > 0.0 {
            (1.0 - (frac / drawn_width).min(1.0)).powf(2.0) * coverage
        } else {
            0.0
        };

        // --- weathering -------------------------------------------------------
        //
        // Water gets in at the joints and works outward, so alteration is a
        // function of how far you are from the nearest break. A slow field on
        // top of it keeps whole areas from altering uniformly, which is what
        // separates weathered rock from rock somebody has painted brown.
        // How far alteration reaches from a joint, in metres. Generous, because
        // this is the part of weathering that is still visible at a distance:
        // the individual crack may be a pixel wide, but the stained margin
        // around it is half a metre and reads as banding across the whole face.
        let reach = 0.55;
        let from_joints = (1.0 - (frac / reach).min(1.0)).powf(1.5);
        let patchy = turbulence([p[0] * 0.5, p[1] * 0.5, p[2] * 0.5], self.seed ^ 0xBEEF, 3);
        let weathered =
            (self.weathering * (0.45 + 0.55 * from_joints) * (0.7 + 0.3 * patchy)).clamp(0.0, 1.0);

        // --- colour -----------------------------------------------------------
        //
        // The grain's own colour, darkened toward its boundary -- grain
        // boundaries catch dirt and scatter light, and without this every rock
        // reads as flat coloured confetti. Both fade into the mean as the grain
        // drops below the sampling rate.
        let boundary = (edge * 4.0).min(1.0);
        let shade = 1.0 - 0.45 * (1.0 - boundary) * resolved;
        let grain_colour = Lch::new(mineral.colour.l * shade, mineral.colour.c, mineral.colour.h);
        let base = mix_lch(mean_colour, grain_colour, resolved);

        let altered = mix_lch(base, self.weathered, weathered);
        // A joint is a gap. Whatever the rock is, the inside of a crack is
        // dark, because very little light gets out of it.
        let colour =
            Lch::new(altered.l * (1.0 - 0.75 * in_joint), altered.c, altered.h).to_linear();

        // --- relief and gloss -------------------------------------------------
        //
        // Joints cut in. Soft grains wear down, so a hard grain stands proud of
        // its neighbours -- which is why a weathered granite feels like gravel
        // and a fresh cut does not.
        let hardness_now = mean_hardness + (mineral.hardness - mean_hardness) * resolved;
        let relief_from_grain = (hardness_now - 0.5) * grain * 0.35 * weathered * resolved;
        let depth = in_joint * widest.max(0.004) * 1.5 - relief_from_grain;

        // Weathering roughens everything, and softer minerals more.
        let hardness = (hardness_now * (1.0 - 0.5 * weathered)).clamp(0.0, 1.0);

        Sample {
            colour,
            hardness,
            depth,
            fracture: frac,
            weathered,
        }
    }
}

// ---------------------------------------------------------------------------
// A small library of rocks
// ---------------------------------------------------------------------------
//
// Each of these is a description of a real rock turned into parameters. That is
// the whole authoring method and it is worth stating plainly: you do not tune
// numbers until it looks right, you write down what the rock *is* -- coarse
// grained, three minerals, no bedding, two joint sets a metre apart -- and the
// numbers follow. Getting the sentence right is the work.

const fn m(l: f32, c: f32, h: f32, abundance: f32, hardness: f32) -> Mineral {
    Mineral::new(Lch::new(l, c, h), abundance, hardness)
}

/// No layering at all, for rocks that have none.
pub const UNBEDDED: Bedding = Bedding {
    normal: [0.0, 1.0, 0.0],
    spacing: 0.0,
    fold: 0.0,
    fold_scale: 1.0,
    contrast: 0.0,
};

const NO_JOINTS: JointSet = JointSet {
    normal: [0.0, 1.0, 0.0],
    spacing: 0.0,
    roughness: 0.0,
    width: 0.0,
};

/// **Granite.** Slow-cooled, deep, coarse. Three minerals in plain sight --
/// pale feldspar, grey quartz, dark mica -- which is why it reads as speckled
/// rather than as a colour, and why a granite wall is legible from across an
/// arena while a sandstone one is not.
///
/// Two joint sets roughly at right angles, which is the ordinary case for a
/// massive rock: it has no bedding to break along, so it breaks along the
/// planes the stress field gave it.
pub fn granite() -> Stone {
    Stone {
        fabric: Fabric::Igneous,
        minerals: [
            m(0.62, 0.020, 0.08, 0.45, 0.65), // feldspar, faintly pink
            m(0.48, 0.004, 0.66, 0.30, 0.95), // quartz, barely cool grey
            m(0.18, 0.012, 0.12, 0.20, 0.35), // biotite, near black
            m(0.72, 0.005, 0.14, 0.05, 0.55), // a pale accessory
        ],
        mineral_count: 4,
        grain: 0.012,
        phenocryst: 0.0,
        phenocryst_grain: 0.0,
        bedding: UNBEDDED,
        // Open joints, not hairline ones. An arena wall is weathered outcrop,
        // not a polished countertop: rock that has stood out in the air has had
        // water in its joints for a long time, and they are gaps of a few
        // centimetres with stained margins rather than the pencil lines you see
        // on a cut slab. Which is also what makes them survive being looked at
        // from across an arena.
        joints: [
            JointSet {
                normal: [1.0, 0.06, 0.2],
                spacing: 1.5,
                roughness: 0.09,
                width: 0.045,
            },
            JointSet {
                normal: [0.1, 1.0, 0.0],
                spacing: 2.2,
                roughness: 0.14,
                width: 0.055,
            },
            NO_JOINTS,
        ],
        joint_count: 2,
        weathering: 0.42,
        weathered: Lch::new(0.42, 0.045, 0.11),
        seed: 0x6A17,
    }
}

/// **Sandstone.** Bedded, and the beds differ -- that is the whole look. Warm,
/// soft, and it weathers into ledges because some beds resist and others do
/// not.
///
/// The bedding is tilted a few degrees on purpose. Level bedding reads as a
/// diagram; a wall cut across tilted beds is immediately a piece of geology.
pub fn sandstone() -> Stone {
    Stone {
        fabric: Fabric::Sedimentary,
        minerals: [
            m(0.66, 0.055, 0.10, 0.50, 0.60), // quartz sand, warm
            m(0.54, 0.070, 0.08, 0.28, 0.45), // iron-stained
            m(0.40, 0.040, 0.12, 0.17, 0.35), // a darker, muddier bed
            m(0.76, 0.030, 0.11, 0.05, 0.50), // a pale bed
        ],
        mineral_count: 4,
        grain: 0.006,
        phenocryst: 0.0,
        phenocryst_grain: 0.0,
        bedding: Bedding {
            normal: [0.10, 1.0, 0.06],
            spacing: 0.22,
            fold: 0.05,
            fold_scale: 3.0,
            contrast: 0.85,
        },
        joints: [
            // Sedimentary joints usually run perpendicular to the bedding and
            // stop at bed boundaries, which is why a sandstone face is a grid
            // rather than a mosaic.
            JointSet {
                normal: [1.0, -0.08, 0.0],
                spacing: 0.9,
                roughness: 0.04,
                width: 0.008,
            },
            JointSet {
                normal: [0.0, -0.06, 1.0],
                spacing: 1.3,
                roughness: 0.05,
                width: 0.007,
            },
            NO_JOINTS,
        ],
        joint_count: 2,
        weathering: 0.55,
        weathered: Lch::new(0.50, 0.085, 0.09),
        seed: 0x5A11,
    }
}

/// **Gneiss.** Squeezed hard enough that its minerals gave up and sorted
/// themselves into bands. Strongly folded, because anything that has been
/// through that much stress is not still flat.
///
/// The grains are flattened into the foliation, which is what makes it read as
/// *striped* rather than as speckled granite -- and it is the same minerals.
pub fn gneiss() -> Stone {
    Stone {
        fabric: Fabric::Metamorphic,
        minerals: [
            m(0.62, 0.012, 0.20, 0.40, 0.70), // pale felsic band
            m(0.26, 0.014, 0.68, 0.35, 0.40), // dark mafic band
            m(0.52, 0.010, 0.10, 0.20, 0.85), // quartz
            m(0.34, 0.030, 0.04, 0.05, 0.55), // garnet, sparse and red
        ],
        mineral_count: 4,
        grain: 0.010,
        phenocryst: 0.06,
        phenocryst_grain: 0.05,
        bedding: Bedding {
            normal: [0.25, 1.0, 0.10],
            spacing: 0.10,
            fold: 0.16,
            fold_scale: 0.9,
            contrast: 0.75,
        },
        joints: [
            JointSet {
                normal: [1.0, 0.0, 0.35],
                spacing: 1.8,
                roughness: 0.12,
                width: 0.009,
            },
            NO_JOINTS,
            NO_JOINTS,
        ],
        joint_count: 1,
        weathering: 0.22,
        weathered: Lch::new(0.40, 0.040, 0.10),
        seed: 0x9E15,
    }
}

/// **Basalt.** Cooled fast at the surface, so the grain is too fine to see and
/// the rock is nearly one colour. Its character is entirely in how it broke:
/// cooling contraction cracks it into columns, which is the tightest joint
/// spacing on this list by a long way.
pub fn basalt() -> Stone {
    Stone {
        fabric: Fabric::Igneous,
        minerals: [
            m(0.20, 0.008, 0.70, 0.70, 0.70),
            m(0.26, 0.010, 0.66, 0.22, 0.60),
            m(0.14, 0.006, 0.72, 0.08, 0.75),
            m(0.30, 0.010, 0.68, 0.02, 0.65),
        ],
        mineral_count: 3,
        grain: 0.0025,
        phenocryst: 0.04,
        phenocryst_grain: 0.02,
        bedding: UNBEDDED,
        joints: [
            JointSet {
                normal: [1.0, 0.0, 0.0],
                spacing: 0.34,
                roughness: 0.06,
                width: 0.012,
            },
            JointSet {
                normal: [0.5, 0.0, 0.87],
                spacing: 0.34,
                roughness: 0.06,
                width: 0.012,
            },
            JointSet {
                normal: [-0.5, 0.0, 0.87],
                spacing: 0.34,
                roughness: 0.06,
                width: 0.012,
            },
        ],
        joint_count: 3,
        weathering: 0.18,
        weathered: Lch::new(0.34, 0.030, 0.14),
        seed: 0xBA5A,
    }
}

/// **Slate.** The low-grade end of metamorphism: fine, dark, and split by a
/// cleavage so perfect it is what the rock is quarried for.
pub fn slate() -> Stone {
    Stone {
        fabric: Fabric::Metamorphic,
        minerals: [
            m(0.28, 0.010, 0.72, 0.65, 0.45),
            m(0.22, 0.014, 0.70, 0.25, 0.40),
            m(0.36, 0.008, 0.66, 0.10, 0.50),
            m(0.30, 0.010, 0.70, 0.0, 0.45),
        ],
        mineral_count: 3,
        grain: 0.003,
        phenocryst: 0.0,
        phenocryst_grain: 0.0,
        bedding: Bedding {
            normal: [0.05, 1.0, 0.02],
            spacing: 0.035,
            fold: 0.02,
            fold_scale: 2.0,
            contrast: 0.35,
        },
        joints: [
            JointSet {
                normal: [0.05, 1.0, 0.02],
                spacing: 0.14,
                roughness: 0.012,
                width: 0.005,
            },
            JointSet {
                normal: [1.0, 0.0, 0.1],
                spacing: 1.1,
                roughness: 0.05,
                width: 0.006,
            },
            NO_JOINTS,
        ],
        joint_count: 2,
        weathering: 0.20,
        weathered: Lch::new(0.32, 0.030, 0.12),
        seed: 0x51A7,
    }
}

/// Every rock in the library, for tests and the preview sheet.
pub fn library() -> [(&'static str, Stone); 5] {
    [
        ("granite", granite()),
        ("sandstone", sandstone()),
        ("gneiss", gneiss()),
        ("basalt", basalt()),
        ("slate", slate()),
    ]
}

impl Stone {
    /// The most colourful this rock ever gets.
    ///
    /// For the palette rule: the arena has to stay nearly colourless so that
    /// anything *with* colour in it reads as a thing that matters. A rock is
    /// allowed to be warm -- sandstone is -- but a warm rock cannot be what an
    /// arena is built out of. See [`crate::palette`].
    pub fn max_chroma(&self) -> f32 {
        self.minerals()
            .iter()
            .map(|m| m.colour.c)
            .chain(std::iter::once(self.weathered.c))
            .fold(0.0, f32::max)
    }
}

// ---------------------------------------------------------------------------
// Baking a surface as its intersection with the volume
// ---------------------------------------------------------------------------

/// Where a texture actually sits in the world.
///
/// **This is the whole idea.** A texture is not a picture applied to a face; it
/// is a record of what the rock is like along the patch of space that face
/// occupies. Give the baker the patch and it cuts the stone there.
///
/// Two consequences fall straight out and neither needs any extra machinery:
/// every face of every object gets a *different* piece of rock, because no two
/// faces occupy the same space; and a pattern runs round a corner correctly,
/// because the two faces meeting at that corner are reading adjacent parts of
/// one solid.
#[derive(Clone, Copy, Debug)]
pub struct Placement {
    /// World position of the texture's top-left corner.
    pub origin: [f32; 3],
    /// World vector from the left edge to the right edge.
    pub across: [f32; 3],
    /// World vector from the top edge to the bottom edge.
    pub down: [f32; 3],
}

impl Placement {
    /// The world point a texel reads from.
    pub fn point(&self, u: f32, v: f32) -> [f32; 3] {
        [
            self.origin[0] + self.across[0] * u + self.down[0] * v,
            self.origin[1] + self.across[1] * u + self.down[1] * v,
            self.origin[2] + self.across[2] * u + self.down[2] * v,
        ]
    }

    /// How many metres one texel covers, at a given resolution.
    ///
    /// Handed to [`Stone::at_scale`] so grain finer than a texel fades into the
    /// rock's mean rather than aliasing into a crawling fizz.
    pub fn texel(&self, size: u32) -> f32 {
        let a = dot(self.across, self.across).sqrt();
        let d = dot(self.down, self.down).sqrt();
        a.max(d) / size.max(1) as f32
    }
}
