// **A character painted, in motion** (`brushwork.rs`, exploration 0025).
//
// Each stroke is four corners at one point of the bind pose. The vertex
// stage skins that point as the surface it came from is skinned and turns
// the card square to the eye, its long side along a direction fixed to the
// body, leaned toward the painter's slant, and laid along the outline where
// the form turns away. The fragment stage takes the stroke's one colour
// from the lit picture under its centre (Meier 1996) -- the copy of
// everything solid Bevy keeps for transmission -- simplifies it as a
// painter would (values gathered into bands, light warmed, shade cooled),
// and cuts the card out in the shape of a flat brush.

#import bevy_pbr::{
    mesh_functions,
    skinning,
    mesh_view_bindings as views,
    mesh_view_bindings::{view, globals, view_transmission_texture, view_transmission_sampler},
    view_transformations::depth_ndc_to_view_z,
}

struct BrushLook {
    // How many value bands, and how far the picture is pulled to them; how
    // much richer the colour is made.
    tint: vec4<f32>,
    // What the light and the shade are multiplied by.
    light: vec4<f32>,
    shade: vec4<f32>,
    // How far a stroke is pulled toward the eye (metres); unused; how much
    // each stroke's value wanders; the brush's bristle count.
    knobs: vec4<f32>,
}

@group(2) @binding(0) var<uniform> brush: BrushLook;

struct In {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    // Which corner of the card: -1 or 1 along the stroke and across it.
    @location(3) corner: vec2<f32>,
    // The stroke's direction in the bind pose, and a number of its own.
    @location(4) way: vec4<f32>,
    // Its width and length (metres), how much it lies along an open edge,
    // and whether it is a detail stroke laid over the masses.
    @location(5) size: vec4<f32>,
#ifdef SKINNED
    @location(6) joint_indices: vec4<u32>,
    @location(7) joint_weights: vec4<f32>,
#endif
}

struct Out {
    @builtin(position) clip: vec4<f32>,
    @location(0) corner: vec2<f32>,
    // Its own number; how far it faces the eye; whether it is a wisp.
    @location(1) seed: vec3<f32>,
    // Where its centre is on the screen (0 to 1), and how far its long and
    // short sides reach there, for reading the picture under it.
    @location(2) centre: vec2<f32>,
    @location(3) reach: vec4<f32>,
    // How far in front of the eye its point is (view space, negative).
    @location(4) depth: f32,
}

const LUMA: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

fn to_screen(clip: vec4<f32>) -> vec2<f32> {
    let ndc = clip.xy / clip.w;
    return vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}

