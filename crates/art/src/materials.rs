//! Every material in the game, as rows of numbers.
//!
//! This file is the point of the whole crate. Each entry below is one material
//! — what it looks like, how light leaves it, how rough it is — and it is a
//! handful of constants against a single evaluator. No texture, no import
//! step, no filename, no artist.
//!
//! The useful way to read it is as a set of claims about *what each material
//! actually is*, since that is what the parameters end up encoding:
//!
//! - **Stone** is strata that have been buckled, with cracks along the
//!   boundaries between grains.
//! - **Cloth** is two sets of threads crossing, with the slub of uneven yarn
//!   underneath.
//! - **Armour** is a smooth plate that has been worn brighter where it was
//!   struck and left dirty in its recesses.
//! - **Skin** is a low, wide mottle with pores on top.
//! - **Fire** is a temperature field rolling upward, and its colour is that
//!   temperature rather than a choice.
//!
//! Getting those five sentences right is most of the work. The numbers are
//! just how they are said.
//!
//! ## Structure, not octaves
//!
//! The failure mode of procedural materials is that everything comes out
//! looking like the same grey marble, and the instinct — add more octaves — is
//! the wrong one. Octaves add detail; they never add structure. What separates
//! these materials is **anisotropy** (a frequency per axis, so stone has
//! strata and cloth has a direction), **layering** (two fields with different
//! jobs, not one field with more of itself) and **warping** (a field displaced
//! by another field, so features wander instead of running straight).

use crate::color::{Lch, Ramp};
use crate::surface::{Emission, Field, FieldKind, Surface};

/// How many pixels a side each material is baked at.
///
/// One number for everything, because a per-material resolution is a decision
/// per material and the whole crate is arranged to spend as few of those as
/// possible. 256 is where the set costs about a quarter of a second to build
/// on four cores and no material has visible texels at fighting distance; the
/// tile size in [`Material::extent`] is the parameter that actually varies
/// what "detail" means, and it is already carried per material.
pub const BAKE_SIZE: u32 = 256;

/// What a material is for. Decides which rules a test holds it to — the world
/// has to stay nearly colourless so that the things with colour in them read;
/// a fighter and an effect do not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// The arena. Held to the chroma ceiling.
    World,
    /// A fighter, or something a fighter is wearing.
    Character,
    /// Spells, hits, and everything that exists for a second and a half.
    Effect,
}

/// A material with a name and a job.
#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub name: &'static str,
    pub role: Role,
    pub surface: Surface,
    /// How many metres one tile of this material covers.
    ///
    /// Carried by the material rather than chosen at the call site, because it
    /// is not a rendering preference — it is half of what the frequencies
    /// *mean*. A stone at 0.55 cycles per metre is a different rock baked
    /// across two metres than across eight, and every caller getting to pick
    /// would make the numbers above unreadable.
    pub extent: f32,
}

impl Material {
    /// How this material wants to be baked, at a given resolution.
    ///
    /// Walls slice along the world's vertical so that stone crosses its own
    /// strata; floors slice flat; everything else is sampled without tiling,
    /// since a fighter's arm does not repeat.
    pub fn plan(&self, size: u32) -> crate::bake::Plan {
        match self.role {
            Role::World if self.name == "ground" => crate::bake::Plan::floor(size, self.extent),
            Role::World => crate::bake::Plan::wall(size, self.extent),
            _ => crate::bake::Plan {
                extent: self.extent,
                ..crate::bake::Plan::character(size)
            },
        }
    }

    /// How many metres one tile covers. Convenience for renderers, which think
    /// in surface sizes rather than in fields.
    pub fn surface_extent(&self) -> f32 {
        self.extent
    }

    /// Texels per cycle of the finest thing in this material, at a resolution.
    ///
    /// Under about four, the fine detail stops being detail and becomes
    /// interference. See [`crate::surface::Field::finest`].
    pub fn texels_per_cycle(&self, size: u32) -> f32 {
        let finest = self.surface.finest();
        if finest <= 0.0 {
            return f32::INFINITY;
        }
        size as f32 / (finest * self.extent)
    }
}

// A shorthand, because a hue and a chroma per stop is the point and the word
// `Lch::new` three times per ramp is not.
const fn c(l: f32, chroma: f32, h: f32) -> Lch {
    Lch::new(l, chroma, h)
}

// ---------------------------------------------------------------------------
// The world
// ---------------------------------------------------------------------------

