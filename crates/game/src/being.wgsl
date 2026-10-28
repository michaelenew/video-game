// **The seal the Dual mage's beings rise from** (`being.rs`,
// `view::being`): a disc round her, at her waist or her feet, split down her
// middle -- dark on her left, light on her right. A faint ring whatever the
// bars; each half's rings and runes lit round from behind her toward the
// front as its bar fills, and a band of it washed in its force. The beings
// themselves are their painted models, veiled (`skin.wgsl`).

#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    mesh_view_bindings::globals,
    pbr_functions::main_pass_post_lighting_processing,
    pbr_types::pbr_input_new,
}

struct SpiritLook {
    fill: vec4<f32>,
    left: vec4<f32>,
    feet: vec4<f32>,
    dark_shoulder: vec4<f32>,
    dark_fist: vec4<f32>,
    light_shoulder: vec4<f32>,
    light_fist: vec4<f32>,
    head: vec4<f32>,
    order: vec4<f32>,
    reach: vec4<f32>,
}

@group(2) @binding(0) var<uniform> spirit: SpiritLook;

const PI: f32 = 3.14159265;
const TAU: f32 = 6.28318531;

// Each force's two colours: the veil's own, and its light at the edge.
const DARK_BODY: vec3<f32> = vec3<f32>(0.035, 0.012, 0.07);
const DARK_GLOW: vec3<f32> = vec3<f32>(0.62, 0.30, 1.25);
const LIGHT_BODY: vec3<f32> = vec3<f32>(1.0, 0.93, 0.74);
const LIGHT_GLOW: vec3<f32> = vec3<f32>(1.35, 1.15, 0.72);

fn hash1(x: f32) -> f32 {
    return fract(sin(x * 127.1 + 311.7) * 43758.5453);
}

fn finish(in: VertexOutput, colour: vec3<f32>, a: f32) -> FragmentOutput {
    var pbr_input = pbr_input_new();
    pbr_input.world_position = in.world_position;
    pbr_input.frag_coord = in.position;
    var out: FragmentOutput;
    out.color = main_pass_post_lighting_processing(pbr_input, vec4<f32>(colour, clamp(a, 0.0, 1.0)));
    return out;
}

fn seal(in: VertexOutput) -> FragmentOutput {
    let p = in.world_position.xyz;
    let c = spirit.feet.xyz;
    let radius = spirit.feet.w;
    let rel = vec2<f32>(p.x - c.x, p.z - c.z) / radius;
    let r = length(rel);
    if (r > 1.0) {
        discard;
    }
    let left = normalize(vec2<f32>(spirit.left.x, spirit.left.z));
    // Her facing: up crossed with her left.
    let ahead = vec2<f32>(left.y, -left.x);
    let side = dot(rel, left);
    let dark = smoothstep(-0.01, 0.01, side);
    // Round each half from behind her (0) to in front of her (1).
    let around = atan2(abs(side), -dot(rel, ahead)) / PI;
    let fill = mix(spirit.fill.y, spirit.fill.x, dark);
    let asc = spirit.fill.z;
    let t = globals.time;
    // How lit this part of the half is: the arc its bar has reached.
    let lit = 1.0 - smoothstep(fill * 1.04 - 0.06, fill * 1.04, around);

    let line = rings(r);
    // Runes between the two outer rings, a glyph each twenty-fourth of the
    // way round, turning slowly -- the dark half one way, the light the other.
    let spin = t * 0.05 * (1.0 + 2.0 * asc) * mix(1.0, -1.0, dark);
    let a = atan2(rel.y, rel.x) + spin;
    let cell = fract(a / TAU) * 24.0;
    let k = floor(cell);
    let f = fract(cell);
    var runes = 0.0;
    if (r > 0.64 && r < 0.86) {
        let h = (r - 0.64) / 0.22;
        let s1 = clamp(1.0 - abs(f - (0.3 + 0.4 * hash1(k * 3.0))) / 0.06, 0.0, 1.0)
            * step(0.15, h) * step(h, 0.85);
        let s2 = clamp(1.0 - abs(h - (0.2 + 0.6 * hash1(k * 5.0))) / 0.09, 0.0, 1.0)
            * step(0.2, f) * step(f, 0.8);
        runes = max(s1, s2);
    }
    // The line between the halves, the being's own midline.
    let seam = clamp(1.0 - abs(side) / 0.012, 0.0, 1.0) * step(r, 0.96);
    let ink = clamp(line + runes, 0.0, 1.0);
    // Faint whatever the bars, so the seal is always there to be filled.
    let faint = 0.18 * line;
    let shown = max(faint, ink * lit * (0.55 + 0.45 * fill)) + seam * 0.6;
    // Each half's floor washed in its force, as far round as it is lit --
    // a band, hollow in the middle, where she stands.
    let wash = lit * (0.12 + 0.22 * fill) * smoothstep(1.0, 0.85, r) * smoothstep(0.42, 0.56, r);
    let glow = mix(LIGHT_GLOW, DARK_GLOW, dark) * (1.2 + 1.2 * asc);
    let ground = mix(LIGHT_BODY * 0.6, DARK_BODY, dark);
    let colour = mix(ground, glow, clamp(shown / max(shown + wash, 0.001), 0.0, 1.0));
    return finish(in, colour, clamp(shown + wash, 0.0, 0.95));
}

// The rings: two close together at the rim, two round the runes' inner edge.
fn rings(r: f32) -> f32 {
    var at = array<vec2<f32>, 4>(
        vec2<f32>(0.965, 0.022),
        vec2<f32>(0.885, 0.012),
        vec2<f32>(0.62, 0.016),
        vec2<f32>(0.55, 0.01),
    );
    var sum = 0.0;
    for (var i = 0; i < 4; i = i + 1) {
        sum = sum + clamp(1.0 - abs(r - at[i].x) / at[i].y, 0.0, 1.0);
    }
    return sum;
}

@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    return seal(in);
}
