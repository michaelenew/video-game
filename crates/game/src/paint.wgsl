// **The flat shade of everything that stands still**: the arenas' boxes, the
// land, the town, the trees. See `paint.rs` for why there is one.
//
// A vertex colour, or the material's own, times the light: the ambient, and
// each directional light by how squarely the surface faces it, through its
// shadow. Then the fog and the tonemapper the standard material would apply,
// by the same function, so the air and the response curve are the same as
// everything else's -- `look::palette::lit` was measured off that curve.
//
// The light is Bevy's own diffuse term (Burley's, as the standard material
// has it) and its ambient approximation, so a surface comes out the colour
// it did under the standard material; what is left out is the specular
// lobe, which on matte stone and grass was a sheen nobody could see, and
// everything the standard material reads that these surfaces never set --
// metal, reflectance, emission, clearcoat, transmission, the texture
// lookups -- which is most of the per-pixel cost.

#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    mesh_view_bindings::{view, lights},
    mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
    pbr_functions::main_pass_post_lighting_processing,
    lighting::{EnvBRDFApprox, F_AB, F_Schlick},
    pbr_types::{pbr_input_new, STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT},
    shadows::fetch_directional_shadow,
}
#import game::cut::window

struct Paint {
    color: vec4<f32>,
    // A hole along the line from the eye to `cut.xyz`, `cut.w` across at
    // that end and narrower toward the eye; none where `cut.w` is zero.
    cut: vec4<f32>,
}

