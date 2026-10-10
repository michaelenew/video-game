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

use crate::paint::{Materials, Paint};
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
        ArenaId::CLIMB_WATERFALL => &climb::WATERFALL,
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
/// In the valley, only the sky and the light: the places and their boxes are
/// streamed (`crate::stream`) and come and go with distance instead.
#[derive(Component)]
pub struct Scenery;

/// Which arena is drawn, if any yet, and whether it was drawn as a place on
/// the valley's map.
#[derive(Resource, Default)]
pub struct Drawn(Option<(ArenaId, bool)>);

/// How far the floor runs past the bounds, so the edge of the world is not the
/// edge of the walls.
const APRON: f32 = 12.0;

/// How wide a drop's floor is drawn: past the horizon from any island.
const DEEP: f32 = 2000.0;

/// **Where a drawn piece goes**: into the arena's scenery, cleared with the
/// arena, or under a streamed place's entity, cleared when it is unloaded.
#[derive(Clone, Copy, Debug)]
pub enum Under {
    Scenery,
    Parent(Entity),
}

/// Spawn one drawn piece where it goes.
pub fn put(commands: &mut Commands, under: Under, piece: impl Bundle) -> Entity {
    match under {
        Under::Scenery => commands.spawn((piece, Scenery)).id(),
        Under::Parent(p) => commands.spawn((piece, ChildOf(p))).id(),
    }
}

/// **How a place looks**: its sky, the palette that follows from it, and the
/// brush that puts the accent on its surfaces, stretched over the heights the
/// place actually contains.
#[derive(Clone)]
pub struct PlaceLook {
    /// Whose look it is.
    pub id: ArenaId,
    pub sky: look::Sky,
    pub palette: look::palette::Palette,
    pub brush: crate::shapes::Brush,
    pub dressing: &'static Dressing,
}

impl PlaceLook {
    pub fn of(arena: &sim::arena::Arena) -> PlaceLook {
        // One table, keyed by arena: `look::skies`. Resolved once here rather
        // than per frame, and shared by the dome, the fog and the drop below,
        // which is what keeps all three agreeing about where the horizon is.
        let sky = look::skies::of(arena.id).resolved();
        // Derived from that sky, so an arena that has one has both.
        let palette = look::palette::Palette::under(&sky);
        // The crest half of the rule is stretched over the heights this arena
        // actually contains, read off its own collision data. A fixed ramp
        // from the ground reads beautifully in a proving ground whose walls
        // are waist-high and paints every surface of a jump course at full
        // strength, because a course's *lowest* island is already forty
        // metres up.
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
        PlaceLook {
            id: arena.id,
            sky,
            palette,
            brush,
            dressing: dressing(arena.id),
        }
    }
}

