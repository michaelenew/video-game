// **The fighters' skin** (`still::SkinMaterial`): the standard material, with
// the share of its light that came from the sun taken away where the still
// world's shadow falls -- as the paint does, so a fighter under a giant is in
// the same shade as the ground they stand on.

#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    forward_io::{VertexOutput, FragmentOutput},
    mesh_view_bindings::lights,
    mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
}
#import game::still

@group(2) @binding(100) var<uniform> where_map: still::Shade;
@group(2) @binding(101) var map_a: texture_2d<f32>;
@group(2) @binding(102) var map_b: texture_2d<f32>;

// A grown creature's detail (`still::Detail`, exploration 0030).
struct Detail {
    ember: vec4<f32>,
    on: vec4<f32>,
}
@group(2) @binding(103) var<uniform> detail: Detail;

fn hash3(p: vec3<i32>, seed: u32) -> f32 {
    var h = (u32(p.x) * 73856093u) ^ (u32(p.y) * 19349663u) ^ (u32(p.z) * 83492791u) ^ (seed * 2654435761u);
    h = (h ^ (h >> 16u)) * 0x45d9f3bu;
    h = (h ^ (h >> 16u)) * 0x45d9f3bu;
    h = h ^ (h >> 16u);
    return f32(h & 0xffffffu) / 16777216.0;
}

fn vnoise(q: vec3<f32>) -> f32 {
    let i = vec3<i32>(floor(q));
    let f = fract(q);
    let u = f * f * (3.0 - 2.0 * f);
    let a = mix(hash3(i, 1u), hash3(i + vec3<i32>(1, 0, 0), 1u), u.x);
    let b = mix(hash3(i + vec3<i32>(0, 1, 0), 1u), hash3(i + vec3<i32>(1, 1, 0), 1u), u.x);
    let c = mix(hash3(i + vec3<i32>(0, 0, 1), 1u), hash3(i + vec3<i32>(1, 0, 1), 1u), u.x);
    let d = mix(hash3(i + vec3<i32>(0, 1, 1), 1u), hash3(i + vec3<i32>(1, 1, 1), 1u), u.x);
    return mix(mix(a, b, u.y), mix(c, d, u.y), u.z);
}

// How much of a detail of wavelength `w` metres to keep where a pixel is
// `pixel` metres across.
fn keep(w: f32, pixel: f32) -> f32 {
    return smoothstep(2.0, 4.0, w / max(pixel, 1e-5));
}

fn fbm3(q: vec3<f32>) -> f32 {
    return (vnoise(q) * 4.0 + vnoise(q * 2.0) * 2.0 + vnoise(q * 4.0)) / 7.0;
}

// Worley noise: the distances to the nearest and second-nearest jittered
// lattice points -- creases, pits, cracks and fissures are all this.
fn worley(q: vec3<f32>, seed: u32) -> vec2<f32> {
    let b = vec3<i32>(floor(q));
    var f1 = 9.0;
    var f2 = 9.0;
    for (var z = -1; z <= 1; z++) {
        for (var y = -1; y <= 1; y++) {
            for (var x = -1; x <= 1; x++) {
                let c = b + vec3<i32>(x, y, z);
                let j = vec3<f32>(hash3(c, seed), hash3(c, seed + 1u), hash3(c, seed + 2u)) * 0.85 + 0.075;
                let d = length(vec3<f32>(c) + j - q);
                if (d < f1) {
                    f2 = f1;
                    f1 = d;
                } else if (d < f2) {
                    f2 = d;
                }
            }
        }
    }
    return vec2<f32>(f1, f2);
}

