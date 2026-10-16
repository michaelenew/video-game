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
    let across = 1.0 - smoothstep(0.3 * r, r, off);
    let along = 1.0 - smoothstep(0.7, 0.85, t);
    return across * along;
}

// Is this point at the window's rim or inside it: the window grown a little
// either way. What the outline asks of the nearer side of an edge before it
// inks it (`outline.wgsl`): the wood around the window stands in front of
// what is behind it, and a line drawn round that step is a ring that says
// there is a hole in the tree, which there is not -- it is the camera's.
fn by_window(cut: vec4<f32>, eye: vec3<f32>, at: vec3<f32>) -> bool {
    if (cut.w <= 0.0) {
        return false;
    }
    let to = cut.xyz - eye;
    let t = dot(at - eye, to) / max(dot(to, to), 1e-4);
    if (t <= 0.0 || t >= 0.9) {
        return false;
    }
    let off = length(at - (eye + to * t));
    return off < 1.4 * cut.w * (0.35 + 0.65 * t);
}
