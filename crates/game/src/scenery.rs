//! The two things the arena can be standing in, and the switch between them.
//!
//! **Arena** is the fight: one rock, carved. **Terrain** is the same machinery
//! asked a harder question -- hills, ruins and scattered boulders, all cut from
//! the same stone volume the arena walls are.
//!
//! It is a *look* toggle and nothing else. The simulation's collision geometry
//! is a constant in `sim::arena` and neither scene touches it, so a fighter
//! standing on a hillside is really standing on the arena floor with the walls
//! hidden. That is the right trade for a harness whose job is to show what the
//! generators make -- and it is stated plainly here rather than left to be
//! discovered, because a world you can walk through is the kind of thing
//! someone reports as a bug.
//!
//! The terrain is built once at startup and hidden, rather than built on the
//! first click. It is about a second of baking, and a scene switch that stalls
//! is a scene switch nobody uses twice.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use art::stone::Stone;
use art::terrain::{Scatter, Terrain};

/// Which scene is showing.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Scene {
    #[default]
    Arena,
    Terrain,
}

impl Scene {
    /// `SCENE=terrain` starts in the landscape.
    ///
    /// For the headless capture, which has no pointer to click the button
    /// with -- the same reason `SHOT_FRAME` exists. Anything you can only
    /// reach by clicking is something a screenshot can never show, and a look
    /// you cannot screenshot is a look nobody reviews.
    pub fn from_env() -> Scene {
        match std::env::var("SCENE").as_deref() {
            Ok("terrain") => Scene::Terrain,
            _ => Scene::Arena,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Scene::Arena => "Arena",
            Scene::Terrain => "Terrain",
        }
    }

    pub fn next(self) -> Scene {
        match self {
            Scene::Arena => Scene::Terrain,
            Scene::Terrain => Scene::Arena,
        }
    }
}

/// Scenery that belongs to the terrain, hidden while the arena is showing.
#[derive(Component)]
pub struct TerrainScenery;

/// How far the hills reach. Well past the far ground's visible detail, so the
/// landscape does not end at a visible edge.
const REACH: f32 = 340.0;

/// How many metres one repeat of the detail texture covers.
///
/// **The hills are the hybrid case, and they are what makes the argument
/// concrete.** At four metres between vertices the stone volume's own structure
/// -- joints a metre or two apart -- is entirely below the sampling rate, so
/// asking the volume for colour at each vertex correctly returns the rock's
/// mean and the hills come out as smooth dunes. Every joint is averaged away.
///
/// The volume is still the right thing for the large scale: it says which
/// hillside is pale and which is stained, across hundreds of metres, uniquely.
/// What it cannot do at this vertex spacing is grain. So a small tiling texture
/// goes underneath and the vertex colours multiply it -- repetition invisible,
/// because a tile with no large features has no period to see.
const DETAIL_TILE: f32 = 4.0;

/// Vertices per side of the terrain mesh.
///
/// A hundred and sixty gives about four metres between vertices at this reach,
/// which is finer than the hills and coarser than the rocks -- exactly the
/// split that matters, since a hill has to be geometry and a boulder does not.
const GRID: usize = 160;

pub fn build(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) {
    let land = Terrain::HILLS;
    // Shifted so the origin is at ground level, because the fighters stand at
    // zero and the simulation has no idea the terrain is there.
    let datum = land.datum();
    // The hills are made of the same rock the arena is cut from. Not a
    // matching colour -- the same volume, sampled where the hillside actually
    // is, which is the claim the stone module was written to support.
    let bedrock = art::stone::granite();
    let marble = art::stone::marble();

    // Grain from a small tiling texture; the large-scale variation comes from
    // the vertex colours, which Bevy multiplies in. See `DETAIL_TILE`.
    let ground = surfaces::build_repeating(&art::materials::GROUND, images, materials);

    commands.spawn((
        Mesh3d(meshes.add(hills(&land, &bedrock, datum))),
        MeshMaterial3d(ground),
        Transform::default(),
        Visibility::Hidden,
        TerrainScenery,
    ));

    ruins(commands, meshes, materials, images, &land, &marble, datum);
    boulders(commands, meshes, materials, &land, &bedrock, datum);
}

