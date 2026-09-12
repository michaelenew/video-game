//! A material is a point in a parameter space, not a file.
//!
//! This is the whole paradigm in one struct. There is **one evaluator**; a
//! material is the numbers you hand it. Adding stone after fire costs a row of
//! constants, not a shader, an asset, an import step or a filename.
//!
//! That shape is deliberately the same one the Oven already uses for the 396
//! feel numbers, and for the same reason: uniformity is what makes a live
//! editor, a bake step and a search index each cost an afternoon rather than a
//! project. What is *not* the same is where the numbers live — tuning values
//! are folded into the simulation's checksum so mistuned peers desync loudly,
//! and art must never do that. Two people playing each other have to be able
//! to look different and still agree on the fight. Art sits on the camera's
//! side of that line.
//!
//! ## The shape
//!
//! ```text
//! surface(p) = ramp( shape( field(p + warp * detail(p)) ) )
//! ```
//!
//! A **form** field gives the large structure, a **detail** field gives the
//! fine grain and also displaces the form's lookup, and a **ramp** turns the
//! resulting scalar into colour. Roughness comes from the same scalar through
//! its own two-ended range, so a material that is mottled in colour is mottled
//! in gloss, which is what real surfaces do and what sells them.
//!
//! Two fields rather than one because one is not enough for any real material
//! and three has not yet been needed for any. Stone is strata plus cracks,
//! cloth is weave plus slub, armour is plate plus wear, skin is mottle plus
//! pores. Every one of them is two scales of the same story.
//!
//! ## Domain warping is worth more than octaves
//!
//! Evaluating a field at a position that has itself been displaced by another
//! field turns straight features into wandering ones. It is the difference
//! between noise and something that looks like it was *formed*: flame rolls,
//! stone strata buckle, wood grain flows around a knot. Two noise evaluations
//! buy more than four extra octaves ever do, because octaves add detail and
//! warping adds *structure*.

use crate::color::{Ramp, Rgb};
use crate::noise::{gradient_noise, worley};

/// The shape of a scalar field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldKind {
    /// Summed octaves of gradient noise. Smooth hills. The default for
    /// anything organic.
    Fbm,
    /// The same, folded at zero so peaks become creases. Rock, bark, anything
    /// eroded.
    Ridged,
    /// Distance to the nearest scattered point. Cells, pebbles, scales.
    Cells,
    /// Distance between the nearest two, which is zero on the boundary between
    /// cells. Cracks, mortar, plate seams.
    Seams,
    /// A sine along the axes. The thing that makes cloth read as woven.
    Stripe,
    /// No variation. For the materials that genuinely have none.
    Flat,
}

/// One scalar field. Output is normalised to 0..1.
#[derive(Clone, Copy, Debug)]
pub struct Field {
    pub kind: FieldKind,
    /// Cycles per metre, **per axis**. This is the parameter that separates
    /// stone from marble: strata are a high frequency in Y against a low one
    /// in X and Z, a weave is high in X and Z against nothing in Y. One scalar
    /// frequency can only ever produce isotropic mush.
    pub frequency: [f32; 3],
    pub octaves: u8,
    /// Frequency multiplier between octaves. Around 2 is standard; drifting off
    /// it stops successive octaves lining up on the lattice, which kills the
    /// faint grid that shows up in large flat areas.
    pub lacunarity: f32,
    /// Amplitude multiplier between octaves. Below 0.5 reads as smooth, above
    /// as rough.
    pub gain: f32,
    pub seed: u32,
}

impl Field {
    pub const fn new(kind: FieldKind, frequency: [f32; 3], seed: u32) -> Field {
        Field {
            kind,
            frequency,
            octaves: 4,
            lacunarity: 2.03,
            gain: 0.5,
            seed,
        }
    }

    pub const fn octaves(mut self, n: u8) -> Field {
        self.octaves = n;
        self
    }

