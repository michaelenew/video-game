// **The paper half of the wash** (`look::wash`): two passes over the finished
// picture. `brush` runs before the line is drawn and is what the pigment does
// while it is wet -- dragged along the form, pooled dark at the edge of every
// wash, thick and thin across the sheet. `paper` runs after the line, so the
// ink sits in the same paper as the colour: the sheet's relief, the pigment
// settling in its hollows or skipping them altogether under a dry brush.
//
// Two open sources, ported rather than reinvented:
//
// - **MNPR** (Montesdeoca et al., "MNPR: A Framework for Real-Time Expressive
//   Non-Photorealistic Rendering of 3D Computer Graphics", Expressive 2018;
//   github.com/semontesdeoca/MNPR, MIT): the watercolour model -- colour as
//   transmittance raised to a pigment density, edge darkening, pigment
//   turbulence, pigment application over a paper heightmap (granulation and
//   dry brush), paper distortion and paper relief -- and the paper scan
//   itself (`paper.jpg`, its `rough_default` substrate: normal in red and
//   green, height in blue).
// - **Acerola's anisotropic Kuwahara** (Garrett Gunnell,
//   github.com/GarrettGunnell/Post-Processing, MIT), after Kyprianidis, Kang
//   and Döllner, "Anisotropic Kuwahara Filtering on the GPU" (GPU Pro, 2010):
//   the brush. Eight overlapping sectors of an ellipse turned to run along the
//   picture's edges; each is averaged, and the calmest win. A flat area stays
//   flat, an edge stays an edge, and a gradient comes out as strokes laid
//   along the form. Its structure tensor is three passes there; here it is
//   gathered in place from nine Sobel taps, so the pass needs no targets of
//   its own.
//
// Everything is in screen space on purpose: the paper does not move with the
// camera. Sizes are in pixels of a 760-line picture, scaled to this one.

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct Wash {
    a: vec4<f32>,      // brush, drawn, haze, haze_falloff
    b: vec4<f32>,      // haze tint, haze tint amount
    c: vec4<f32>,      // near_dark, ao, glow, shafts
    d: vec4<f32>,      // grade shade, saturation
    e: vec4<f32>,      // grade light, lift
    f: vec4<f32>,      // dark, key
    g: vec4<f32>,      // on paper, pool, bleed, granulate
    h: vec4<f32>,      // dry, distort, relief, turbulence
    paper: vec4<f32>,
    cam: vec4<f32>,    // camera height; height per unit along right, up, back
    lens: vec4<f32>,   // tan half fov, aspect, near
    sun: vec4<f32>,    // sun ndc x, y, in front
    i: vec4<f32>,      // edges, the smear's scale
    j: vec4<f32>,      // crisp until, painted from (metres), broad strokes, frame
    right: vec4<f32>,  // the camera's axes in the world
    up: vec4<f32>,
    back: vec4<f32>,
    fog: vec4<f32>,    // the fog's colour: the sky at the horizon, as seen
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var<uniform> wash: Wash;
@group(0) @binding(3) var paper_texture: texture_2d<f32>;
@group(0) @binding(4) var paper_sampler: sampler;
@group(0) @binding(5) var depth_texture: texture_depth_2d;
@group(0) @binding(6) var depth_sampler: sampler;
// The brush laid by `smear`, at its own resolution (a 1x1 blank in every
// pass but `brush`).
@group(0) @binding(7) var smear_texture: texture_2d<f32>;

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

fn fbm(p: vec2<f32>) -> f32 {
    return 0.5 * noise2(p) + 0.3 * noise2(p * 2.07 + 7.1) + 0.2 * noise2(p * 4.13 + 3.7);
}

// The arithmetic is done on the colour as it is seen, not as light: pigment
// density is a perceptual idea, and the texture hands back linear.
fn seen(c: vec3<f32>) -> vec3<f32> {
    return pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
}

fn light(c: vec3<f32>) -> vec3<f32> {
    return pow(max(c, vec3<f32>(0.0)), vec3<f32>(2.2));
}

fn read(px: vec2<f32>, size: vec2<f32>) -> vec3<f32> {
    return seen(textureSampleLevel(screen_texture, screen_sampler, px / size, 0.0).rgb);
}

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.299, 0.587, 0.114));
}

