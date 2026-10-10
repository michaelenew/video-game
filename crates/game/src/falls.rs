//! **The Waterfall, drawn** (`sim::arena::waterfall`): the river over the
//! cliff's top, the curtain of water falling from the lip, the white streaks
//! running down it, and the foam where it lands. The cliff itself is basalt,
//! drawn as a form (`crate::forms`, `Form::Basalt`).
//!
//! Every shape is read off the waterfall's table, so a sheet moved in `sim`
//! moves here; every colour is `look::Palette`'s (`falling_water`, `foam`).
//! **The water runs on the simulation's frame**, not the renderer's clock:
//! a streak's place is a function of `World::frame`, so a rollback that
//! re-simulates a few frames puts it back where it was rather than popping.
//! None of it collides: the push of the falling water is the simulation's.

use bevy::prelude::*;
use sim::arena::Arena;
use sim::arena::waterfall::{self, HEIGHT, Piece};

use crate::arenas::{Under, put};
use crate::paint::Materials;
use crate::shapes::hash01;

/// A streak of white water running down the curtain, from `top` to `bottom`.
#[derive(Component)]
pub struct Streak {
    top: f32,
    bottom: f32,
    /// Metres a second.
    speed: f32,
    /// Where in its run it starts, metres.
    phase: f32,
}

/// A puff of foam at the foot, swelling and settling.
#[derive(Component)]
pub struct Foam {
    size: Vec3,
    phase: f32,
}

fn m(cm: i32) -> f32 {
    cm as f32 / 100.0
}

/// A piece's corners in metres, in the frame `at` (centimetres).
fn corners(p: &Piece, at: [i32; 3]) -> (Vec3, Vec3) {
    (
        Vec3::new(
            m(p.min[0] + at[0]),
            m(p.min[1] + at[1]),
            m(p.min[2] + at[2]),
        ),
        Vec3::new(
            m(p.max[0] + at[0]),
            m(p.max[1] + at[1]),
            m(p.max[2] + at[2]),
        ),
    )
}

fn translucent(rgb: [f32; 3], alpha: f32) -> StandardMaterial {
    let c = look::tint::linear(rgb);
    StandardMaterial {
        base_color: Color::linear_rgba(c[0], c[1], c[2], alpha),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    }
}

/// **The curtain**: a sheet bowed out from the lip, as water leaving an edge
/// falls away from it, in strips down its height. Corners in metres.
fn curtain(lip: f32, out: f32, top: f32, bottom: f32, z0: f32, z1: f32) -> Mesh {
    const ROWS: usize = 12;
    let mut ps = Vec::new();
    let mut ix = Vec::new();
    for r in 0..=ROWS {
        let t = r as f32 / ROWS as f32;
        // Out from the lip as it falls: fast at first, then nearly straight.
        let x = lip - out * (1.0 - (1.0 - t) * (1.0 - t));
        let y = top + (bottom - top) * t;
        ps.push([x, y, z0]);
        ps.push([x, y, z1]);
        if r > 0 {
            let k = (2 * r) as u32;
            ix.extend_from_slice(&[k - 2, k - 1, k, k - 1, k + 1, k]);
        }
    }
    let ns = vec![[-1.0, 0.0, 0.0]; ps.len()];
    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, ps)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, ns)
    .with_inserted_indices(bevy::render::mesh::Indices::U32(ix))
}

