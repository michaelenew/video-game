//! **The valley, drawn** (`docs/design/valley.md`, `sim::valley`): what the
//! simulation's seams, waystones, vines and updrafts look like, and the
//! lookout's view from Hearth's wall.
//!
//! Nothing here decides a colour -- `look::palette` does (`beacon`, `vine`,
//! `draft`, `far_ridge`) -- and nothing here decides where anything is: every
//! shape is read off the place's table, so a seam moved in `sim` moves here.
//!
//! - **A seam** is a column of light standing in its zone and a glow on its
//!   floor: the arena's own pastel when the way is open, a cold grey while a
//!   dark waystone holds it, brightening as somebody holds it.
//! - **A waystone** carries a light on its top, lit or dark with its gate.
//! - **A vine** is a few dark-green strands down the face it climbs.
//! - **An updraft** is a faint column of rising air from the floor to its top.
//! - **The lookout**: from Hearth, the reaches on the sky to the east, one
//!   ridge behind another, rising as the valley does.

use bevy::prelude::*;
use sim::arena::{Arena, Terrain};
use sim::valley::{self, Kind};

use crate::arenas::Scenery;

/// A seam's light: which seam, and its two colours.
#[derive(Component)]
pub struct Beacon {
    seam: usize,
    lit: [f32; 3],
    dark: [f32; 3],
    /// What it is showing: lit, and how far through a hold, in tenths.
    shown: Option<(bool, u8)>,
    /// How strong this piece of it is: the column is fainter than the floor.
    strength: f32,
}

fn fx(v: sim::Fx) -> f32 {
    v.to_f32_for_render()
}

fn translucent(rgb: [f32; 3], alpha: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::linear_rgba(rgb[0], rgb[1], rgb[2], alpha),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    }
}

fn linear(rgb: [f32; 3]) -> [f32; 3] {
    look::tint::linear(rgb)
}