// MNPR's colour transmittance model: a transparent colour laid `d` times as
// thick lets through its transmittance to the power `d`.
fn thicken(c: vec3<f32>, d: f32) -> vec3<f32> {
    return pow(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(max(d, 0.0)));
}

// The paper at this pixel: its normal's slope (x, y, -1 to 1) and its height
// (z, 0 to 1). The scan is 1024 square from a 2048 original, so it is drawn
// at two pixels a texel of a 760-line picture to keep the original's tooth.
fn sheet(px: vec2<f32>, k: f32) -> vec3<f32> {
    let t = textureSampleLevel(paper_texture, paper_sampler, px / (2048.0 * k), 0.0).rgb;
    return vec3<f32>(t.rg * 2.0 - 1.0, t.b);
}

// Acerola's structure tensor: a Sobel of the colour, as (xx, yy, xy).
fn tensor(at: vec2<f32>, size: vec2<f32>, d: f32) -> vec3<f32> {
    let a = read(at + vec2<f32>(-d, -d), size);
    let b = read(at + vec2<f32>(-d, 0.0), size);
    let c = read(at + vec2<f32>(-d, d), size);
    let e = read(at + vec2<f32>(d, -d), size);
    let f = read(at + vec2<f32>(d, 0.0), size);
    let g = read(at + vec2<f32>(d, d), size);
    let h = read(at + vec2<f32>(0.0, -d), size);
    let i = read(at + vec2<f32>(0.0, d), size);
    let sx = (a + 2.0 * b + c - e - 2.0 * f - g) / 4.0;
    let sy = (a + 2.0 * h + e - c - 2.0 * i - g) / 4.0;
    return vec3<f32>(dot(sx, sx), dot(sy, sy), dot(sx, sy));
}

// Forty-eight taps over the unit disc, four rings each turned half a step
// from the last (and the middle makes forty-nine): what the Kuwahara reads,
// in place of every pixel of its ellipse, where a dense radius of seven was
// eight hundred. Twenty-five taps was tried first and smeared the grass into
// streaks along the ellipse: with three samples a sector, the variance that
// picks the calmest sector is noise. Each tap's eight sector weights are
// worked out in the loop: a table of them, indexed per tap, was slower under
// the software renderer (279 ms to 465 ms a pass), not faster.
const TAPS = array<vec2<f32>, 48>(
    vec2<f32>(0.2500, 0.0000), vec2<f32>(0.1768, 0.1768), vec2<f32>(0.0000, 0.2500), vec2<f32>(-0.1768, 0.1768),
    vec2<f32>(-0.2500, 0.0000), vec2<f32>(-0.1768, -0.1768), vec2<f32>(-0.0000, -0.2500), vec2<f32>(0.1768, -0.1768),
    vec2<f32>(0.4830, 0.1294), vec2<f32>(0.3536, 0.3536), vec2<f32>(0.1294, 0.4830), vec2<f32>(-0.1294, 0.4830),
    vec2<f32>(-0.3536, 0.3536), vec2<f32>(-0.4830, 0.1294), vec2<f32>(-0.4830, -0.1294), vec2<f32>(-0.3536, -0.3536),
    vec2<f32>(-0.1294, -0.4830), vec2<f32>(0.1294, -0.4830), vec2<f32>(0.3536, -0.3536), vec2<f32>(0.4830, -0.1294),
    vec2<f32>(0.7500, 0.0000), vec2<f32>(0.6757, 0.3254), vec2<f32>(0.4676, 0.5864), vec2<f32>(0.1669, 0.7312),
    vec2<f32>(-0.1669, 0.7312), vec2<f32>(-0.4676, 0.5864), vec2<f32>(-0.6757, 0.3254), vec2<f32>(-0.7500, 0.0000),
    vec2<f32>(-0.6757, -0.3254), vec2<f32>(-0.4676, -0.5864), vec2<f32>(-0.1669, -0.7312), vec2<f32>(0.1669, -0.7312),
    vec2<f32>(0.4676, -0.5864), vec2<f32>(0.6757, -0.3254), vec2<f32>(0.9262, 0.2114), vec2<f32>(0.7427, 0.5923),
    vec2<f32>(0.4122, 0.8559), vec2<f32>(0.0000, 0.9500), vec2<f32>(-0.4122, 0.8559), vec2<f32>(-0.7427, 0.5923),
    vec2<f32>(-0.9262, 0.2114), vec2<f32>(-0.9262, -0.2114), vec2<f32>(-0.7427, -0.5923), vec2<f32>(-0.4122, -0.8559),
    vec2<f32>(-0.0000, -0.9500), vec2<f32>(0.4122, -0.8559), vec2<f32>(0.7427, -0.5923), vec2<f32>(0.9262, -0.2114),
);

