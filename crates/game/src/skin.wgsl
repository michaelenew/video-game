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
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