    pub const fn gain(mut self, g: f32) -> Field {
        self.gain = g;
        self
    }

    pub const FLAT: Field = Field {
        kind: FieldKind::Flat,
        frequency: [0.0; 3],
        octaves: 1,
        lacunarity: 2.0,
        gain: 0.5,
        seed: 0,
    };

    /// The finest thing this field produces, in cycles per metre.
    ///
    /// The top octave, along whichever axis runs fastest. Used to check that a
    /// material is actually *resolvable* at the size it will be baked: a
    /// feature finer than about four texels does not come out fine, it comes
    /// out as **moiré** — a coarse interference pattern that is not in the
    /// material at all. Sampling theory's oldest result, and it is the reason
    /// the first cloth looked like watered silk instead of canvas.
    pub fn finest(&self) -> f32 {
        if self.kind == FieldKind::Flat {
            return 0.0;
        }
        let top = self.frequency[0]
            .max(self.frequency[1])
            .max(self.frequency[2]);
        top * self.lacunarity.powi(self.octaves.max(1) as i32 - 1)
    }

    /// The field at a point in metres. Returns 0..1.
    pub fn at(&self, p: [f32; 3]) -> f32 {
        if self.kind == FieldKind::Flat {
            return 0.5;
        }
        let mut freq = self.frequency;
        let mut amp = 1.0f32;
        let mut total = 0.0f32;
        let mut norm = 0.0f32;

        for _ in 0..self.octaves.max(1) {
            let q = [p[0] * freq[0], p[1] * freq[1], p[2] * freq[2]];
            let v = match self.kind {
                FieldKind::Fbm => gradient_noise(q, self.seed) * 0.5 + 0.5,
                FieldKind::Ridged => 1.0 - gradient_noise(q, self.seed).abs() * 2.0,
                FieldKind::Cells => worley(q, self.seed).0.min(1.0),
                FieldKind::Seams => {
                    let (f1, f2) = worley(q, self.seed);
                    (f2 - f1).min(1.0)
                }
                FieldKind::Stripe => {
                    use std::f32::consts::TAU;
                    // Multiplied, not summed: a sum gives a plaid of light and
                    // dark squares, a product gives crossing threads, which is
                    // what a weave is.
                    //
                    // Axes with no frequency are **skipped**, not multiplied
                    // in. A zero frequency means "no threads run this way",
                    // and folding `sin(0)` into the product instead means "no
                    // threads at all": it zeroes the whole field and leaves a
                    // flat half. That is not a hypothetical -- cloth was
                    // written with no frequency in Y, sliced across X and Y,
                    // and the only thing left on screen was the domain warp
                    // wobbling a constant. It looked like corduroy and nothing
                    // in the parameters said why.
                    let mut v = 1.0;
                    let mut threads = 0;
                    for axis in 0..3 {
                        if freq[axis] != 0.0 {
                            v *= (q[axis] * TAU).sin();
                            threads += 1;
                        }
                    }
                    if threads == 0 { 0.5 } else { v * 0.5 + 0.5 }
                }
                FieldKind::Flat => 0.5,
            };
            total += v * amp;
            norm += amp;
            amp *= self.gain;
            freq = [
                freq[0] * self.lacunarity,
                freq[1] * self.lacunarity,
                freq[2] * self.lacunarity,
            ];
        }
        (total / norm.max(1e-6)).clamp(0.0, 1.0)
    }
}

/// Whether the surface is all there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Transparency {
    /// Solid. Everything the arena is built out of.
    Opaque,
    /// The field decides how much of the surface exists: below `floor` there
    /// is nothing, and it comes in over the range above it.
    ///
    /// This is what separates a flame from a lampshade. A fire drawn on a
    /// solid cylinder is a bright orange *object* with a hard silhouette and a
    /// visible top edge, however good the emission texture on it is -- the eye
    /// reads the outline before it reads anything else. Eating the edges away
    /// with the same field that makes the flame gives it a ragged, moving
    /// boundary, which is most of what fire *is*.
    ///
    /// Also the right answer for the Reaver's shadow, for the opposite reason:
    /// a shadow with a crisp edge is a solid object.
    Fades { floor: f32 },
}