/// Draw a place's seams, waystones, vines and updrafts, and the lookout's
/// view if it is the town. Called by `arenas::dress` with the arena it has
/// just drawn; everything spawned is scenery, cleared with the arena.
pub fn draw(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    arena: &'static Arena,
    palette: &look::Palette,
    sky: &look::Sky,
) {
    let Some(place) = valley::place(arena.id) else {
        return;
    };
    let ground = Terrain::bare(arena);
    let (lit, dark) = (linear(palette.beacon(true)), linear(palette.beacon(false)));
    for (i, seam) in place.seams.iter().enumerate() {
        let z = seam.zone;
        let mid = z.middle();
        let floor = fx(ground.floor_below(sim::V3::new(mid.x, z.max.y, mid.z)));
        let (w, d) = (fx(z.max.x.sub(z.min.x)), fx(z.max.z.sub(z.min.z)));
        let (cx, cz) = (fx(mid.x), fx(mid.z));
        // The glow on its floor.
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(w, d))),
            MeshMaterial3d(materials.add(translucent(lit, 0.3))),
            Transform::from_xyz(cx, floor + 0.04, cz),
            bevy::pbr::NotShadowCaster,
            Beacon {
                seam: i,
                lit,
                dark,
                shown: None,
                strength: 0.32,
            },
            Scenery,
        ));
        // The column of light standing in it -- but not in a room, whose way
        // out is where the hunters arrive: a column there is a wall of light
        // between the camera and the creature for the first second of every
        // hunt. The glow on the floor is enough to find a door you came in by.
        let r = (w.min(d) * 0.32).max(0.6);
        let room = matches!(place.kind, Kind::Room(_));
        if !room {
            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(r, 7.0).mesh().resolution(20).build())),
                MeshMaterial3d(materials.add(translucent(lit, 0.12))),
                Transform::from_xyz(cx, floor + 3.5, cz),
                bevy::pbr::NotShadowCaster,
                Beacon {
                    seam: i,
                    lit,
                    dark,
                    shown: None,
                    strength: 0.12,
                },
                Scenery,
            ));
        }
        // The waystone's light, on its top.
        if let Some(k) = seam.waystone {
            let s = arena.solids()[k as usize];
            let top = Vec3::new(
                (fx(s.min.x) + fx(s.max.x)) * 0.5,
                fx(s.max.y) + 0.35,
                (fx(s.min.z) + fx(s.max.z)) * 0.5,
            );
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.38).mesh().ico(3).unwrap())),
                MeshMaterial3d(materials.add(translucent(lit, 0.95))),
                Transform::from_translation(top),
                bevy::pbr::NotShadowCaster,
                Beacon {
                    seam: i,
                    lit,
                    dark,
                    shown: None,
                    strength: 0.95,
                },
                Scenery,
            ));
        }
    }

    // Vines: a few strands down the face each one climbs.
    let vine = materials.add(StandardMaterial {
        base_color: {
            let c = linear(palette.vine());
            Color::linear_rgb(c[0], c[1], c[2])
        },
        perceptual_roughness: 0.95,
        ..default()
    });
    for v in place.vines {
        let (lo, hi) = (
            Vec3::new(fx(v.min.x), fx(v.min.y), fx(v.min.z)),
            Vec3::new(fx(v.max.x), fx(v.max.y), fx(v.max.z)),
        );
        // The face is on whichever thin side of the zone has rock against
        // it; the strands hang a hand in front of it.
        let along_z = (hi.x - lo.x) < (hi.z - lo.z);
        let probe = |p: Vec3| {
            arena.solids().iter().any(|s| {
                p.x > fx(s.min.x)
                    && p.x < fx(s.max.x)
                    && p.y > fx(s.min.y)
                    && p.y < fx(s.max.y)
                    && p.z > fx(s.min.z)
                    && p.z < fx(s.max.z)
            })
        };
        let mid_y = (lo.y + hi.y) * 0.5;
        let height = (hi.y - lo.y - 1.0).max(1.0);
        let base = lo.y + 1.0;
        let strands = 5;
        for k in 0..strands {
            let t = (k as f32 + 0.5) / strands as f32;
            let (x, z) = if along_z {
                let face = if probe(Vec3::new(hi.x + 0.2, mid_y, (lo.z + hi.z) * 0.5)) {
                    hi.x - 0.06
                } else {
                    lo.x + 0.06
                };
                (face, lo.z + (hi.z - lo.z) * t)
            } else {
                let face = if probe(Vec3::new((lo.x + hi.x) * 0.5, mid_y, hi.z + 0.2)) {
                    hi.z - 0.06
                } else {
                    lo.z + 0.06
                };
                (lo.x + (hi.x - lo.x) * t, face)
            };
            let sway = 0.85 + 0.3 * ((k * 7 % 5) as f32 / 5.0);
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.09, height * sway, 0.09))),
                MeshMaterial3d(vine.clone()),
                Transform::from_xyz(x, base + height * sway * 0.5, z),
                Scenery,
            ));
        }
    }

    // Updrafts: a faint column of rising air, from the floor to its top.
    let draft = linear(palette.draft());
    for v in place.vents {
        let (x, z) = (fx(v.at.0), fx(v.at.1));
        let floor = fx(ground.floor_below(sim::V3::new(v.at.0, v.top, v.at.1)));
        let top = fx(v.top);
        let h = (top - floor).max(1.0);
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(fx(v.radius), h).mesh().resolution(24).build())),
            MeshMaterial3d(materials.add(translucent(draft, 0.10))),
            Transform::from_xyz(x, floor + h * 0.5, z),
            bevy::pbr::NotShadowCaster,
            Scenery,
        ));
        commands.spawn((
            Mesh3d(meshes.add(Annulus::new(fx(v.radius) * 0.8, fx(v.radius)))),
            MeshMaterial3d(materials.add(translucent(draft, 0.35))),
            Transform::from_xyz(x, floor + 0.05, z)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            bevy::pbr::NotShadowCaster,
            Scenery,
        ));
    }

    if place.kind == Kind::Town {
        lookout(commands, meshes, materials, arena, palette, sky);
    }
}

