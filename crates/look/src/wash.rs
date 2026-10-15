//! **How a picture is painted**: the treatment, as numbers.
//!
//! Everything else in this crate decides what colour a thing *is*. This
//! decides how that colour is *laid down and lit*: whether the light on a
//! wall turns to shade along a smooth curve or in a few painted tones, what
//! colour the shade is, how thick the air is and what colour, what glows,
//! whether there is paper under it all.
//!
//! It is in three places, kept apart because they are different things:
//!
//! - **On the surface** (`game/src/paint.wgsl`), in world space: the light as
//!   a few soft painted tones with the brush pushing the border between them
//!   about, a shade mixed from the sky and a light from the sun, pigment that
//!   wanders over a few metres, tops lit pale by the sky.
//! - **In the air** (`game/src/wash.wgsl`, `brush`), from the depth buffer:
//!   haze that thickens with distance and toward the ground and takes the
//!   sky's own colour, near things darkened into silhouettes, corners and
//!   contacts shaded, glow round what is bright, shafts from the sun, and a
//!   grade that tints the shade cool and the light warm. A painted surface
//!   filter (Kuwahara) before all of it.
//! - **On the paper** (`game/src/wash.wgsl`, `paper`), in screen space: the
//!   watercolour treatment -- pigment pooling, granulation, dry brush, paper
//!   relief -- for the styles that sit on a sheet.
//!
//! [`STYLES`] holds the candidates being judged; `--look <name>` picks one.

/// One treatment, everything it does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wash {
    pub name: &'static str,

    // ---- On the surface ------------------------------------------------
    /// How far the light is pulled into tones, 0 (a smooth gradient, as it
    /// was) to 1 (flat tones only).
    pub bands: f32,
    /// How many tones between full shade and full light.
    pub tones: f32,
    /// How soft the border between two tones is, in units of light.
    pub soft: f32,
    /// How far a brush stroke pushes that border, in units of light.
    pub stroke: f32,
    /// How long a stroke is, in metres; it is a sixth as wide.
    pub stroke_len: f32,
    /// The shade's colour, multiplied in: mixed from the sky.
    pub shade: [f32; 3],
    /// The light's colour, multiplied in: mixed from the sun.
    pub light: [f32; 3],
    /// How much the pigment wanders over a few metres, as a fraction of it.
    pub mottle: f32,
    /// How far the lit tone is thinned toward pale (MNPR's dilution).
    pub dilute: f32,
    /// How much paler a face is the more it looks up at the sky.
    pub top: f32,
    /// How strongly the brush's own marks show on a surface: short strokes a
    /// little lighter, darker, warmer and cooler.
    pub grain: f32,
    /// How strongly surfaces are laid in broad flat-brush strokes, fixed to
    /// the world and kept a constant size on screen.
    pub broad: f32,
    /// How long such a stroke is, as a share of its distance from the eye.
    pub broad_size: f32,
    /// How strongly the land's colour follows its contours: its hollows
    /// deepened toward `hollow`, its crests toward `crest`. A seasoning, not a
    /// colour: at full strength a hollow is multiplied by `hollow` once.
    pub rich: f32,
    /// What a hollow's colour is multiplied toward: deeper, cooler.
    pub hollow: [f32; 3],
    /// What a crest's colour is multiplied toward: warmer.
    pub crest: [f32; 3],
    /// Whether a rock is cut in flat facets (a stone blocked in with a flat
    /// brush) rather than a rounded lump.
    pub cut: bool,

    // ---- In the air ----------------------------------------------------
    /// The radius of the painted-surface filter in pixels at 760 lines.
    pub brush: f32,
    /// How long the filter is drawn out along an edge.
    pub drawn: f32,
    /// Crisp near, painted far: the distance in metres inside which the
    /// picture is rendered as it is, and the distance past which it is all
    /// brushwork. A `painted_from` of zero paints everything alike.
    pub crisp: f32,
    pub painted_from: f32,
    /// How strongly the distance and the sky are laid in broad dry-brush
    /// strokes (GW2's concept art).
    pub strokes: f32,
    /// How far the picture's edges break up into bare paper, in dry brush.
    pub frame: f32,
    /// How thick the air is, per metre, at the camera's height.
    pub haze: f32,
    /// How fast it thins with height, per metre.
    pub haze_falloff: f32,
    /// What colour the air is, mixed into the sky's own.
    pub haze_tint: [f32; 3],
    /// How much of that tint; the rest is the sky's colour.
    pub haze_tint_amount: f32,
    /// How dark the nearest things go, as silhouettes against the distance.
    pub near_dark: f32,
    /// How much corners and contacts are shaded.
    pub ao: f32,
    /// How strongly an edge turned toward you is caught with a lighter
    /// stroke, and a crease darkened.
    pub edges: f32,
    /// How much light spills round what is bright.
    pub glow: f32,
    /// How strong the shafts from the sun are.
    pub shafts: f32,
    /// What the shade is tinted toward in the grade.
    pub grade_shade: [f32; 3],
    /// What the light is tinted toward in the grade.
    pub grade_light: [f32; 3],
    /// Colourfulness, 1 as it was.
    pub saturation: f32,
    /// How far the darks are lifted toward the darkest colour.
    pub lift: f32,
    /// The darkest colour: never black.
    pub dark: [f32; 3],
    /// How high-key: how far the mid tones are lifted.
    pub key: f32,

    // ---- On the paper --------------------------------------------------
    /// Whether there is a sheet at all; the rest of this block is ignored
    /// without one.
    pub on_paper: bool,
    /// How hard pigment pools where colour changes (edge darkening).
    pub pool: f32,
    /// How far a wet colour wanders, in pixels.
    pub bleed: f32,
    /// How much the paper's tooth catches pigment.
    pub granulate: f32,
    /// How dry the brush is: how much a light wash skips the paper.
    pub dry: f32,
    /// How far the wet colour runs along the paper's slopes, in pixels.
    pub distort: f32,
    /// How much the paper's own relief shades the picture.
    pub relief: f32,
    /// How uneven a wash is over a hand's width of paper.
    pub turbulence: f32,
    /// The paper's colour: the whitest the picture may be.
    pub paper: [f32; 3],

    // ---- The line ------------------------------------------------------
    /// How much the outline wavers, in pixels.
    pub wobble: f32,
    /// How much the outline is one sepia ink.
    pub sepia: f32,
    /// How strongly the outline is laid on; scales the line's own weight.
    pub line: f32,
}

