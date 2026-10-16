// **The paint's depth, with its window cut** (`Paint::cut`, `cut.wgsl`).
//
// Only a windowed paint gets here: it is drawn alpha-masked so that Bevy runs
// a fragment shader in the depth prepass and the shadow pass at all, and this
// is that shader -- Bevy's own prepass fragment, with the window's discard in
// front of it.

#import bevy_pbr::{
    prepass_io,
    prepass_io::VertexOutput,
    mesh_view_bindings::view,
}
#import game::cut::in_window

struct Paint {
    color: vec4<f32>,
    cut: vec4<f32>,
}

@group(2) @binding(0) var<uniform> paint: Paint;

// The window is cut for the camera's eye only. A shadow map is drawn from the
// sun, through an orthographic projection, and a tree's shadow does not open
// because the camera is behind it.
fn cut_away(in: VertexOutput) -> bool {
    let orthographic = view.clip_from_view[3][3] == 1.0;
    return !orthographic && in_window(paint.cut, view.world_position, in.world_position.xyz, in.position.xy);
}

// Bevy defines `FragmentOutput` only when the pass writes something besides
// depth (a normal, a motion vector, a clamped depth), so the depth-only
// prepass and most shadow passes take the second form, with no output: a
// shader that named it there would not compile, and a paint whose prepass
// does not compile is simply missing from the depth and the shadows.
#ifdef PREPASS_FRAGMENT
@fragment
fn fragment(in: VertexOutput) -> prepass_io::FragmentOutput {
    if (cut_away(in)) {
        discard;
    }
    var out: prepass_io::FragmentOutput;
#ifdef NORMAL_PREPASS
    out.normal = vec4(in.world_normal * 0.5 + vec3(0.5), 1.0);
#endif
#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    out.frag_depth = in.unclipped_depth;
#endif
#ifdef MOTION_VECTOR_PREPASS
    out.motion_vector = vec2(0.0);
#endif
    return out;
}
#else
@fragment
fn fragment(in: VertexOutput) {
    if (cut_away(in)) {
        discard;
    }
}
#endif