// Acerola's anisotropic Kuwahara, eight sectors, read on the sparse taps
// above and with its structure tensor one Sobel rather than a blurred nine.
fn kuwahara(at: vec2<f32>, size: vec2<f32>, k: f32, reach: f32) -> vec3<f32> {
    let g = tensor(at, size, 1.5 * k);
    let root = sqrt(max(g.y * g.y - 2.0 * g.x * g.y + g.x * g.x + 4.0 * g.z * g.z, 0.0));
    let l1 = 0.5 * (g.y + g.x + root);
    let l2 = 0.5 * (g.y + g.x - root);
    let v = vec2<f32>(l1 - g.x, -g.z);
    // On nothing at all, a slow wander, so the strokes across a flat sky do
    // not all lie the same way.
    var t = vec2<f32>(cos(noise2(at / (70.0 * k)) * 6.28), sin(noise2(at / (70.0 * k)) * 6.28));
    if (length(v) > 1e-6) {
        t = normalize(v);
    }
    let phi = -atan2(t.y, t.x);
    var anisotropy = 0.0;
    if (l1 + l2 > 1e-6) {
        anisotropy = (l1 - l2) / (l1 + l2);
    }
    anisotropy = max(anisotropy, 0.35);

    let alpha = wash.a.y;
    let radius = reach * k;
    // Stretched along the edge no more than half again: further, and the
    // sparse taps along the long axis read as a smear.
    let a = radius * clamp((alpha + anisotropy) / alpha, 0.1, 1.5);
    let b = radius * clamp(alpha / (alpha + anisotropy), 0.1, 1.5);
    let cp = cos(phi);
    let sp = sin(phi);
    let zeta = 2.0 / max(radius, 1.0);
    let zero_cross = 0.58;
    let szc = sin(zero_cross);
    let eta = (zeta + cos(zero_cross)) / (szc * szc);

    var m = array<vec4<f32>, 8>();
    var s = array<vec3<f32>, 8>();
    for (var i = 0; i < 49; i = i + 1) {
        // The tap in the ellipse's own frame, a unit-half disc...
        var v = vec2<f32>(0.0);
        if (i < 48) {
            v = TAPS[i] * 0.5;
        }
        // ...and on the screen: the ellipse's inverse, R^T S^-1.
        let o = vec2<f32>(
            cp * (2.0 * a * v.x) + sp * (2.0 * b * v.y),
            -sp * (2.0 * a * v.x) + cp * (2.0 * b * v.y),
        );
        let c = clamp(read(at + o, size), vec3<f32>(0.0), vec3<f32>(1.0));
        var w = array<f32, 8>();
        var sum = 0.0;
        var vxx = zeta - eta * v.x * v.x;
        var vyy = zeta - eta * v.y * v.y;
        var z = max(0.0, v.y + vxx);
        w[0] = z * z;
        z = max(0.0, -v.x + vyy);
        w[2] = z * z;
        z = max(0.0, -v.y + vxx);
        w[4] = z * z;
        z = max(0.0, v.x + vyy);
        w[6] = z * z;
        v = 0.70710678 * vec2<f32>(v.x - v.y, v.x + v.y);
        vxx = zeta - eta * v.x * v.x;
        vyy = zeta - eta * v.y * v.y;
        z = max(0.0, v.y + vxx);
        w[1] = z * z;
        z = max(0.0, -v.x + vyy);
        w[3] = z * z;
        z = max(0.0, -v.y + vxx);
        w[5] = z * z;
        z = max(0.0, v.x + vyy);
        w[7] = z * z;
        for (var n = 0; n < 8; n = n + 1) {
            sum += w[n];
        }
        let gauss = exp(-3.125 * dot(v, v)) / max(sum, 1e-6);
        for (var n = 0; n < 8; n = n + 1) {
            let wk = w[n] * gauss;
            m[n] += vec4<f32>(c * wk, wk);
            s[n] += c * c * wk;
        }
    }
    var out = vec4<f32>(0.0);
    for (var n = 0; n < 8; n = n + 1) {
        if (m[n].w <= 1e-6) {
            continue;
        }
        let mean = m[n].rgb / m[n].w;
        let spread2 = abs(s[n] / m[n].w - mean * mean);
        let sigma2 = spread2.r + spread2.g + spread2.b;
        let w = 1.0 / (1.0 + pow(8.0 * 1000.0 * sigma2, 4.0));
        out += vec4<f32>(mean * w, w);
    }
    if (out.w <= 1e-6) {
        return read(at, size);
    }
    return out.rgb / out.w;
}