/// The hills, as a grid with the rock's colour baked into its vertices.
fn hills(land: &Terrain, rock: &Stone, datum: f32) -> Mesh {
    // **The vertex colour is a tint, not an albedo.**
    //
    // Bevy multiplies the base colour texture by the vertex colour, so handing
    // it the rock's actual reflectance multiplies two albedos together -- a
    // 0.09 texture times a 0.10 tint is 0.009, and the hills came out about ten
    // times too dark. Dividing by what the detail texture averages to makes the
    // product land on the rock's colour, which is what was meant all along.
    let detail_mean = art::materials::GROUND.mean_albedo().max(1e-3);
    let mut positions = Vec::with_capacity(GRID * GRID);
    let mut normals = Vec::with_capacity(GRID * GRID);
    let mut colours = Vec::with_capacity(GRID * GRID);
    let mut uvs = Vec::with_capacity(GRID * GRID);
    let mut indices = Vec::with_capacity((GRID - 1) * (GRID - 1) * 6);

    // One vertex covers this much ground, which is what the stone is told so
    // its grain averages out instead of turning into noise between vertices.
    let step = REACH * 2.0 / (GRID - 1) as f32;

    for iz in 0..GRID {
        for ix in 0..GRID {
            let x = -REACH + ix as f32 * step;
            let z = -REACH + iz as f32 * step;
            let y = land.height(x, z) + datum;
            positions.push([x, y, z]);
            normals.push(land.normal(x, z));

            let mut s = rock.at_scale([x, y, z], step);
            // Steep ground is bare rock; gentle ground has had somewhere for
            // dust to settle. One line, and it is most of what stops a
            // hillside reading as a single painted surface.
            let bare = (land.slope(x, z) * 3.5).min(1.0);
            for c in s.colour.iter_mut() {
                *c *= 0.55 + 0.45 * bare;
            }
            colours.push([
                s.colour[0] / detail_mean,
                s.colour[1] / detail_mean,
                s.colour[2] / detail_mean,
                1.0,
            ]);
            // World-space texture coordinates, so the detail layer keeps the
            // same size however the grid is resampled.
            uvs.push([x / DETAIL_TILE, z / DETAIL_TILE]);
        }
    }

    for iz in 0..GRID - 1 {
        for ix in 0..GRID - 1 {
            let a = (iz * GRID + ix) as u32;
            let (b, c, d) = (a + 1, a + GRID as u32, a + GRID as u32 + 1);
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Ruined marble columns, on the high ground.
///
/// **A group per site, not a column per site.** A single shaft standing alone
/// on a hill is a monolith; what says *ruin* is a row of them at different
/// stages of falling down -- one nearly intact, one snapped halfway, one down
/// to a stump, and the drums that came off them lying where they rolled. The
/// intact one is what tells you what the stumps used to be, and without it the
/// stumps are just cylinders.
///
/// Sited on summits and level footings, because that is where somebody would
/// have built and where anything is still standing. A column on a steep slope
/// reads as having been dropped there.
fn ruins(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    land: &Terrain,
    marble: &Stone,
    datum: f32,
) {
    // Few, and far apart. A landscape with ruins every thirty metres is not a
    // landscape with ruins in it, it is a quarry.
    let scatter = Scatter {
        spacing: 78.0,
        jitter: 0.8,
        seed: 0x0C01,
    };
    let sites = scatter.over(land, 210.0, |s| s.y > 3.0 && s.slope < 0.10);

    // One mesh and one material for every drum. They differ in size and in how
    // far they have shifted, which a transform says; a ruin is not made more
    // convincing by each stump owning a vertex buffer.
    let drum = meshes.add(surfaces::tangented(
        Cylinder::new(0.5, 1.0).mesh().resolution(20).build(),
    ));
    let stone = marble_material(marble, images, materials);

    for site in sites {
        // Column proportions from a real temple: about a metre across and five
        // to nine tall. The first version was a third of that and read as a
        // row of bollards.
        let radius = 0.46 + site.roll[1] * 0.18;
        let drum_height = radius * 1.9;
        // Which way the colonnade ran. Ruins line up because buildings do.
        let along = Vec2::from_angle(site.roll[0] * std::f32::consts::TAU);

        let count = 3 + (site.roll[1] * 3.0) as usize;
        for i in 0..count {
            // How much of this one is left. Deliberately spread rather than
            // random: the first is nearly whole, the last is a footing, and the
            // ones between are the stages in order -- which reads as one
            // building coming apart rather than as unrelated stumps.
            let wear = i as f32 / (count - 1).max(1) as f32;
            let standing = (8.4 * (1.0 - wear).powf(1.7) + 0.5).max(0.5);

            let step = radius * 5.2;
            let x = site.x + along.x * (i as f32 - count as f32 * 0.5) * step;
            let z = site.z + along.y * (i as f32 - count as f32 * 0.5) * step;
            let base = land.height(x, z) + datum;

            let drums = (standing / drum_height).ceil().max(1.0) as usize;
            for k in 0..drums {
                let h = (standing - k as f32 * drum_height).min(drum_height);
                if h < 0.2 {
                    break;
                }
                // The lean grows up the shaft, because the drums that were
                // highest had furthest to shift before they settled.
                let lean = (k as f32 / drums as f32).powi(2) * (site.roll[0] - 0.5) * 0.22;
                commands.spawn((
                    Mesh3d(drum.clone()),
                    MeshMaterial3d(stone.clone()),
                    Transform {
                        translation: Vec3::new(
                            x + lean * 1.6,
                            base + k as f32 * drum_height + h * 0.5,
                            z + lean * 1.2,
                        ),
                        rotation: Quat::from_rotation_z(lean)
                            * Quat::from_rotation_y(k as f32 * 1.7 + site.roll[1] * 6.0),
                        scale: Vec3::new(radius * 2.0, h, radius * 2.0),
                    },
                    Visibility::Hidden,
                    TerrainScenery,
                ));
            }

            // What came down. Drums roll, so they end up beside the column
            // rather than under it, and they lie on their sides.
            let fallen = ((wear * 3.0) as usize).min(2);
            for f in 0..fallen {
                let angle = site.roll[1] * std::f32::consts::TAU + f as f32 * 2.1 + i as f32 * 0.7;
                let away = Vec2::from_angle(angle) * (radius * 3.0 + f as f32 * radius * 2.4);
                let fx = x + away.x;
                let fz = z + away.y;
                commands.spawn((
                    Mesh3d(drum.clone()),
                    MeshMaterial3d(stone.clone()),
                    Transform {
                        translation: Vec3::new(fx, land.height(fx, fz) + datum + radius, fz),
                        rotation: Quat::from_rotation_y(angle)
                            * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                        scale: Vec3::new(radius * 2.0, drum_height, radius * 2.0),
                    },
                    Visibility::Hidden,
                    TerrainScenery,
                ));
            }
        }
    }
}

/// A boulder: a sphere pushed around until it stops being one.
///
/// An icosphere at a low subdivision is not a rock, it is a ball, and scaling
/// it unevenly gives an egg. What makes a rock a rock is that its surface has
/// *facets and hollows at several scales* -- which is the same noise the stone
/// volume is built from, applied to the radius instead of to the colour.
///
/// Ridged noise rather than plain, for the reason the terrain uses it: folding
/// at zero turns bumps into creases, and a boulder's edges are creases where
/// it broke.
fn boulder_mesh(seed: u32) -> Mesh {
    let mut mesh = Sphere::new(0.5).mesh().ico(3).unwrap();
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|a| a.as_float3())
        .map(|p| p.to_vec())
        .unwrap_or_default();

    let moved: Vec<[f32; 3]> = positions
        .iter()
        .map(|p| {
            let dir = Vec3::from_array(*p).normalize_or_zero();
            // Two scales: big lobes that decide the overall lumpiness, and a
            // finer crease that breaks up the silhouette.
            let lobe = art::noise::gradient_noise((dir * 1.7).to_array(), seed);
            let crease =
                1.0 - art::noise::gradient_noise((dir * 4.3).to_array(), seed ^ 0x5A).abs() * 2.0;
            let r = 0.5 * (1.0 + lobe * 0.34 + crease * 0.13);
            (dir * r).to_array()
        })
        .collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, moved);
    // The normals are the sphere's until they are recomputed, which would light
    // the rock as though it were still round -- every facet this just carved
    // would be invisible.
    mesh.compute_normals();
    surfaces::tangented(mesh)
}

/// Boulders, anywhere they could have come to rest.
fn boulders(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    land: &Terrain,
    rock: &Stone,
    datum: f32,
) {
    let scatter = Scatter {
        spacing: 17.0,
        jitter: 0.95,
        seed: 0xB001,
    };
    // Not on the steepest faces: anything loose there left a long time ago.
    let spots = scatter.over(land, 210.0, |s| s.slope < 0.30 && s.roll[0] > 0.55);

    // Four shapes, so a field of boulders is not one boulder repeated.
    let shapes: Vec<Handle<Mesh>> = (0..4)
        .map(|i| meshes.add(boulder_mesh(0xB010 + i)))
        .collect();
    let stone = materials.add(boulder_material(rock));

    for spot in spots {
        // Mostly knee-high, occasionally something you would take cover behind.
        // The square is what makes big ones rare rather than merely larger --
        // a uniform size gives a field of identical lumps.
        let size = 0.45 + spot.roll[1] * spot.roll[1] * 2.1;
        let shape = shapes[(spot.roll[0] * 1000.0) as usize % shapes.len()].clone();
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(stone.clone()),
            Transform {
                // Sunk a little, because a boulder sitting exactly on the
                // surface reads as a ball that has been placed.
                translation: Vec3::new(spot.x, spot.y + datum + size * 0.28, spot.z),
                rotation: Quat::from_euler(
                    EulerRot::XYZ,
                    spot.roll[0] * 6.0,
                    spot.roll[1] * 6.0,
                    spot.roll[0] * spot.roll[1] * 6.0,
                ),
                scale: Vec3::new(size, size * (0.62 + spot.roll[0] * 0.45), size * 0.9),
            },
            Visibility::Hidden,
            TerrainScenery,
        ));
    }
}

