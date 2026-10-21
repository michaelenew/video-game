//! **The still world's shadow, drawn once.**
//!
//! The sun's shadow is a depth map seen from the sun: how far from it the
//! first surface along each of its rays is, so a pixel further than that is
//! in shadow. Bevy draws four of them every frame, round the camera, out to
//! 150 m, and draws everything that casts into each -- and in the valley
//! almost nothing that casts ever moves. Under the software renderer the
//! Great Wood's giants alone were 416 ms of a 666 ms frame in those four maps
//! (`docs/design/architecture.md`, "The still world's shadow").
//!
//! So in the valley, what is still -- everything the stream puts under the
//! map's root, drawn in the [`Paint`] -- casts no shadow into Bevy's maps.
//! It is drawn instead into **one map, square to the sun, [`SIZE`] metres
//! across round the camera**, which is drawn again only when it has to be:
//! the camera has walked [`RECENTRE`] metres from its middle, the sun has
//! turned, or something still has come or gone near it. Bevy's maps keep
//! the fighters and creatures, which is a few dozen boxes. The paint
//! ([`paint.wgsl`]) and the fighters' skin ([`SkinMaterial`]) read both.
//!
//! **A redraw never costs a frame.** There are two maps. The one being read
//! stays as it is while the other is drawn a tile a frame -- [`TILES`] by
//! [`TILES`] tiles, each a viewport of a camera that sees only the still
//! world -- and when its last tile is drawn the two change places. A redraw
//! is a sixteenth of the still world drawn on each of sixteen frames, about
//! a quarter of a second, during which the old map is the one on screen.
//!
//! The camera writes what it sees as a colour: the paint, drawn by an
//! orthographic view (nothing else is), puts out its depth along the sun in
//! two halves (`hi`, `lo`), because the camera's picture is half-floats and
//! one half-float holds a depth to a few centimetres at best.
//!
//! **An arena fought on alone does the same** (2026-10-21): its still world
//! is everything drawn as its scenery ([`crate::arenas::Scenery`], and what
//! hangs under it), in the world's own coordinates. It used to keep every
//! still thing in Bevy's cascades, and on a King of the Hill map the trees
//! and boulders on the hills behind the rim were seventy milliseconds of a
//! software-rendered frame in the two far cascades alone. In a browser
//! nothing is baked and Bevy's cascades hold everything as they always have
//! ([`crate::platform::bakes_shadows`]).

use bevy::asset::weak_handle;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::pbr::{ExtendedMaterial, MaterialExtension, NotShadowCaster};
use bevy::prelude::*;
use bevy::render::camera::{ClearColorConfig, ScalingMode, Viewport};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::view::RenderLayers;

use crate::paint::Paint;

/// How far across the map is, in metres: a square round the camera, so
/// [`SIZE`]` / 2` ahead of it in every direction. Bevy's cascades reached
/// 150 m; this reaches further at the side and about as far at the corner.
pub const SIZE: f32 = 400.0;
/// How many texels across: [`SIZE`] over this is a texel, 20 cm.
const TEXELS: u32 = 2048;
/// How many tiles across a redraw is cut into: one is drawn a frame.
const TILES: u32 = 4;
/// How far behind the middle of the map, toward the sun, the camera stands,
/// and how far it sees: everything that can shade the map is between.
const BACK: f32 = 1000.0;
const FAR: f32 = 2000.0;
/// How far the camera walks from the middle before the map is drawn again
/// round where it is now. With [`SIZE`] 400, the map still reaches 160 m
/// ahead at the moment it is redrawn.
const RECENTRE: f32 = 40.0;
/// How much the sun turns (radians) before the map is drawn again: a little
/// under a degree, which moves a shadow at the foot of a twenty-metre tree by
/// a quarter of a metre.
const TURN: f32 = 0.015;
/// The render layer the still world is drawn on as well as the picture's,
/// and the only one the map's camera sees.
pub const LAYER: usize = 7;

/// The two maps. Fixed handles so a paint made anywhere can name them.
pub const MAP_A: Handle<Image> = weak_handle!("3b7d2e14-8c5f-4a91-b6e0-7f2a9c4d1e58");
pub const MAP_B: Handle<Image> = weak_handle!("9e4a6c21-3f8b-4d70-a5c2-1b8e7d3f6a94");

/// Map `which`, 0 or 1.
fn map(which: usize) -> Handle<Image> {
    if which == 0 { MAP_A } else { MAP_B }
}