fn ring(at: vec2<f32>, size: vec2<f32>, r: f32) -> vec3<f32> {
    var sum = vec3<f32>(0.0);
    for (var i = 0; i < 8; i = i + 1) {
        let a = f32(i) * 0.7853982 + r;
        sum += read(at + vec2<f32>(cos(a), sin(a)) * r, size);
    }
    return sum / 8.0;
}

// The depth at a pixel, as Bevy writes it: reversed and infinite, so 1 at the
// near plane, 0 at the sky.
fn depth_at(px: vec2<f32>, size: vec2<f32>) -> f32 {
    let on = clamp(px, vec2<f32>(0.5), size - vec2<f32>(0.5));
    return textureSampleLevel(depth_texture, depth_sampler, on / size, 0i);
}

// Distance along the view, in metres.
fn along(depth: f32) -> f32 {
    return wash.lens.z / max(depth, 1e-7);
}

// **A broad dry brush**, in screen space: wide bands laid across the picture
// at a sweeping angle that wanders slowly, each band broken into strokes a
// few hundred pixels long. Returns the stroke's own shade (x, -1 to 1) and
// how much paint it laid here (y, 0 to 1): full in its body, breaking into
// bristle streaks toward its dry ends and its ragged sides.
fn broad(px: vec2<f32>, k: f32, layer: f32) -> vec3<f32> {
    let turn = -0.42 + layer * 0.9 + 0.55 * (noise2(px / (420.0 * k) + layer * 9.0) - 0.5);
    let c = cos(turn);
    let s = sin(turn);
    let u = (c * px.x + s * px.y) / k;
    let v = (-s * px.x + c * px.y) / k;
    let width = 120.0;
    let band = floor(v / width);
    let fv = fract(v / width);
    let length = 620.0 + 380.0 * hash2(vec2<f32>(band, layer));
    let slid = u + hash2(vec2<f32>(band, 7.0 + layer)) * 900.0;
    let seg = floor(slid / length);
    let fu = fract(slid / length);
    let shade = hash2(vec2<f32>(band * 1.7 + layer, seg)) * 2.0 - 1.0;
    // Bristles: streaks a pixel or two across, long along the stroke.
    let bristle = 0.6 * noise2(vec2<f32>(u / 70.0, v / 1.4)) + 0.4 * noise2(vec2<f32>(u / 23.0, v / 0.7) + 5.0);
    // Paint runs out toward the end of a stroke and thins at its sides.
    let body = smoothstep(0.0, 0.08, fu) * (1.0 - smoothstep(0.55, 1.0, fu))
        * smoothstep(0.0, 0.18, fv) * (1.0 - smoothstep(0.82, 1.0, fv));
    let laid = smoothstep(0.15, 0.45, body + (bristle - 0.5) * 0.9);
    return vec3<f32>(shade * (0.6 + 0.4 * bristle), laid, bristle);
}

