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

/// **The fill light**, which is the sky rather than the sun.
///
/// Nothing outdoors is lit by one light. The sun is one and the whole sky is
/// the other, and the sky is a different colour -- so the shadowed side of a
/// rock is not a darker version of its lit side, it is a *bluer* one. A shadow
/// painted as plain darkness looks dead; the same shadow in the complement of
/// the light looks like a real afternoon.
///
/// So this light and the ambient term both take `Palette::shade`, which is the
/// arena's own sky overhead -- exactly what is shining into every shadow in it.
/// Under a warm dawn the shadows come out violet; under a blue midday, deeper
/// blue. Nothing had to be chosen.
#[derive(Component)]
pub struct Skylight;

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
    mut fill: Query<&mut DirectionalLight, With<Skylight>>,
    mut ambient: ResMut<AmbientLight>,
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
    //
    // The crest half of the rule is stretched over the heights this arena
    // actually contains, read off its own collision data. A fixed ramp from the
    // ground reads beautifully in a proving ground whose walls are waist-high
    // and paints every surface of a jump course at full strength, because a
    // course's *lowest* island is already forty metres up.
    let (lo, hi) = arena
        .solids()
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), s| {
            (lo.min(fx(s.min.y)), hi.max(fx(s.max.y)))
        });
    let brush = crate::shapes::Brush {
        palette,
        edge: if lo <= hi {
            look::edge::EDGE.across(lo, hi)
        } else {
            look::edge::EDGE
        },
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
        // The line fades over the same distance the air does -- where there is
        // a line at all.
        if crate::platform::draws_outlines() {
            commands
                .entity(eye)
                .insert(crate::outline::Outline::over(look::edge::LINE, &sky));
        }
    }
    // The pooled hazard and raised-solid materials take this arena's colours.
    if let Some(looks) = &looks {
        crate::ground::repaint(looks, &mut materials, arena.id);
    }
    // Both the ambient term and the fill light take the sky's colour, so every
    // shadow in the arena is the complement of what cast it.
    let shade = palette.shade;
    ambient.color = Color::srgb(shade[0], shade[1], shade[2]);
    for mut light in &mut fill {
        light.color = Color::srgb(shade[0], shade[1], shade[2]);
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

    // **The floor casts no shadow.** Shadows fall on it, never from it: it
    // is the lowest thing there is, so a shadow map that has drawn it has
    // drawn eight thousand triangles -- the whole of its gradient's
    // subdivision -- that nothing can ever be in the shadow of. Every flat
    // thing lying on it (the regions, the drop below a course) is the same.
    // What stands on it -- the solids, the props, the bodies -- still casts.
    //
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
                // Nothing is under it to throw a shadow on, and it is two
                // kilometres across: drawn into every shadow cascade it was
                // most of each one's triangles. See the note on the floor.
                bevy::pbr::NotShadowCaster,
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
            // Ground, not paint: the colour wanders a little across it.
            crate::shapes::mottle(&mut floor, 0.07, 2.5, arena.id.0 as u32);
            commands.spawn((
                Mesh3d(meshes.add(floor)),
                MeshMaterial3d(white.clone()),
                Transform::from_xyz((lo_x + hi_x) * 0.5, 0.0, (lo_z + hi_z) * 0.5),
                bevy::pbr::NotShadowCaster,
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
                    bevy::pbr::NotShadowCaster,
                    Scenery,
                ));
            }
            Area::Disc { at, radius } => {
                commands.spawn((
                    Mesh3d(meshes.add(Circle::new(fx(radius)))),
                    MeshMaterial3d(look),
                    Transform::from_xyz(fx(at.0), lift, fx(at.1))
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                    bevy::pbr::NotShadowCaster,
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
        // The form follows the material (`docs/design/forms.md`): bare rock
        // is a rock, soft ground is rounded, dressed stone and timber keep
        // their edges. Every form stays inside the box the body collides
        // against.
        let rgb = palette.of(solid.material);
        let seed = (at.x * 7.0 + at.z * 13.0 + size.y * 3.0) as i32 as u32;
        let mesh = match solid.material {
            Material::Rock => crate::shapes::rock(size, seed, Some((at, rgb, &brush))),
            Material::Ground
            | Material::Grass
            | Material::Sand
            | Material::Snow
            | Material::Ash
            | Material::Peat => crate::shapes::soft_box(size, 0.3, 12, Some((at, rgb, &brush))),
            // Dressed stone is cut square and keeps its arris, chamfered just
            // enough not to catch the light as a wire; timber a little more.
            Material::Stone => crate::shapes::chamfered_box(size, 0.06, Some((at, rgb, &brush))),
            Material::Wood => crate::shapes::chamfered_box(size, 0.04, Some((at, rgb, &brush))),
            Material::Water => crate::shapes::boxy(size, at, rgb, &brush),
        };
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
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
            Shape::Box => {
                crate::shapes::chamfered_box(Vec3::new(w, h, d), 0.04, Some((at, rgb, &brush)))
            }
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
            Transform::from_translation(at)
                .with_rotation(Quat::from_rotation_y(-prop.yaw * std::f32::consts::TAU)),
            Scenery,
        ));
    }

    // **The floor broken up**, by dressing rather than geometry: the floor's
    // height is the simulation's and cannot move, so what lies on it does the
    // work -- tussocks in grass, pebbles on earth and sand, drifts on snow,
    // chips on rock -- small enough to walk through unnoticed and placed by a
    // hash of the arena, so the same arena scatters the same way every time.
    // Not on a course (no floor), not in water, never inside a solid.
    if !dressing.drop {
        scatter(&mut commands, &mut meshes, arena, &palette, &brush, &white);
    }
}