/// Arena walls and the Elementalist's structures.
///
/// Strata come from the frequency being four times higher in Y than across, so
/// the noise is squashed into horizontal bands — that one number is the
/// difference between rock and marble. The warp then buckles those bands, so
/// they bend the way real bedding planes do rather than running dead straight.
/// `Seams` cuts the cracks: it goes to zero exactly on the boundaries between
/// scattered points, which is a closed line obtained without drawing one. Its
/// cells are flattened the same way the strata are, so the cracks run *along*
/// the bedding rather than across it, which is both what real rock does and —
/// less obviously — the only reason the strata survive at all. An isotropic
/// detail layer at this grain drowns them: measured, it took stone from a
/// 2.6-to-1 ratio between its vertical and horizontal variation down to
/// 1.1-to-1, which is another way of spelling "grey marble". A test pins it.
pub const STONE: Material = Material {
    name: "stone",
    role: Role::World,
    surface: Surface {
        form: Field::new(FieldKind::Ridged, [0.55, 1.7, 0.55], 0x5701).octaves(3),
        detail: Field::new(FieldKind::Seams, [2.2, 5.5, 2.2], 0x5702).octaves(1),
        warp: 0.45,
        grain: 0.30,
        contrast: 1.0,
        ramp: Ramp::three(
            c(0.18, 0.012, 0.68),
            c(0.36, 0.018, 0.66),
            c(0.50, 0.010, 0.62),
        ),
        roughness: (0.98, 0.82),
        metallic: 0.0,
        emission: Emission::None,
        relief: 0.035,
    },
    extent: 2.0,
};

/// The arena floor. Stone at a different scale, and flatter.
///
/// A separate entry rather than the same one reused, because the floor is seen
/// at a grazing angle across forty metres and the walls are seen head-on from
/// two. The same frequency cannot serve both: what reads as grain on a wall
/// reads as static on a floor.
///
/// This is also the material that found the `contrast` parameter. Its first
/// frequency was low enough that three metres of floor did not cross one whole
/// feature, and four octaves of noise cluster toward their middle on top of
/// that -- between them the floor spanned 0.40 to 0.64 of a ramp that runs
/// from 0 to 1, and came out an even grey whatever the ramp said. Raising the
/// frequency fixed half of it and stretching the range fixed the rest.
pub const GROUND: Material = Material {
    name: "ground",
    role: Role::World,
    surface: Surface {
        form: Field::new(FieldKind::Fbm, [0.85, 0.85, 0.85], 0x6101).octaves(4),
        detail: Field::new(FieldKind::Cells, [2.6, 2.6, 2.6], 0x6102).octaves(2),
        warp: 0.35,
        grain: 0.26,
        contrast: 1.9,
        // Dark: this is asphalt, not concrete. Raising the lightness was the
        // wrong half of the fix when the ramp came out unreachable -- the
        // frequency and the contrast were what was actually wrong, and a
        // lighter floor on top of them put the arena a stop over.
        ramp: Ramp::three(
            c(0.17, 0.012, 0.70),
            c(0.28, 0.018, 0.68),
            c(0.38, 0.009, 0.65),
        ),
        roughness: (0.99, 0.90),
        metallic: 0.0,
        emission: Emission::None,
        relief: 0.05,
    },
    extent: 4.0,
};

// ---------------------------------------------------------------------------
// Fighters
// ---------------------------------------------------------------------------

/// Skin.
///
/// Two scales and nothing else: a wide, low mottle for the colour variation
/// across a limb, and a fine one for pores. Deliberately low chroma — see
/// [`crate::palette`] — which is also what keeps it out of the way of the
/// reserved red.
///
/// The honest limit: real skin is translucent, and light that enters it
/// scatters below the surface before leaving. No albedo texture reproduces
/// that. What this gets is the *variation*; the softness wants a lighting term
/// the renderer has to provide, and until it does, skin will read a little
/// like painted clay.
pub const SKIN: Material = Material {
    name: "skin",
    role: Role::Character,
    surface: Surface {
        form: Field::new(FieldKind::Fbm, [1.6, 1.6, 1.6], 0x3301)
            .octaves(3)
            .gain(0.45),
        detail: Field::new(FieldKind::Fbm, [8.0, 8.0, 8.0], 0x3302).octaves(2),
        warp: 0.08,
        grain: 0.16,
        contrast: 1.0,
        ramp: Ramp::three(
            c(0.46, 0.048, 0.085),
            c(0.60, 0.056, 0.100),
            c(0.70, 0.038, 0.115),
        ),
        roughness: (0.62, 0.48),
        metallic: 0.0,
        emission: Emission::None,
        relief: 0.004,
    },
    extent: 1.0,
};

