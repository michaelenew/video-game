// **The arc a Champion's cut leaves** (`swing.rs`): a strip laid along the
// path the drawn blade took, painted here from two numbers a vertex.
//
// `uv.x` is how recently the blade was there: 1 where it is now, 0 as far
// back as the arc reaches. `uv.y` is which edge a point is nearer, 0 the
// inner and 1 the outer, the one the blade's tip drew. The vertex colour is
// the weapon's tint, and in its alpha how bright the cut was there: dark
// through the wind-up, full while the volume is out, fading through the
// follow-through -- so the arc grows out of nothing and goes back into it,
// and never parts from the blade.
//
// What it draws:
// - **A crescent**: full width at the blade, drawn to a point at the tail, as
//   a brush stroke is -- the shape every drawn sword arc has.
// - **Streaks**: the body is fine lanes running along the arc, each fraying
//   off at its own place near the tail, so the end is wisps, not a cut line.
// - **The edge**: a thin hot line where the tip passed, whitest at the blade.
// - **A flare** at the blade.

#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::main_pass_post_lighting_processing,
    pbr_types::pbr_input_new,
}

// How many lanes the body is drawn in.
const LANES: f32 = 22.0;

// A number in [0, 1) that looks unrelated to its neighbour's.
fn hash(x: f32) -> f32 {
    return fract(sin(x * 127.1 + 311.7) * 43758.5453);
}

@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    var u = 0.0;
    var v = 1.0;
    var tint = vec3<f32>(1.0);
    var glow = 1.0;
#ifdef VERTEX_UVS_A
    u = in.uv.x;
    v = in.uv.y;
#endif
#ifdef VERTEX_COLORS
    tint = in.color.rgb;
    glow = in.color.a;
#endif
    // How long ago, as a share of how far back the arc reaches.
    let life = u;

    // How far in from the outer edge, 0 on it and 1 at the inner.
    let depth = 1.0 - v;

    // The crescent: as wide as the band at the blade, a sliver at the tail.
    let width = mix(0.15, 1.0, pow(life, 0.5));
    let inside = 1.0 - smoothstep(width * 0.6, width, depth);

    // The streaks: each lane its own brightness and its own end.
    let lane = floor(v * LANES);
    let ends = hash(lane) * 0.3;
    let fray = smoothstep(ends, ends + 0.15, life);
    let across = fract(v * LANES);
    let line = 0.75 + 0.25 * smoothstep(0.0, 0.5, across) * smoothstep(1.0, 0.5, across);
    let shade = 0.8 + 0.2 * hash(lane + 17.0);
    let body = (0.5 + 0.5 * (1.0 - depth)) * 0.9 * fray * line * shade;

    // The edge, narrower toward the tail.
    let thick = 0.07 + 0.09 * life;
    let edge = exp(-(depth * depth) / (thick * thick)) * (0.35 + 0.65 * life);

    // The flare where the weapon is.
    let lead = smoothstep(0.85, 1.0, life) * (1.0 - smoothstep(0.0, 0.6, depth));

    // Older is fainter: solid through the newer part, thinning to nothing
    // over the oldest.
    let along = smoothstep(0.0, 0.65, life);
    var a = ((body + edge) * along + 0.3 * lead) * inside * glow;
    // Never a hard line at the tail's very end or the inner edge.
    a = a * smoothstep(0.0, 0.04, life) * smoothstep(0.0, 0.08, v);
    a = clamp(a, 0.0, 1.0);

    // Deeper toward the inner edge, white where it is hot.
    let hot = clamp(edge * life * 1.2 + lead, 0.0, 1.0);
    let deep = mix(tint, tint * tint * 0.7, depth);
    let colour = mix(deep, vec3<f32>(1.0), hot * 0.9) * (1.0 + 0.8 * hot);

    var pbr_input = pbr_input_new();
    pbr_input.world_position = in.world_position;
    pbr_input.frag_coord = in.position;
    var out: FragmentOutput;
    out.color = main_pass_post_lighting_processing(pbr_input, vec4<f32>(colour, a));
#ifdef SWING_ALPHA
    out.color = vec4<f32>(vec3<f32>(a), 1.0);
#endif
#ifdef SWING_UV
    out.color = vec4<f32>(u, v, glow, 1.0);
#endif
    return out;
}
