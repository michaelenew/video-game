//! Drawing what a fight puts on the ground (bestiary P4 and P7), any species,
//! from the simulation's own lists.
//!
//! **The floor hazards are drawn from the list the simulation feels**:
//! `World::terrain().floor`, each hazard placed in the world exactly where
//! its slow, its pull and its fire are tested -- a disc as a disc of its
//! radius, a strand as a line of its width, a cloud as a column of its
//! height, a vent carried by the part it is on. The solids a fight raises
//! (slag) are the boxes the collision reads, and a defended thing is the box
//! a creature's blow reaches, with a bar over it for what it has left. One
//! source of truth: if you can see it, it is there.
//!
//! A fixed pool of meshes spawned once, so nothing is spawned in a fight.
//! The debug overlay (F1) outlines all of it, and draws the noise ring -- the
//! loud things a creature that hears can perceive -- as rings that fade.

use bevy::prelude::*;
use sim::arena::{MAX_RAISED, Material};
use sim::hazard::{MAX_HAZARDS, Placed, Shape};
use sim::objective::{self, MAX_STANDING};

use crate::arenas::colour;

/// Which pool a mesh belongs to, and its index in it.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Piece {
    Hazard(usize),
    Raised(usize),
    Objective(usize),
    /// What is left of an objective: a bar over it.
    Bar(usize),
}

/// Materials, one per floor material, and a burning one and a cloud.
#[derive(Resource)]
pub struct Looks {
    by: Vec<Handle<StandardMaterial>>,
    fire: Handle<StandardMaterial>,
    cloud: Handle<StandardMaterial>,
    bar: Handle<StandardMaterial>,
    disc: Handle<Mesh>,
    cube: Handle<Mesh>,
}

const MATERIALS: [Material; 10] = [
    Material::Ground,
    Material::Grass,
    Material::Rock,
    Material::Stone,
    Material::Sand,
    Material::Snow,
    Material::Ash,
    Material::Peat,
    Material::Water,
    Material::Wood,
];

impl Looks {
    fn of(&self, m: Material) -> Handle<StandardMaterial> {
        let i = MATERIALS.iter().position(|x| *x == m).unwrap_or(0);
        self.by[i].clone()
    }

    /// What a hazard is drawn in: fire glows, a cloud is see-through, the
    /// rest is its material, a little lighter so it reads against the floor.
    fn hazard(&self, h: &Placed) -> Handle<StandardMaterial> {
        if h.decl.burns {
            self.fire.clone()
        } else if h.decl.blocks_sight {
            self.cloud.clone()
        } else {
            self.of(h.decl.material)
        }
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let by = MATERIALS
        .iter()
        .map(|m| {
            let [r, g, b] = colour(*m);
            materials.add(StandardMaterial {
                base_color: Color::srgb(r * 1.25, g * 1.25, b * 1.25),
                perceptual_roughness: 0.6,
                ..default()
            })
        })
        .collect();
    let fire = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.35, 0.08),
        emissive: LinearRgba::rgb(2.4, 0.7, 0.1),
        ..default()
    });
    let cloud = materials.add(StandardMaterial {
        base_color: Color::srgba(0.55, 0.55, 0.58, 0.55),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        ..default()
    });
    let bar = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.75, 0.25),
        emissive: LinearRgba::rgb(0.6, 0.5, 0.1),
        unlit: true,
        ..default()
    });
    let disc = meshes.add(Cylinder::new(1.0, 1.0));
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let pieces = (0..MAX_HAZARDS)
        .map(Piece::Hazard)
        .chain((0..MAX_RAISED).map(Piece::Raised))
        .chain((0..MAX_STANDING).map(Piece::Objective))
        .chain((0..MAX_STANDING).map(Piece::Bar));
    for piece in pieces {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(bar.clone()),
            Transform::default(),
            Visibility::Hidden,
            piece,
        ));
    }
    commands.insert_resource(Looks {
        by,
        fire,
        cloud,
        bar,
        disc,
        cube,
    });
}

/// How thick a hazard lying on the floor is drawn: enough to show over the
/// floor's own regions, not enough to read as a step.
const SKIN: f32 = 0.03;