/// Cloth, in a player's colour.
///
/// `Stripe` multiplies a sine along X by a sine along Y, which gives crossing
/// threads. Summing them instead gives a plaid of squares, which is the first
/// thing anyone tries and it does not look like fabric. The two axes have to
/// be the two the texture is *sliced* on, or the weave has no second direction
/// to cross in -- see [`crate::bake::Plan::wall`] for the same trap caught
/// from the other side. The weave frequency is
/// high and the relief is tiny, because a weave is something you notice at
/// arm's length and never at fighting distance — what carries at distance is
/// the fold shading from the low-frequency layer underneath.
pub fn cloth(tint: Lch) -> Material {
    Material {
        name: "cloth",
        role: Role::Character,
        surface: Surface {
            form: Field::new(FieldKind::Stripe, [11.0, 11.0, 0.0], 0x4401).octaves(1),
            detail: Field::new(FieldKind::Fbm, [2.4, 2.4, 2.4], 0x4402).octaves(3),
            warp: 0.10,
            // Most of the look is the fold layer, not the weave.
            grain: 0.62,
            contrast: 1.0,
            ramp: Ramp::three(
                Lch::new(tint.l * 0.55, tint.c * 0.85, tint.h),
                tint,
                Lch::new((tint.l * 1.20).min(0.95), tint.c * 0.75, tint.h),
            ),
            roughness: (0.94, 0.80),
            metallic: 0.0,
            emission: Emission::None,
            relief: 0.004,
        },
        extent: 1.0,
    }
}

/// Plate armour, tinted toward a player's colour.
///
/// The wear is the whole trick, and it runs *backwards* from the intuition:
/// roughness is high where the field is low and low where it is high, so the
/// recesses stay dull and the raised parts come up polished. That is what
/// handling does to metal — it burnishes what it touches and leaves what it
/// cannot reach — and it is the single cheapest thing that stops a metal
/// surface reading as plastic.
///
/// `Seams` again for the boundaries between plates, at a low frequency so the
/// plates are hand-sized rather than scale-sized.
pub fn armour(tint: Lch) -> Material {
    Material {
        name: "armour",
        role: Role::Character,
        surface: Surface {
            form: Field::new(FieldKind::Seams, [2.6, 2.6, 2.6], 0x2201).octaves(1),
            detail: Field::new(FieldKind::Fbm, [7.5, 7.5, 7.5], 0x2202)
                .octaves(3)
                .gain(0.55),
            warp: 0.12,
            grain: 0.34,
            contrast: 1.0,
            ramp: Ramp::four(
                Lch::new(0.30, 0.014, tint.h),
                Lch::new(0.52, 0.030, tint.h),
                Lch::new(0.70, tint.c * 0.50, tint.h),
                Lch::new(0.86, tint.c * 0.26, tint.h),
            ),
            // Dull in the recesses, burnished on the high ground.
            roughness: (0.70, 0.16),
            metallic: 1.0,
            emission: Emission::None,
            relief: 0.016,
        },
        extent: 1.0,
    }
}

/// Leather and strapping. The connective tissue of a kit — belts, boots,
/// grips, the parts that are neither cloth nor plate.
pub const LEATHER: Material = Material {
    name: "leather",
    role: Role::Character,
    surface: Surface {
        form: Field::new(FieldKind::Cells, [9.0, 9.0, 9.0], 0x7701).octaves(2),
        detail: Field::new(FieldKind::Fbm, [3.0, 3.0, 3.0], 0x7702).octaves(3),
        warp: 0.25,
        grain: 0.40,
        contrast: 1.0,
        ramp: Ramp::three(
            c(0.18, 0.030, 0.095),
            c(0.30, 0.044, 0.090),
            c(0.40, 0.036, 0.085),
        ),
        roughness: (0.92, 0.68),
        metallic: 0.0,
        emission: Emission::None,
        relief: 0.008,
    },
    extent: 1.0,
};

// ---------------------------------------------------------------------------
// Effects
// ---------------------------------------------------------------------------