/// How much floor one scattered thing stands for, in square metres.
const SCATTER_SPACING: f32 = 40.0;

fn scatter(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    arena: &sim::arena::Arena,
    palette: &look::palette::Palette,
    brush: &crate::shapes::Brush,
    white: &Handle<StandardMaterial>,
) {
    let b = arena.bounds;
    let (lo_x, hi_x) = (fx(b.lo_x), fx(b.hi_x));
    let (lo_z, hi_z) = (fx(b.lo_z), fx(b.hi_z));
    let (w, d) = (hi_x - lo_x, hi_z - lo_z);
    let count = ((w * d / SCATTER_SPACING) as usize).clamp(12, 200);
    let seed = arena.id.0 as u32 + 1;
    let shade = |rgb: [f32; 3], k: f32| [rgb[0] * k, rgb[1] * k, rgb[2] * k];
    for i in 0..count {
        let i = i as u32;
        let r = |salt: u32| crate::shapes::hash01(seed, i, salt);
        let x = lo_x + 1.0 + (w - 2.0) * r(1);
        let z = lo_z + 1.0 + (d - 2.0) * r(2);
        // RENDER-ONLY: metres to fixed point, to ask the arena what is here.
        let to_fx = |v: f32| sim::Fx::from_raw((v * 65536.0) as i32);
        let at = sim::V3::new(to_fx(x), sim::Fx::ZERO, to_fx(z));
        // Not inside anything, and not where a region says something else
        // stands on the floor.
        if arena.solids().iter().any(|s| {
            x > fx(s.min.x) - 0.4
                && x < fx(s.max.x) + 0.4
                && z > fx(s.min.z) - 0.4
                && z < fx(s.max.z) + 0.4
        }) {
            continue;
        }
        let y = fx(arena.ground_under(at));
        let material = arena.material_under(at);
        let base = palette.of(material);
        let (mesh, size) = match material {
            Material::Grass => {
                let s = 0.25 + 0.3 * r(3);
                let h = 0.07 + 0.08 * r(4);
                let rgb = shade(base, 0.78 + 0.1 * r(5));
                (
                    crate::shapes::soft_box(
                        Vec3::new(s, h * 2.0, s * (0.8 + 0.4 * r(6))),
                        0.9,
                        8,
                        Some((Vec3::new(x, y, z), rgb, brush)),
                    ),
                    h,
                )
            }
            Material::Ground | Material::Peat | Material::Ash | Material::Sand => {
                let s = 0.1 + 0.16 * r(3);
                let h = 0.05 + 0.06 * r(4);
                let rock = palette.of(Material::Rock);
                let mix = 0.55;
                let rgb = [
                    base[0] * (1.0 - mix) + rock[0] * mix,
                    base[1] * (1.0 - mix) + rock[1] * mix,
                    base[2] * (1.0 - mix) + rock[2] * mix,
                ];
                (
                    crate::shapes::rock(
                        Vec3::new(s, h * 2.0, s * (0.7 + 0.6 * r(6))),
                        seed ^ i,
                        Some((Vec3::new(x, y, z), shade(rgb, 0.9 + 0.2 * r(5)), brush)),
                    ),
                    h,
                )
            }
            Material::Snow => {
                let s = 0.8 + 1.0 * r(3);
                let h = 0.05 + 0.07 * r(4);
                (
                    crate::shapes::soft_box(
                        Vec3::new(s, h * 2.0, s * (0.5 + 0.5 * r(6))),
                        0.95,
                        8,
                        Some((Vec3::new(x, y, z), shade(base, 1.04), brush)),
                    ),
                    h,
                )
            }
            Material::Rock | Material::Stone => {
                // A paved floor is swept: half as many, and small.
                if i % 2 == 1 {
                    continue;
                }
                let s = 0.08 + 0.1 * r(3);
                let h = 0.03 + 0.04 * r(4);
                (
                    crate::shapes::rock(
                        Vec3::new(s, h * 2.0, s * (0.6 + 0.6 * r(6))),
                        seed ^ i,
                        Some((Vec3::new(x, y, z), shade(base, 0.85 + 0.2 * r(5)), brush)),
                    ),
                    h,
                )
            }
            Material::Water | Material::Wood => continue,
        };
        // Half buried: the form's middle sits on the floor, so only its top
        // half shows and nothing has a visible underside to float on.
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(white.clone()),
            Transform::from_xyz(x, y, z)
                .with_rotation(Quat::from_rotation_y(r(7) * std::f32::consts::TAU)),
            bevy::pbr::NotShadowCaster,
            Scenery,
        ));
        let _ = size;
    }
}

/// RENDER-ONLY. Fixed point to metres.
fn fx(v: sim::Fx) -> f32 {
    v.to_f32_for_render()
}