/// How light leaves the surface.
#[derive(Clone, Copy, Debug)]
pub enum Emission {
    /// It does not. Ordinary matter.
    None,
    /// A fixed colour times a strength, taken from the ramp. Arcane light,
    /// glowing runes.
    Glow { strength: f32 },
    /// The colour of something at this temperature, in kelvin, interpolated
    /// across the field. Fire and anything else that is hot -- see
    /// `color::blackbody` for why this is a physical question rather than an
    /// aesthetic one.
    Hot { cool: f32, hot: f32, strength: f32 },
}

/// A material, entire.
#[derive(Clone, Copy, Debug)]
pub struct Surface {
    pub form: Field,
    pub detail: Field,
    /// How far the detail field displaces the form's lookup, in metres.
    pub warp: f32,
    /// How much the detail field shows through in the final scalar, 0..1.
    pub grain: f32,
    /// How far the scalar is stretched away from its middle before the ramp
    /// reads it. 1.0 leaves it alone.
    ///
    /// This exists because of a measurement, not a preference. **Summed
    /// octaves of noise cluster around the middle** — it is the central limit
    /// theorem doing what it always does, and the more octaves you add the
    /// narrower the result gets. The floor material was the case that found
    /// it: four octaves of gradient noise spanned 0.40 to 0.64 over three
    /// metres, so three quarters of its colour ramp were unreachable and the
    /// arena floor came out a flat grey no matter what the ramp said.
    ///
    /// The alternative was to normalise each field against its own measured
    /// range, which is a table of magic numbers that goes stale the moment a
    /// frequency changes. One number that means "use more of the ramp" is
    /// cheaper and says what it does.
    pub contrast: f32,
    pub ramp: Ramp,
    /// Roughness where the field is 0 and where it is 1. Ordering is free:
    /// `(0.2, 0.9)` is polished in the hollows, `(0.9, 0.2)` is polished on
    /// the peaks, which is what wear looks like.
    pub roughness: (f32, f32),
    pub metallic: f32,
    pub emission: Emission,
    pub transparency: Transparency,
    /// Height relief in metres, for the normal map. This is the parameter that
    /// decides whether a surface reads as textured or as painted.
    pub relief: f32,
}

impl Surface {
    /// The finest feature anywhere in this material, in cycles per metre.
    pub fn finest(&self) -> f32 {
        let detail = if self.grain > 0.0 {
            self.detail.finest()
        } else {
            0.0
        };
        self.form.finest().max(detail)
    }
}

/// What the evaluator returns at a point.
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub albedo: Rgb,
    pub roughness: f32,
    pub metallic: f32,
    pub emissive: Rgb,
    /// How much of the surface is here, 0..1.
    pub alpha: f32,
    /// The raw scalar, 0..1. Kept because the normal map is its gradient and
    /// recomputing colour to get it would be three times the work.
    pub height: f32,
}

