// **The window on to the fighter** (`Paint::cut`), shared by the paint's two
// passes: the colour (`paint.wgsl`) and the depth drawn before it
// (`paint_prepass.wgsl`). Both have to cut the same pixels. A hole cut only in
// the colour leaves the depth of what was cut away standing in the depth
// prepass, so nothing behind it is drawn there either, and the hole is left
// the clear colour, the sky's: the tree comes out white where it should have
// been a window.
#define_import_path game::cut

// Is this pixel in the hole along the line from `eye` to `cut.xyz`, `cut.w`
// across at that end and narrower toward the eye? Never where `cut.w` is zero.
// The edge is stippled by the pixel's own place on the screen, so it reads as
// a window rather than a missing piece -- and so both passes stipple it alike.
fn in_window(cut: vec4<f32>, eye: vec3<f32>, at: vec3<f32>, pixel: vec2<f32>) -> bool {
    if (cut.w <= 0.0) {
        return false;
    }
    let to = cut.xyz - eye;
    let t = dot(at - eye, to) / max(dot(to, to), 1e-4);
    if (t <= 0.0 || t >= 0.85) {
        return false;
    }
    let off = length(at - (eye + to * t));
    let r = cut.w * (0.35 + 0.65 * t);
    let grain = fract(sin(dot(pixel, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    return off < r * (0.75 + 0.25 * grain);
}