const SHADE_SHADER: Handle<Shader> = weak_handle!("c5a83e17-2d6f-4b9a-8e14-6f3c9b2d7a05");
const SKIN_SHADER: Handle<Shader> = weak_handle!("e2f7b4a9-6c1d-4e38-9a52-8d4f1c7e3b60");

pub use shade::Shade;

/// The uniform on its own, because `ShaderType`'s derive writes a layout
/// check per field beside it that nothing calls.
mod shade {
    #![allow(dead_code)]
    use bevy::prelude::*;
    use bevy::render::render_resource::ShaderType;

    /// **Where the map is**, as the shaders read it: the point the map's camera
    /// stands at (`origin.xyz`, in the picture's coordinates) and the map's size
    /// (`origin.w`); the camera's right and up, which are the map's axes, and how
    /// far it sees (`right.w`); whether there is a map to read (`up.w`, 0 or 1);
    /// the way the sunlight goes, and which of [`MAPS`] is the one to read
    /// (`along.w`, 0 or 1: [`MAP_A`] or [`MAP_B`]).
    #[derive(ShaderType, Reflect, Clone, Copy, Debug, Default, PartialEq)]
    pub struct Shade {
        pub origin: Vec4,
        pub right: Vec4,
        pub up: Vec4,
        pub along: Vec4,
    }

    impl Shade {
        /// No map: everything reads as lit by it.
        pub const NONE: Shade = Shade {
            origin: Vec4::ZERO,
            right: Vec4::ZERO,
            up: Vec4::ZERO,
            along: Vec4::ZERO,
        };
    }
}

/// **The fighters' skin**: the standard material, darkened by the still
/// world's shadow the way the paint is. A fighter under a giant is in its
/// shade; without this they would be the one lit thing in the wood.
pub type SkinMaterial = ExtendedMaterial<StandardMaterial, Shaded>;

/// What the skin adds to the standard material: where the map is, and the
/// two maps.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct Shaded {
    #[uniform(100)]
    pub shade: Shade,
    #[texture(101)]
    pub map_a: Handle<Image>,
    #[texture(102)]
    pub map_b: Handle<Image>,
}

impl Default for Shaded {
    fn default() -> Self {
        Shaded {
            shade: Shade::NONE,
            map_a: MAP_A,
            map_b: MAP_B,
        }
    }
}

impl MaterialExtension for Shaded {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SKIN_SHADER)
    }
}

/// A skin of this colour and roughness.
pub fn skin(base: StandardMaterial) -> SkinMaterial {
    SkinMaterial {
        base,
        extension: Shaded::default(),
    }
}

/// Marks a piece of the still world that has been taken out of Bevy's
/// shadow maps and put on [`LAYER`].
#[derive(Component)]
pub struct Still;

/// The camera that draws the map.
#[derive(Component)]
struct MapCamera;

/// A map: where it was drawn round, in the map's own coordinates (which do
/// not move when the world's frame does), and which way the sun went.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Drawn {
    middle: Vec3,
    sun: Vec3,
    which: usize,
}

/// **What the map is doing**: the map being read, if one has been drawn;
/// the one being drawn, and the next tile of it; and whether something still
/// has come or gone near enough to want another.
#[derive(Resource, Default)]
pub struct Maps {
    read: Option<Drawn>,
    drawing: Option<(Drawn, u32)>,
    stale: bool,
    /// What the paints were last told, and a count that moves whenever it
    /// changes, so a paint made since can be caught up.
    told: Shade,
    stamp: u32,
    /// Which map the camera was last pointed at.
    target: Option<usize>,
    /// How many maps have been finished, for `--profile`: a count that
    /// climbs while nothing moves is something asking for a redraw every
    /// frame.
    pub finished: u32,
}

pub struct StillPlugin;