/// **The lookout's view**: the reaches on the sky east of the town, one ridge
/// behind another and each higher than the last, as the valley climbs --
/// what you see from the wall's walk. Painted, not built: nothing out there
/// is in the arena, so it is far past the wall and nothing reaches it.
fn lookout(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    arena: &Arena,
    palette: &look::Palette,
    sky: &look::Sky,
) {
    // From the town's east wall, out along the road. The rises are the
    // reaches' own: twenty, fifty, ninety, a hundred and twenty, a hundred
    // and sixty metres -- squashed, since a ridge that far off is a line.
    const RISES: [f32; 5] = [20.0, 50.0, 90.0, 120.0, 160.0];
    let east = fx(arena.bounds.hi_x);
    for (k, rise) in RISES.iter().enumerate() {
        let t = k as f32 / (RISES.len() - 1) as f32;
        let x = east + 140.0 + 120.0 * k as f32;
        let rgb = look::tint::linear(look::palette::far_ridge(
            sky.horizon,
            palette.of(sim::arena::Material::Rock),
            t,
        ));
        let mesh = ridge(rise * 0.6 + 10.0, 900.0, k as u32);
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::linear_rgb(rgb[0], rgb[1], rgb[2]),
                unlit: true,
                cull_mode: None,
                ..default()
            })),
            Transform::from_xyz(x, -2.0, 0.0),
            bevy::pbr::NotShadowCaster,
            Scenery,
        ));
    }
}

