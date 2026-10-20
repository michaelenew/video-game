// What the ink's colour pass and its depth pass agree on (`ink.rs`): where
// her silhouette has broken into smoke, so the depth has the same holes the
// picture does.

#define_import_path game::ink

struct InkLook {
    body: vec4<f32>,
    glow: vec4<f32>,
    fade: vec4<f32>,
}

fn hash3(p: vec3<f32>) -> f32 {
    let q = fract(p * 0.3183099 + vec3<f32>(0.1, 0.2, 0.3)) * 17.0;
    return fract(q.x * q.y * q.z * (q.x + q.y + q.z));
}

fn noise3(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(
            mix(hash3(i), hash3(i + vec3<f32>(1.0, 0.0, 0.0)), u.x),
            mix(hash3(i + vec3<f32>(0.0, 1.0, 0.0)), hash3(i + vec3<f32>(1.0, 1.0, 0.0)), u.x),
            u.y,
        ),
        mix(
            mix(hash3(i + vec3<f32>(0.0, 0.0, 1.0)), hash3(i + vec3<f32>(1.0, 0.0, 1.0)), u.x),
            mix(hash3(i + vec3<f32>(0.0, 1.0, 1.0)), hash3(i + vec3<f32>(1.0, 1.0, 1.0)), u.x),
            u.y,
        ),
        u.z,
    );
}

// How side-on the surface is seen: one at the silhouette. From the face's
// own slope on the screen rather than the mesh's normal, because the depth
// prepass is not given normals, and the two passes must cut the same holes.
fn rim(p: vec3<f32>, eye: vec3<f32>) -> f32 {
    let n = normalize(cross(dpdx(p), dpdy(p)));
    let v = normalize(eye - p);
    return pow(1.0 - abs(dot(n, v)), 2.0);
}

// How far smoke has eaten into her here: past one is gone. Smoke rises
// through her, finer near the edge; the side-on edge goes first, and the
// fainter she is (a copy folding home) the further in it eats.
fn eaten(look: InkLook, p: vec3<f32>, rim: f32, t: f32) -> f32 {
    let smoke = noise3(p * 6.0 + vec3<f32>(0.0, -t * 1.4, 0.0)) * 0.6
        + noise3(p * 15.0 + vec3<f32>(0.3, -t * 2.6, 0.7)) * 0.4;
    return (rim * 0.9 + (smoke - 0.5) * 0.7 + (1.0 - look.fade.x) * 1.1) / max(look.fade.y, 0.01);
}