impl Surface {
    /// The combined scalar at a point, 0..1. Colour, roughness and relief are
    /// all functions of this one number, which is what keeps a material
    /// coherent instead of looking like three unrelated textures stacked.
    pub fn scalar(&self, p: [f32; 3]) -> f32 {
        let warped = if self.warp > 0.0 {
            // A dedicated, cheap, **low**-frequency displacement rather than a
            // second look at the detail field, and both halves of that matter.
            //
            // *Low*, because warping by a high-frequency field is jitter, not
            // flow: it shakes the form apart instead of bending it. Half the
            // form's own base frequency is the scale at which features wander
            // rather than rattle.
            //
            // *Cheap*, because this runs three times per texel. Re-evaluating
            // the whole detail field for it -- which the first version did --
            // made the warp cost as much as the rest of the material put
            // together, and for the floor, whose detail is a cellular field
            // searching twenty-seven neighbours per octave, it was most of a
            // second per bake on its own.
            let f = self.form.frequency;
            let w = self.warp;
            let q = [p[0] * f[0] * 0.5, p[1] * f[1] * 0.5, p[2] * f[2] * 0.5];
            [
                p[0] + w * gradient_noise([q[0] + 11.3, q[1], q[2]], self.form.seed ^ 0xA1),
                p[1] + w * gradient_noise([q[0], q[1] + 7.7, q[2]], self.form.seed ^ 0xB2),
                p[2] + w * gradient_noise([q[0], q[1], q[2] + 3.1], self.form.seed ^ 0xC3),
            ]
        } else {
            p
        };
        let f = self.form.at(warped);
        let t = if self.grain > 0.0 {
            f * (1.0 - self.grain) + self.detail.at(p) * self.grain
        } else {
            f
        };
        (0.5 + (t - 0.5) * self.contrast).clamp(0.0, 1.0)
    }

    /// Everything the surface looks like at a point.
    pub fn at(&self, p: [f32; 3]) -> Sample {
        self.shade(self.scalar(p))
    }

    /// The same, from an already-computed scalar.
    ///
    /// Split out because the baker computes the scalar its own way -- a
    /// tileable bake blends four wrapped samples -- and then needs the colour
    /// that belongs to *that* number rather than to a fresh evaluation. Two
    /// evaluations of the same point would agree; a tiled one would not, and
    /// the height map and the colour map would quietly disagree along the seam
    /// that tiling exists to remove.
    pub fn shade(&self, t: f32) -> Sample {
        let t = t.clamp(0.0, 1.0);
        let albedo = self.ramp.at(t);
        let emissive = match self.emission {
            Emission::None => [0.0; 3],
            Emission::Glow { strength } => [
                albedo[0] * strength,
                albedo[1] * strength,
                albedo[2] * strength,
            ],
            Emission::Hot {
                cool,
                hot,
                strength,
            } => {
                let c = crate::color::blackbody(cool + (hot - cool) * t);
                // Cubed, because a flame's brightness falls off far faster than
                // its colour changes. A linear falloff gives a flat orange slab;
                // the cube is what makes a core and a fading edge.
                let i = strength * t * t * t;
                [c[0] * i, c[1] * i, c[2] * i]
            }
        };
        let alpha = match self.transparency {
            Transparency::Opaque => 1.0,
            Transparency::Fades { floor } => {
                // **A thing you can only see because it glows is exactly as
                // present as it is bright.**
                //
                // Fading on the raw field instead gets this backwards, and the
                // pillar showed it: where the flame was dim the surface was
                // still more than half opaque, so the bottom of it rendered as
                // a dark grey drum with flames on top. Near-black albedo is
                // *correct* for fire -- it is emitted light, not a lit surface
                // -- which is exactly why the dim parts have to be absent
                // rather than dark.
                //
                // So the fade reads whatever actually drives the brightness.
                // `Hot` cubes its field, so its alpha does too; `Glow` tracks
                // its ramp, which is near enough linear; and something that
                // does not emit at all falls back to the field itself, which
                // is right for the Reaver's shadow -- an absence is not dim,
                // it is *there*, and it is the arena behind it that is dark.
                let driver = match self.emission {
                    Emission::Hot { .. } => t * t * t,
                    _ => t,
                };
                let a = ((driver - floor) / (1.0 - floor).max(1e-3)).clamp(0.0, 1.0);
                // Smoothstepped rather than linear, so the boundary reads as a
                // soft edge rather than as a band where the alpha ramp starts.
                a * a * (3.0 - 2.0 * a)
            }
        };
        Sample {
            albedo,
            alpha,
            roughness: self.roughness.0 + (self.roughness.1 - self.roughness.0) * t,
            metallic: self.metallic,
            emissive,
            height: t,
        }
    }
}