// Whether a pixel is the sky: its distance along its own ray is past where
// the dome sits (`lens.w`).
fn is_sky(depth: f32, size: vec2<f32>, px: vec2<f32>) -> bool {
    let ndc = vec2<f32>(px.x / size.x * 2.0 - 1.0, 1.0 - px.y / size.y * 2.0);
    let ray = vec3<f32>(ndc.x * wash.lens.x * wash.lens.y, ndc.y * wash.lens.x, -1.0);
    return depth <= 1e-7 || along(depth) * length(ray) > wash.lens.w;
}

// **The wet colour wanders** (on paper only): where everything is read
// from is pushed a pixel or two by a slow noise.
fn wandered(px: vec2<f32>, k: f32) -> vec2<f32> {
    let wander = vec2<f32>(noise2(px / (17.0 * k)), noise2(px / (17.0 * k) + vec2<f32>(31.0, 17.0)));
    return px + (wander - 0.5) * 2.0 * wash.g.z * k;
}

// **Crisp near, painted far**: how far the brush reaches at a pixel, by how
// far into the painted distance it is -- nothing where it is rendered as it
// is, the whole brush where it is brushwork.
fn reach_at(px: vec2<f32>, size: vec2<f32>) -> f32 {
    let depth = depth_at(px, size);
    var far = 1.0;
    if (wash.j.y > 0.0 && !is_sky(depth, size, px)) {
        let ndc = vec2<f32>(px.x / size.x * 2.0 - 1.0, 1.0 - px.y / size.y * 2.0);
        let ray = vec3<f32>(ndc.x * wash.lens.x * wash.lens.y, ndc.y * wash.lens.x, -1.0);
        far = smoothstep(wash.j.x, wash.j.y, along(depth) * length(ray));
    }
    return wash.a.x * far;
}

// **The brush, on its own**: the Kuwahara, the costliest thing in the
// picture by far -- forty-nine taps of eight sectors a pixel -- drawn into a
// texture of its own at `wash.i.y` of the screen's resolution, which
// `governor` lowers when a frame runs over and `brush` reads back filtered.
// In the screen's own pixels throughout, so only how many places it is
// worked out at changes, never the brush's size; at a scale of one it is
// the brush exactly as it was.
@fragment
fn smear(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(screen_texture));
    let k = size.y / 760.0;
    let px = in.uv * size;
    let at = wandered(px, k);
    let reach = reach_at(px, size);
    if (reach >= 0.75) {
        return vec4<f32>(kuwahara(at, size, k, reach), 1.0);
    }
    return vec4<f32>(read(at, size), 1.0);
}

