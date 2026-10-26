// **The fighters' skin** (`still::SkinMaterial`): the standard material, with
// the share of its light that came from the sun taken away where the still
// world's shadow falls -- as the paint does, so a fighter under a giant is in
// the same shade as the ground they stand on.

#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    forward_io::{VertexOutput, FragmentOutput},
    mesh_view_bindings::{lights, view},
    mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
}
#import game::still

@group(2) @binding(100) var<uniform> where_map: still::Shade;
@group(2) @binding(101) var map_a: texture_2d<f32>;
@group(2) @binding(102) var map_b: texture_2d<f32>;

// `still::Lustre`: a studio's reflections for what has to shine (zero: none).
struct Lustre {
    gloss: vec4<f32>,
    sheen: vec4<f32>,
}
@group(2) @binding(103) var<uniform> lustre: Lustre;

// A soft panel of light in the mirror direction `r`, centred on `d`, its
// edge wider the rougher the surface.
fn panel(r: vec3<f32>, d: vec3<f32>, size: f32, rough: f32) -> f32 {
    let c = dot(r, normalize(d));
    let edge = size + rough * 0.6;
    return smoothstep(cos(edge), cos(size * 0.55), c);
}

// The studio, in the camera's own frame (x right, y up, z toward the eye):
// a sky that is dark below the horizon and bright overhead, a key panel high
// to the left, a long strip across the top and a dimmer fill to the right.
fn studio(r: vec3<f32>, rough: f32) -> vec3<f32> {
    let up = r.y;
    let sky = mix(vec3<f32>(0.015), vec3<f32>(0.22, 0.23, 0.26), smoothstep(-0.25, 0.2, up))
        + vec3<f32>(0.35) * smoothstep(0.35, 1.0, up);
    let k = lustre.gloss.y;
    let key = panel(r, vec3<f32>(-0.55, 0.6, 0.6), 0.32, rough) * k;
    let strip = panel(r, vec3<f32>(0.1, 0.95, 0.25), 0.22, rough) * k * 0.7;
    let fill = panel(r, vec3<f32>(0.75, 0.15, 0.62), 0.28, rough) * k * 0.35;
    return sky + vec3<f32>(key + strip + fill);
}

const PI: f32 = 3.14159265;
const LUMA: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
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
    if (lustre.gloss.x > 0.0) {
        let n = pbr_input.N;
        let v = pbr_input.V;
        let r = reflect(-v, n);
        let rv = (view.view_from_world * vec4<f32>(r, 0.0)).xyz;
        let rough = pbr_input.material.perceptual_roughness;
        let metal = pbr_input.material.metallic;
        let base = pbr_input.material.base_color.rgb;
        let f0 = mix(vec3<f32>(0.04), base, metal);
        let nv = saturate(dot(n, v));
        let fres = f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - nv, 5.0);
        let gloss = (1.0 - rough) * (1.0 - rough);
        let env = studio(normalize(rv), rough) * fres * gloss;
        let edge = lustre.sheen.rgb * pow(1.0 - nv, max(lustre.sheen.w, 0.5));
        out.color = vec4<f32>(out.color.rgb + (env * lustre.gloss.x + edge) * view.exposure, out.color.a);
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
