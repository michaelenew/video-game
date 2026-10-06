//! How each arena is drawn: its floor, its solids and its dressing.
//!
//! **The geometry is the simulation's.** Every solid is drawn from
//! `sim::arena::Arena::solids`, the same boxes bodies collide against, and the
//! floor from its bounds and material regions -- if you can see it, you
//! collide with it, and a region you can see is the region the simulation
//! answers for. What an arena adds here is presentation only: [`Dressing`],
//! things to look at that nothing collides with (bones in the sand, reeds, a
//! banner), and the colour of its sky.
//!
//! Every planned creature's arena already has a line in [`dressing`],
//! commented out and one blank line from the next, so two branches each adding
//! an arena do not conflict (the same arrangement as `crate::species`).
//!
//! The scenery is rebuilt when the arena changes -- the picker, a reset into
//! another one -- and not otherwise. That is the one time this spends meshes,
//! and it is on the frame the fight starts over anyway.

use bevy::prelude::*;
use sim::arena::{Area, ArenaId, Material};

use crate::sky;

pub mod proving_ground;

pub mod range;

pub mod gnawers;

pub mod hornback;

pub mod mireback;

pub mod sandmaw;

pub mod pair;

pub mod broodmother;

pub mod veilstalker;

pub mod mantis;

pub mod galewing;

pub mod siegeshell;

pub mod climb;

/// One thing to look at that nothing collides with.
#[derive(Clone, Copy, Debug)]
pub struct Prop {
    pub shape: Shape,
    /// The middle of its base, in metres.
    pub at: [f32; 3],
    /// Width, height, depth: a box's sides, a cylinder's diameter and height,
    /// a sphere's diameter.
    pub size: [f32; 3],
    /// Turned about the vertical, in turns.
    pub yaw: f32,
    pub rgb: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Box,
    Cylinder,
    Sphere,
}

/// What an arena adds to its geometry, for the eye only.
#[derive(Clone, Copy, Debug)]
pub struct Dressing {
    pub drop: bool,
    pub props: &'static [Prop],
}

/// An arena's dressing. One with none is drawn bare under the proving
/// ground's sky, which is plain but honest: the geometry is all there.
pub fn dressing(id: ArenaId) -> &'static Dressing {
    match id {
        ArenaId::RANGE => &range::DRESSING,

        ArenaId::GNAWERS => &gnawers::DRESSING,

        ArenaId::HORNBACK => &hornback::DRESSING,

        ArenaId::HORNBACK_CROSSING => &hornback::CROSSING,

        ArenaId::MIREBACK => &mireback::DRESSING,

        ArenaId::SANDMAW => &sandmaw::DRESSING,

        ArenaId::PAIR => &pair::DRESSING,

        ArenaId::BROODMOTHER => &broodmother::DRESSING,

        ArenaId::VEILSTALKER => &veilstalker::DRESSING,

        ArenaId::MANTIS => &mantis::DRESSING,

        ArenaId::GALEWING => &galewing::DRESSING,
        ArenaId::SIEGESHELL => &siegeshell::DRESSING,

        ArenaId::CLIMB_STAIR => &climb::STAIR,
        ArenaId::CLIMB_CAUSEWAY => &climb::CAUSEWAY,
        ArenaId::CLIMB_SPIRAL => &climb::SPIRAL,
        ArenaId::CLIMB_FALLS => &climb::FALLS,
        ArenaId::CLIMB_SLALOM => &climb::SLALOM,
        ArenaId::CLIMB_FORK => &climb::FORK,
        ArenaId::CLIMB_SPIRE => &climb::SPIRE,
        ArenaId::CLIMB_GULF => &climb::GULF,
        ArenaId::CLIMB_REACH => &climb::REACH,
        _ => &proving_ground::DRESSING,
    }
}

/// A material's colour in an arena, and the one place the game asks.
///
/// Thin on purpose: the answer is `look::palette`, which derives a whole
/// scheme from the arena's sky. Ten colours written here once, for the whole
/// game, is what made every arena the same ten colours.
pub fn colour(id: ArenaId, material: Material) -> [f32; 3] {
    look::palette::of(id).of(material)
}

/// **The key light**, the one that casts shadows: put where [`sun`] says
/// when the arena is drawn.
#[derive(Component)]
pub struct Sun;