impl Plugin for StillPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
        shaders.insert(
            SHADE_SHADER.id(),
            Shader::from_wgsl(include_str!("still.wgsl"), "game/src/still.wgsl"),
        );
        shaders.insert(
            SKIN_SHADER.id(),
            Shader::from_wgsl(include_str!("skin.wgsl"), "game/src/skin.wgsl"),
        );
        // `--no-still`: the still world in Bevy's cascades as it was, so
        // the two can be measured and looked at side by side.
        let drawn = crate::platform::bakes_shadows() && !crate::platform::flag("--no-still");
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        for which in 0..2 {
            images.insert(map(which).id(), map_image(drawn));
        }
        app.add_plugins(MaterialPlugin::<SkinMaterial>::default())
            .init_resource::<Maps>();
        if !drawn {
            return;
        }
        // The camera is put where this frame's tile is before the transforms
        // are propagated, so it is drawn from there this frame and not from
        // where the last tile was. What is new is looked at after, where it
        // has been put; a map it wants is started the frame after.
        app.add_systems(Startup, spawn_camera)
            .add_systems(
                PostUpdate,
                (draw, tell)
                    .chain()
                    .before(TransformSystem::TransformPropagate),
            )
            .add_systems(
                PostUpdate,
                gather.after(TransformSystem::TransformPropagate),
            );
    }
}

/// A map's texture: two half-floats a texel, drawn into by a camera and read
/// by the shaders; nothing kept on the CPU. Where no map is drawn, a texel,
/// so the paint's binding exists and costs nothing.
fn map_image(drawn: bool) -> Image {
    let texels = if drawn { TEXELS } else { 1 };
    let mut image = Image::new_uninit(
        Extent3d {
            width: texels,
            height: texels,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        TextureFormat::Rg16Float,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage = if drawn {
        TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST
    } else {
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST
    };
    image
}

/// The camera, made once and switched on only while a map is drawn.
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            // Before the picture's camera, so a map finished this frame is
            // finished before anything reads it.
            order: -1,
            is_active: false,
            hdr: true,
            target: MAP_A.into(),
            // Nothing in the way: as far as the camera sees, `hi` 1 `lo` 0.
            clear_color: ClearColorConfig::Custom(Color::linear_rgba(1.0, 0.0, 0.0, 1.0)),
            // The tile drawn this frame goes into its square of the map and
            // the rest of the map is left as it is.
            output_mode: bevy::render::camera::CameraOutputMode::Write {
                blend_state: None,
                clear_color: ClearColorConfig::None,
            },
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: SIZE / TILES as f32,
                height: SIZE / TILES as f32,
            },
            near: 0.0,
            far: FAR,
            ..OrthographicProjection::default_3d()
        }),
        // A depth is not a colour: nothing may be done to it on the way out.
        Tonemapping::None,
        DebandDither::Disabled,
        Msaa::Off,
        RenderLayers::layer(LAYER),
        MapCamera,
        Name::new("the still world's shadow"),
    ));
}

/// **Where the still world is drawn from**: in the valley, the stream's map
/// origin; in an arena fought on alone, the world's own origin -- its
/// scenery is drawn where it is. `None` in the valley before the stream has
/// put anything down.
fn origin_of(sim: &crate::Sim, stream: &crate::stream::Stream) -> Option<Vec3> {
    if sim.cur.valley.on {
        stream.origin()
    } else {
        Some(Vec3::ZERO)
    }
}

/// **Take the still world out of Bevy's shadow maps**: every piece of paint
/// that casts, as it is made -- under the map's root in the valley, or part
/// of an arena's scenery anywhere else. One that arrives inside the map, or
/// one already in it that is shown or hidden (a door), wants the map drawn
/// again.
#[allow(clippy::type_complexity, clippy::too_many_arguments)] // A Bevy query: one filter per thing it watches.
fn gather(
    mut commands: Commands,
    sim: Res<crate::Sim>,
    stream: Res<crate::stream::Stream>,
    scenery: Query<(), With<crate::arenas::Scenery>>,
    mut maps: ResMut<Maps>,
    new: Query<
        (Entity, &GlobalTransform),
        (
            With<Mesh3d>,
            With<MeshMaterial3d<Paint>>,
            Without<NotShadowCaster>,
            Without<Still>,
        ),
    >,
    shown: Query<&GlobalTransform, (With<Still>, Changed<Visibility>)>,
    parents: Query<&ChildOf>,
) {
    let Some(origin) = origin_of(&sim, &stream) else {
        return;
    };
    let valley = sim.cur.valley.on;
    let root = stream.root();
    let still = |e: Entity| {
        if valley {
            root.is_some_and(|r| parents.iter_ancestors(e).any(|a| a == r))
        } else {
            scenery.contains(e) || parents.iter_ancestors(e).any(|a| scenery.contains(a))
        }
    };
    if valley && root.is_none() {
        return;
    }
    let near = |at: Vec3, maps: &Maps| {
        let middles = maps.read.iter().chain(maps.drawing.iter().map(|d| &d.0));
        let mut any = false;
        for m in middles {
            let d = (at + origin) - m.middle;
            any |= d.x.abs().max(d.z.abs()) < SIZE;
        }
        // No map yet: the first one is coming anyway.
        any || maps.read.is_none()
    };
    for (e, at) in &new {
        if !still(e) {
            continue;
        }
        commands.entity(e).insert((
            NotShadowCaster,
            RenderLayers::from_layers(&[0, LAYER]),
            Still,
        ));
        if near(at.translation(), &maps) {
            maps.stale = true;
        }
    }
    for at in &shown {
        if near(at.translation(), &maps) {
            maps.stale = true;
        }
    }
}