@group(2) @binding(0) var<uniform> paint: Paint;
// The brush strokes, baked (`look::strokes`): four tiling sheets, one a
// rigidity, each texel a stroke's shade and two colour shifts.
@group(2) @binding(1) var strokes_tex: texture_2d_array<f32>;
@group(2) @binding(2) var strokes_sampler: sampler;
// How many stroke lengths a sheet is across (`look::strokes::PERIOD`), and
// how many angles a rigidity is drawn at (`look::strokes::ANGLES`).
const STROKE_PERIOD: f32 = 8.0;
const STROKE_ANGLES: f32 = 6.0;
const STROKE_ANGLES_I: i32 = 6;
// How far from the eye the brush is worked out at the pixel, in metres
// (`WASH_NEAR_EXACT`, thousandths): negative is everywhere, 0 is never.
const NEAR_EXACT: f32 = f32(#{WASH_NEAR_EXACT}) / 1000.0;

const PI: f32 = 3.14159265;
// A matte surface: the standard material's perceptual roughness 0.92,
// squared, as the white material these surfaces used to be drawn in had
// it. The sheen a surface keeps in the ambient is read from the same.
const ROUGHNESS: f32 = 0.85;
const PERCEPTUAL_ROUGHNESS: f32 = 0.92;
const F0: f32 = 0.04;
// How much of the wood is left in the middle of the window on to the
// fighter (`Paint::ghost`): enough to read as the tree, little enough to see
// them through it.
const GHOST_ALPHA: f32 = 0.3;

// **The wash** (`look::wash`, the surface half): the light laid down as a few
// flat washes with a brushed border between them, a cool shade and a warm
// light, and pigment that wanders. Handed in as shader defs in thousandths,
// because the treatment is chosen once per run and a constant folds away.
const BANDS: f32 = f32(#{WASH_BANDS}) / 1000.0;
const TONES: f32 = f32(#{WASH_TONES}) / 1000.0;
const SOFT: f32 = f32(#{WASH_SOFT}) / 1000.0;
const STROKE: f32 = f32(#{WASH_STROKE}) / 1000.0;
const STROKE_LEN: f32 = f32(#{WASH_STROKE_LEN}) / 1000.0;
const SHADE: vec3<f32> = vec3<f32>(f32(#{WASH_SHADE_R}), f32(#{WASH_SHADE_G}), f32(#{WASH_SHADE_B})) / 1000.0;
const LIGHT: vec3<f32> = vec3<f32>(f32(#{WASH_LIGHT_R}), f32(#{WASH_LIGHT_G}), f32(#{WASH_LIGHT_B})) / 1000.0;
const MOTTLE: f32 = f32(#{WASH_MOTTLE}) / 1000.0;
const DILUTE: f32 = f32(#{WASH_DILUTE}) / 1000.0;
const TOP: f32 = f32(#{WASH_TOP}) / 1000.0;
const GRAIN: f32 = f32(#{WASH_GRAIN}) / 1000.0;
const BROAD: f32 = f32(#{WASH_BROAD}) / 1000.0;
const BROAD_SIZE: f32 = f32(#{WASH_BROAD_SIZE}) / 1000.0;
const RICH: f32 = f32(#{WASH_RICH}) / 1000.0;
const HOLLOW: vec3<f32> = vec3<f32>(f32(#{WASH_HOLLOW_R}), f32(#{WASH_HOLLOW_G}), f32(#{WASH_HOLLOW_B})) / 1000.0;
const CREST: vec3<f32> = vec3<f32>(f32(#{WASH_CREST_R}), f32(#{WASH_CREST_G}), f32(#{WASH_CREST_B})) / 1000.0;

fn hash2(p: vec2<f32>) -> f32 {
    var q = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    q = q + dot(q, q.yzx + 33.33);
    return fract((q.x + q.y) * q.z);
}

fn noise2(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash2(i), hash2(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash2(i + vec2<f32>(0.0, 1.0)), hash2(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y,
    );
}

fn hash3(p: vec3<f32>) -> f32 {
    var q = fract(p * 0.1031);
    q = q + dot(q, q.zyx + 31.32);
    return fract((q.x + q.y) * q.z);
}

fn noise3(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = mix(hash3(i), hash3(i + vec3<f32>(1.0, 0.0, 0.0)), u.x);
    let b = mix(hash3(i + vec3<f32>(0.0, 1.0, 0.0)), hash3(i + vec3<f32>(1.0, 1.0, 0.0)), u.x);
    let c = mix(hash3(i + vec3<f32>(0.0, 0.0, 1.0)), hash3(i + vec3<f32>(1.0, 0.0, 1.0)), u.x);
    let d = mix(hash3(i + vec3<f32>(0.0, 1.0, 1.0)), hash3(i + vec3<f32>(1.0, 1.0, 1.0)), u.x);
    return mix(mix(a, b, u.y), mix(c, d, u.y), u.z);
}

/// Where the brush went, -1 to 1: strokes stretched along the surface, laid
/// on the plane it most nearly faces. Across a wall they run level, the way
/// a wall is washed; on the ground they run on a slant. Each stroke's angle
/// wanders a little, so they are a hand's and not a ruler's.
fn brush(p: vec3<f32>, n: vec3<f32>) -> f32 {
    let a = abs(n);
    var uv = p.xy;
    var tilt = 0.0;
    if (a.y > max(a.x, a.z)) {
        uv = p.xz;
        tilt = 0.6;
    } else if (a.x > a.z) {
        uv = p.zy;
    }
    let turn = tilt + 0.2 * (noise2(uv * 0.17) - 0.5);
    let c = cos(turn);
    let s = sin(turn);
    let r = vec2<f32>(c * uv.x + s * uv.y, -s * uv.x + c * uv.y);
    let q = vec2<f32>(r.x / STROKE_LEN, r.y * 6.0 / STROKE_LEN);
    let v = 0.62 * noise2(q) + 0.38 * noise2(q * 2.3 + vec2<f32>(5.2, 1.3));
    return v * 2.0 - 1.0;
}

/// **Dabs**: the marks a flat brush leaves, -1 to 1. The surface is laid in
/// rows a sixth of a dab long, each row offset like brickwork, each dab its
/// own shade with soft ends and soft sides where it overlaps its neighbours,
/// and a little streak along it where the bristles dragged.
fn dabs(p: vec3<f32>, n: vec3<f32>, len: f32) -> f32 {
    let a = abs(n);
    var uv = p.xy;
    var tilt = 0.12;
    if (a.y > max(a.x, a.z)) {
        uv = p.xz;
        tilt = 0.7;
    } else if (a.x > a.z) {
        uv = p.zy;
    }
    let c = cos(tilt);
    let s = sin(tilt);
    let r = vec2<f32>(c * uv.x + s * uv.y, -s * uv.x + c * uv.y) / len;
    let width = 0.32;
    let row = floor(r.y / width);
    let fy = fract(r.y / width);
    let x = r.x + hash2(vec2<f32>(row, 3.0)) * 7.0;
    let cell = floor(x);
    let fx = fract(x);
    let shade = hash2(vec2<f32>(row, cell)) * 2.0 - 1.0;
    let next = hash2(vec2<f32>(row, cell + 1.0)) * 2.0 - 1.0;
    let up = hash2(vec2<f32>(row + 1.0, floor(x + hash2(vec2<f32>(row + 1.0, 3.0)) * 7.0 - hash2(vec2<f32>(row, 3.0)) * 7.0))) * 2.0 - 1.0;
    var v = mix(shade, next, smoothstep(0.55, 1.0, fx));
    v = mix(v, up, smoothstep(0.5, 1.0, fy) * 0.5);
    let bristle = noise2(vec2<f32>(x * 3.0, r.y / width * 9.0)) - 0.5;
    return clamp(v * 0.8 + bristle * 0.5, -1.0, 1.0);
}

/// **Which way the brush runs here**, in the surface: along `flow` where the
/// mesh gave one (a road), else uphill on a slope, along the courses on an
/// upright built face, and on the flat a direction that wanders slowly.
fn stroke_way(p: vec3<f32>, n: vec3<f32>, flow: vec2<f32>, rigid: f32) -> vec3<f32> {
    let up = vec3<f32>(0.0, 1.0, 0.0);
    // Level along the face: the way a course of stone runs.
    var level = cross(up, n);
    if (length(level) < 1e-3) {
        level = vec3<f32>(1.0, 0.0, 0.0);
    }
    level = normalize(level);
    if (length(flow) > 0.1) {
        let f = vec3<f32>(flow.x, 0.0, flow.y);
        let on = f - n * dot(f, n);
        if (length(on) > 1e-3) {
            return normalize(on);
        }
    }
    // Uphill: straight up, flattened into the surface.
    let lean = sqrt(max(1.0 - n.y * n.y, 0.0));
    var uphill = level;
    if (lean > 1e-3) {
        uphill = normalize(up - n * n.y);
    }
    // On the flat, a slow wander, so a meadow is not combed one way.
    let a = noise2(p.xz / 38.0) * 6.28;
    var wander = vec3<f32>(cos(a), 0.0, sin(a));
    wander = wander - n * dot(wander, n);
    if (dot(wander, uphill) < 0.0) {
        wander = -wander;
    }
    let on_slope = smoothstep(0.03, 0.15, lean);
    let natural = normalize(mix(normalize(wander + n * 1e-4), uphill, on_slope) + n * 1e-5);
    // Built work runs along its courses however it is turned; nature runs
    // uphill. An upright face is built unless it was marked otherwise.
    let built = smoothstep(0.55, 0.8, rigid) * (1.0 - smoothstep(0.5, 0.8, abs(n.y)));
    return normalize(mix(natural, level, built));
}

/// **One layer of strokes**, each `len` metres long at its usual size: the
/// baked sheet (`look::strokes`, which says what a stroke is) read on the
/// plane the surface most nearly faces, scaled to `len`, from the sheet
/// nearest this rigidity and one of the two angle sheets nearest the way the
/// strokes run here. Four texels where there were thirty-six strokes worked
/// out a pixel.
///
/// **The lookup is never turned.** Turning it about the origin to point the
/// strokes along `way` moved the sample by the distance from the origin times
/// the field's change between two pixels -- metres, far out on the map -- and
/// smeared every stroke on the ground into a streak. The angle sheets hold
/// the same strokes turned about their own middles; which of the two nearest
/// a pixel shows is decided by the stroke's own phase, never by a blend.
///
/// Returns shade, warmth and a second colour axis (each -1 to 1).
fn strokes(p: vec3<f32>, n: vec3<f32>, way: vec3<f32>, len: f32, rigid: f32) -> vec3<f32> {
    let a = abs(n);
    var u_axis = vec3<f32>(1.0, 0.0, 0.0);
    var v_axis = vec3<f32>(0.0, 1.0, 0.0);
    if (a.y > max(a.x, a.z)) {
        v_axis = vec3<f32>(0.0, 0.0, 1.0);
    } else if (a.x > a.z) {
        u_axis = vec3<f32>(0.0, 0.0, 1.0);
    }
    let q = vec2<f32>(dot(p, u_axis), dot(p, v_axis));
    var d = vec2<f32>(dot(way, u_axis), dot(way, v_axis));
    if (length(d) < 1e-4) {
        d = vec2<f32>(1.0, 0.0);
    }
    let uv = q / (len * STROKE_PERIOD);
    // `look::strokes::RIGIDITIES`: 0, 0.25, 0.5, 0.85, nearest.
    var sheet = 3;
    if (rigid < 0.125) {
        sheet = 0;
    } else if (rigid < 0.375) {
        sheet = 1;
    } else if (rigid < 0.675) {
        sheet = 2;
    }
    // The angle, over a half turn (a stroke turned half a turn is the same
    // stroke), as a place between two of the angle sheets.
    var turn = atan2(d.y, d.x);
    if (turn < 0.0) {
        turn = turn + PI;
    }
    let place = turn / PI * STROKE_ANGLES;
    let lo = i32(floor(place)) % STROKE_ANGLES_I;
    let hi = (lo + 1) % STROKE_ANGLES_I;
    let t = fract(place);
    let base = sheet * STROKE_ANGLES_I;
    let one = textureSampleLevel(strokes_tex, strokes_sampler, uv, base + lo, 0.0);
    let two = textureSampleLevel(strokes_tex, strokes_sampler, uv, base + hi, 0.0);
    // One or the other, never a blend: the earlier sheet until the place
    // between the two passes the phase of the stroke it shows here, then the
    // later. Each stroke switches at its own moment, so nothing lines up into
    // a seam, and a stroke is always a whole hard stroke.
    if (t < one.a) {
        return one.rgb * 2.0 - 1.0;
    }
    return two.rgb * 2.0 - 1.0;
}

/// **The brush worked out at the pixel** -- the painting: every candidate
/// stroke's place, angle, size, curve, dry ends and bristles, for this very
/// pixel, and every stroke the same size on the screen whatever its
/// distance, as a brush is the same size on a canvas. The sheets below
/// approximate it and read as a decal: the owner's call, 2026-10-16. The
/// sheets are read beyond `NEAR_EXACT` metres, or everywhere with
/// `--exact 0`, for a machine that cannot afford this.
fn strokes_exact(p: vec3<f32>, n: vec3<f32>, way: vec3<f32>, len: f32, rigid: f32) -> vec3<f32> {
    let a = abs(n);
    var u_axis = vec3<f32>(1.0, 0.0, 0.0);
    var v_axis = vec3<f32>(0.0, 1.0, 0.0);
    if (a.y > max(a.x, a.z)) {
        v_axis = vec3<f32>(0.0, 0.0, 1.0);
    } else if (a.x > a.z) {
        u_axis = vec3<f32>(0.0, 0.0, 1.0);
    }
    let q = vec2<f32>(dot(p, u_axis), dot(p, v_axis));
    var d = vec2<f32>(dot(way, u_axis), dot(way, v_axis));
    if (length(d) < 1e-4) {
        d = vec2<f32>(1.0, 0.0);
    }
    let base_angle = atan2(d.y, d.x);
    let loose = 1.0 - rigid;
    let cell = len * 0.5;
    let home = floor(q / cell);
    var best = -1.0;
    var out = vec3<f32>(0.0);
    for (var j = -1; j <= 1; j = j + 1) {
        for (var i = -1; i <= 1; i = i + 1) {
            let c = home + vec2<f32>(f32(i), f32(j));
            for (var k = 0; k < 2; k = k + 1) {
                let seed = c + vec2<f32>(f32(k) * 17.13, f32(k) * 31.71);
                let h1 = hash2(seed);
                let h2 = hash2(seed + 1.37);
                let h3 = hash2(seed + 2.71);
                let h4 = hash2(seed + 3.97);
                let h5 = hash2(seed + 5.21);
                let h6 = hash2(seed + 6.83);
                // Where it sits: on the grid when ruled, anywhere in its cell
                // when loose.
                let centre = (c + 0.5 + (vec2<f32>(h1, h2) - 0.5) * mix(0.2, 1.0, loose)) * cell;
                let angle = base_angle + (h3 - 0.5) * loose * 1.6;
                let size = 1.0 + (h4 - 0.5) * loose * 1.4;
                let long = len * size * mix(0.5, 0.55, h5);
                let wide = len * size * mix(0.15, 0.13 + 0.12 * h6, loose);
                let curve = (h5 - 0.5) * loose * 0.8;
                let cs = cos(angle);
                let sn = sin(angle);
                let rel = q - centre;
                let local = vec2<f32>(cs * rel.x + sn * rel.y, -sn * rel.x + cs * rel.y);
                let u = local.x / long;
                if (abs(u) > 1.2) {
                    continue;
                }
                let v = (local.y - curve * wide * u * u) / wide;
                // Ends: square and ruled, or tapered and dry, by how loose.
                let taper = mix(1.0, sqrt(max(1.0 - u * u, 0.0)), 0.25 + 0.6 * loose);
                let bristle = noise2(vec2<f32>(u * 4.0 + h1 * 9.0, v * 5.0 + h2 * 9.0)) - 0.5;
                let inside = abs(v) < taper * (1.0 + bristle * 0.35 * loose)
                    && abs(u) < 1.0 + bristle * 0.5;
                let order = hash2(seed + 8.09);
                if (inside && order > best) {
                    best = order;
                    out = vec3<f32>(
                        hash2(seed + 9.41) * 2.0 - 1.0,
                        hash2(seed + 10.7) * 2.0 - 1.0,
                        hash2(seed + 11.3) * 2.0 - 1.0,
                    ) * (0.85 + 0.15 * bristle * 2.0);
                }
            }
        }
    }
    return out;
}

/// `broad_strokes` with the brush worked out at the pixel.
fn broad_exact(p: vec3<f32>, n: vec3<f32>, way: vec3<f32>, dist: f32, rigid: f32, size: f32) -> vec3<f32> {
    let l = log2(max(BROAD_SIZE * dist * size, 0.05));
    let lo = floor(l);
    let t = l - lo;
    let one = strokes_exact(p, n, way, exp2(lo), rigid);
    let two = strokes_exact(p + vec3<f32>(17.3, 5.1, 9.7), n, way, exp2(lo + 1.0), rigid);
    return mix(one, two, t) / sqrt(t * t + (1.0 - t) * (1.0 - t));
}

/// **Strokes that hold still and keep their size.** Fixed to the world so
/// they do not swim when the camera moves; their length picked from the
/// distance in powers of two, the two nearest cross-faded, so a stroke stays
/// about the same size on screen at any range (Bénard et al., "Dynamic 2D
/// Patterns for Shading 3D Scenes", 2010). `size` scales them per surface.
fn broad_strokes(p: vec3<f32>, n: vec3<f32>, way: vec3<f32>, dist: f32, rigid: f32, size: f32) -> vec3<f32> {
    let l = log2(max(BROAD_SIZE * dist * size, 0.05));
    let lo = floor(l);
    let t = l - lo;
    let one = strokes(p, n, way, exp2(lo), rigid);
    let two = strokes(p + vec3<f32>(17.3, 5.1, 9.7), n, way, exp2(lo + 1.0), rigid);
    return mix(one, two, t) / sqrt(t * t + (1.0 - t) * (1.0 - t));
}

/// The light as washes: `TONES` flat levels from shade to light, the step
/// between two of them `SOFT` wide.
fn washed(x: f32) -> f32 {
    let steps = max(TONES - 1.0, 1.0);
    let y = clamp(x, 0.0, 1.0) * steps;
    let f = min(floor(y), steps - 1.0);
    let w = clamp(SOFT * steps, 0.001, 0.5);
    let s = smoothstep(0.5 - w, 0.5 + w, y - f);
    return (f + s) / steps;
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    // **Out of the way of the fighter** (`Paint::cut`): what stands between
    // the eye and them, near the line, is not drawn solid. The ghost
    // (`Paint::ghost`, `PAINT_GHOST`) draws exactly that part again, see-
    // through: thinnest in the middle and solid at the rim, where the solid
    // wood takes over.
    let in_window = window(paint.cut, view.world_position, in.world_position.xyz);
#ifdef PAINT_GHOST
    if (in_window <= 0.0) {
        discard;
    }
#else
    if (in_window > 0.0) {
        discard;
    }
#endif
    var base = paint.color;
#ifdef VERTEX_COLORS
    base = base * in.color;
#endif
    var n = normalize(in.world_normal);
    if (!is_front) {
        n = -n;
    }
    let p = in.world_position.xyz;

    // **The pigment wanders**: a little lighter and darker over a few metres,
    // and a little warmer and cooler over a few more, as no wash goes on even.
    if (MOTTLE > 0.0) {
        let v = noise3(p * 0.21) * 0.7 + noise3(p * 0.63 + 11.0) * 0.3;
        let h = noise3(p * 0.09 + 23.0) - 0.5;
        base = vec4<f32>(
            base.rgb * (1.0 + MOTTLE * 2.0 * (v - 0.5)) * (1.0 + MOTTLE * vec3<f32>(h, 0.0, -h)),
            base.a,
        );
    }
    // **The brush's own marks**: short strokes laid along the surface, a
    // little lighter and darker and a touch warmer or cooler each, so a face
    // reads as painted rather than filled.
    if (GRAIN > 0.0) {
        let v = 0.7 * dabs(p, n, 0.55) + 0.3 * dabs(p + vec3<f32>(7.3, 1.1, 4.7), n, 0.23);
        // Softer on the ground, where a dab is seen from a slant and
        // its ends crowd into blocks.
        let g = GRAIN * (1.0 - 0.5 * smoothstep(0.6, 0.95, n.y));
        base = vec4<f32>(base.rgb * (1.0 + g * v) * (1.0 + g * 0.5 * vec3<f32>(v, 0.0, -v)), base.a);
    }
    if (BROAD > 0.0) {
        // What kind of surface this is, for the brush: the mesh's word if it
        // gave one (`land::ground_stroke`, `shapes::lay_natural`), else read
        // off the face -- upright is built, flat is open ground.
        var flow = vec2<f32>(0.0);
        var rigid = mix(0.85, 0.3, smoothstep(0.5, 0.85, abs(n.y)));
        var size = mix(0.7, 1.2, smoothstep(0.5, 0.85, abs(n.y)));
#ifdef VERTEX_UVS_A
        flow = in.uv;
#endif
#ifdef VERTEX_UVS_B
        rigid = in.uv_b.x;
        size = in.uv_b.y;
#endif
        let dist = length(view.world_position - p);
        let way = stroke_way(p, n, flow, rigid);
        var b: vec3<f32>;
        if (NEAR_EXACT < 0.0 || dist < NEAR_EXACT) {
            b = broad_exact(p, n, way, dist, rigid, size);
        } else {
            b = broad_strokes(p, n, way, dist, rigid, size);
        }
        // The looser the brush the further its colours wander: a meadow's
        // strokes go yellow, blue and dark; a wall's stay close to its own.
        let wander = BROAD * mix(0.45, 1.0, 1.0 - rigid);
        let shift = vec3<f32>(b.y, 0.15 * b.y, -b.y) * 0.3 + vec3<f32>(0.25 * b.z, 0.2 * b.z, -0.35 * b.z) * (1.0 - rigid);
        base = vec4<f32>(base.rgb * (1.0 + wander * 0.4 * b.x) * (1.0 + wander * shift), base.a);
    }
    // **Colour that follows the land's contours**: a pinch of blue-violet
    // laid into its hollows and folds and a pinch of red-orange onto its
    // crests -- not to make them blue or red but to deepen them, the salt in
    // the painting -- so the colour itself models the form. The bend is the
    // land's own (`land::bend`), smooth over metres, so the tint is too.
#ifdef VERTEX_TANGENTS
    if (RICH > 0.0) {
        let bend = in.world_tangent.w;
        let hollow = smoothstep(0.03, 0.5, bend);
        let crest = smoothstep(0.03, 0.5, -bend);
        // Multiplied, and a little added: a multiple of nearly black is
        // still nearly black, so a dark rock would take no seasoning at all.
        let lift = (HOLLOW - 1.0) * hollow + (CREST - 1.0) * crest;
        base = vec4<f32>(
            max(
                base.rgb * mix(vec3<f32>(1.0), HOLLOW, hollow * RICH) * mix(vec3<f32>(1.0), CREST, crest * RICH)
                    + lift * RICH * 0.035,
                vec3<f32>(0.0),
            ),
            base.a,
        );
    }
#endif
    var stroke = 0.0;
    if (STROKE > 0.0) {
        stroke = brush(p, n) * STROKE;
    }
    var key = 0.0;
    var key_strength = 0.0;
    let v = normalize(view.world_position - in.world_position.xyz);
    let ndotv = max(dot(n, v), 0.0001);
    let view_z = dot(vec4<f32>(
        view.view_from_world[0].z,
        view.view_from_world[1].z,
        view.view_from_world[2].z,
        view.view_from_world[3].z
    ), in.world_position);

    // The ambient, as the standard material approximates it.
    let ambient = EnvBRDFApprox(base.rgb, F_AB(1.0, ndotv))
        + EnvBRDFApprox(vec3(F0), F_AB(PERCEPTUAL_ROUGHNESS, ndotv));
    var lit = ambient * lights.ambient_color.rgb;

    for (var i = 0u; i < lights.n_directional_lights; i++) {
        let light = &lights.directional_lights[i];
        let l = (*light).direction_to_light.xyz;
        let ndotl = saturate(dot(n, l));
        if (ndotl <= 0.0 && BANDS <= 0.0) {
            continue;
        }
        var shadow = 1.0;
        if (ndotl > 0.0 && ((*light).flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u) {
            shadow = fetch_directional_shadow(i, in.world_position, n, view_z);
        }
        // How much of this light the surface takes: as it was, or laid down
        // as washes with the brush pushing the border between them about.
        let smooth_light = ndotl * shadow;
        let reach = mix(smooth_light, washed(smooth_light + stroke), BANDS);
        let strength = dot((*light).color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        if (strength > key_strength) {
            key_strength = strength;
            key = reach;
        }
        // Burley's diffuse, as `pbr_lighting::Fd_Burley`.
        let h = normalize(l + v);
        let ldoth = saturate(dot(l, h));
        let f90 = 0.5 + 2.0 * ROUGHNESS * ldoth * ldoth;
        let burley = F_Schlick(1.0, f90, ndotl) * F_Schlick(1.0, f90, ndotv) / PI;
        lit += base.rgb * burley * reach * (*light).color.rgb;
    }
    // The shade is mixed from the sky and the light from the sun: one tint
    // across the whole surface, chosen by how far into the light it is.
    lit = lit * mix(SHADE, LIGHT, key);
    // **Dilution** (MNPR): the lit wash is thinned toward the paper -- paler
    // and lighter, never whiter than the sheet -- which is what makes a
    // watercolour's light luminous instead of merely bright.
    // **Tops are lit by the sky**: a face paler the more it looks up, so the
    // top of every stone and step reads against its side.
    lit = lit * (1.0 + TOP * smoothstep(0.35, 0.95, n.y));
    let pale = dot(lit, vec3<f32>(0.2126, 0.7152, 0.0722));
    lit = mix(lit, mix(lit, vec3<f32>(pale), 0.45) * 1.35, DILUTE * key);

    var pbr_input = pbr_input_new();
    pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT;
    pbr_input.world_position = in.world_position;
    pbr_input.frag_coord = in.position;
    var out: FragmentOutput;
    out.color = main_pass_post_lighting_processing(pbr_input, vec4(lit * view.exposure, base.a));
#ifdef PAINT_GHOST
    out.color.a = mix(1.0, GHOST_ALPHA, in_window);
#endif
    return out;
}
