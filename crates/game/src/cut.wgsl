// **The window on to the fighter** (`Paint::cut`), shared by every pass that
// draws a windowed paint: the colour (`paint.wgsl`), the depth drawn before
// it (`paint_prepass.wgsl`), and the ghost drawn over the hole
// (`Paint::ghost`). They have to agree on the pixels. A hole cut only in the
// colour leaves the depth of what was cut away standing in the depth
// prepass, so nothing behind it is drawn there either, and the hole is left
// the clear colour, the sky's: the tree came out white where it should have
// been a window.
#define_import_path game::cut

// How much of the window this point is in: 0 outside it, 1 well inside, and
// a soft rim between, so the ghost thins into the wood instead of the wood
// ending at an edge. The window runs along the line from `eye` to `cut.xyz`,
// `cut.w` across at that end and narrower toward the eye, and stops short of
// the fighter, so what they stand on stays solid. None where `cut.w` is
// zero.
fn window(cut: vec4<f32>, eye: vec3<f32>, at: vec3<f32>) -> f32 {
    if (cut.w <= 0.0) {
        return 0.0;
    }
    let to = cut.xyz - eye;
    let t = dot(at - eye, to) / max(dot(to, to), 1e-4);
    if (t <= 0.0 || t >= 0.85) {
        return 0.0;
    }
    let off = length(at - (eye + to * t));
    let r = cut.w * (0.35 + 0.65 * t);
    let across = 1.0 - smoothstep(0.6 * r, r, off);
    let along = 1.0 - smoothstep(0.7, 0.85, t);
    return across * along;
}