/// The treatment as it was before any of this.
pub const PLAIN: Wash = Wash {
    name: "plain",
    bands: 0.0,
    tones: 3.0,
    soft: 0.1,
    stroke: 0.0,
    stroke_len: 1.0,
    shade: [1.0, 1.0, 1.0],
    light: [1.0, 1.0, 1.0],
    mottle: 0.0,
    dilute: 0.0,
    top: 0.0,
    grain: 0.0,
    broad: 0.0,
    broad_size: 0.05,
    rich: 0.0,
    hollow: [1.0, 1.0, 1.0],
    crest: [1.0, 1.0, 1.0],
    cut: false,
    brush: 0.0,
    drawn: 1.0,
    crisp: 0.0,
    painted_from: 0.0,
    strokes: 0.0,
    frame: 0.0,
    haze: 0.0,
    haze_falloff: 0.05,
    haze_tint: [1.0, 1.0, 1.0],
    haze_tint_amount: 0.0,
    near_dark: 0.0,
    ao: 0.0,
    edges: 0.0,
    glow: 0.0,
    shafts: 0.0,
    grade_shade: [1.0, 1.0, 1.0],
    grade_light: [1.0, 1.0, 1.0],
    saturation: 1.0,
    lift: 0.0,
    dark: [0.0, 0.0, 0.0],
    key: 0.0,
    on_paper: false,
    pool: 0.0,
    bleed: 0.0,
    granulate: 0.0,
    dry: 0.0,
    distort: 0.0,
    relief: 0.0,
    turbulence: 0.0,
    paper: [1.0, 1.0, 1.0],
    wobble: 0.0,
    sepia: 0.0,
    line: 1.0,
};