/// The sun's rays and the map's two axes: the camera's own, so the shaders
/// and the camera cannot disagree about which way is right.
fn axes(sun: Vec3) -> (Quat, Vec3, Vec3) {
    // Any up that is not the sun's own direction; the sun is never straight
    // overhead in any place, and +z is as good as any other.
    let hint = if sun.cross(Vec3::Z).length_squared() > 1e-4 {
        Vec3::Z
    } else {
        Vec3::X
    };
    let turn = Transform::IDENTITY.looking_to(sun, hint).rotation;
    (turn, turn * Vec3::X, turn * Vec3::Y)
}

/// **Draw the next tile, or start a map, or finish one.** In the map's own
/// coordinates throughout, so the world's frame moving under the fighters
/// does not count as the camera walking.
fn draw(
    sim: Res<crate::Sim>,
    stream: Res<crate::stream::Stream>,
    mut maps: ResMut<Maps>,
    // Neither has a parent, so its transform is where it is.
    suns: Query<&Transform, (With<crate::arenas::Sun>, Without<MapCamera>)>,
    eyes: Query<&Transform, (With<crate::MainCamera>, Without<MapCamera>)>,
    mut camera: Query<(&mut Camera, &mut Transform), With<MapCamera>>,
) {
    let Ok((mut cam, mut place)) = camera.single_mut() else {
        return;
    };
    let origin = origin_of(&sim, &stream);
    let (Some(origin), Ok(sun), Ok(eye)) = (origin, suns.single(), eyes.single()) else {
        // Nothing put down yet: nothing to draw, and nothing to read.
        *maps = Maps {
            told: maps.told,
            stamp: maps.stamp,
            target: maps.target,
            finished: maps.finished,
            ..default()
        };
        cam.is_active = false;
        return;
    };
    let sun = sun.forward().as_vec3();
    let here = eye.translation + origin;
    // The map just finished goes on.
    if let Some((drawn, tile)) = maps.drawing
        && tile == TILES * TILES
    {
        maps.read = Some(drawn);
        maps.drawing = None;
        maps.finished += 1;
    }
    if maps.drawing.is_none() {
        let wanted = match maps.read {
            None => true,
            Some(r) => {
                let d = here - r.middle;
                maps.stale || d.x.abs().max(d.z.abs()) > RECENTRE || r.sun.angle_between(sun) > TURN
            }
        };
        if !wanted {
            cam.is_active = false;
            return;
        }
        maps.stale = false;
        let which = maps.read.map_or(0, |r| 1 - r.which);
        maps.drawing = Some((
            Drawn {
                // On a metre grid, so the map does not swim by fractions of
                // a texel from one drawing to the next.
                middle: here.round(),
                sun,
                which,
            },
            0,
        ));
    }
    let Some((drawn, tile)) = maps.drawing else {
        return;
    };
    let (turn, right, up) = axes(drawn.sun);
    let (i, j) = (tile % TILES, tile / TILES);
    let step = SIZE / TILES as f32;
    let across = -SIZE / 2.0 + (i as f32 + 0.5) * step;
    let down = SIZE / 2.0 - (j as f32 + 0.5) * step;
    let at = drawn.middle - origin - drawn.sun * BACK + right * across + up * down;
    *place = Transform {
        translation: at,
        rotation: turn,
        scale: Vec3::ONE,
    };
    let px = TEXELS / TILES;
    cam.viewport = Some(Viewport {
        physical_position: UVec2::new(i * px, j * px),
        physical_size: UVec2::splat(px),
        ..default()
    });
    if maps.target != Some(drawn.which) {
        cam.target = map(drawn.which).into();
        maps.target = Some(drawn.which);
    }
    cam.is_active = true;
    maps.drawing = Some((drawn, tile + 1));
}