/// Fire.
///
/// The one material here whose colour was not chosen. `Emission::Hot` runs the
/// field through the blackbody curve, so the parameters are two temperatures —
/// the cool edge and the hot core — and the gradient between them is what
/// physics says it is. 1700 K is a dull ember, 2600 K is a hot flame; push the
/// pair up together and it whitens exactly the way real fire does, from one
/// knob that means something. The cool end sits at the floor of the blackbody
/// fit deliberately -- see [`crate::color::COOLEST`].
///
/// Albedo is almost black on purpose. Flame is emitted light, not a lit
/// surface — give it a base colour and it picks up the arena's lighting and
/// starts reading as orange plastic.
///
/// The heavy warp is what makes it roll. Fire's shape is vorticity, and a
/// field displaced by another field is the cheapest thing that looks like it.
pub const FIRE: Material = Material {
    name: "fire",
    role: Role::Effect,
    surface: Surface {
        form: Field::new(FieldKind::Fbm, [1.1, 0.55, 1.1], 0x1101)
            .octaves(4)
            .gain(0.58),
        detail: Field::new(FieldKind::Fbm, [2.6, 1.3, 2.6], 0x1102).octaves(3),
        warp: 0.85,
        grain: 0.30,
        contrast: 1.0,
        ramp: Ramp::two(c(0.02, 0.004, 0.08), c(0.06, 0.010, 0.10)),
        roughness: (1.0, 1.0),
        metallic: 0.0,
        emission: Emission::Hot {
            cool: 1700.0,
            hot: 2600.0,
            strength: 9.0,
        },
        relief: 0.0,
    },
    extent: 2.0,
};

/// Blood, for the Blood mage's cost and the Reaver's drain field.
///
/// Wet where it is thick and matte where it has dried, which is the roughness
/// pair running from 0.2 to 0.8. Sits inside the reserved red band on purpose:
/// it is a thing that hurts someone.
pub const BLOOD: Material = Material {
    name: "blood",
    role: Role::Effect,
    surface: Surface {
        form: Field::new(FieldKind::Fbm, [2.0, 2.0, 2.0], 0x1201).octaves(4),
        detail: Field::new(FieldKind::Cells, [6.0, 6.0, 6.0], 0x1202).octaves(1),
        warp: 0.30,
        grain: 0.25,
        contrast: 1.0,
        ramp: Ramp::three(
            c(0.12, 0.060, 0.055),
            c(0.26, 0.130, 0.055),
            c(0.38, 0.150, 0.045),
        ),
        roughness: (0.80, 0.18),
        metallic: 0.0,
        emission: Emission::Glow { strength: 0.35 },
        relief: 0.006,
    },
    extent: 1.5,
};

/// The Shadow Reaver's shadow.
///
/// Barely a material. It is an *absence*, and the way to draw an absence is to
/// take almost all the light out and keep just enough structure that it does
/// not read as a hole in the rendering. Very slightly blue, because a shadow
/// that is neutral grey reads as fog and one that is warm reads as a burn.
///
/// Wants to be drawn unlit. A shadow that catches a highlight is a solid
/// object and stops being a shadow.
pub const SHADOW: Material = Material {
    name: "shadow",
    role: Role::Effect,
    surface: Surface {
        form: Field::new(FieldKind::Fbm, [1.8, 0.9, 1.8], 0x1301).octaves(3),
        detail: Field::new(FieldKind::Fbm, [5.0, 5.0, 5.0], 0x1302).octaves(2),
        warp: 0.55,
        grain: 0.35,
        contrast: 1.0,
        ramp: Ramp::two(c(0.02, 0.008, 0.72), c(0.14, 0.030, 0.72)),
        roughness: (1.0, 1.0),
        metallic: 0.0,
        emission: Emission::None,
        relief: 0.0,
    },
    extent: 2.0,
};

/// Arcane light, in whatever colour the caster's identity is.
///
/// For the mage bolts, the Dual mage's meter, and anything that is spectacle
/// rather than substance. Emission carries all of it; the albedo is nearly
/// black for the same reason fire's is.
pub fn arcane(tint: Lch) -> Material {
    Material {
        name: "arcane",
        role: Role::Effect,
        surface: Surface {
            form: Field::new(FieldKind::Fbm, [2.2, 2.2, 2.2], 0x1401).octaves(3),
            detail: Field::new(FieldKind::Ridged, [5.5, 5.5, 5.5], 0x1402).octaves(2),
            warp: 0.65,
            grain: 0.45,
            contrast: 1.0,
            ramp: Ramp::three(
                Lch::new(0.10, tint.c * 0.5, tint.h),
                Lch::new(0.55, tint.c * 1.4, tint.h),
                Lch::new(0.92, tint.c * 0.4, tint.h),
            ),
            roughness: (1.0, 1.0),
            metallic: 0.0,
            emission: Emission::Glow { strength: 5.0 },
            relief: 0.0,
        },
        extent: 1.5,
    }
}

/// Every material that does not need an argument, for the tests and the
/// preview sheet.
///
/// The tinted ones ([`cloth`], [`armour`], [`arcane`]) are functions because a
/// player's colour is not knowable here, and they are checked at a
/// representative tint rather than listed.
pub const FIXED: &[Material] = &[STONE, GROUND, SKIN, LEATHER, FIRE, BLOOD, SHADOW];
