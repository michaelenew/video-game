// **The strokes as splats**: every stroke of the land within sight, drawn
// as a quad lying on the ground, with `paint.wgsl`'s original inner loop as
// its fragment -- the taper, the bristles, inside or not -- so a stroke's
// edge is still decided at the pixel. Written to a screen-sized texture the
// surface shader reads at its pixel in place of searching a hundred strokes.
// Which level is in season is decided here from the distance, as
// `broad_strokes` decided it: the brush is the same size on the screen.

#import bevy_render::view::View

struct Splats {
    // The map's origin: a stroke is placed in map coordinates and the
    // world is the map less this.
    origin: vec4<f32>,
    // x: the look's broad_size.
    params: vec4<f32>,
}

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<uniform> splats: Splats;
// The prepass depth: what is at each pixel, to keep a stroke on its
// ground and off the air in front of it.
@group(0) @binding(2) var depth_tex: texture_depth_2d;

// One stroke, drawn as six vertices of one quad (`corner`): what the
// placer wrote, once a stroke.
struct VertexIn {
    @builtin(vertex_index) corner: u32,
    // The stroke's middle on the ground, and its angle on the plane.
    @location(0) p: vec4<f32>,
    // The ground's normal there (x, z), and the stroke's half length and
    // half width.
    @location(1) q: vec4<f32>,
    // order, level, the ground's stroke size, loose
    @location(2) a: vec4<f32>,
    // shade, warmth, the third axis, curve
    @location(3) b: vec4<f32>,
    // the bristle seeds, half the stroke's length
    @location(4) c: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) b: vec4<f32>,
    // seeds, loose, the ground's stroke size
    @location(2) c: vec4<f32>,
    // 1 for an odd level: which texture is this stroke's; the level
    @location(3) @interpolate(flat) odd: vec2<f32>,
    // How far along the view this fragment is, in metres.
    @location(4) depth: f32,
    // Where in the world this fragment is.
    @location(5) world: vec3<f32>,
}

struct FragmentOut {
    @location(0) even: vec4<f32>,
    @location(1) odd: vec4<f32>,
}

@vertex
fn vertex(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    // The quad's corner, in half-lengths and half-widths, with room for
    // the bristles' reach; laid on the ground's plane at the stroke's
    // middle.
    var corners = array<vec2<f32>, 4>(
        vec2<f32>(-1.2, -1.8), vec2<f32>(1.2, -1.8), vec2<f32>(1.2, 1.8), vec2<f32>(-1.2, 1.8),
    );
    var order = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u);
    let uv = corners[order[in.corner % 6u]];
    let sn = sin(in.p.w);
    let cs = cos(in.p.w);
    let dx = uv.x * in.q.z * cs - uv.y * in.q.w * sn;
    let dz = uv.x * in.q.z * sn + uv.y * in.q.w * cs;
    let ny = sqrt(max(1.0 - in.q.x * in.q.x - in.q.y * in.q.y, 1e-3));
    let y = in.p.y - (in.q.x * dx + in.q.y * dz) / ny;
    let pos = vec3<f32>(in.p.x + dx, y, in.p.z + dz);
    let world = pos - splats.origin.xyz;
    let to_eye = view.world_position - world;
    let dist = length(to_eye);
    let level = in.a.y;
    let size = in.a.z;
    // Whether any of this stroke can be in season: the season is judged
    // at the pixel (`season`), but a quad none of which can be need not be
    // drawn at all. A level is in season between two distances (where
    // `log2(broad_size * d * size)` is within one of it), and the quad
    // reaches a little over its length either way from this corner.
    let band = exp2(level) / (splats.params.x * size);
    let reach = 2.6 * in.c.z;
    if (dist + reach < band * 0.5 || dist - reach > band * 2.0) {
        // Out of season: no area.
        out.clip = vec4<f32>(2.0, 2.0, 0.0, 1.0);
        return out;
    }
    out.clip = view.clip_from_world * vec4<f32>(world, 1.0);
    out.depth = -(view.view_from_world * vec4<f32>(world, 1.0)).z;
    out.world = world;
    out.uv = uv;
    out.b = in.b;
    out.c = vec4<f32>(in.c.xy, in.a.w, size);
    out.odd = vec2<f32>(f32(i32(level) & 1), level);
    return out;
}

// `broad_strokes`: the length the brush wants at a point, as a power of
// two, and this level's share of it -- the two nearest levels blended, the
// blend kept at full strength. Judged at the pixel: a wide stroke reaches
// from where it is in season to where it is not.
fn season(dist: f32, size: f32, level: f32) -> f32 {
    let l = log2(max(splats.params.x * dist * size, 0.05));
    let t = fract(l);
    let k = 1.0 / sqrt(t * t + (1.0 - t) * (1.0 - t));
    return max(0.0, 1.0 - abs(l - level)) * k;
}

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

@fragment
fn fragment(in: VertexOut) -> FragmentOut {
    // On the ground, or not drawn: the scene's depth here, against this
    // fragment's, within what the mesh can bulge between its vertices
    // plus a share of the distance.
    let scene = textureLoad(depth_tex, vec2<i32>(in.clip.xy), 0);
    // Reversed infinite depth: the view distance is the near plane over it.
    let scene_depth = view.clip_from_view[3][2] / max(scene, 1e-7);
    if (abs(in.depth - scene_depth) > 0.08 + 0.02 * scene_depth) {
        discard;
    }
    let w = season(length(view.world_position - in.world), in.c.w, in.odd.y);
    if (w <= 0.0) {
        discard;
    }
    let u = in.uv.x;
    let curve = in.b.w;
    let loose = in.c.z;
    let v = in.uv.y - curve * u * u;
    // Ends: square and ruled, or tapered and dry, by how loose.
    let taper = mix(1.0, sqrt(max(1.0 - u * u, 0.0)), 0.25 + 0.6 * loose);
    let bristle = noise2(vec2<f32>(u * 4.0 + in.c.x * 9.0, v * 5.0 + in.c.y * 9.0)) - 0.5;
    let inside = abs(v) < taper * (1.0 + bristle * 0.35 * loose) && abs(u) < 1.0 + bristle * 0.5;
    if (!inside) {
        discard;
    }
    let col = in.b.xyz * (0.85 + 0.15 * bristle * 2.0);
    // Weighted by the season, at a quarter plus a half: the weight reaches
    // 1.4. Into this stroke's level's texture; the other is left alone.
    let texel = vec4<f32>(col * w * 0.25 + 0.5, 1.0);
    let none = vec4<f32>(0.0);
    var out: FragmentOut;
    if (in.odd.x > 0.5) {
        out.even = none;
        out.odd = texel;
    } else {
        out.even = texel;
        out.odd = none;
    }
    return out;
}