@fragment
fn brush(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(screen_texture));
    let k = size.y / 760.0;
    let px = in.position.xy;
    let paper = wash.paper.rgb;
    let on_paper = wash.g.x > 0.5;

    let at = wandered(px, k);

    let depth = depth_at(px, size);
    let z = along(depth);
    let ndc = vec2<f32>(px.x / size.x * 2.0 - 1.0, 1.0 - px.y / size.y * 2.0);
    let ray = vec3<f32>(ndc.x * wash.lens.x * wash.lens.y, ndc.y * wash.lens.x, -1.0);
    // How much height the ray gains per metre along the view.
    let climb = wash.cam.y * ray.x + wash.cam.z * ray.y - wash.cam.w;
    let metres = z * length(ray);
    // The sky is a dome a fixed way off (`sky::RADIUS`), not empty depth.
    let sky = is_sky(depth, size, px);

    var c = read(at, size);
    if (reach_at(px, size) >= 0.75) {
        c = textureSampleLevel(smear_texture, screen_sampler, px / size, 0.0).rgb;
    }

    // **Broad strokes across the sky** (GW2's concept art): wide dry-brush
    // bands, two layers crossing, each stroke its own shade with bristle
    // streaks through it. Laid on the sky's own directions, not the screen's,
    // so they turn with the world as the camera does instead of sliding over
    // it. (The land's strokes are on the land, in `paint.wgsl`.)
    if (wash.j.z > 0.0 && sky) {
        let dir = normalize(wash.right.xyz * ray.x + wash.up.xyz * ray.y + wash.back.xyz * ray.z);
        let per_radian = size.y / (2.0 * wash.lens.x);
        let at_sky = vec2<f32>(atan2(dir.z, dir.x), -asin(clamp(dir.y, -1.0, 1.0))) * per_radian;
        let one = broad(at_sky, k, 0.0);
        let two = broad(at_sky + vec2<f32>(130.0, 57.0) * k, k, 1.0);
        let shade = (one.x * one.y + two.x * two.y * 0.6) / (1.0 + 0.6 * two.y);
        c = c * (1.0 + wash.j.z * shade * 0.22);
    }

    // **The air's colour**: the sky at the horizon, which the camera's fog
    // already fades into (`sky::fog`), handed in rather than hunted for across
    // the top of the picture by every pixel.
    let air = mix(wash.fog.xyz, wash.b.xyz, wash.b.w);

    if (!sky) {
        // **Corners and contacts are shaded**: the nearer what surrounds a
        // pixel stands, the more it is tucked in. Measured in metres round it,
        // so the shading is as wide on a far wall as on a near one, and
        // tinted toward the grade's shade rather than toward black.
        if (wash.c.y > 0.0) {
            var tucked = 0.0;
            let reach = clamp(26.0 * k / max(z, 1.0) * 6.0, 4.0 * k, 40.0 * k);
            for (var i = 0; i < 10; i = i + 1) {
                let ang = f32(i) * 2.399 + hash2(px) * 0.6;
                let r = reach * (0.35 + 0.65 * f32(i % 5) / 4.0);
                let other = along(depth_at(px + vec2<f32>(cos(ang), sin(ang)) * r, size));
                let nearer = z - other;
                tucked += clamp(nearer / 0.6, 0.0, 1.0) * (1.0 - smoothstep(1.5, 4.0, nearer));
            }
            let shade = wash.c.y * tucked / 10.0;
            c = c * mix(vec3<f32>(1.0), wash.d.xyz * 0.6, shade);
        }

        // **Edges caught, creases shaded.** A painter runs a lighter stroke
        // along every edge that turns toward the light and darkens every
        // crease. Both are found in the depth: across a flat face depth is
        // linear on screen, so the pixel between two others sits exactly at
        // their average; on an edge turned toward you it stands proud of it,
        // in a crease it sinks behind. A step in depth (a silhouette) is not
        // a fold and is left alone.
        if (wash.i.x > 0.0) {
            var fold = 0.0;
            let r = 2.0 * k;
            for (var i = 0; i < 4; i = i + 1) {
                let a = f32(i) * 0.7853982;
                let o = vec2<f32>(cos(a), sin(a)) * r;
                let one = depth_at(px + o, size);
                let two = depth_at(px - o, size);
                let step = max(abs(one - depth), abs(two - depth)) / depth;
                if (step < 0.04) {
                    let bend = (depth - 0.5 * (one + two)) / depth;
                    if (abs(bend) > abs(fold)) {
                        fold = bend;
                    }
                }
            }
            // Broken along its length, as a stroke is.
            let stroke = 0.55 + 0.45 * noise2(px / (6.0 * k));
            // The thresholds are well above the bend between two triangles of
            // the land, which is a fold too, and a gentle one.
            let caught = smoothstep(0.002, 0.008, fold) * stroke;
            let crease = smoothstep(0.002, 0.008, -fold) * stroke;
            c = mix(c, c * 1.45 + wash.e.xyz * 0.08, caught * wash.i.x);
            c = mix(c, c * wash.d.xyz * 0.55, crease * wash.i.x * 0.8);
        }

        // **The near is dark against the far**: the nearest things a little
        // darker and deeper, the way a painter knocks back a foreground to
        // throw the eye past it.
        let near = (1.0 - smoothstep(2.0, 9.0, metres)) * wash.c.x;
        c = mix(c, c * wash.f.xyz * 3.0 + c * 0.25, near * 0.8);

        // **The air**: exponential haze thinning with height, integrated
        // along the ray, in the sky's colour.
        if (wash.a.z > 0.0) {
            let fall = wash.a.w;
            let rise = climb / length(ray) * metres;
            var spread = 1.0;
            if (abs(fall * rise) > 1e-4) {
                spread = (1.0 - exp(-fall * rise)) / (fall * rise);
            }
            let thick = wash.a.z * metres * spread;
            let hazed = 1.0 - exp(-max(thick, 0.0));
            c = mix(c, air, hazed);
        }
    }

    // **Glow**: what is bright spills a little into what is round it.
    if (wash.c.z > 0.0) {
        var bloom = vec3<f32>(0.0);
        for (var i = 0; i < 12; i = i + 1) {
            let ang = f32(i) * 2.399;
            let r = (5.0 + 20.0 * f32(i) / 11.0) * k;
            let s = read(px + vec2<f32>(cos(ang), sin(ang)) * r, size);
            bloom += max(s - vec3<f32>(0.72), vec3<f32>(0.0));
        }
        c = c + bloom / 12.0 * wash.c.z * 1.8;
    }

    // **Shafts from the sun**: the sky's light carried toward the sun across
    // whatever stands in front of it, so the light comes through gaps.
    if (wash.c.w > 0.0 && wash.sun.z > 0.0) {
        let sun_px = vec2<f32>((wash.sun.x * 0.5 + 0.5) * size.x, (0.5 - wash.sun.y * 0.5) * size.y);
        let toward = sun_px - px;
        var lit = 0.0;
        let jitter = hash2(px);
        for (var i = 0; i < 12; i = i + 1) {
            let t = (f32(i) + jitter) / 12.0;
            let q = px + toward * t * 0.8;
            if (is_sky(depth_at(q, size), size, q)) {
                lit += 1.0 - t * 0.6;
            }
        }
        let fade = 1.0 - smoothstep(0.3, 1.3, length(toward / size.y));
        let shaft = lit / 12.0 * fade * wash.sun.z;
        c = c + air * shaft * wash.c.w * 0.6;
    }

    // ---- The grade --------------------------------------------------------
    // Shade toward the shade's tint, light toward the light's, then
    // colourfulness, then the darks lifted toward the darkest pigment.
    let l = luma(c);
    c = c * mix(wash.d.xyz, wash.e.xyz, smoothstep(0.1, 0.8, l));
    c = mix(vec3<f32>(luma(c)), c, wash.d.w);
    c = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));

    if (on_paper) {
        // **Pigment turbulence** (MNPR): no wash goes on evenly.
        let d = 1.0 + wash.h.w * (fbm(px / (120.0 * k)) - 0.5) * 2.0;
        c = mix(paper, thicken(c, d), clamp(d, 0.0, 1.0));

        // **Edge darkening** (MNPR): pigment carried to the edge of a wash
        // sets darker there. A difference of blurs, gated off near the paper.
        let near_ring = ring(at, size, 2.0 * k);
        let far_ring = ring(at, size, 4.5 * k);
        let edge = length(near_ring - c) * 0.6 + length(far_ring - c) * 0.4;
        let pigmented = clamp(length(c - paper) * 5.0, 0.0, 1.0);
        c = thicken(c, 1.0 + wash.g.y * min(edge * 3.0, 0.9) * pigmented);
    }

    // The darks lifted toward the darkest colour, and the high key.
    c = mix(c, wash.f.xyz + c * (vec3<f32>(1.0) - wash.f.xyz), wash.e.w);
    c = vec3<f32>(1.0) - pow(vec3<f32>(1.0) - clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(1.0 + wash.f.w));

    return vec4<f32>(light(c), 1.0);
}