/// **Painted**: the first reference. Soft hand-painted gradients on every
/// face, violet shade and warm light, pale sky-lit tops, a coloured haze the
/// distance sinks into, near things dark against it, glow, sun shafts. No
/// paper, almost no line.
pub const PAINTED: Wash = Wash {
    name: "painted",
    bands: 0.45,
    tones: 3.0,
    soft: 0.16,
    stroke: 0.14,
    stroke_len: 1.4,
    shade: [0.80, 0.80, 1.14],
    light: [1.08, 1.01, 0.90],
    mottle: 0.12,
    dilute: 0.08,
    top: 0.22,
    grain: 0.22,
    broad: 0.0,
    broad_size: 0.05,
    rich: 0.0,
    hollow: [1.0, 1.0, 1.0],
    crest: [1.0, 1.0, 1.0],
    cut: true,
    brush: 5.0,
    drawn: 1.5,
    crisp: 0.0,
    painted_from: 0.0,
    strokes: 0.0,
    frame: 0.0,
    haze: 0.006,
    haze_falloff: 0.03,
    haze_tint: [0.50, 0.46, 0.92],
    haze_tint_amount: 0.4,
    near_dark: 0.25,
    ao: 0.55,
    edges: 0.9,
    glow: 0.55,
    shafts: 0.35,
    grade_shade: [0.86, 0.84, 1.16],
    grade_light: [1.08, 1.02, 0.90],
    saturation: 1.22,
    lift: 0.3,
    dark: [0.10, 0.08, 0.20],
    key: 0.1,
    on_paper: false,
    pool: 0.0,
    bleed: 0.0,
    granulate: 0.0,
    dry: 0.0,
    distort: 0.0,
    relief: 0.0,
    turbulence: 0.0,
    paper: [1.0, 1.0, 1.0],
    wobble: 0.0,
    sepia: 0.0,
    line: 0.3,
};

/// **Brushed**: Guild Wars 2's concept art. The near world -- the fighters,
/// what they stand on -- rendered crisp; the distance and the sky laid in
/// broad dry-brush strokes with bristle streaks; the picture's edges breaking
/// into bare paper. Natural colour, warm light and cool shade.
pub const BRUSHED: Wash = Wash {
    name: "brushed",
    bands: 0.35,
    tones: 3.0,
    soft: 0.18,
    stroke: 0.06,
    stroke_len: 1.4,
    shade: [0.86, 0.88, 1.08],
    light: [1.06, 1.02, 0.94],
    mottle: 0.04,
    dilute: 0.08,
    top: 0.22,
    grain: 0.0,
    broad: 0.75,
    broad_size: 0.22,
    rich: 1.0,
    hollow: [0.74, 0.82, 1.12],
    crest: [1.14, 0.96, 0.84],
    cut: true,
    brush: 7.0,
    drawn: 1.8,
    crisp: 12.0,
    painted_from: 70.0,
    strokes: 1.0,
    frame: 0.0,
    haze: 0.004,
    haze_falloff: 0.03,
    haze_tint: [0.80, 0.84, 0.95],
    haze_tint_amount: 0.25,
    near_dark: 0.1,
    ao: 0.55,
    edges: 0.25,
    glow: 0.4,
    shafts: 0.3,
    grade_shade: [0.90, 0.90, 1.10],
    grade_light: [1.06, 1.02, 0.94],
    saturation: 1.15,
    lift: 0.2,
    dark: [0.10, 0.09, 0.16],
    key: 0.05,
    on_paper: false,
    pool: 0.0,
    bleed: 0.0,
    granulate: 0.0,
    dry: 0.0,
    distort: 0.0,
    relief: 0.0,
    turbulence: 0.0,
    paper: [1.0, 1.0, 1.0],
    wobble: 0.0,
    sepia: 0.0,
    line: 0.35,
};

