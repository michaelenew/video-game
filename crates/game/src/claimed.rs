//! **The Blood mage's ground, drawn as one surface** (2026-10-18).
//!
//! Her ground is many discs in the simulation -- patches (`sim::ground`), the
//! ground each stain and heart of hers holds -- and they overlap all the
//! time. Drawn as a disc each, every overlap was another layer of
//! see-through violet, so the floor went darker wherever two met, which reads
//! as "more ground" when it is not. So the discs are drawn as **their union**:
//! one mesh, rebuilt every frame from a grid laid over them, with the edge
//! found by marching squares on the distance to the nearest edge, so it comes
//! out round rather than stepped.
//!
//! Every vertex is **draped on the floor the simulation stands on**, a few
//! centimetres over it (`LIFT`): a flat disc laid on a slope, or on a town's
//! flagstones standing proud of the floor, had half of itself under them.
//!
//! Presentation only: what is hers is `sim::World::on_blood`, and this draws
//! what that function answers, to the same discs and the same height slack.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::view::NoFrustumCulling;
use std::collections::HashMap;

/// Metres between grid points. Small enough that an edge is round, large
/// enough that a field across the arena is a few thousand triangles.
const STEP: f32 = 0.3;
/// How far over the floor the surface is drawn: over a town's flagstones,
/// which stand three centimetres proud of the floor the simulation knows.
pub const LIFT: f32 = 0.06;
/// Frames a patch takes to fade out at the end of its life.
const FADE: f32 = 60.0;

/// The one entity her ground is drawn on.
#[derive(Component)]
pub struct Claimed;

/// One disc of ground, in the renderer's numbers.
#[derive(Clone, Copy)]
struct Disc {
    x: f32,
    y: f32,
    z: f32,
    r: f32,
    fade: f32,
}

/// One grid point: how far inside the union it is (negative outside), how
/// solid, and the floor's height there.
#[derive(Clone, Copy)]
struct Point {
    inside: f32,
    fade: f32,
    y: f32,
}

/// What the drape keeps between frames: the floor's height under each grid
/// point it has asked about, for the place it was asked in, and the buffers
/// it builds into.
#[derive(Default)]
pub struct Drape {
    place: Option<(sim::arena::ArenaId, i32, i32)>,
    heights: HashMap<(i32, i32, i32), f32>,
    points: HashMap<(i32, i32), Point>,
    discs: Vec<Disc>,
    positions: Vec<[f32; 3]>,
    colours: Vec<[f32; 4]>,
}

/// Spawn the surface, with its material.
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(empty());
    let material = materials.add(StandardMaterial {
        // Her essence's colour, darker and flatter, so the floor she holds
        // reads from across the arena without hiding the stains and bodies
        // on it. The vertex colour carries how solid each part is.
        base_color: Color::srgba(0.17, 0.01, 0.12, 0.6),
        emissive: LinearRgba::rgb(0.27, 0.006, 0.19),
        perceptual_roughness: 1.0,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::default(),
        Visibility::Hidden,
        Claimed,
        // Rebuilt every frame, so the bounds it was spawned with mean
        // nothing.
        NoFrustumCulling,
        bevy::pbr::NotShadowCaster,
    ));
}

fn empty() -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32; 4]; 3]);
    mesh.insert_indices(Indices::U32(vec![0, 1, 2]));
    mesh
}

/// **Drape her ground**: gather every disc, lay the grid over them, march
/// the squares and write the mesh.
pub fn drape(
    sim: Res<crate::Sim>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut surface: Query<(&Mesh3d, &mut Visibility), With<Claimed>>,
    mut keep: Local<Drape>,
) {
    let Ok((handle, mut seen)) = surface.single_mut() else {
        return;
    };
    let w = &sim.cur;
    let keep = &mut *keep;
    // A new place, or the same place moved under us: the heights asked
    // before were for another floor.
    let origin = w.map_origin();
    let place = (
        w.arena,
        origin.x.to_f32_for_render() as i32,
        origin.z.to_f32_for_render() as i32,
    );
    if keep.place != Some(place) {
        keep.place = Some(place);
        keep.heights.clear();
    }
    gather(w, &mut keep.discs);
    if keep.discs.is_empty() {
        *seen = Visibility::Hidden;
        return;
    }
    *seen = Visibility::Inherited;
    lay_grid(w, keep);
    march(keep);
    let Some(mesh) = meshes.get_mut(&handle.0) else {
        return;
    };
    let n = keep.positions.len();
    if n < 3 {
        *seen = Visibility::Hidden;
        return;
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, keep.positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; n]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, keep.colours.clone());
    mesh.insert_indices(Indices::U32((0..n as u32).collect()));
}

/// Every disc of her ground there is: the patches, fading over their last
/// second, and the ground each stain and heart of a Blood mage holds.
fn gather(w: &sim::World, discs: &mut Vec<Disc>) {
    discs.clear();
    let f = |v: sim::Fx| v.to_f32_for_render();
    for p in w.ground.iter().filter(|p| p.live()) {
        discs.push(Disc {
            x: f(p.at.x),
            y: f(p.at.y),
            z: f(p.at.z),
            r: f(p.radius()),
            fade: (p.life as f32 / FADE).min(1.0),
        });
    }
    for e in w.effects.iter().flatten() {
        if e.class != sim::Class::BloodMage {
            continue;
        }
        let r = if e.is_a_pool() {
            e.stain_ground()
        } else if e.is_a_heart() {
            sim::tuning::heart_ground()
        } else {
            continue;
        };
        discs.push(Disc {
            x: f(e.pos.x),
            y: f(e.pos.y),
            z: f(e.pos.z),
            r: f(r),
            fade: 1.0,
        });
    }
}