@vertex
fn vertex(v: In) -> Out {
    var out: Out;
#ifdef SKINNED
    let world_from_local = skinning::skin_model(v.joint_indices, v.joint_weights, v.instance_index);
#else
    let world_from_local = mesh_functions::get_world_from_local(v.instance_index);
#endif
    let centre = (world_from_local * vec4<f32>(v.position, 1.0)).xyz;
    let n = normalize((world_from_local * vec4<f32>(v.normal, 0.0)).xyz);
    let way = (world_from_local * vec4<f32>(v.way.xyz, 0.0)).xyz;
    let to_eye = normalize(view.world_position - centre);
    let facing = dot(n, to_eye);

    let own = v.way.w;
    let kind = v.size.w;
    var d = (view.view_from_world * vec4<f32>(way, 0.0)).xy;
    if (length(d) < 1e-4) {
        d = vec2<f32>(0.0, 1.0);
    }
    d = normalize(d);
    var nv = (view.view_from_world * vec4<f32>(n, 0.0)).xy;
    let edge = 1.0 - smoothstep(0.1, 0.55, abs(facing));
    var wisp = kind > 1.5;
    if (!wisp) {
        // The painter's hand: every stroke leans a little toward one slant
        // on the canvas -- only its angle; it stays where it is on the body.
        // Not one laid along an open edge (`size.z`): it keeps the edge.
        let hand = normalize(vec2<f32>(0.55, 1.0));
        if (dot(d, hand) < 0.0) {
            d = -d;
        }
        d = normalize(mix(d, hand, 0.25 * (1.0 - v.size.z)));
        if (length(nv) > 1e-4) {
            let out_v = normalize(nv);
            // At the outline, a third of the strokes lift off it as wisps;
            // the rest lie along it, as a painter draws an edge.
            // The outline is where the form turns out of view, not where it
            // ends: strokes there lie along it and stay inside it (the
            // owner, 2026-10-22). Only a real edge of the cloth frays.
            {
                var along = vec2<f32>(-out_v.y, out_v.x);
                if (dot(along, d) < 0.0) {
                    along = -along;
                }
                d = normalize(mix(d, along, edge));
            }
        }
    }
    var len = v.size.y;
    var wide = v.size.x * mix(0.5, 1.0, smoothstep(0.05, 0.5, abs(facing)));
    if (wisp) {
        // A wisp of paint, a quarter of the way to flame: it lifts as it
        // leaves the cloth, sways, and breathes in length. On the renderer's
        // clock, as `fx::flow` is: decoration on a body whose every place is
        // the snapshot's.
        let t = globals.time;
        d = normalize(d + vec2<f32>(0.0, 0.7));
        let sway = sin(t * 2.1 + own * 40.0) * 0.25 + sin(t * 3.7 + own * 13.0) * 0.1;
        let cs = cos(sway);
        let sn = sin(sway);
        d = vec2<f32>(d.x * cs - d.y * sn, d.x * sn + d.y * cs);
        len = len * (0.85 + 0.2 * sin(t * 2.9 + own * 17.0)) * select(1.0, 1.5, kind < 1.5);
        wide = v.size.x;
    }
    let across = vec2<f32>(-d.y, d.x);

    // The card, square to the eye. Pulled toward the eye so the surface it
    // came from does not cut it, detail strokes more, so they lie on the
    // masses. A stroke's card has room round it for its bristles (the
    // land's: a fifth along, four-fifths across); a wisp's root is on the
    // body and its length runs out from there.
    let pull = brush.knobs.x * (1.0 + min(kind, 1.0));
    let c = (view.view_from_world * vec4<f32>(centre + to_eye * pull, 1.0)).xyz;
    var off: vec2<f32>;
    if (wisp) {
        off = d * (v.corner.x + 1.0) * 0.5 * len + across * v.corner.y * wide * 0.5;
    } else {
        off = d * v.corner.x * 1.2 * len * 0.5 + across * v.corner.y * 1.8 * wide * 0.5;
    }
    out.clip = view.clip_from_view * vec4<f32>(c.xy + off, c.z, 1.0);

    // Where the picture is read: the stroke's own point, and a little way
    // along and across it.
    let at = view.clip_from_view * vec4<f32>(c, 1.0);
    let tip = view.clip_from_view * vec4<f32>(c.xy + d * v.size.y * 0.3, c.z, 1.0);
    let side = view.clip_from_view * vec4<f32>(c.xy + across * v.size.x * 0.3, c.z, 1.0);
    out.centre = to_screen(at);
    out.reach = vec4<f32>(to_screen(tip) - out.centre, to_screen(side) - out.centre);
    out.depth = c.z + pull;
    out.corner = v.corner;
    // 0 a stroke, 1 a wisp off a hem, 2 paint leaving a legendary.
    out.seed = vec3<f32>(v.way.w, smoothstep(-0.15, 0.3, facing), select(0.0, 1.0, wisp) + select(0.0, 1.0, kind > 2.5));
    return out;
}

fn hash12(p: vec2<f32>) -> f32 {
    var q = fract(p * vec2<f32>(123.34, 456.21));
    q += dot(q, q + 45.32);
    return fract(q.x * q.y);
}

fn picture(uv: vec2<f32>) -> vec3<f32> {
    return textureSampleLevel(view_transmission_texture, view_transmission_sampler, uv, 0.0).rgb;
}

// How much a point of the picture counts toward a stroke: only if what is
// drawn there is the stroke's own body (about its depth), so a stroke at
// the silhouette never takes the wall's colour. Where depth cannot be read
// (the browser), every point counts.
fn ours(uv: vec2<f32>, depth: f32) -> f32 {
#ifdef DEPTH_PREPASS
#ifndef MULTISAMPLED
    let px = vec2<i32>(uv * view.viewport.zw + view.viewport.xy);
    let z = depth_ndc_to_view_z(textureLoad(views::depth_prepass_texture, px, 0));
    return 1.0 - smoothstep(0.2, 0.45, abs(z - depth));
#else
    return 1.0;
#endif
#else
    return 1.0;
#endif
}