/// Draw the arena the simulation is in, when it is not the one already drawn:
/// its sky and light, and -- alone, outside the valley -- all of it. In the
/// valley the sky and light follow the place you are in, and everything else
/// is `crate::stream`'s.
#[allow(clippy::too_many_arguments)] // A Bevy system: one argument per resource it reads.
pub fn dress(
    mut commands: Commands,
    sim: Res<crate::Sim>,
    mut drawn: ResMut<Drawn>,
    old: Query<Entity, With<Scenery>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard: ResMut<Assets<StandardMaterial>>,
    mut paints: ResMut<Assets<Paint>>,
    mut clear: ResMut<ClearColor>,
    mut suns: Query<&mut Transform, With<Sun>>,
    mut fill: Query<&mut DirectionalLight, With<Skylight>>,
    mut ambient: ResMut<AmbientLight>,
    mut camera: Query<Entity, With<crate::MainCamera>>,
    looks: Option<Res<crate::ground::Looks>>,
) {
    let arena = sim.cur.arena();
    let mapped = sim.cur.valley.on;
    if drawn.0 == Some((arena.id, mapped)) {
        return;
    }
    drawn.0 = Some((arena.id, mapped));
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let look = PlaceLook::of(arena);
    let sky = &look.sky;
    let sun_at = sun(arena.id);

    // The clear colour still matters: it is what shows in the sliver of a frame
    // before the dome is drawn, and anywhere the dome does not reach. Set to
    // the horizon so that sliver is never a different colour from the sky.
    let h = sky.horizon;
    clear.0 = Color::srgb(h[0], h[1], h[2]);
    sky::raise(
        &mut commands,
        &mut meshes,
        &mut standard,
        sky,
        sun_at,
        Scenery,
    );
    // Fog on the camera rather than on the scene, because it is a property of
    // looking rather than of the things looked at.
    if let Ok(eye) = camera.single_mut() {
        commands.entity(eye).insert(sky::fog(sky));
        // The line fades over the same distance the air does -- where there is
        // a line at all.
        if crate::platform::draws_outlines() {
            commands
                .entity(eye)
                .insert(crate::outline::Outline::over(look::edge::LINE, sky));
        }
    }
    // The pooled hazard and raised-solid materials take this arena's colours.
    if let Some(looks) = &looks {
        crate::ground::repaint(looks, &mut standard, arena.id);
    }
    // Both the ambient term and the fill light take the sky's colour, so every
    // shadow in the arena is the complement of what cast it.
    let shade = look.palette.shade;
    ambient.color = Color::srgb(shade[0], shade[1], shade[2]);
    for mut light in &mut fill {
        light.color = Color::srgb(shade[0], shade[1], shade[2]);
    }
    for mut light in &mut suns {
        *light = Transform::from_translation(sun_at).looking_at(Vec3::ZERO, Vec3::Y);
    }
    if mapped {
        return;
    }

    // Everything the arena is made of is painted with this: the palette says
    // what colour a thing is, the edge rule says where its accent goes, and
    // `shapes` puts the answer in the vertices. One white material serves all
    // of it, because every mesh carries its own colour.
    let mut materials = Materials {
        standard: &mut standard,
        paint: &mut paints,
    };
    let white = materials.paint.add(Paint::white());
    draw_place(
        &mut commands,
        &mut meshes,
        &mut materials,
        arena,
        &look,
        &white,
        Under::Scenery,
    );
    // The geometry, straight from the simulation's own collision data. One
    // source of truth: if you can see it, you collide with it.
    for solid in arena.solids() {
        draw_solid(
            &mut commands,
            &mut meshes,
            solid,
            Vec3::ZERO,
            solid.hangs(),
            &look,
            &white,
            Under::Scenery,
            true,
        );
    }
}