/// The floor under grid point `(i, k)`, as seen from a little over `from`:
/// the simulation's own answer, asked once and kept.
fn floor(
    w: &sim::World,
    heights: &mut HashMap<(i32, i32, i32), f32>,
    i: i32,
    k: i32,
    from: f32,
) -> f32 {
    let band = (from * 2.0).round() as i32;
    *heights.entry((i, k, band)).or_insert_with(|| {
        let to = |v: f32| sim::Fx::from_raw((v * 65536.0) as i32);
        let over = sim::V3::new(
            to(i as f32 * STEP),
            to(band as f32 * 0.5 + 1.0),
            to(k as f32 * STEP),
        );
        w.terrain().ground_under(over).to_f32_for_render()
    })
}

/// Every grid point near a disc: how far inside the union it is, how solid,
/// and the floor there. A disc counts at a point only where the floor is
/// near its own height (`tuning::ground_slack`), as `on_blood` asks: a disc
/// beside a wall does not climb it.
fn lay_grid(w: &sim::World, keep: &mut Drape) {
    let slack = sim::tuning::ground_slack().to_f32_for_render();
    keep.points.clear();
    for d in &keep.discs {
        let reach = d.r + STEP;
        let (i0, i1) = (
            ((d.x - reach) / STEP).floor() as i32,
            ((d.x + reach) / STEP).ceil() as i32,
        );
        let (k0, k1) = (
            ((d.z - reach) / STEP).floor() as i32,
            ((d.z + reach) / STEP).ceil() as i32,
        );
        for i in i0..=i1 {
            for k in k0..=k1 {
                let (px, pz) = (i as f32 * STEP, k as f32 * STEP);
                let inside = d.r - ((px - d.x).powi(2) + (pz - d.z).powi(2)).sqrt();
                let y = floor(w, &mut keep.heights, i, k, d.y);
                let inside = if (y - d.y).abs() > slack {
                    -STEP
                } else {
                    inside
                };
                let fade = if inside > 0.0 { d.fade } else { 0.0 };
                keep.points
                    .entry((i, k))
                    .and_modify(|p| {
                        if inside > p.inside {
                            p.inside = inside;
                            p.y = y;
                        }
                        p.fade = p.fade.max(fade);
                    })
                    .or_insert(Point { inside, fade, y });
            }
        }
    }
}

/// **Marching squares**: every grid square with a corner inside the union
/// becomes the part of itself that is inside, its edges cut where the
/// distance crosses nought, so the boundary is round.
fn march(keep: &mut Drape) {
    keep.positions.clear();
    keep.colours.clear();
    let outside = Point {
        inside: -STEP,
        fade: 0.0,
        y: 0.0,
    };
    let corners: Vec<(i32, i32)> = keep.points.keys().copied().collect();
    for (i, k) in corners {
        // Corners counter-clockwise from the square's own: (i, k), (i+1, k),
        // (i+1, k+1), (i, k+1).
        let at = [(i, k), (i + 1, k), (i + 1, k + 1), (i, k + 1)];
        let pts = at.map(|c| *keep.points.get(&c).unwrap_or(&outside));
        if pts.iter().all(|p| p.inside <= 0.0) {
            continue;
        }
        // The heights of an outside corner are its neighbours', so a cut
        // edge does not dive to nought.
        let known = pts
            .iter()
            .filter(|p| p.inside > 0.0)
            .map(|p| p.y)
            .fold(f32::MIN, f32::max);
        let corner = |n: usize| {
            let p = pts[n];
            let y = if p.inside > 0.0 { p.y } else { known };
            (
                Vec3::new(at[n].0 as f32 * STEP, y, at[n].1 as f32 * STEP),
                p,
            )
        };
        let cut = |a: usize, b: usize| {
            let ((va, pa), (vb, pb)) = (corner(a), corner(b));
            let t = (pa.inside / (pa.inside - pb.inside)).clamp(0.0, 1.0);
            (va.lerp(vb, t), pa.fade + (pb.fade - pa.fade) * t)
        };
        // The polygon inside the square: walk its edges, keeping inside
        // corners and the cuts where an edge leaves or enters.
        let mut poly: [(Vec3, f32); 8] = [(Vec3::ZERO, 0.0); 8];
        let mut n = 0;
        for e in 0..4 {
            let (a, b) = (e, (e + 1) % 4);
            if pts[a].inside > 0.0 {
                let (v, p) = corner(a);
                poly[n] = (v, p.fade);
                n += 1;
            }
            if (pts[a].inside > 0.0) != (pts[b].inside > 0.0) {
                poly[n] = cut(a, b);
                n += 1;
            }
        }
        // A fan from the first point, counter-clockwise seen from above.
        for t in 1..n.saturating_sub(1) {
            for (v, fade) in [poly[0], poly[t + 1], poly[t]] {
                keep.positions.push([v.x, v.y + LIFT, v.z]);
                keep.colours.push([1.0, 1.0, 1.0, fade]);
            }
        }
    }
}