/// Draw a place's waterfall, if it has one. Called by `arenas::draw_place`,
/// so everything goes where the place's other pieces go.
pub fn draw(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Materials,
    arena: &'static Arena,
    palette: &look::Palette,
    under: Under,
) {
    let Some(at) = waterfall::frame_in(arena.id) else {
        return;
    };
    let (water, foam) = (palette.falling_water(), palette.foam());
    let (sheet_lo, sheet_hi) = corners(&waterfall::SHEET, at);
    let (river_lo, river_hi) = corners(&waterfall::RIVER, at);
    let (pool_lo, _) = corners(&waterfall::POOL, at);
    let surface = pool_lo.y + m(waterfall::POOL.max[1] - waterfall::POOL.min[1]);
    let lip = river_lo.x;
    let top = m(at[1] + HEIGHT);
    let (z0, z1) = (sheet_lo.z, sheet_hi.z);

    // The river over the top, to the lip.
    put(
        commands,
        under,
        (
            Mesh3d(
                meshes.add(
                    Plane3d::default()
                        .mesh()
                        .size(river_hi.x - river_lo.x, river_hi.z - river_lo.z),
                ),
            ),
            MeshMaterial3d(materials.standard.add(translucent(water, 0.7))),
            Transform::from_xyz(
                (river_lo.x + river_hi.x) * 0.5,
                top + 0.04,
                (river_lo.z + river_hi.z) * 0.5,
            ),
            bevy::pbr::NotShadowCaster,
        ),
    );
    // The curtain, two layers: a fainter one behind, so it has depth.
    let out = (lip - (sheet_lo.x + sheet_hi.x) * 0.5).max(0.5);
    for (k, alpha) in [(0.0f32, 0.55f32), (0.25, 0.3)] {
        put(
            commands,
            under,
            (
                Mesh3d(meshes.add(curtain(lip + k, out, top, surface, z0, z1))),
                MeshMaterial3d(materials.standard.add(translucent(water, alpha))),
                Transform::default(),
                bevy::pbr::NotShadowCaster,
            ),
        );
    }
    // The streaks: white, running down the front of the curtain.
    let streak = materials.standard.add(translucent(foam, 0.75));
    let fall = top - surface;
    for i in 0..18u32 {
        let z = z0 + 0.2 + (z1 - z0 - 0.4) * hash01(i, 1, 31);
        let len = 2.0 + 3.0 * hash01(i, 2, 31);
        let wide = 0.18 + 0.25 * hash01(i, 3, 31);
        put(
            commands,
            under,
            (
                Mesh3d(meshes.add(Cuboid::new(0.06, len, wide))),
                MeshMaterial3d(streak.clone()),
                Transform::from_xyz(lip - out - 0.08, top, z),
                bevy::pbr::NotShadowCaster,
                Streak {
                    top,
                    bottom: surface,
                    speed: 9.0 + 4.0 * hash01(i, 4, 31),
                    phase: fall * hash01(i, 5, 31),
                },
            ),
        );
    }
    // The foam at the foot, and the spray hanging over it.
    let foot = Vec3::new(lip - out, surface, (z0 + z1) * 0.5);
    let puff = materials.standard.add(translucent(foam, 0.5));
    for i in 0..9u32 {
        let off = Vec3::new(
            (hash01(i, 6, 37) - 0.5) * 3.0,
            0.1,
            (hash01(i, 7, 37) - 0.5) * (z1 - z0 + 2.0),
        );
        let size = Vec3::new(1.4, 0.5, 1.4) * (0.7 + 0.8 * hash01(i, 8, 37));
        put(
            commands,
            under,
            (
                Mesh3d(meshes.add(Sphere::new(0.5))),
                MeshMaterial3d(puff.clone()),
                Transform::from_translation(foot + off).with_scale(size),
                bevy::pbr::NotShadowCaster,
                Foam {
                    size,
                    phase: hash01(i, 9, 37) * std::f32::consts::TAU,
                },
            ),
        );
    }
    put(
        commands,
        under,
        (
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.standard.add(translucent(foam, 0.07))),
            Transform::from_translation(foot + Vec3::new(-1.0, 2.0, 0.0)).with_scale(Vec3::new(
                6.0,
                4.5,
                z1 - z0 + 5.0,
            )),
            bevy::pbr::NotShadowCaster,
        ),
    );
}

/// **Each frame**: the water runs, on the simulation's clock.
pub fn update(
    sim: Res<crate::Sim>,
    mut streaks: Query<(&Streak, &mut Transform), Without<Foam>>,
    mut foam: Query<(&Foam, &mut Transform), Without<Streak>>,
) {
    let seconds = sim.cur.frame as f32 / 60.0;
    for (s, mut t) in &mut streaks {
        let span = (s.top - s.bottom).max(1.0);
        let down = (seconds * s.speed + s.phase).rem_euclid(span);
        t.translation.y = s.top - down;
    }
    for (f, mut t) in &mut foam {
        let k = 1.0 + 0.18 * (seconds * 3.0 + f.phase).sin();
        t.scale = f.size * Vec3::new(k, 2.0 - k, k);
    }
}