/// **A place, less its boxes**: its floor and the regions on it, its props,
/// what is scattered on its floor, and -- in the valley -- its seams,
/// waystones, vines and updrafts. In the place's own coordinates, under
/// `under`.
pub fn draw_place(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Materials,
    arena: &'static sim::arena::Arena,
    look: &PlaceLook,
    white: &Handle<Paint>,
    under: Under,
) {
    let (sky, palette, brush, dressing) = (&look.sky, &look.palette, &look.brush, look.dressing);
    // On the map the land is drawn by itself, a tile at a time
    // (`crate::land`), and the floor of every place with it.
    let mapped = matches!(under, Under::Parent(_));
    // The drop's floor, unlit: made before `paint` borrows the materials.
    //
    // Unlit but **fogged**, which is the whole point of it now. A drop drawn
    // crisp from edge to edge is a dark floor somebody put under the level; the
    // same floor hazing toward the sky as it recedes is a long way down.
    let below = dressing.drop.then(|| {
        let rgb = sky.ground;
        materials.standard.add(StandardMaterial {
            base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
            unlit: true,
            fog_enabled: true,
            ..default()
        })
    });

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
    //
    // On the valley's map the floor is drawn over the place's whole footprint
    // and the ground round a room (`sim::atlas`), with no apron of its own
    // past that: the next place's floor starts where this one's ends.
    let (lo_x, hi_x, lo_z, hi_z, apron) = match under {
        Under::Parent(_) => {
            let (lo, hi) = sim::atlas::footprint(arena);
            let pad = (sim::valley::layout::APRON as f32 / 100.0)
                * matches!(
                    sim::valley::place(arena.id).map(|p| p.kind),
                    Some(sim::valley::Kind::Room(_) | sim::valley::Kind::Ring)
                ) as i32 as f32;
            (
                fx(lo.0) - pad,
                fx(hi.0) + pad,
                fx(lo.1) - pad,
                fx(hi.1) + pad,
                0.0,
            )
        }
        Under::Scenery => {
            let b = arena.bounds;
            (fx(b.lo_x), fx(b.hi_x), fx(b.lo_z), fx(b.hi_z), APRON)
        }
    };
    match below.filter(|_| !mapped) {
        _ if mapped => {}
        // A fight with a rim: its floor and the rim round it, as land.
        _ if arena.rim.is_some() => {
            crate::land::draw_arena(commands, meshes, materials, arena, palette, white);
        }
        Some(dark) => {
            // Under the far land, a floor past its edge to the horizon.
            put(
                commands,
                under,
                (
                    Mesh3d(meshes.add(Plane3d::default().mesh().size(DEEP, DEEP))),
                    MeshMaterial3d(dark),
                    Transform::from_xyz((lo_x + hi_x) * 0.5, -3.0, (lo_z + hi_z) * 0.5),
                    bevy::pbr::NotShadowReceiver,
                    // Nothing is under it to throw a shadow on, and it is two
                    // kilometres across: drawn into every shadow cascade it was
                    // most of each one's triangles. See the note on the floor.
                    bevy::pbr::NotShadowCaster,
                ),
            );
            crate::land::draw_below(commands, meshes, materials, arena, palette, white);
        }
        None => {
            let (w, d) = (hi_x - lo_x + apron * 2.0, hi_z - lo_z + apron * 2.0);
            // Subdivided, because a four-vertex plane has nowhere to put a
            // gradient. The accent runs in from the arena's own perimeter,
            // which is the edge of the world as far as anyone standing on it
            // is concerned.
            // Flat, a subdivided plane; with relief (`sim::arena::relief`),
            // a grid at the simulation's own heights, half a metre a cell,
            // so what is drawn is the floor feet are held to.
            let hilly = !sim::arena::relief::is_flat(arena.id);
            let mut floor = if hilly {
                let to_fx = |v: f32| sim::Fx::from_raw((v * 65536.0) as i32);
                crate::shapes::ground_grid(
                    Vec2::new(w, d),
                    Vec3::new((lo_x + hi_x) * 0.5, 0.0, (lo_z + hi_z) * 0.5),
                    0.5,
                    &|x, z| fx(arena.relief_at(to_fx(x), to_fx(z))),
                )
            } else {
                Plane3d::default()
                    .mesh()
                    .size(w, d)
                    .subdivisions(crate::shapes::cuts_across(w.max(d), brush.edge.reach))
                    .build()
            };
            crate::shapes::paint(
                &mut floor,
                Vec3::new(w * 0.5, 0.0, d * 0.5),
                Vec3::ZERO,
                palette.of(arena.floor),
                brush,
            );
            // Ground, not paint: the colour wanders a little across it.
            crate::shapes::mottle(&mut floor, 0.07, 2.5, arena.id.0 as u32);
            // A crown lighter and a hollow darker, by the look's rule.
            crate::shapes::shade_by_height(&mut floor, &look::palette::relief_shade);
            put(
                commands,
                under,
                (
                    Mesh3d(meshes.add(floor)),
                    MeshMaterial3d(white.clone()),
                    Transform::from_xyz((lo_x + hi_x) * 0.5, 0.0, (lo_z + hi_z) * 0.5),
                    bevy::pbr::NotShadowCaster,
                ),
            );
        }
    }
    let mut paint = |rgb: [f32; 3]| {
        materials.standard.add(StandardMaterial {
            base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
            perceptual_roughness: 0.92,
            ..default()
        })
    };
    let hilly = !sim::arena::relief::is_flat(arena.id);
    let regions = if mapped { &[][..] } else { arena.regions };
    for (i, region) in regions.iter().enumerate() {
        let look = paint(palette.of(region.material));
        // A hair above the floor, each region a hair above the last.
        let lift = 0.004 * (i + 1) as f32;
        // On a floor with relief the patch follows it, vertex by vertex, at
        // the simulation's own heights; on a flat one it is a flat shape.
        let ground = |x: f32, z: f32| {
            let to_fx = |v: f32| sim::Fx::from_raw((v * 65536.0) as i32);
            fx(arena.relief_at(to_fx(x), to_fx(z)))
        };
        match region.area {
            Area::Rect { lo, hi } => {
                let (x0, z0, x1, z1) = (fx(lo.0), fx(lo.1), fx(hi.0), fx(hi.1));
                let centre = Vec3::new((x0 + x1) * 0.5, lift, (z0 + z1) * 0.5);
                let mesh = if hilly {
                    crate::shapes::ground_grid(
                        Vec2::new(x1 - x0, z1 - z0),
                        Vec3::new(centre.x, 0.0, centre.z),
                        0.5,
                        &ground,
                    )
                } else {
                    Plane3d::default().mesh().size(x1 - x0, z1 - z0).build()
                };
                put(
                    commands,
                    under,
                    (
                        Mesh3d(meshes.add(mesh)),
                        MeshMaterial3d(look),
                        Transform::from_translation(centre),
                        bevy::pbr::NotShadowCaster,
                    ),
                );
            }
            Area::Disc { at, radius } => {
                let centre = Vec3::new(fx(at.0), lift, fx(at.1));
                if hilly {
                    put(
                        commands,
                        under,
                        (
                            Mesh3d(meshes.add(crate::shapes::ground_disc(
                                fx(radius),
                                Vec3::new(centre.x, 0.0, centre.z),
                                0.5,
                                &ground,
                            ))),
                            MeshMaterial3d(look),
                            Transform::from_translation(centre),
                            bevy::pbr::NotShadowCaster,
                        ),
                    );
                } else {
                    put(
                        commands,
                        under,
                        (
                            Mesh3d(meshes.add(Circle::new(fx(radius)))),
                            MeshMaterial3d(look),
                            Transform::from_translation(centre)
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                            bevy::pbr::NotShadowCaster,
                        ),
                    );
                }
            }
        }
    }

    for prop in dressing.props {
        // Past a rim, the rim's own woods and rocks stand instead
        // (`crate::land::draw_arena`, and the valley's land on the map).
        if let Some(rim) = arena.rim.filter(|r| !r.drop) {
            let to_fx = |v: f32| sim::Fx::from_raw((v * 65536.0) as i32);
            if rim.outside(to_fx(prop.at[0]), to_fx(prop.at[2])) {
                continue;
            }
        }
        let [w, h, d] = prop.size;
        // Under a course, the land far below has woods of its own
        // (`crate::land::draw_below`): the old spires with a ball of trees on
        // top that stood on the floor give way to it.
        if dressing.drop && prop.at[1] <= 0.0 && h > 8.0 {
            continue;
        }
        let at = Vec3::new(prop.at[0], prop.at[1] + h * 0.5, prop.at[2]);
        let rgb = palette.surface(prop.rgb);
        let half = Vec3::new(w * 0.5, h * 0.5, d * 0.5);
        let mesh = match prop.shape {
            // A far peak hanging in the air over a course: a lump of rock,
            // not a ball.
            Shape::Sphere if dressing.drop && w > 6.0 => crate::shapes::rock(
                Vec3::new(w, h, d),
                (prop.at[0] * 3.0 + prop.at[2] * 7.0) as i32 as u32,
                Some((at, rgb, brush)),
            ),
            Shape::Box => {
                crate::shapes::chamfered_box(Vec3::new(w, h, d), 0.04, Some((at, rgb, brush)))
            }
            Shape::Cylinder => {
                let mut m = Cylinder::new(w * 0.5, h).mesh().build();
                crate::shapes::paint(&mut m, half, at, rgb, brush);
                m
            }
            Shape::Sphere => {
                let mut m = Sphere::new(w * 0.5).mesh().build();
                crate::shapes::paint(&mut m, Vec3::splat(w * 0.5), at, rgb, brush);
                m
            }
        };
        put(
            commands,
            under,
            (
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(white.clone()),
                Transform::from_translation(at)
                    .with_rotation(Quat::from_rotation_y(-prop.yaw * std::f32::consts::TAU)),
            ),
        );
    }

    // **The floor broken up**, by dressing rather than geometry: the floor's
    // height is the simulation's and cannot move, so what lies on it does the
    // work -- tussocks in grass, pebbles on earth and sand, drifts on snow,
    // chips on rock -- small enough to walk through unnoticed and placed by a
    // hash of the arena, so the same arena scatters the same way every time.
    // Not on a course (no floor), not in water, never inside a solid.
    if !dressing.drop && !mapped {
        scatter(commands, meshes, arena, palette, brush, white, under);
    }

    // The town's buildings and everything about its streets.
    if arena.id == ArenaId::HEARTH {
        crate::town::draw(commands, meshes, materials, palette, white, under);
    }

    // The valley's seams, waystones, vines and updrafts: `crate::valley`, for
    // a place that is one.
    crate::valley::draw(commands, meshes, materials, arena, palette, under);
    // The Waterfall's water, where the place has one.
    crate::falls::draw(commands, meshes, materials, arena, palette, under);
}

