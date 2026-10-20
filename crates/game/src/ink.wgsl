// **Her shadow, in ink** (`ink.rs`): the standard material's light on her
// costume, sunk to violet-black, with a rim of violet light round the
// silhouette and the side-on edge broken into smoke that drifts upward.
// Solid, not see-through: a blended body shows its own insides.

#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    forward_io::{VertexOutput, FragmentOutput},
    mesh_view_bindings::{view, globals},
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
    pbr_input.material.base_color = vec4<f32>(ink.body.rgb * (0.6 + 2.5 * lum), 1.0);
    pbr_input.material.metallic = 0.0;
    pbr_input.material.perceptual_roughness = 0.75;
    pbr_input.material.reflectance = vec3<f32>(0.1);
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
    return out;
}