/// **Tell the paints and the skins where the map is**, when that changes --
/// a new map, or the world's frame moving under it -- and catch up any made
/// since.
fn tell(
    sim: Res<crate::Sim>,
    stream: Res<crate::stream::Stream>,
    mut maps: ResMut<Maps>,
    mut paints: ResMut<Assets<Paint>>,
    mut skins: ResMut<Assets<SkinMaterial>>,
) {
    let shade = match (maps.read, origin_of(&sim, &stream)) {
        (Some(r), Some(origin)) => {
            let (_, right, up) = axes(r.sun);
            let at = r.middle - origin - r.sun * BACK;
            Shade {
                origin: at.extend(SIZE),
                right: right.extend(FAR),
                up: up.extend(1.0),
                along: r.sun.extend(r.which as f32),
            }
        }
        _ => Shade::NONE,
    };
    if shade != maps.told {
        maps.told = shade;
        maps.stamp = maps.stamp.wrapping_add(1);
    }
    let stamp = maps.stamp;
    // Looked at before taken mutably: `get_mut` marks a material changed,
    // and a changed material is rebuilt on the GPU.
    let late: Vec<AssetId<Paint>> = paints
        .iter()
        .filter(|(_, p)| p.stamp != stamp)
        .map(|(id, _)| id)
        .collect();
    for id in late {
        if let Some(p) = paints.get_mut(id) {
            p.shade = shade;
            p.stamp = stamp;
        }
    }
    let late: Vec<AssetId<SkinMaterial>> = skins
        .iter()
        .filter(|(_, s)| s.extension.shade != shade)
        .map(|(id, _)| id)
        .collect();
    for id in late {
        if let Some(s) = skins.get_mut(id) {
            s.extension.shade = shade;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shader turns a depth back into metres with its own copy of how
    /// far the camera sees; the two must be the same number.
    #[test]
    fn the_shader_knows_how_far_the_camera_sees() {
        let wgsl = include_str!("still.wgsl");
        assert!(
            wgsl.contains(&format!("const FAR: f32 = {FAR:.1};")),
            "still.wgsl's FAR is not still::FAR ({FAR})"
        );
    }

    /// The map's axes are the camera's: square to each other and to the
    /// sun, whichever way it shines.
    #[test]
    fn the_axes_are_square_to_the_sun() {
        for sun in [
            Vec3::new(-6.0, -14.0, -5.0),
            Vec3::new(0.3, -1.0, 0.0),
            Vec3::new(0.0, -1.0, -0.001),
        ] {
            let sun = sun.normalize();
            let (turn, right, up) = axes(sun);
            assert!((turn * Vec3::NEG_Z - sun).length() < 1e-4, "{sun}");
            assert!(right.dot(up).abs() < 1e-4, "{sun}");
            assert!(right.dot(sun).abs() < 1e-4, "{sun}");
            assert!(up.dot(sun).abs() < 1e-4, "{sun}");
        }
    }

    /// The tiles are the map: as many texels across as it has, none
    /// overlapping, every one drawn once a redraw.
    #[test]
    fn the_tiles_cover_the_map() {
        assert_eq!(TEXELS % TILES, 0);
        let px = TEXELS / TILES;
        let mut seen = vec![false; (TILES * TILES) as usize];
        for tile in 0..TILES * TILES {
            let (i, j) = (tile % TILES, tile / TILES);
            assert!((i + 1) * px <= TEXELS && (j + 1) * px <= TEXELS);
            seen[(j * TILES + i) as usize] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }
}

#[cfg(test)]
mod shader {
    /// `still.wgsl` stands alone (it imports nothing), so it can be checked
    /// without a GPU: a shader that fails to compile is otherwise silent in
    /// this game -- what is drawn in it is simply not drawn.
    #[test]
    fn the_shadow_module_compiles() {
        let source: String = include_str!("still.wgsl")
            .lines()
            .filter(|l| !l.starts_with("#define_import_path"))
            .collect::<Vec<_>>()
            .join("\n");
        let module = naga::front::wgsl::parse_str(&source)
            .unwrap_or_else(|e| panic!("{}", e.emit_to_string(&source)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap_or_else(|e| panic!("{}", e.emit_to_string(&source)));
    }
}
