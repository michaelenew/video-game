// **The flat shade of everything that stands still**: the arenas' boxes, the
// land, the town, the trees. See `paint.rs` for why there is one.
//
// A vertex colour, or the material's own, times the light: the ambient, and
// each directional light by how squarely the surface faces it, through its
// shadow. Then the fog and the tonemapper the standard material would apply,
// by the same function, so the air and the response curve are the same as
// everything else's -- `look::palette::lit` was measured off that curve.
//
// The light is Bevy's own diffuse term (Burley's, as the standard material
// has it) and its ambient approximation, so a surface comes out the colour
// it did under the standard material; what is left out is the specular
// lobe, which on matte stone and grass was a sheen nobody could see, and
// everything the standard material reads that these surfaces never set --
// metal, reflectance, emission, clearcoat, transmission, the texture
// lookups -- which is most of the per-pixel cost.

#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    mesh_view_bindings::{view, lights},
    mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
    pbr_functions::main_pass_post_lighting_processing,
    lighting::{EnvBRDFApprox, F_AB, F_Schlick},
    pbr_types::{pbr_input_new, STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT},
    shadows::fetch_directional_shadow,
}

struct Paint {
    color: vec4<f32>,
}

@group(2) @binding(0) var<uniform> paint: Paint;

const PI: f32 = 3.14159265;
// A matte surface: the standard material's perceptual roughness 0.92,
// squared, as the white material these surfaces used to be drawn in had
// it. The sheen a surface keeps in the ambient is read from the same.
const ROUGHNESS: f32 = 0.85;
const PERCEPTUAL_ROUGHNESS: f32 = 0.92;
const F0: f32 = 0.04;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var base = paint.color;
#ifdef VERTEX_COLORS
    base = base * in.color;
#endif
    var n = normalize(in.world_normal);
    if (!is_front) {
        n = -n;
    }
    let v = normalize(view.world_position - in.world_position.xyz);
    let ndotv = max(dot(n, v), 0.0001);
    let view_z = dot(vec4<f32>(
        view.view_from_world[0].z,
        view.view_from_world[1].z,
        view.view_from_world[2].z,
        view.view_from_world[3].z
    ), in.world_position);

    // The ambient, as the standard material approximates it.
    let ambient = EnvBRDFApprox(base.rgb, F_AB(1.0, ndotv))
        + EnvBRDFApprox(vec3(F0), F_AB(PERCEPTUAL_ROUGHNESS, ndotv));
    var lit = ambient * lights.ambient_color.rgb;

    for (var i = 0u; i < lights.n_directional_lights; i++) {
        let light = &lights.directional_lights[i];
        let l = (*light).direction_to_light.xyz;
        let ndotl = saturate(dot(n, l));
        if (ndotl <= 0.0) {
            continue;
        }
        var shadow = 1.0;
        if (((*light).flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u) {
            shadow = fetch_directional_shadow(i, in.world_position, n, view_z);
        }
        // Burley's diffuse, as `pbr_lighting::Fd_Burley`.
        let h = normalize(l + v);
        let ldoth = saturate(dot(l, h));
        let f90 = 0.5 + 2.0 * ROUGHNESS * ldoth * ldoth;
        let burley = F_Schlick(1.0, f90, ndotl) * F_Schlick(1.0, f90, ndotv) / PI;
        lit += base.rgb * burley * ndotl * shadow * (*light).color.rgb;
    }

    var pbr_input = pbr_input_new();
    pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT;
    pbr_input.world_position = in.world_position;
    pbr_input.frag_coord = in.position;
    var out: FragmentOutput;
    out.color = main_pass_post_lighting_processing(pbr_input, vec4(lit * view.exposure, base.a));
    return out;
}
