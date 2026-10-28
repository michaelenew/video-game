// **Her shadow, in ink** (`ink.rs`): the standard material's light on her
// costume, sunk to violet-black, with a rim of violet light round the
// silhouette and the side-on edge broken into smoke that drifts upward.
// See-through, but only her nearest surface: drawn after everything solid,
// over a copy of it (`ink.rs` says how), so the depth test keeps her front
// and what is behind her is the world, never her own far side.

#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    forward_io::{VertexOutput, FragmentOutput},
    mesh_view_bindings::{view, globals, view_transmission_texture, view_transmission_sampler},
}
#import game::ink::{InkLook, rim, eaten}

@group(2) @binding(100) var<uniform> ink: InkLook;

const LUMA: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let n = normalize(pbr_input.N);
    let v = normalize(view.world_position - in.world_position.xyz);
    let r = rim(in.world_position.xyz, view.world_position);
    let gone = eaten(ink, in.world_position.xyz, r, globals.time);
    if (gone > 0.62) {
        discard;
    }

    // Her costume's own value, kept: the dark has her shape in it.
    let albedo = pbr_input.material.base_color.rgb;
    let lum = dot(albedo, LUMA);
    pbr_input.material.base_color = vec4<f32>(ink.body.rgb * (0.7 + 1.2 * lum), 1.0);
    pbr_input.material.metallic = 0.0;
    pbr_input.material.perceptual_roughness = 0.75;
    pbr_input.material.reflectance = vec3<f32>(0.1);
    pbr_input.material.specular_transmission = 0.0;
    // Lit at the very edge, where the smoke is about to take her.
    let lip = smoothstep(0.35, 0.62, gone);
    pbr_input.material.emissive = vec4<f32>(
        ink.glow.rgb * ink.glow.a * (pow(1.0 - abs(dot(n, v)), 6.0) * 0.8 + lip * 1.2),
        1.0,
    );

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color.a = 1.0;
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    // How much of her is there: thin across her middle, solid at her rim
    // and where the smoke is taking her, so her edges and light stay hers.
    let side = pow(1.0 - abs(dot(n, v)), 2.0);
    let solid = clamp(mix(ink.body.a, 1.0, max(side, lip)) * ink.fade.x, 0.0, 1.0);
    let uv = (in.position.xy - view.viewport.xy) / view.viewport.zw;
    let behind = textureSampleLevel(view_transmission_texture, view_transmission_sampler, uv, 0.0).rgb;
    // A shadow darkens what it stands in front of.
    out.color = vec4<f32>(mix(behind * mix(1.0, ink.fade.z, ink.fade.x), out.color.rgb, solid), 1.0);
    return out;
}