/// Put every piece where the snapshot says.
pub fn place(
    sim: Res<crate::Sim>,
    looks: Res<Looks>,
    mut pieces: Query<(
        &Piece,
        &mut Transform,
        &mut Visibility,
        &mut Mesh3d,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let w = &sim.cur;
    if w.lore.owner.is_none() {
        for (_, _, mut visible, _, _) in pieces.iter_mut() {
            *visible = Visibility::Hidden;
        }
        return;
    }
    let ground = w.terrain();
    let hazards: Vec<Placed> = ground.floor.iter().copied().collect();
    let (raised, raised_n) = ground.floor.solids::<MAX_RAISED>();
    let standing: Vec<objective::Standing> = objective::standing(&w.lore, w.arena()).collect();
    for (piece, mut transform, mut visible, mut mesh, mut material) in pieces.iter_mut() {
        let shown = match *piece {
            Piece::Hazard(i) => hazards.get(i).map(|h| {
                let a = v3(h.a);
                let b = v3(h.b);
                let r = fx(h.radius);
                // A column is drawn as tall as it reaches; a puddle as a skin.
                let tall = if h.decl.blocks_sight || fx(h.height) > 1.0 {
                    fx(h.height).max(SKIN)
                } else {
                    SKIN
                };
                let lift = if tall > SKIN {
                    0.0
                } else {
                    0.01 + 0.002 * i as f32
                };
                match h.decl.shape {
                    Shape::Disc => (
                        looks.disc.clone(),
                        Transform {
                            translation: a + Vec3::Y * (lift + tall * 0.5),
                            rotation: Quat::IDENTITY,
                            scale: Vec3::new(r, tall, r),
                        },
                        looks.hazard(h),
                    ),
                    Shape::Strand => {
                        let along = b - a;
                        let len = along.length().max(0.01);
                        let yaw = (-along.z).atan2(along.x);
                        (
                            looks.cube.clone(),
                            Transform {
                                translation: (a + b) * 0.5 + Vec3::Y * (lift + tall * 0.5),
                                rotation: Quat::from_rotation_y(yaw),
                                scale: Vec3::new(len, tall, r * 2.0),
                            },
                            looks.hazard(h),
                        )
                    }
                }
            }),
            Piece::Raised(i) => (i < raised_n).then(|| {
                let s = raised[i];
                let (lo, hi) = (v3(s.min), v3(s.max));
                (
                    looks.cube.clone(),
                    Transform {
                        translation: (lo + hi) * 0.5,
                        rotation: Quat::IDENTITY,
                        scale: hi - lo,
                    },
                    looks.of(s.material),
                )
            }),
            Piece::Objective(i) => standing.get(i).filter(|o| !o.state.broken).map(|o| {
                let s = o.solid();
                let (lo, hi) = (v3(s.min), v3(s.max));
                (
                    looks.cube.clone(),
                    Transform {
                        translation: (lo + hi) * 0.5,
                        rotation: Quat::IDENTITY,
                        scale: hi - lo,
                    },
                    looks.of(o.site.material),
                )
            }),
            Piece::Bar(i) => standing.get(i).filter(|o| !o.state.broken).map(|o| {
                let s = o.solid();
                let share = (o.state.health as f32 / o.full.max(1) as f32).clamp(0.0, 1.0);
                let width = fx(s.max.x.sub(s.min.x)).max(fx(s.max.z.sub(s.min.z)));
                let top = v3(o.at) + Vec3::Y * (fx(s.max.y.sub(s.min.y)) + 0.6);
                (
                    looks.cube.clone(),
                    Transform {
                        translation: top,
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(width * share, 0.15, 0.15),
                    },
                    looks.bar.clone(),
                )
            }),
        };
        match shown {
            Some((m, t, paint)) => {
                if mesh.0 != m {
                    mesh.0 = m;
                }
                if material.0 != paint {
                    material.0 = paint;
                }
                *transform = t;
                *visible = Visibility::Inherited;
            }
            None => *visible = Visibility::Hidden,
        }
    }
}

/// The overlay: each hazard's outline at the radius its effects are tested
/// at, each raised solid's and objective's box, and the noise ring.
pub fn overlay(show: Res<crate::debug::ShowDebug>, sim: Res<crate::Sim>, mut gizmos: Gizmos) {
    if !show.0 {
        return;
    }
    let w = &sim.cur;
    if w.lore.owner.is_none() {
        return;
    }
    let flat = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    let ground = w.terrain();
    for h in ground.floor.iter() {
        let colour = if h.decl.burns {
            FIRE
        } else if h.decl.solid {
            SOLID
        } else {
            HAZARD
        };
        match h.decl.shape {
            Shape::Disc => {
                for y in [0.0, fx(h.height)] {
                    gizmos.circle(
                        Isometry3d::new(v3(h.a) + Vec3::Y * (y + 0.02), flat),
                        fx(h.radius),
                        colour,
                    );
                }
            }
            Shape::Strand => {
                let (a, b) = (v3(h.a), v3(h.b));
                let across = (b - a).normalize_or_zero().cross(Vec3::Y) * fx(h.radius);
                gizmos.line(a + across, b + across, colour);
                gizmos.line(a - across, b - across, colour);
            }
        }
    }
    for s in ground.raised() {
        let (lo, hi) = (v3(s.min), v3(s.max));
        gizmos.cuboid(
            Transform::from_translation((lo + hi) * 0.5).with_scale(hi - lo),
            SOLID,
        );
    }
    for n in sim::noise::all(&w.lore) {
        let age = w.frame.wrapping_sub(n.born) as f32;
        let fade = (1.0 - age / 90.0).clamp(0.0, 1.0);
        if fade <= 0.0 {
            continue;
        }
        gizmos.circle(
            Isometry3d::new(v3(n.pos()) + Vec3::Y * 0.03, flat),
            0.3 + age * 0.05,
            NOISE.with_alpha(fade),
        );
    }
}

const HAZARD: Color = Color::srgb(0.3, 0.9, 0.6);
const FIRE: Color = Color::srgb(1.0, 0.45, 0.1);
const SOLID: Color = Color::srgb(0.85, 0.85, 0.9);
const NOISE: Color = Color::srgb(0.95, 0.95, 0.4);

/// RENDER-ONLY. Fixed point to metres.
fn fx(v: sim::Fx) -> f32 {
    v.to_f32_for_render()
}

fn v3(v: sim::V3) -> Vec3 {
    Vec3::new(fx(v.x), fx(v.y), fx(v.z))
}