/// **One box**, from the simulation's own collision data: one source of
/// truth, so if you can see it, you collide with it. `offset` is where its
/// coordinates' origin is drawn -- zero for an arena's own box, the place's
/// origin taken off a map box's -- and the colour is worked out where the box
/// stands in its own place, so a box looks the same however it is loaded.
#[allow(clippy::too_many_arguments)] // What is drawn, where, and in what.
pub fn draw_solid(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    solid: &sim::arena::Solid,
    offset: Vec3,
    hangs: bool,
    look: &PlaceLook,
    white: &Handle<Paint>,
    under: Under,
    near: bool,
) -> Entity {
    let Some((mesh, relief, at)) = solid_mesh(solid, offset, hangs, look, near) else {
        return put(
            commands,
            under,
            (Transform::default(), Visibility::default()),
        );
    };
    let e = put(
        commands,
        under,
        (
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(white.clone()),
            Transform::from_translation(at + offset),
        ),
    );
    if let Some(relief) = relief {
        commands.spawn((
            Mesh3d(meshes.add(relief)),
            MeshMaterial3d(white.clone()),
            Transform::default(),
            bevy::pbr::NotShadowCaster,
            ChildOf(e),
        ));
    }
    e
}

/// **What a box is drawn as**: its mesh, in its own coordinates, and where
/// its middle is relative to `offset` -- or nothing, for a box the town
/// draws whole; and its relief that casts no shadow, as
/// [`crate::forms::build_parts`] has it, as does `near`.
pub fn solid_mesh(
    solid: &sim::arena::Solid,
    offset: Vec3,
    hangs: bool,
    look: &PlaceLook,
    near: bool,
) -> Option<(Mesh, Option<Mesh>, Vec3)> {
    let (palette, brush) = (&look.palette, &look.brush);
    let min = Vec3::new(fx(solid.min.x), fx(solid.min.y), fx(solid.min.z)) - offset;
    let max = Vec3::new(fx(solid.max.x), fx(solid.max.y), fx(solid.max.z)) - offset;
    let size = max - min;
    let at = (min + max) * 0.5;
    // Part of something the town draws whole: drawn there.
    if crate::town::whole(look.id, min, max) {
        return None;
    }
    // **A form**, where the box is one (`crate::forms`): a wall in courses,
    // a log, a column, an island. Built in the box's own coordinates.
    let form = crate::forms::form_of(look.id, solid, hangs);
    if let Some((mesh, relief)) = crate::forms::build_parts(
        form,
        size,
        crate::forms::seed_of(at, size),
        solid.material,
        palette,
        near,
    ) {
        return Some((mesh, relief, at));
    }
    // The form follows the material (`docs/design/forms.md`): bare rock
    // is a rock, soft ground is rounded, dressed stone and timber keep
    // their edges. Every form stays inside the box the body collides
    // against.
    let rgb = palette.of(solid.material);
    let seed = (at.x * 7.0 + at.z * 13.0 + size.y * 3.0) as i32 as u32;
    // **Terrain** -- a terrace, a bank, a valley's side, a hedge -- is a
    // cliff, roughened in metres rather than in shares of its size, so a
    // thirty-metre block's edge is where its collision is. Dressed stone
    // and timber stay square at any size. A soft top on a tall block (a
    // turf-topped terrace) shows rock in its faces; a thin one (a hedge)
    // is its own stuff all the way down.
    let soft = matches!(
        solid.material,
        Material::Ground
            | Material::Grass
            | Material::Sand
            | Material::Snow
            | Material::Ash
            | Material::Peat
            | Material::Rock
    );
    // **A crag**: a pillar of rock standing on the land, drawn as one --
    // tapering, lumpy, flat on top where feet stand -- rather than as a
    // block of cliff.
    let crag = solid.material == Material::Rock && size.y > 8.0 && size.x <= 12.0 && size.z <= 12.0;
    let mesh = if crag {
        let mut m = crate::shapes::rock_column(seed);
        if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(ps)) =
            m.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            for p in ps.iter_mut() {
                *p = [p[0] * size.x, p[1] * size.y, p[2] * size.z];
            }
        }
        if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(ns)) =
            m.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
        {
            for n in ns.iter_mut() {
                let v =
                    Vec3::new(n[0] / size.x, n[1] / size.y, n[2] / size.z).normalize_or(Vec3::Y);
                *n = v.to_array();
            }
        }
        crate::shapes::paint(&mut m, size * 0.5, at, rgb, brush);
        m
    } else if soft && crate::shapes::is_terrain(size) {
        let side = palette.cliff_face(solid.material, size.x.min(size.z) < 3.0);
        // A low outcrop of rock is mossed over on top, as old rock is.
        let top = if solid.material == Material::Rock && size.y < 4.0 {
            look::tint::mix(rgb, palette.moss(), 0.55)
        } else {
            rgb
        };
        crate::shapes::cliff(size, seed, top, side, at, brush)
    } else {
        match solid.material {
            Material::Rock => crate::shapes::rock(size, seed, Some((at, rgb, brush))),
            Material::Ground
            | Material::Grass
            | Material::Sand
            | Material::Snow
            | Material::Ash
            | Material::Peat => crate::shapes::soft_box(size, 0.3, 12, Some((at, rgb, brush))),
            // Dressed stone is cut square and keeps its arris, chamfered just
            // enough not to catch the light as a wire; timber a little more.
            Material::Stone => crate::shapes::chamfered_box(size, 0.06, Some((at, rgb, brush))),
            Material::Wood => crate::shapes::chamfered_box(size, 0.04, Some((at, rgb, brush))),
            Material::Water => crate::shapes::boxy(size, at, rgb, brush),
        }
    };
    Some((mesh, None, at))
}

