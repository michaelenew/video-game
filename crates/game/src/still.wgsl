// **The still world's shadow** (`still.rs`): the depth along the sun of the
// first still thing, drawn once into a map square to the sun, and read here
// by the paint and the fighters' skin.

#define_import_path game::still

// Where the map is (`still::Shade`): the point its camera stands at and the
// map's size; its right and up, which are the map's axes, and how far its
// camera sees; whether there is a map; the way the sunlight goes, and which
// of the two maps to read.
struct Shade {
    origin: vec4<f32>,
    right: vec4<f32>,
    up: vec4<f32>,
    along: vec4<f32>,
}

// How far the map's camera sees, in metres: `still::FAR`.
const FAR: f32 = 2000.0;
// A depth is kept in two half-floats: the whole number of 2048ths, and what
// is left over, in 2048ths of one of those -- about half a millimetre over
// the camera's two kilometres.
const SPLIT: f32 = 2048.0;

// **What the map's camera writes**: a depth, 0 at the camera and 1 as far as
// it sees, in two halves.
fn encode(metres: f32) -> vec4<f32> {
    let d = clamp(metres / FAR, 0.0, 1.0);
    let hi = floor(d * SPLIT) / SPLIT;
    return vec4<f32>(hi, (d - hi) * SPLIT, 0.0, 1.0);
}

// One texel's verdict: is a point this deep in front of what the map holds
// there.
fn tap(a: texture_2d<f32>, b: texture_2d<f32>, which: f32, t: vec2<i32>, d: f32) -> f32 {
    var v: vec4<f32>;
    if (which < 0.5) {
        v = textureLoad(a, t, 0);
    } else {
        v = textureLoad(b, t, 0);
    }
    return select(0.0, 1.0, d <= v.x + v.y / SPLIT);
}

// Four texels' verdicts round `x` (in texels, centres at the halves),
// weighted by how near each is.
fn bilinear(a: texture_2d<f32>, b: texture_2d<f32>, w: f32, x: vec2<f32>, d: f32, top: i32) -> f32 {
    let corner = floor(x - 0.5);
    let f = x - 0.5 - corner;
    let i = clamp(vec2<i32>(corner), vec2<i32>(0), vec2<i32>(top));
    let j = clamp(vec2<i32>(corner) + 1, vec2<i32>(0), vec2<i32>(top));
    return mix(
        mix(tap(a, b, w, i, d), tap(a, b, w, vec2<i32>(j.x, i.y), d), f.x),
        mix(tap(a, b, w, vec2<i32>(i.x, j.y), d), tap(a, b, w, j, d), f.x),
        f.y,
    );
}

// **How much of the sun reaches `p`** past the still world, 0 to 1: the
// verdicts of the texels round it, four bilinear readings half a texel
// either side averaged, so a shadow's edge is a slope two texels wide (40
// cm) rather than a stair. 1 where there is no map, and faded to 1 over the
// map's last few metres, so its edge is not a line.
fn shade(s: Shade, a: texture_2d<f32>, b: texture_2d<f32>, p: vec3<f32>, n: vec3<f32>) -> f32 {
    if (s.up.w < 0.5) {
        return 1.0;
    }
    let size = s.origin.w;
    let texels = f32(textureDimensions(a).x);
    let texel = size / texels;
    // Off the surface by two texels along its normal, and a few
    // centimetres toward the sun: a surface's own depth, spread over the
    // texels round it, must not be read as something in front of it.
    let q = p + n * texel * 2.0 - s.origin.xyz;
    let u = dot(q, s.right.xyz) / size + 0.5;
    let v = 0.5 - dot(q, s.up.xyz) / size;
    let inside = min(min(u, 1.0 - u), min(v, 1.0 - v));
    if (inside <= 0.0) {
        return 1.0;
    }
    let d = (dot(q, s.along.xyz) - 0.05) / s.right.w;
    let x = vec2<f32>(u, v) * texels;
    let top = i32(texels) - 1;
    let w = s.along.w;
    let lit = 0.25 * (
        bilinear(a, b, w, x + vec2<f32>(-0.5, -0.5), d, top)
        + bilinear(a, b, w, x + vec2<f32>(0.5, -0.5), d, top)
        + bilinear(a, b, w, x + vec2<f32>(-0.5, 0.5), d, top)
        + bilinear(a, b, w, x + vec2<f32>(0.5, 0.5), d, top)
    );
    return mix(1.0, lit, smoothstep(0.0, 0.04, inside));
}