/// A ridge's silhouette: a wall facing west, `width` across, its top line
/// about `height` with a few peaks and saddles by a hash of `seed`.
fn ridge(height: f32, width: f32, seed: u32) -> Mesh {
    use bevy::render::mesh::{Indices, PrimitiveTopology};
    use bevy::render::render_asset::RenderAssetUsages;
    let n = 64;
    let mut positions = Vec::with_capacity((n + 1) * 2);
    for i in 0..=n {
        let z = -width * 0.5 + width * i as f32 / n as f32;
        let a = crate::shapes::hash01(seed, i as u32, 3);
        let b = crate::shapes::hash01(seed, i as u32 / 4, 5);
        let top = height * (0.7 + 0.25 * b + 0.12 * a);
        positions.push([0.0, 0.0, z]);
        positions.push([0.0, top, z]);
    }
    let mut indices = Vec::with_capacity(n * 6);
    for i in 0..n as u32 {
        let (a, b, c, d) = (2 * i, 2 * i + 1, 2 * i + 2, 2 * i + 3);
        indices.extend_from_slice(&[a, c, b, b, c, d]);
    }
    let normals = vec![[-1.0f32, 0.0, 0.0]; positions.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

/// **Each frame**: a seam's light shows its gate -- lit or dark -- and
/// brightens while somebody holds it, toward going.
pub fn update(
    sim: Res<crate::Sim>,
    mut beacons: Query<(
        &mut Beacon,
        &MeshMaterial3d<StandardMaterial>,
        &mut Visibility,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let w = &sim.cur;
    let Some(place) = valley::place(w.arena) else {
        return;
    };
    let hold = sim::tuning::seam_hold().max(1) as u32;
    for (mut b, material, mut shown) in &mut beacons {
        // A room is a creature's own arena too, and a hunt picked from the
        // menu is not on a journey: its exit leads nowhere, so nothing marks
        // it. `set_if_neq`, so an unchanged beacon is not re-sent.
        shown.set_if_neq(if w.valley.on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        let Some(seam) = place.seams.get(b.seam) else {
            continue;
        };
        let lit = seam.gate.lit(w.valley.beaten);
        let held = if w.valley.held as usize == b.seam + 1 {
            (w.valley.hold as u32 * 10 / hold).min(10) as u8
        } else {
            0
        };
        if b.shown == Some((lit, held)) {
            continue;
        }
        b.shown = Some((lit, held));
        let rgb = if lit { b.lit } else { b.dark };
        let alpha = (b.strength * (1.0 + 1.5 * held as f32 / 10.0)).min(1.0);
        if let Some(m) = materials.get_mut(&material.0) {
            m.base_color = Color::linear_rgba(rgb[0], rgb[1], rgb[2], alpha);
        }
    }
}

/// **The next creature this player has beaten that the journey has not got**,
/// to credit on the wire -- or every creature there is, with `open`. `None`
/// when the journey is up to date.
pub fn credit_due(
    w: &sim::World,
    trophies: &crate::trophies::Trophies,
    open: bool,
) -> Option<sim::species::SpeciesId> {
    sim::species::shown()
        .map(|s| s.id)
        .find(|&id| (open || trophies.beaten(id, 0)) && !w.valley.has_beaten(id))
}

/// **What the valley says on screen**: where you are, and -- when somebody is
/// standing in a seam -- where it leads and what it is waiting for. Empty
/// outside the valley. Pure, so it is tested without a window.
pub fn text(w: &sim::World) -> String {
    if !w.valley.on {
        return String::new();
    }
    let Some(place) = valley::place(w.arena) else {
        return String::new();
    };
    let name = w.arena().name;
    let mut out = match place.kind {
        Kind::Town => format!("{name} -- the valley's town. Nobody fights here."),
        Kind::Ring => format!("The {name} -- fight each other here."),
        Kind::Reach => format!("The {name}"),
        Kind::Room(s) if w.hunting() => format!("The {name} -- {}", s.get().name),
        Kind::Room(_) => format!("The {name} -- quiet now. Walk out the way you came."),
    };
    let beaten = sim::species::shown()
        .filter(|s| w.valley.has_beaten(s.id))
        .count();
    out.push_str(&format!("   ({beaten} beaten)"));
    let seats = (w.seats as usize).clamp(1, sim::state::MAX_PLAYERS);
    let standing = (0..seats).find_map(|i| {
        let p = &w.players[i];
        place.seams.iter().find(|s| s.zone.holds(p.pos))
    });
    if let Some(seam) = standing {
        out.push('\n');
        match seam.gate {
            valley::Gate::Waystone { need, of, tier } if !seam.gate.lit(w.valley.beaten) => {
                let names: Vec<&str> = of.iter().map(|s| s.get().name).collect();
                out.push_str(&format!(
                    "{} -- the waystone of {tier} is dark: beat {need} of {} ({} so far)",
                    seam.says,
                    names.join(", "),
                    seam.gate.count(w.valley.beaten)
                ));
            }
            _ => {
                let hold = sim::tuning::seam_hold() as f32 / sim::TICK_HZ as f32;
                let held = w.valley.hold as f32 / sim::TICK_HZ as f32;
                if seats > 1 {
                    out.push_str(&format!(
                        "To {} -- both of you in, or hold it ({:.1} of {:.1} s)",
                        seam.says, held, hold
                    ));
                } else {
                    out.push_str(&format!("To {} -- {:.1} of {:.1} s", seam.says, held, hold));
                }
            }
        }
    }
    out
}

/// The valley's line on screen.
#[derive(Component)]
pub struct ValleyText;

pub fn setup_text(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::srgba(0.95, 0.95, 0.92, 0.92)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(96.0),
            left: Val::Percent(15.0),
            right: Val::Percent(15.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        ValleyText,
    ));
}

/// Rewritten only when what it says changes.
pub fn update_text(
    sim: Res<crate::Sim>,
    mut text: Query<&mut Text, With<ValleyText>>,
    mut shown: Local<String>,
) {
    let now = self::text(&sim.cur);
    if *shown == now {
        return;
    }
    if let Ok(mut t) = text.single_mut() {
        *t = Text::new(now.clone());
    }
    *shown = now;
}