/// How much floor one scattered thing stands for, in square metres.
const SCATTER_SPACING: f32 = 1.6;

/// **The floor broken up**: tufts of grass and a few flowers in a meadow,
/// pebbles on earth and sand, drifts on snow, chips on rock -- one mesh of
/// thousands of small things, none of which anything collides with, placed by
/// a hash of the arena so it scatters the same way every time.
fn scatter(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    arena: &'static sim::arena::Arena,
    palette: &look::palette::Palette,
    _brush: &crate::shapes::Brush,
    white: &Handle<Paint>,
    under: Under,
) {
    let b = arena.bounds;
    let (lo_x, hi_x) = (fx(b.lo_x), fx(b.hi_x));
    let (lo_z, hi_z) = (fx(b.lo_z), fx(b.hi_z));
    let (w, d) = (hi_x - lo_x, hi_z - lo_z);
    let count = ((w * d / SCATTER_SPACING) as usize).clamp(60, 6000);
    let seed = arena.id.0 as u32 + 1;
    let shade = |rgb: [f32; 3], k: f32| [rgb[0] * k, rgb[1] * k, rgb[2] * k];
    let mut kit = crate::forms::Kit::new();
    let grass = palette.of(Material::Grass);
    let rock = palette.of(Material::Rock);
    for i in 0..count {
        let i = i as u32;
        let r = |salt: u32| crate::shapes::hash01(seed, i, salt);
        let x = lo_x + 0.5 + (w - 1.0) * r(1);
        let z = lo_z + 0.5 + (d - 1.0) * r(2);
        // RENDER-ONLY: metres to fixed point, to ask the arena what is here.
        let to_fx = |v: f32| sim::Fx::from_raw((v * 65536.0) as i32);
        let at = sim::V3::new(to_fx(x), sim::Fx::ZERO, to_fx(z));
        // On the floor, or on the top of something broad enough to be ground
        // (a plateau, a bank): never against a side, nor on a wall's top.
        let near_side = arena.solids().iter().any(|s| {
            let over = x > fx(s.min.x) - 0.4
                && x < fx(s.max.x) + 0.4
                && z > fx(s.min.z) - 0.4
                && z < fx(s.max.z) + 0.4;
            let broad = fx(s.max.x) - fx(s.min.x) > 6.0 && fx(s.max.z) - fx(s.min.z) > 6.0;
            let inner = x > fx(s.min.x) + 0.6
                && x < fx(s.max.x) - 0.6
                && z > fx(s.min.z) + 0.6
                && z < fx(s.max.z) - 0.6;
            over && !(broad && inner)
        });
        if near_side {
            continue;
        }
        let at = sim::V3::new(at.x, sim::Fx::from_int(1000), at.z);
        let y = fx(arena.ground_under(at));
        let at = sim::V3::new(at.x, arena.ground_under(at), at.z);
        let material = arena.material_under(at);
        let base = palette.of(material);
        let foot = Vec3::new(x, y, z);
        match material {
            Material::Grass | Material::Peat | Material::Ground => {
                // Thinner on bare earth and marsh than in a meadow.
                let thin = match material {
                    Material::Grass => 1.0,
                    _ => 0.3,
                };
                if r(3) > thin {
                    if r(4) < 0.12 {
                        pebble(&mut kit, foot, r(5), r(6), shade(rock, 0.85 + 0.3 * r(7)));
                    }
                    continue;
                }
                // A tuft: a few broad, short blades, the ground's own green a
                // shade either way -- grass, not wire. (Tall thin blades took
                // the outline round every one and drew the meadow in ink.)
                let tone = match material {
                    Material::Grass => shade(grass, 0.92 + 0.22 * r(5)),
                    _ => look::tint::mix(base, grass, 0.5),
                };
                let tall = 0.1 + 0.14 * r(6);
                for k in 0..4u32 {
                    let a = std::f32::consts::TAU * (k as f32 / 4.0 + r(7));
                    let lean = Vec3::new(a.cos(), 0.0, a.sin()) * (0.06 + 0.06 * r(8 + k));
                    let off = Vec3::new(a.cos(), 0.0, a.sin()) * 0.04;
                    let h = tall * (0.7 + 0.5 * crate::shapes::hash01(seed, i, 20 + k));
                    kit.tube(
                        foot + off - Vec3::Y * 0.02,
                        foot + off + lean + Vec3::Y * h,
                        0.05,
                        0.008,
                        3,
                        shade(tone, 0.95 + 0.12 * crate::shapes::hash01(seed, i, 30 + k)),
                        None,
                    );
                }
                // Now and then a flower over the tuft.
                if material == Material::Grass && r(9) < 0.07 {
                    let top = foot + Vec3::new(0.02, tall + 0.08, 0.0);
                    kit.tube(foot, top, 0.012, 0.01, 3, shade(grass, 0.8), None);
                    let petal = palette.bloom((r(10) * 3.0) as u32);
                    kit.turned(top, Vec3::new(0.05, 0.025, 0.05), r(11) * 3.0, petal, petal);
                }
            }
            Material::Sand | Material::Ash => {
                if r(3) < 0.25 {
                    let rgb = look::tint::mix(base, rock, 0.55);
                    pebble(&mut kit, foot, r(5), r(6), shade(rgb, 0.85 + 0.3 * r(7)));
                }
            }
            Material::Snow => {
                if r(3) < 0.04 {
                    let s = 0.8 + 1.0 * r(4);
                    let h = 0.05 + 0.07 * r(5);
                    let drift = crate::shapes::soft_box(
                        Vec3::new(s, h * 2.0, s * (0.5 + 0.5 * r(6))),
                        0.95,
                        6,
                        None,
                    );
                    kit.mesh(&drift, Transform::from_translation(foot), shade(base, 1.04));
                }
            }
            Material::Rock | Material::Stone => {
                // A paved floor is swept: few, and small.
                if r(3) < 0.08 {
                    pebble(
                        &mut kit,
                        foot,
                        r(5) * 0.5,
                        r(6),
                        shade(base, 0.8 + 0.25 * r(7)),
                    );
                }
            }
            Material::Water | Material::Wood => {}
        }
    }
    if kit.is_empty() {
        return;
    }
    put(
        commands,
        under,
        (
            Mesh3d(meshes.add(kit.build())),
            MeshMaterial3d(white.clone()),
            Transform::default(),
            bevy::pbr::NotShadowCaster,
        ),
    );
}

/// A pebble: a small stone, half sunk.
fn pebble(kit: &mut crate::forms::Kit, foot: Vec3, size: f32, turn: f32, rgb: [f32; 3]) {
    let s = 0.08 + 0.18 * size;
    kit.nugget(
        foot + Vec3::Y * s * 0.12,
        Vec3::new(s, s * 0.6, s * 0.8),
        (turn * 1.0e6) as u32,
        rgb,
    );
}

/// RENDER-ONLY. Fixed point to metres.
fn fx(v: sim::Fx) -> f32 {
    v.to_f32_for_render()
}