// **The frame, in dry brush**: toward the picture's corners the paint runs
// out the way a broad stroke does -- along its length, in bristle streaks --
// and leaves the paper, as a painting's ground shows round its subject. After
// the line, so no line is drawn over bare paper.
fn framed(c: vec3<f32>, px: vec2<f32>, size: vec2<f32>, k: f32) -> vec3<f32> {
    let uv = px / size;
    let dd = (uv - 0.5) * 2.0;
    // A rounded rectangle, so the paper comes in at the corners first.
    let q = abs(dd) - vec2<f32>(0.55, 0.45);
    let rim = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0);
    let stroke = broad(px, k, 2.0);
    let reach = 0.62 - 0.3 * wash.j.w;
    // The stroke's own body and its bristles make the ragged end.
    let ragged = (0.5 - stroke.y) * 0.14 + (0.5 - stroke.z) * 0.13;
    let bare = smoothstep(reach - 0.02, reach + 0.02, rim + ragged);
    return mix(c, vec3<f32>(0.95, 0.94, 0.91), bare);
}

@fragment
fn paper(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(screen_texture));
    let k = size.y / 760.0;
    let px = in.position.xy;
    let paper = wash.paper.rgb;
    if (wash.g.x < 0.5) {
        if (wash.j.w <= 0.0) {
            return textureLoad(screen_texture, vec2<i32>(px), 0);
        }
        return vec4<f32>(light(framed(seen(textureLoad(screen_texture, vec2<i32>(px), 0).rgb), px, size, k)), 1.0);
    }
    let s = sheet(px, k);

    // **Paper distortion** (MNPR): the wet colour runs along the paper's
    // slopes, so everything is read a little downhill on the sheet.
    var c = read(px + s.xy * wash.h.y * k, size);

    // **Pigment application** (MNPR). Where the brush was dry it skips the
    // hollows of the paper and leaves them white; elsewhere the pigment
    // settles into them and granulates, more in a light wash than a dark one.
    // The dry passes are long strokes across the sheet, and a thin (light)
    // wash is dry more often than a thick one: at a gradient from dark to
    // light, the light end breaks up into the paper.
    let slant = vec2<f32>(px.x * 0.94 + px.y * 0.34, -px.x * 0.34 + px.y * 0.94);
    let stroke = fbm(vec2<f32>(slant.x / (160.0 * k), slant.y / (14.0 * k)));
    let thin = smoothstep(0.35, 0.95, luma(c));
    let dryness = wash.h.x * smoothstep(0.45, 0.75, stroke) * (0.35 + 0.65 * thin);
    let pigmented = clamp(length(c - paper) * 5.0, 0.0, 1.0);
    let dry_diff = s.z - dryness * 1.2;
    if (dry_diff < 0.0) {
        c = mix(c, paper, clamp(-dry_diff * 4.0, 0.0, 1.0) * pigmented);
    } else {
        // MNPR granulates a light wash five times as hard as a dark one,
        // which on a game's sky -- all light wash, and never paper -- is
        // dirt. Twice, here.
        var application = 0.2 + wash.g.w;
        application = mix(application, application * 2.0, luma(c));
        let height = s.z * 0.2 + 0.8;
        let accumulation = 1.0 + application * (1.0 - height);
        c = mix(c, thicken(c, accumulation), pigmented);
    }

    // **The paper's colour under everything**: the whitest thing in the
    // picture is the sheet.
    c = c * paper;

    // **The paper's relief** (MNPR's deferred substrate lighting): the sheet
    // lit from the top left at a low angle, under everything.
    let light_dir = normalize(vec3<f32>(-0.5, -0.7, 0.75));
    let n = normalize(vec3<f32>(-s.xy * 0.5, 1.0));
    let lambert = 1.0 - dot(n, light_dir);
    c = c * (1.0 - lambert * lambert * wash.h.z);
    if (wash.j.w > 0.0) {
        c = framed(c, px, size, k);
    }

    return vec4<f32>(light(c), 1.0);
}