/// **Depths**: the second reference. The same painting pushed toward one
/// hue and toward layers: thick blue air, the foreground in silhouette, the
/// sun coming through in shafts.
pub const DEPTHS: Wash = Wash {
    name: "depths",
    bands: 0.6,
    tones: 3.0,
    soft: 0.12,
    stroke: 0.12,
    stroke_len: 1.6,
    shade: [0.70, 0.80, 1.15],
    light: [1.02, 1.04, 1.04],
    mottle: 0.10,
    dilute: 0.12,
    top: 0.3,
    grain: 0.25,
    broad: 0.0,
    broad_size: 0.05,
    rich: 0.0,
    hollow: [1.0, 1.0, 1.0],
    crest: [1.0, 1.0, 1.0],
    cut: true,
    brush: 3.0,
    drawn: 1.0,
    crisp: 0.0,
    painted_from: 0.0,
    strokes: 0.0,
    frame: 0.0,
    haze: 0.014,
    haze_falloff: 0.03,
    haze_tint: [0.22, 0.52, 0.82],
    haze_tint_amount: 0.8,
    near_dark: 0.6,
    ao: 0.5,
    edges: 0.7,
    glow: 0.8,
    shafts: 0.9,
    grade_shade: [0.70, 0.88, 1.15],
    grade_light: [0.92, 1.04, 1.10],
    saturation: 1.05,
    lift: 0.4,
    dark: [0.03, 0.08, 0.16],
    key: 0.0,
    on_paper: false,
    pool: 0.0,
    bleed: 0.0,
    granulate: 0.0,
    dry: 0.0,
    distort: 0.0,
    relief: 0.0,
    turbulence: 0.0,
    paper: [1.0, 1.0, 1.0],
    wobble: 0.0,
    sepia: 0.0,
    line: 0.0,
};

/// **Wash**: transparent watercolour on a sheet (MNPR's model and paper).
/// Soft washes with brushed borders, pigment pooling at every edge, the
/// paper's grain and dry-brush breaks through the light, a wavering line.
pub const WASH: Wash = Wash {
    name: "wash",
    bands: 0.85,
    tones: 3.0,
    soft: 0.07,
    stroke: 0.16,
    stroke_len: 1.6,
    shade: [0.90, 0.95, 1.10],
    light: [1.07, 1.02, 0.92],
    mottle: 0.10,
    dilute: 0.35,
    top: 0.1,
    grain: 0.08,
    broad: 0.0,
    broad_size: 0.05,
    rich: 0.0,
    hollow: [1.0, 1.0, 1.0],
    crest: [1.0, 1.0, 1.0],
    cut: false,
    brush: 2.0,
    drawn: 1.6,
    crisp: 0.0,
    painted_from: 0.0,
    strokes: 0.0,
    frame: 0.0,
    haze: 0.008,
    haze_falloff: 0.05,
    haze_tint: [0.97, 0.94, 0.87],
    haze_tint_amount: 0.5,
    near_dark: 0.0,
    ao: 0.3,
    edges: 0.3,
    glow: 0.0,
    shafts: 0.0,
    grade_shade: [0.94, 0.96, 1.06],
    grade_light: [1.04, 1.01, 0.95],
    saturation: 0.95,
    lift: 0.55,
    dark: [0.17, 0.16, 0.25],
    key: 0.45,
    on_paper: true,
    pool: 1.3,
    bleed: 1.6,
    granulate: 0.35,
    dry: 0.55,
    distort: 2.5,
    relief: 0.15,
    turbulence: 0.16,
    paper: [0.97, 0.94, 0.87],
    wobble: 0.9,
    sepia: 0.35,
    line: 0.8,
};

/// Every candidate, by the name `--look` takes.
pub const STYLES: [Wash; 5] = [PLAIN, PAINTED, DEPTHS, WASH, BRUSHED];

/// The treatment called `name`, if there is one.
pub fn named(name: &str) -> Option<Wash> {
    STYLES.into_iter().find(|w| w.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_style_answers_to_its_name() {
        for w in STYLES {
            assert_eq!(named(w.name), Some(w));
        }
        assert_eq!(named("nothing"), None);
    }

    #[test]
    fn the_paper_is_never_a_light() {
        for w in STYLES {
            for c in w.paper {
                assert!((0.8..=1.0).contains(&c), "{}: paper {:?}", w.name, w.paper);
            }
        }
    }

    #[test]
    fn a_shade_is_cooler_than_its_light() {
        for w in STYLES.iter().filter(|w| w.bands > 0.0) {
            let warmth = |c: [f32; 3]| c[0] - c[2];
            assert!(warmth(w.shade) < warmth(w.light), "{}", w.name);
            assert!(warmth(w.grade_shade) < warmth(w.grade_light), "{}", w.name);
        }
    }
}