/// The average look of a rock, as a plain material.
///
/// Boulders and column drums are small and there are hundreds of them, so each
/// getting its own cut of the volume would be hundreds of textures for
/// something a metre across. They take the rock's *mean* instead -- which is
/// what [`Stone::at_scale`] hands back for anything sampled coarsely anyway, so
/// this is the same answer arrived at directly.
fn boulder_material(rock: &Stone) -> StandardMaterial {
    let s = rock.at_scale([3.7, 1.1, -2.3], 0.4);
    StandardMaterial {
        base_color: crate::linear(s.colour),
        perceptual_roughness: 1.0 - s.hardness * 0.6,
        ..default()
    }
}

fn marble_material(
    rock: &Stone,
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    surfaces::from_stone_patch(rock, [4.0, 1.0, -2.0], 1.6, 2.4, images, materials, 0.42)
}

use crate::surfaces;

/// Show whichever scene is selected.
pub fn apply(
    scene: Res<Scene>,
    mut arena: Query<&mut Visibility, (With<crate::ArenaScenery>, Without<TerrainScenery>)>,
    mut terrain: Query<&mut Visibility, (With<TerrainScenery>, Without<crate::ArenaScenery>)>,
) {
    if !scene.is_changed() {
        return;
    }
    let showing_arena = *scene == Scene::Arena;
    for mut v in arena.iter_mut() {
        *v = if showing_arena {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for mut v in terrain.iter_mut() {
        *v = if showing_arena {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_toggle_comes_back_to_where_it_started() {
        // Two scenes, so pressing twice has to return. Obvious, and the sort of
        // thing that stops being obvious the moment a third is added.
        assert_eq!(Scene::Arena.next().next(), Scene::Arena);
        assert_eq!(Scene::Terrain.next().next(), Scene::Terrain);
        assert_ne!(Scene::Arena.next(), Scene::Arena);
    }

    #[test]
    fn the_default_is_the_fight() {
        // The harness is for the fight. The landscape is a showcase of what the
        // generators make, and a harness that opens on the showcase is one that
        // has forgotten what it is for.
        assert_eq!(Scene::default(), Scene::Arena);
    }

    #[test]
    fn every_scene_says_what_it_is() {
        for scene in [Scene::Arena, Scene::Terrain] {
            assert!(!scene.label().is_empty());
        }
    }
}