/// Where the key light shines from, in metres, looking at the middle of the
/// arena. Most arenas take the low afternoon sun every fight was first lit
/// by. **The Cliffs take it nearly overhead** (galewing.md §11): the bird's
/// shadow on the plateau is a tell -- where it is over you when it is out of
/// frame -- and a low sun throws it off the plateau altogether.
pub fn sun(id: ArenaId) -> Vec3 {
    match id {
        ArenaId::GALEWING => Vec3::new(1.5, 20.0, 1.0),
        _ => Vec3::new(6.0, 14.0, 5.0),
    }
}

/// Everything drawn for the arena, so all of it can go when the arena does.
#[derive(Component)]
pub struct Scenery;

/// Which arena is drawn, if any yet.
#[derive(Resource, Default)]
pub struct Drawn(Option<ArenaId>);

/// How far the floor runs past the bounds, so the edge of the world is not the
/// edge of the walls.
const APRON: f32 = 12.0;

/// How wide a drop's floor is drawn: past the horizon from any island.
const DEEP: f32 = 2000.0;

/// Draw the arena the simulation is in, when it is not the one already drawn.
#[allow(clippy::too_many_arguments)] // A Bevy system: one argument per resource it reads.
pub fn dress(
    mut commands: Commands,
    sim: Res<crate::Sim>,
    mut drawn: ResMut<Drawn>,
    old: Query<Entity, With<Scenery>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut clear: ResMut<ClearColor>,
    mut suns: Query<&mut Transform, With<Sun>>,
    mut camera: Query<Entity, With<crate::MainCamera>>,
    looks: Option<Res<crate::ground::Looks>>,
) {
    let arena = sim.cur.arena();
    if drawn.0 == Some(arena.id) {
        return;
    }
    drawn.0 = Some(arena.id);
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let dressing = dressing(arena.id);
    let sun_at = sun(arena.id);
    // One table, keyed by arena: `look::skies`. Resolved once here rather than
    // per frame, and shared by the dome, the fog and the drop below, which is
    // what keeps all three agreeing about where the horizon is.
    let sky = look::skies::of(arena.id).resolved();
    // Derived from that sky, so an arena that has one has both. Everything
    // drawn below goes through it, props included.
    let palette = look::palette::Palette::under(&sky);
    // Everything the arena is made of is painted with this: the palette says
    // what colour a thing is, the edge rule says where its accent goes, and
    // `shapes` puts the answer in the vertices. One white material serves all
    // of it, because every mesh carries its own colour.
    let brush = crate::shapes::Brush {
        palette,
        edge: look::edge::EDGE,
    };
    let white = materials.add(crate::shapes::plain());

    // The clear colour still matters: it is what shows in the sliver of a frame
    // before the dome is drawn, and anywhere the dome does not reach. Set to
    // the horizon so that sliver is never a different colour from the sky.
    let h = sky.horizon;
    clear.0 = Color::srgb(h[0], h[1], h[2]);
    sky::raise(
        &mut commands,
        &mut meshes,
        &mut materials,
        &sky,
        sun_at,
        Scenery,
    );
    // Fog on the camera rather than on the scene, because it is a property of
    // looking rather than of the things looked at.
    if let Ok(eye) = camera.single_mut() {
        commands.entity(eye).insert(sky::fog(&sky));
    }
    // The pooled hazard and raised-solid materials take this arena's colours.
    if let Some(looks) = &looks {
        crate::ground::repaint(looks, &mut materials, arena.id);
    }
    for mut light in &mut suns {
        *light = Transform::from_translation(sun_at).looking_at(Vec3::ZERO, Vec3::Y);
    }

    // The drop's floor, unlit: made before `paint` borrows the materials.
    //
    // Unlit but **fogged**, which is the whole point of it now. A drop drawn
    // crisp from edge to edge is a dark floor somebody put under the level; the
    // same floor hazing toward the sky as it recedes is a long way down.
    let below = dressing.drop.then(|| {
        let rgb = sky.ground;
        materials.add(StandardMaterial {
            base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
            unlit: true,
            fog_enabled: true,
            ..default()
        })
    });
    let mut paint = |rgb: [f32; 3]| {
        materials.add(StandardMaterial {
            base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
            perceptual_roughness: 0.92,
            ..default()
        })
    };

    // The floor: its own material under everything, then each region on top,
    // a hair higher than the one before so a later region wins on screen the
    // way it wins in `Arena::floor_at`.
    let b = arena.bounds;
    let (lo_x, hi_x) = (fx(b.lo_x), fx(b.hi_x));
    let (lo_z, hi_z) = (fx(b.lo_z), fx(b.hi_z));
    match below {
        Some(dark) => {
            commands.spawn((
                Mesh3d(meshes.add(Plane3d::default().mesh().size(DEEP, DEEP))),
                MeshMaterial3d(dark),
                Transform::from_xyz((lo_x + hi_x) * 0.5, 0.0, (lo_z + hi_z) * 0.5),
                bevy::pbr::NotShadowReceiver,
                Scenery,
            ));
        }
        None => {
            let (w, d) = (hi_x - lo_x + APRON * 2.0, hi_z - lo_z + APRON * 2.0);
            // Subdivided, because a four-vertex plane has nowhere to put a
            // gradient. The accent runs in from the arena's own perimeter,
            // which is the edge of the world as far as anyone standing on it
            // is concerned.
            let mut floor = Plane3d::default()
                .mesh()
                .size(w, d)
                .subdivisions(crate::shapes::cuts_across(w.max(d), brush.edge.reach))
                .build();
            crate::shapes::paint(
                &mut floor,
                Vec3::new(w * 0.5, 0.0, d * 0.5),
                Vec3::ZERO,
                palette.of(arena.floor),
                &brush,
            );
            commands.spawn((
                Mesh3d(meshes.add(floor)),
                MeshMaterial3d(white.clone()),
                Transform::from_xyz((lo_x + hi_x) * 0.5, 0.0, (lo_z + hi_z) * 0.5),
                Scenery,
            ));
        }
    }
    for (i, region) in arena.regions.iter().enumerate() {
        let lift = 0.004 * (i + 1) as f32;
        let look = paint(palette.of(region.material));
        match region.area {
            Area::Rect { lo, hi } => {
                let (x0, z0, x1, z1) = (fx(lo.0), fx(lo.1), fx(hi.0), fx(hi.1));
                commands.spawn((
                    Mesh3d(meshes.add(Plane3d::default().mesh().size(x1 - x0, z1 - z0))),
                    MeshMaterial3d(look),
                    Transform::from_xyz((x0 + x1) * 0.5, lift, (z0 + z1) * 0.5),
                    Scenery,
                ));
            }
            Area::Disc { at, radius } => {
                commands.spawn((
                    Mesh3d(meshes.add(Circle::new(fx(radius)))),
                    MeshMaterial3d(look),
                    Transform::from_xyz(fx(at.0), lift, fx(at.1))
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                    Scenery,
                ));
            }
        }
    }

    // The geometry, straight from the simulation's own collision data. One
    // source of truth: if you can see it, you collide with it.
    for solid in arena.solids() {
        let min = Vec3::new(fx(solid.min.x), fx(solid.min.y), fx(solid.min.z));
        let max = Vec3::new(fx(solid.max.x), fx(solid.max.y), fx(solid.max.z));
        let size = max - min;
        let at = (min + max) * 0.5;
        commands.spawn((
            Mesh3d(meshes.add(crate::shapes::boxy(
                size,
                at,
                palette.of(solid.material),
                &brush,
            ))),
            MeshMaterial3d(white.clone()),
            Transform::from_translation(at),
            Scenery,
        ));
    }

    for prop in dressing.props {
        let [w, h, d] = prop.size;
        let at = Vec3::new(prop.at[0], prop.at[1] + h * 0.5, prop.at[2]);
        let rgb = palette.surface(prop.rgb);
        let half = Vec3::new(w * 0.5, h * 0.5, d * 0.5);
        let mesh = match prop.shape {
            Shape::Box => crate::shapes::boxy(Vec3::new(w, h, d), at, rgb, &brush),
            Shape::Cylinder => {
                let mut m = Cylinder::new(w * 0.5, h).mesh().build();
                crate::shapes::paint(&mut m, half, at, rgb, &brush);
                m
            }
            Shape::Sphere => {
                let mut m = Sphere::new(w * 0.5).mesh().build();
                crate::shapes::paint(&mut m, Vec3::splat(w * 0.5), at, rgb, &brush);
                m
            }
        };
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(white.clone()),
            Transform::from_xyz(prop.at[0], prop.at[1] + h * 0.5, prop.at[2])
                .with_rotation(Quat::from_rotation_y(-prop.yaw * std::f32::consts::TAU)),
            Scenery,
        ));
    }
}

/// RENDER-ONLY. Fixed point to metres.
fn fx(v: sim::Fx) -> f32 {
    v.to_f32_for_render()
}
