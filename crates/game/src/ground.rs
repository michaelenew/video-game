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
use sim::species::{MAX_MARKS, Mark, MarkLook};

use crate::arenas::colour;
use sim::arena::ArenaId;

/// Which pool a mesh belongs to, and its index in it.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Piece {
    Hazard(usize),
    Raised(usize),
    Objective(usize),
    /// What is left of an objective: a bar over it.
    Bar(usize),
    /// A species' own mark (`World::marks`): a ring on the floor or a thing
    /// standing in the arena.
    Mark(usize),
    /// The same mark's progress: a ring filling to its edge, embers rising.
    MarkFill(usize),
}

/// Materials, one per floor material, and a burning one and a cloud.
#[derive(Resource)]
pub struct Looks {
    by: Vec<Handle<StandardMaterial>>,
    fire: Handle<StandardMaterial>,
    cloud: Handle<StandardMaterial>,
    bar: Handle<StandardMaterial>,
    /// A species' marks, by `MarkLook`, and a ring's fill.
    warning: Handle<StandardMaterial>,
    warning_fill: Handle<StandardMaterial>,
    kindling: Handle<StandardMaterial>,
    iron: Handle<StandardMaterial>,
    embers: Handle<StandardMaterial>,
    /// Raised sand, a fin, a noise heard, what a creature feels.
    sand: Handle<StandardMaterial>,
    fin: Handle<StandardMaterial>,
    heard: Handle<StandardMaterial>,
    feel: Handle<StandardMaterial>,
    /// The Mantis's notches: one per class, dim, then lit (`MarkLook::Notch`).
    notches: Vec<Handle<StandardMaterial>>,
    disc: Handle<Mesh>,
    cube: Handle<Mesh>,
    /// For a raised solid of bare rock (the herd's boulders, the Mireback's
    /// slag): a rock, inside the box the body collides against
    /// (`docs/design/forms.md`). Two of them, so a field of boulders is not
    /// one boulder repeated.
    rocks: [Handle<Mesh>; 2],
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

/// What a hazard or a raised solid is drawn in: the arena's own colour for
/// that material, lifted a little so a thing standing on the floor is not the
/// floor.
///
/// The lift used to be a flat 1.25x on an already-dark colour, which is how a
/// raised stone and the floor it stands on ended up the same grey. A step in
/// lightness reads as a step; a step in brightness reads as a lighting bug.
fn floor_colour(id: ArenaId, m: Material) -> [f32; 3] {
    look::tint::Lch::of(look::palette::of(id).of(m))
        .lighter(0.05)
        .rgb()
}

/// Repaint the pooled materials for an arena, in place.
///
/// The pool is spawned once and never respawned -- a fight must not allocate --
/// so an arena change edits the materials the handles already point at.
pub fn repaint(looks: &Looks, materials: &mut Assets<StandardMaterial>, id: ArenaId) {
    for (m, handle) in MATERIALS.iter().zip(&looks.by) {
        if let Some(mat) = materials.get_mut(handle) {
            let [r, g, b] = floor_colour(id, *m);
            mat.base_color = Color::srgb(r, g, b);
        }
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // The proving ground's, to start. `repaint` swaps them for the arena's own
    // the moment one is drawn, in place, so nothing here is spawned twice.
    let by: Vec<_> = MATERIALS
        .iter()
        .map(|m| {
            let [r, g, b] = floor_colour(ArenaId::PROVING_GROUND, *m);
            materials.add(StandardMaterial {
                base_color: Color::srgb(r, g, b),
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
    let see_through = |rgb: [f32; 3], alpha: f32, glow: f32| StandardMaterial {
        base_color: Color::srgba(rgb[0], rgb[1], rgb[2], alpha),
        emissive: LinearRgba::rgb(rgb[0] * glow, rgb[1] * glow, rgb[2] * glow),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    };
    let warning = materials.add(see_through([0.95, 0.25, 0.12], 0.22, 0.4));
    let warning_fill = materials.add(see_through([1.0, 0.35, 0.15], 0.45, 0.8));
    let kindling = materials.add(see_through([1.0, 0.62, 0.15], 0.5, 1.6));
    let iron = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.15, 0.14),
        perceptual_roughness: 0.5,
        metallic: 0.6,
        ..default()
    });
    let embers = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.12, 0.04),
        emissive: LinearRgba::rgb(0.9, 0.18, 0.02),
        ..default()
    });
    // The Sandmaw's: a wake a shade darker than the sand it is in, so it
    // reads at dusk; a dark fin; a pale ring for a noise it heard; and the
    // faintest disc for how far it feels.
    let [sr, sg, sb] = colour(ArenaId::SANDMAW, Material::Sand);
    let sand = materials.add(StandardMaterial {
        base_color: Color::srgb(sr * 0.82, sg * 0.78, sb * 0.72),
        perceptual_roughness: 0.95,
        ..default()
    });
    let fin = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.14, 0.11),
        perceptual_roughness: 0.6,
        ..default()
    });
    let heard = materials.add(see_through([1.0, 0.95, 0.8], 0.35, 0.9));
    let feel = materials.add(see_through([0.55, 0.45, 0.3], 0.12, 0.2));
    // A notch in its thrower's class colour: dim while it is only
    // remembered, burning while a Ready waits for it.
    let notches = [false, true]
        .into_iter()
        .flat_map(|lit| {
            NOTCH_COLOURS.map(|rgb| {
                let glow = if lit { 2.0 } else { 0.35 };
                StandardMaterial {
                    base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
                    emissive: LinearRgba::rgb(rgb[0] * glow, rgb[1] * glow, rgb[2] * glow),
                    ..default()
                }
            })
        })
        .map(|m| materials.add(m))
        .collect();
    let disc = meshes.add(Cylinder::new(1.0, 1.0));
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let rocks = [
        meshes.add(crate::shapes::rock(Vec3::ONE, 3, None)),
        meshes.add(crate::shapes::rock(Vec3::ONE, 11, None)),
    ];
    let pieces = (0..MAX_HAZARDS)
        .map(Piece::Hazard)
        .chain((0..MAX_RAISED).map(Piece::Raised))
        .chain((0..MAX_STANDING).map(Piece::Objective))
        .chain((0..MAX_STANDING).map(Piece::Bar))
        .chain((0..MAX_MARKS).map(Piece::Mark))
        .chain((0..MAX_MARKS).map(Piece::MarkFill));
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
        warning,
        warning_fill,
        kindling,
        iron,
        embers,
        sand,
        fin,
        heard,
        feel,
        notches,
        disc,
        cube,
        rocks,
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
    let marks = w.marks();
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
                let form = if s.material == sim::arena::Material::Rock {
                    looks.rocks[i % 2].clone()
                } else {
                    looks.cube.clone()
                };
                (
                    form,
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
            Piece::Mark(i) => marks.items[i].map(|m| mark(&looks, &m, false)),
            Piece::MarkFill(i) => marks.items[i]
                .filter(|m| {
                    m.progress.raw() > 0 && matches!(m.look, MarkLook::Warning | MarkLook::Embers)
                })
                .map(|m| mark(&looks, &m, true)),
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

/// The classes' colours, in `sim::class::ALL_CLASSES` order, for the
/// Mantis's notches: Bulwark, Champion, Shadow Reaver, Elementalist, Blood
/// mage, Dual mage.
const NOTCH_COLOURS: [[f32; 3]; 6] = [
    [0.35, 0.55, 0.95],
    [0.95, 0.75, 0.20],
    [0.55, 0.30, 0.85],
    [0.25, 0.85, 0.55],
    [0.90, 0.15, 0.20],
    [0.95, 0.95, 0.95],
];

/// How high a ring on the floor is drawn: over any hazard's skin, so a
/// warning on tar still reads.
const RING_LIFT: f32 = 0.07;

/// A species' mark: a ring on the floor (filling to its edge with progress),
/// or a column standing in the arena (its fill rising with progress).
fn mark(
    looks: &Looks,
    m: &Mark,
    fill: bool,
) -> (Handle<Mesh>, Transform, Handle<StandardMaterial>) {
    let at = v3(m.at);
    let r = fx(m.radius);
    let progress = fx(m.progress).clamp(0.0, 1.0);
    let height = fx(m.height);
    if height <= 0.0 {
        let (radius, lift, paint) = match (m.look, fill) {
            (MarkLook::Kindling, _) => (r, RING_LIFT, looks.kindling.clone()),
            // A noise heard spreads a little as it fades.
            (MarkLook::Heard, _) => (
                r * (0.6 + 0.4 * progress),
                RING_LIFT + 0.002,
                looks.heard.clone(),
            ),
            (MarkLook::Feel, _) => (r, RING_LIFT - 0.02, looks.feel.clone()),
            (_, false) => (r, RING_LIFT, looks.warning.clone()),
            (_, true) => (r * progress, RING_LIFT + 0.005, looks.warning_fill.clone()),
        };
        return (
            looks.disc.clone(),
            Transform {
                translation: at + Vec3::Y * lift,
                rotation: Quat::IDENTITY,
                scale: Vec3::new(radius, 0.01, radius),
            },
            paint,
        );
    }
    let (tall, radius, paint) = match (m.look, fill) {
        (MarkLook::Flame, _) => (height, r, looks.fire.clone()),
        (MarkLook::Iron, _) => (height, r, looks.iron.clone()),
        (MarkLook::Embers, false) => (height, r, looks.iron.clone()),
        (MarkLook::Embers, true) => (height * progress, r * 1.05, looks.embers.clone()),
        (MarkLook::Sand, _) => (height, r, looks.sand.clone()),
        (MarkLook::Fin, _) => (height, r, looks.fin.clone()),
        (MarkLook::Notch { class, lit }, _) => {
            let i = (class as usize).min(NOTCH_COLOURS.len() - 1)
                + if lit { NOTCH_COLOURS.len() } else { 0 };
            (height, r, looks.notches[i].clone())
        }
        (_, _) => (height, r, looks.warning.clone()),
    };
    (
        looks.disc.clone(),
        Transform {
            translation: at + Vec3::Y * (tall * 0.5),
            rotation: Quat::IDENTITY,
            scale: Vec3::new(radius, tall.max(0.01), radius),
        },
        paint,
    )
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