// A stroke's one colour: the picture under it, averaged over the middle of
// the stroke, then simplified -- its value pulled to the nearest of a few
// bands (soft, so a band's edge is a step of strokes rather than a line),
// its hue kept and a little enriched, the light warmed and the shade
// cooled, and its value nudged by its own number.
// How much two colours of the picture are the same stuff: 1 for the same
// cloth in a little more or less light, 0 for another material beside it
// (the light mantle on the dark shirt). What keeps a stroke inside its own
// colour shape, as a painter keeps a shape.
fn same(a: vec3<f32>, b: vec3<f32>) -> f32 {
    let diff = length(a - b) / (length(b) + 0.04);
    return 1.0 - smoothstep(0.3, 0.55, diff);
}

// The picture under a stroke: its own point's colour, and the average of
// the points round it that are its own body and its own stuff -- so a
// stroke at the edge of a material is that material's colour, not a mix.
// `a` is 0 where none of its body is under it.
fn under(centre: vec2<f32>, reach: vec4<f32>, depth: f32) -> vec4<f32> {
    if (ours(centre, depth) < 0.5) {
        return vec4<f32>(0.0);
    }
    let middle = picture(centre);
    let a = reach.xy;
    let b = reach.zw;
    var sum = middle * 2.0;
    var weight = 2.0;
    let taps = array<vec2<f32>, 4>(a, -a, b, -b);
    for (var i = 0; i < 4; i++) {
        let uv = centre + taps[i];
        let seen = picture(uv);
        let w = ours(uv, depth) * same(seen, middle);
        sum += seen * w;
        weight += w;
    }
    return vec4<f32>(sum / weight, 1.0);
}

// **A painter's black is never black** (2026-10-22, painting the Blood
// mage's coat). Every variation a stroke makes -- its streaks, its warmer
// and cooler lanes, its own value -- multiplies the colour under it, and on
// near-black cloth a multiple of nothing is nothing: her coat took the
// strokes and showed none. So a variation is made on the eye's scale (the
// square root of the light, as the bands are), and no smaller there than it
// would be on a colour of `DARK`. At `DARK` and brighter this is exactly
// the multiply it was.
const DARK: f32 = 0.3;

fn shift(c: vec3<f32>, m: vec3<f32>) -> vec3<f32> {
    let p = sqrt(max(c, vec3<f32>(0.0)));
    let q = max(p + (sqrt(max(m, vec3<f32>(0.0))) - 1.0) * max(p, vec3<f32>(DARK)), vec3<f32>(0.0));
    return q * q;
}

