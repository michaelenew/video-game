// **The arc a Champion's cut leaves** (`swing.rs`): a strip laid along the
// path the weapon's volume took, painted here from two numbers a vertex.
//
// `uv.x` is how far along the arc a point is, 0 at the tail (where the cut
// began) and 1 at the head (where the weapon is now); `uv.y` which edge it is
// nearer, 0 the inner and 1 the outer, the one the blade's tip drew. The
// vertex colour is the weapon's tint, and in its alpha the fade: one while the
// cut is out, running to nothing through the recovery.
//
// What it draws, in the order it is built:
// - **Erosion**: as it fades, the tail is eaten up toward the head, so a cut
//   goes the way it came rather than everywhere at once.
// - **A crescent**: full width at the head, drawn to a point at the tail, as
//   a brush stroke is -- the shape every drawn sword arc has.
// - **Streaks**: the body is fine lanes running along the arc, each fraying
//   off at its own place near the tail, so the end is wisps, not a cut line.
// - **The edge**: a thin hot line where the tip passed, whitest at the head.
// - **A flare** at the head, where the weapon is.

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
    var fade = 1.0;
#ifdef VERTEX_UVS_A
    u = in.uv.x;
    v = in.uv.y;
#endif
#ifdef VERTEX_COLORS
    tint = in.color.rgb;
    fade = in.color.a;
#endif
    let gone = 1.0 - fade;

    // Erosion: `life` is how far along what is left of the arc a point is.
    let tail = gone * 0.85;
    let life = clamp((u - tail) / max(1.0 - tail, 1e-3), 0.0, 1.0);

    // How far in from the outer edge, 0 on it and 1 at the inner.
    let depth = 1.0 - v;

    // The crescent: as wide as the band at the head, a sliver at the tail,
    // and thinner as it fades.
    let width = mix(0.15, 1.0, pow(life, 0.5)) * mix(1.0, 0.55, gone);
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
    let lead = smoothstep(0.82, 1.0, life) * (1.0 - smoothstep(0.0, 0.6, depth));

    // Older is fainter: solid through the newer part, thinning to nothing
    // over the oldest.
    let along = smoothstep(0.0, 0.65, life);
    var a = ((body + edge) * along + 0.3 * lead) * inside * fade;
    // Never a hard line at the tail's very end or the inner edge.
    // Nor at the head once the weapon has gone on: there the arc rounds off
    // as it thins.
    let cap = mix(0.995, 0.85, gone);
    a = a * smoothstep(0.0, 0.04, life) * smoothstep(0.0, 0.08, v) * (1.0 - smoothstep(cap, 1.0, u) * gone);
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
    out.color = vec4<f32>(u, v, fade, 1.0);
#endif
    return out;
}