const PI: f32 = 3.14159265;
const LUMA: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
#ifdef VERTEX_UVS_A
#ifdef VERTEX_UVS_B
    if (detail.on.x > 0.5) {
        // Where on the creature at rest, and what it is there -- stone, soft
        // flesh or a hard part (`grown::grow`) -- so the detail rides on the
        // skin as it moves.
        let rest = vec3<f32>(in.uv.x, in.uv.y, in.uv_b.x);
        let what = in.uv_b.y;
        let hard = select(0.0, 1.0, what < -0.5);
        let soft = select(0.0, clamp(what - 2.0, 0.0, 1.0), what > 1.5);
        let stone = select(0.0, clamp(what, 0.0, 1.0), what > -0.5 && what < 1.5);
        let skin = max(1.0 - stone - soft - hard, 0.0);
        var h = 0.0;
        var dark = 0.0;
        var glow = 0.0;
        // How big a pixel is on the creature, in metres: a detail finer than
        // about two of them would only sparkle, so each fades out before it
        // gets there.
        let pixel = max(length(dpdx(rest)), length(dpdy(rest)));
        // What the skin is: plumage (1), chitin (2), or hide.
        let plumage = select(0.0, skin, detail.on.y > 0.5 && detail.on.y < 1.5);
        let chitin = select(0.0, skin, detail.on.y > 1.5);
        if (plumage > 0.01) {
            // Feathers: cells longer than wide along the body, each a
            // cushion with a dark seam where the next overlaps it, and fine
            // barbs down it.
            let fw = worley(rest * vec3<f32>(2.6, 4.6, 4.6), 41u);
            let seam = 1.0 - smoothstep(0.0, 0.09, fw.y - fw.x);
            let cushion = 1.0 - fw.x;
            let barbs = sin((rest.y * 0.7 + rest.z) * 250.0 + fbm3(rest * 2.0) * 4.0);
            let kf = keep(0.25, pixel);
            let kb = keep(0.03, pixel);
            h += plumage * (0.010 * cushion * kf - 0.006 * seam * kf + 0.0006 * barbs * kb);
            dark += plumage * (seam * 0.35 * kf - cushion * 0.08);
        }
        if (chitin > 0.01) {
            // Chitin: smooth, a few broad plates with sutures between, fine
            // pits, and a gloss the hide does not have.
            let pw = worley(rest * vec3<f32>(4.0, 2.0, 4.0), 43u);
            let suture = 1.0 - smoothstep(0.0, 0.025, pw.y - pw.x);
            let pits = smoothstep(0.25, 0.0, worley(rest * 140.0, 44u).x);
            let ks = keep(0.05, pixel);
            let kp = keep(0.008, pixel);
            h += chitin * (-0.003 * suture * ks - 0.0005 * pits * kp + 0.002 * (fbm3(rest * 6.0) - 0.5));
            dark += chitin * (suture * 0.5 * ks + pits * 0.1 * kp);
            pbr_input.material.perceptual_roughness = mix(pbr_input.material.perceptual_roughness, 0.32, chitin);
        }
        let hide = skin - plumage - chitin;
        if (hide > 0.01) {
            let w1 = worley(rest * 9.0, 5u);
            let crease = 1.0 - smoothstep(0.0, 0.1, w1.y - w1.x);
            let w2 = worley(rest * vec3<f32>(22.0, 46.0, 22.0), 9u);
            let fine = 1.0 - smoothstep(0.0, 0.12, w2.y - w2.x);
            let kc = keep(0.11, pixel);
            let kf = keep(0.022, pixel);
            h += hide * (-0.004 * crease * kc - 0.0015 * fine * kf + 0.005 * (fbm3(rest * 3.0) - 0.5));
            dark += hide * (crease * 0.3 * kc + fine * 0.08 * kf);
        }
        if (stone > 0.01) {
            let erode = fbm3(rest * 2.6) - 0.5;
            let pw = worley(rest * 26.0, 23u);
            let pits = smoothstep(0.3, 0.0, pw.x) * step(0.5, vnoise(rest * 4.0));
            let cw = worley(rest * 3.2, 25u);
            let crack = (1.0 - smoothstep(0.0, 0.03, cw.y - cw.x)) * smoothstep(0.5, 0.6, vnoise(rest * 1.8));
            let kp = keep(0.04, pixel);
            let kg = keep(0.07, pixel);
            h += stone * (0.012 * erode + 0.003 * (vnoise(rest * 14.0) - 0.5) * kg - 0.004 * pits * kp - 0.014 * crack);
            dark += stone * (crack * 0.55 + pits * 0.25 * kp - erode * 0.25);
        }
        if (soft > 0.01) {
            let lw = worley(rest * 5.5, 31u);
            let lava = 1.0 - smoothstep(0.0, 0.06, lw.y - lw.x);
            h += soft * (-0.026 * lava);
            dark += soft * (1.0 - lava) * 0.4;
            glow += soft * (0.08 + lava);
        }
        // Bump from the height alone, with no texture layout to lean on:
        // the surface gradient from the screen's own derivatives (Mikkelsen,
        // "Bump Mapping Unparametrized Surfaces on the GPU").
        let pos = in.world_position.xyz;
        let sx = dpdx(pos);
        let sy = dpdy(pos);
        let n = pbr_input.N;
        let r1 = cross(sy, n);
        let r2 = cross(n, sx);
        let det = dot(sx, r1);
        let g = sign(det) * (dpdx(h) * r1 + dpdy(h) * r2);
        pbr_input.N = normalize(abs(det) * n - g);
        let c = pbr_input.material.base_color.rgb * clamp(1.0 - dark, 0.35, 1.25);
        pbr_input.material.base_color = vec4<f32>(c, pbr_input.material.base_color.a);
        pbr_input.material.emissive = vec4<f32>(detail.ember.rgb * detail.ember.w * glow, 1.0);
    }
#endif
#endif
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    let s = still::shade(where_map, map_a, map_b, in.world_position.xyz, pbr_input.N);
    if (s < 1.0) {
        // How much of what lights this point is the sun: each directional
        // light by how squarely the surface faces it, as Bevy's diffuse term
        // weighs it, against the ambient.
        let n = pbr_input.N;
        var sun = 0.0;
        var rest = dot(lights.ambient_color.rgb, LUMA);
        for (var i = 0u; i < lights.n_directional_lights; i++) {
            let light = &lights.directional_lights[i];
            let k = dot((*light).color.rgb, LUMA) * saturate(dot(n, (*light).direction_to_light.xyz)) / PI;
            if (((*light).flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u) {
                sun += k;
            } else {
                rest += k;
            }
        }
        let share = sun / max(sun + rest, 1e-4);
        out.color = vec4<f32>(out.color.rgb * (1.0 - share * (1.0 - s)), out.color.a);
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