fn painted(seen: vec3<f32>, own: f32) -> vec4<f32> {
    let l = max(dot(seen, LUMA), 1e-5);
    // Bands on a perceptual scale.
    let v = sqrt(l);
    let k = brush.tint.x;
    let f = v * k;
    let stepped = (floor(f) + smoothstep(0.35, 0.65, fract(f))) / k;
    let v2 = mix(v, stepped, brush.tint.y);
    var colour = seen * (v2 * v2) / l;
    colour = mix(vec3<f32>(dot(colour, LUMA)), colour, brush.tint.z);
    let warm = smoothstep(0.25, 0.6, v2);
    colour *= mix(brush.shade.rgb, brush.light.rgb, warm);
    return vec4<f32>(shift(colour, vec3<f32>(1.0 + (own - 0.5) * 2.0 * brush.knobs.z)), 1.0);
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

// How freely the body's strokes lie (`brushwork::LOOSE`).
const LOOSE: f32 = 0.3;

@fragment
fn fragment(in: Out) -> @location(0) vec4<f32> {
    let own = in.seed.x;
    let s1 = fract(own * 7.31);
    let s2 = fract(own * 13.17);
    // How the paint lies at this point of the stroke: `along` from the tail
    // (-1, dry) to the head (1, loaded) and `lane`, which bristle it is.
    var along = 0.0;
    var lane = 0.0;
    var alpha = 0.0;
    if (in.seed.z < 0.5) {
        // **The land's stroke** (`splats.wgsl`): along it `u`, across it `v`,
        // bowed by its curve, its ends tapered and dry by how loose, its
        // edge broken by the bristles.
        let u = in.corner.x * 1.2;
        let curve = (s1 - 0.5) * LOOSE * 0.8;
        let v = in.corner.y * 1.8 - curve * u * u;
        let taper = mix(1.0, sqrt(max(1.0 - u * u, 0.0)), 0.25 + 0.6 * LOOSE);
        let bristle = noise2(vec2<f32>(u * 4.0 + s1 * 9.0, v * 5.0 + s2 * 9.0)) - 0.5;
        let reach = taper * (1.0 + bristle * 0.35 * LOOSE);
        if (abs(v) >= reach || abs(u) >= 1.0 + bristle * 0.5) {
            discard;
        }
        along = u;
        lane = v / max(reach, 0.05);
        // Thinner in the middle, fuller at the edges where the brush leaves
        // its ridge; soft at its very edge and thinning out at the dry tail.
        let across = abs(lane);
        alpha = mix(0.55, 0.85, smoothstep(0.35, 0.85, across))
            * (1.0 - smoothstep(0.85, 1.0, across))
            * mix(0.55, 1.0, smoothstep(-1.1, -0.2, u));
    } else {
        // **A wisp**: a tongue of paint from its root on the body to a dry
        // tip, narrowing as it goes, thinning out toward the end.
        let t = in.corner.x * 0.5 + 0.5;
        let v = in.corner.y - (s1 - 0.5) * 1.4 * t * t;
        let bristle = noise2(vec2<f32>(t * 6.0 + s1 * 9.0, v * 4.0 + s2 * 9.0)) - 0.5;
        let width = sqrt(max(1.0 - pow(t, 2.2), 0.0)) * 0.9;
        let reach = width * (1.0 + bristle * 0.5);
        if (abs(v) >= reach) {
            discard;
        }
        along = 1.0 - 2.0 * t;
        lane = v / max(reach, 0.05);
        alpha = mix(0.8, 0.0, smoothstep(0.2, 1.0, t)) * (1.0 - smoothstep(0.8, 1.0, abs(lane)));
    }
    // Turning away, a stroke fades.
    alpha *= in.seed.y;
    let base = under(in.centre, in.reach, in.depth);
    if (base.a < 0.5 || alpha < 0.01) {
        discard;
    }
    let paint = painted(base.rgb, own);
    // **Colour inside the stroke**: each bristle laid a little lighter or
    // darker, warmer or cooler than its neighbours -- streaks along the
    // stroke -- and the paint richer where the brush was loaded, paler
    // where it ran thin at the tail.
    let streak = noise2(vec2<f32>(lane * 4.5 + s1 * 31.0, along * 0.6 + s2 * 17.0)) - 0.5;
    let hue = noise2(vec2<f32>(lane * 2.5 + s2 * 23.0, along * 0.4 + s1 * 11.0)) - 0.5;
    let load = along * 0.5 + 0.5;
    var colour = shift(paint.rgb, vec3<f32>(1.0 + streak * 0.22));
    colour = shift(colour, vec3<f32>(1.0 + hue * 0.12, 1.0, 1.0 - hue * 0.12));
    colour = mix(vec3<f32>(dot(colour, LUMA)), colour, mix(0.85, 1.15, load));
    colour = shift(colour, vec3<f32>(mix(1.08, 0.96, load)));
    // The stroke's own shift, as the land's: warmer or cooler as a whole.
    let warm = (s2 - 0.5) * 0.08;
    colour = shift(colour, vec3<f32>(1.0 + warm, 1.0, 1.0 - warm));
    alpha = clamp(alpha * (1.0 + streak * 0.3), 0.0, 1.0);
    // Below a little paint, no stroke: what it would write is the cloth.
    if (alpha < 0.1) {
        discard;
    }
    let here = (in.clip.xy - view.viewport.xy) / view.viewport.zw;
    // **Inside the lines.** A stroke paints only where its own body is
    // drawn -- not the wall past the outline, not an arm in front -- except
    // a wisp off the hem, where the cloth really ends and the paint runs
    // on by intent.
    let there = picture(here);
    if (in.seed.z < 0.5) {
        if (ours(here, in.depth) < 0.5) {
            discard;
        }
        // And only over its own colour shape: where the picture here is
        // another material, the stroke stops at the boundary.
        alpha *= same(there, base.rgb);
        if (alpha < 0.1) {
            discard;
        }
    } else if (in.seed.z < 1.5 && ours(here, in.depth) > 0.5) {
        // A wisp runs on into the air past the cloth's end, never onto
        // other cloth of the same body: a bodice's edge lies on the coat,
        // and a tongue of red up the black coat is the bleed the owner
        // ruled out (`docs/design/cast-look.md` P7, 2026-10-22).
        alpha *= same(there, base.rgb);
        if (alpha < 0.1) {
            discard;
        }
    }
    return vec4<f32>(mix(there, colour, alpha), 1.0);
}
