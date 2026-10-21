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
    mesh_view_bindings::{view, view_transmission_texture, view_transmission_sampler},
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
    // Its own number; how far it faces the eye; its aspect.
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

    var d = (view.view_from_world * vec4<f32>(way, 0.0)).xy;
    if (length(d) < 1e-4) {
        d = vec2<f32>(0.0, 1.0);
    }
    d = normalize(d);
    // The painter's hand: every stroke leans toward one slant on the canvas.
    // Only the angle -- the stroke stays where it is on the body.
    let hand = normalize(vec2<f32>(0.55, 1.0));
    if (dot(d, hand) < 0.0) {
        d = -d;
    }
    // Not a stroke laid along an open edge (`size.z`): it keeps the edge.
    d = normalize(mix(d, hand, 0.45 * (1.0 - v.size.z)));
    // At the edge of the form a painter draws along the outline, not out of
    // it: as the surface turns away the stroke turns to lie along it.
    var nv = (view.view_from_world * vec4<f32>(n, 0.0)).xy;
    if (length(nv) > 1e-4) {
        nv = normalize(nv);
        var along = vec2<f32>(-nv.y, nv.x);
        if (dot(along, d) < 0.0) {
            along = -along;
        }
        let edge = 1.0 - smoothstep(0.1, 0.55, abs(facing));
        d = normalize(mix(d, along, edge));
    }
    let across = vec2<f32>(-d.y, d.x);

    // The card, square to the eye. Pulled toward the eye so the surface it
    // came from does not cut it, detail strokes more, so they lie on the
    // masses.
    let pull = brush.knobs.x * (1.0 + v.size.w);
    let c = (view.view_from_world * vec4<f32>(centre + to_eye * pull, 1.0)).xyz;
    // Narrower where the form turns away: an edge is drawn with the side of
    // the brush, not by a stroke hanging off it into the air.
    let wide = v.size.x * mix(0.45, 1.0, smoothstep(0.05, 0.5, abs(facing)));
    let off = d * v.corner.x * v.size.y * 0.5 + across * v.corner.y * wide * 0.5;
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
    out.seed = vec3<f32>(v.way.w, smoothstep(-0.15, 0.3, facing), v.size.y / max(v.size.x, 1e-4));
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
fn painted(centre: vec2<f32>, reach: vec4<f32>, depth: f32, own: f32) -> vec4<f32> {
    let a = reach.xy;
    let b = reach.zw;
    var sum = vec3<f32>(0.0);
    var weight = 0.0;
    let taps = array<vec2<f32>, 5>(vec2<f32>(0.0), a, -a, b, -b);
    for (var i = 0; i < 5; i++) {
        let uv = centre + taps[i];
        let w = ours(uv, depth) * select(1.0, 2.0, i == 0);
        sum += picture(uv) * w;
        weight += w;
    }
    // Nothing of the body under it: the stroke is not laid (alpha 0).
    if (weight < 0.05) {
        return vec4<f32>(0.0);
    }
    let seen = sum / weight;
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
    return vec4<f32>(colour * (1.0 + (own - 0.5) * 2.0 * brush.knobs.z), 1.0);
}

@fragment
fn fragment(in: Out) -> @location(0) vec4<f32> {
    let seed = in.seed.x * 97.0;
    let along = in.corner.x;
    // A stroke is not straight: it bows a little, its own way.
    let bow = (fract(seed * 0.37) - 0.5) * 0.5;
    let across = in.corner.y - bow * (1.0 - along * along);
    // Loaded at the head, thinning to the tail: full width through most of
    // its length, a rounded head, a tapered tail.
    let taper = mix(0.35, 1.0, smoothstep(-1.0, -0.2, along));
    let headed = sqrt(max(1.0 - pow(max(along - 0.55, 0.0) / 0.45, 2.0), 0.0));
    // Its sides waver, as a brush's do.
    let waver = 0.9 + 0.08 * sin(along * 6.0 + seed) + 0.05 * sin(along * 13.0 - seed * 1.7);
    if (abs(across) > taper * headed * waver) {
        discard;
    }
    // Bristles: streaks along the stroke; toward the tail some carry no
    // paint and the stroke breaks into dry brush.
    let bristle = hash12(vec2<f32>(floor((in.corner.y * 0.5 + 0.5) * brush.knobs.w), seed));
    let dry = smoothstep(-0.1, -1.0, along);
    if (bristle < dry * 0.9 - 0.05) {
        discard;
    }
    // Turning away, a stroke thins out stroke by stroke rather than all at
    // once: a dither fixed to the stroke, not to the screen.
    if (hash12(vec2<f32>(seed, 3.1)) > in.seed.y) {
        discard;
    }
    let paint = painted(in.centre, in.reach, in.depth, in.seed.x);
    if (paint.a < 0.5) {
        discard;
    }
    // The bristles' streaks and a little more paint at the stroke's edges:
    // what makes a flat colour read as brushed.
    let ridge = 1.0 - 0.06 * smoothstep(0.6, 0.95, abs(across) / max(taper * headed, 0.05))
        + 0.06 * (bristle - 0.5);
    return vec4<f32>(paint.rgb * ridge, 1.0);
}
