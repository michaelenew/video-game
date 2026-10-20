// **The ink's depth** (`ink.rs`): Bevy's prepass fragment, with the same
// holes the smoke makes in the picture (`game::ink::eaten`), so nothing
// behind her is hidden by a part of her that is not drawn. The shape of
// `paint_prepass.wgsl`, for the reason it gives.

#import bevy_pbr::{
    prepass_io,
    prepass_io::VertexOutput,
    mesh_view_bindings::view,
}
#import bevy_render::globals::Globals
#import game::ink::{InkLook, rim, eaten}

// The prepass's view has the clock at another binding than the main
// pass's (`bevy_pbr::prepass`'s view layout): read from the main pass's
// place, the pipeline does not match its layout and is never made.
@group(0) @binding(1) var<uniform> globals: Globals;

@group(2) @binding(100) var<uniform> ink: InkLook;

fn gone(in: VertexOutput) -> bool {
    let r = rim(in.world_position.xyz, view.world_position);
    return eaten(ink, in.world_position.xyz, r, globals.time) > 0.62;
}

#ifdef PREPASS_FRAGMENT
@fragment
fn fragment(in: VertexOutput) -> prepass_io::FragmentOutput {
    if (gone(in)) {
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
    if (gone(in)) {
        discard;
    }
}
#endif
