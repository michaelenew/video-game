//! **The valley, drawn** (`docs/design/valley.md`, `sim::valley`): what the
//! simulation's seams, waystones and updrafts look like.
//!
//! Nothing here decides a colour -- `look::palette` does (`beacon`,
//! `draft`) -- and nothing here decides where anything is: every
//! shape is read off the place's table, so a seam moved in `sim` moves here.
//!
//! - **A seam** is a column of light standing in its zone and a glow on its
//!   floor: the arena's own pastel when the way is open, a cold grey while a
//!   dark waystone shuts it.
//! - **A waystone** carries a light on its top, lit or dark with its gate,
//!   and while it is dark its doorway is a wall of the same grey.
//! - **An updraft** is a faint column of rising air from the floor to its top.
//!
//! The lookout's painted ridges are gone: the valley is one map now, and what
//! you see from the wall's walk is the reaches themselves.

use bevy::prelude::*;
use sim::arena::{Arena, ArenaId, Terrain};
use sim::valley::{self, Kind};

use crate::arenas::{Under, put};
use crate::paint::Materials;

/// A seam's light: which place and which seam, and its two colours.
#[derive(Component)]
pub struct Beacon {
    place: ArenaId,
    seam: usize,
    lit: [f32; 3],
    dark: [f32; 3],
    /// What it is showing: lit or not.
    shown: Option<bool>,
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

/// Draw a place's seams, waystones and updrafts. Called by
/// `arenas::draw_place` with the place it is drawing; everything spawned goes
/// where the place's other pieces go.
pub fn draw(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Materials,
    arena: &'static Arena,
    palette: &look::Palette,
    under: Under,
) {
    let Some(place) = valley::place(arena.id) else {
        return;
    };
    // On the map, the floor under anything here is the land's, in this
    // place's coordinates.
    let ground = match under {
        Under::Parent(_) => Terrain::placed(sim::atlas::valley(), arena),
        Under::Scenery => Terrain::bare(arena),
    };
    let (lit, dark) = (linear(palette.beacon(true)), linear(palette.beacon(false)));
    let mapped = matches!(under, Under::Parent(_));
    for (i, seam) in place.seams.iter().enumerate() {
        let z = seam.zone;
        let mid = z.middle();
        let floor = fx(ground.floor_below(sim::V3::new(mid.x, z.max.y, mid.z)));
        let (w, d) = (fx(z.max.x.sub(z.min.x)), fx(z.max.z.sub(z.min.z)));
        let (cx, cz) = (fx(mid.x), fx(mid.z));
        // **On the map a seam is a signpost**, not a doorway of light: a
        // slim pillar and a pool of light at the foot of a side path to a
        // room, or at a waystone's pass. Where one reach runs on into the
        // next, or out of a room, there is nothing to mark -- you walk on.
        if mapped {
            let to_room = valley::place(seam.to)
                .is_some_and(|p| matches!(p.kind, Kind::Room(_) | Kind::Ring));
            let gated = !matches!(seam.gate, valley::Gate::Open);
            let in_room = matches!(place.kind, Kind::Room(_) | Kind::Ring);
            if in_room || !(to_room || gated) {
                continue;
            }
            let beacon = |strength: f32| Beacon {
                place: arena.id,
                seam: i,
                lit,
                dark,
                shown: None,
                strength,
            };
            put(
                commands,
                under,
                (
                    Mesh3d(meshes.add(Circle::new(1.6))),
                    MeshMaterial3d(materials.standard.add(translucent(lit, 0.3))),
                    Transform::from_xyz(cx, floor + 0.06, cz)
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                    bevy::pbr::NotShadowCaster,
                    beacon(0.32),
                ),
            );
            put(
                commands,
                under,
                (
                    Mesh3d(meshes.add(Cylinder::new(0.22, 3.2).mesh().resolution(12).build())),
                    MeshMaterial3d(materials.standard.add(translucent(lit, 0.8))),
                    Transform::from_xyz(cx, floor + 1.6, cz),
                    bevy::pbr::NotShadowCaster,
                    beacon(0.8),
                ),
            );
            if let Some(k) = seam.waystone {
                let s = arena.solids()[k as usize];
                let top = Vec3::new(
                    (fx(s.min.x) + fx(s.max.x)) * 0.5,
                    fx(s.max.y) + 0.35,
                    (fx(s.min.z) + fx(s.max.z)) * 0.5,
                );
                put(
                    commands,
                    under,
                    (
                        Mesh3d(meshes.add(Sphere::new(0.38).mesh().ico(3).unwrap())),
                        MeshMaterial3d(materials.standard.add(translucent(lit, 0.95))),
                        Transform::from_translation(top),
                        bevy::pbr::NotShadowCaster,
                        beacon(0.95),
                    ),
                );
            }
            continue;
        }
        // The glow on its floor.
        put(
            commands,
            under,
            (
                Mesh3d(meshes.add(Plane3d::default().mesh().size(w, d))),
                MeshMaterial3d(materials.standard.add(translucent(lit, 0.3))),
                Transform::from_xyz(cx, floor + 0.04, cz),
                bevy::pbr::NotShadowCaster,
                Beacon {
                    place: arena.id,
                    seam: i,
                    lit,
                    dark,
                    shown: None,
                    strength: 0.32,
                },
            ),
        );
        // The column of light standing in it -- but not in a room, whose way
        // out is where the hunters arrive: a column there is a wall of light
        // between the camera and the creature for the first second of every
        // hunt. The glow on the floor is enough to find a door you came in by.
        let r = (w.min(d) * 0.32).max(0.6);
        let room = matches!(place.kind, Kind::Room(_));
        if !room {
            put(
                commands,
                under,
                (
                    Mesh3d(meshes.add(Cylinder::new(r, 7.0).mesh().resolution(20).build())),
                    MeshMaterial3d(materials.standard.add(translucent(lit, 0.12))),
                    Transform::from_xyz(cx, floor + 3.5, cz),
                    bevy::pbr::NotShadowCaster,
                    Beacon {
                        place: arena.id,
                        seam: i,
                        lit,
                        dark,
                        shown: None,
                        strength: 0.12,
                    },
                ),
            );
        }
        // The waystone's light, on its top.
        if let Some(k) = seam.waystone {
            let s = arena.solids()[k as usize];
            let top = Vec3::new(
                (fx(s.min.x) + fx(s.max.x)) * 0.5,
                fx(s.max.y) + 0.35,
                (fx(s.min.z) + fx(s.max.z)) * 0.5,
            );
            put(
                commands,
                under,
                (
                    Mesh3d(meshes.add(Sphere::new(0.38).mesh().ico(3).unwrap())),
                    MeshMaterial3d(materials.standard.add(translucent(lit, 0.95))),
                    Transform::from_translation(top),
                    bevy::pbr::NotShadowCaster,
                    Beacon {
                        place: arena.id,
                        seam: i,
                        lit,
                        dark,
                        shown: None,
                        strength: 0.95,
                    },
                ),
            );
        }
    }

    // Updrafts: a faint column of rising air, from the floor to its top.
    for v in place.vents {
        let floor = ground.floor_below(sim::V3::new(v.at.0, v.top, v.at.1));
        draw_vent(commands, meshes, materials, v, floor, palette, under);
    }
}

/// **An updraft**: a faint column of rising air from `floor` (or its own
/// foot, if that is higher) to its top, and a ring where it starts.
pub fn draw_vent(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Materials,
    v: &sim::valley::Vent,
    floor: sim::Fx,
    palette: &look::Palette,
    under: Under,
) {
    let draft = linear(palette.draft());
    let (x, z) = (fx(v.at.0), fx(v.at.1));
    let floor = fx(floor.max(v.from));
    let top = fx(v.top);
    let h = (top - floor).max(1.0);
    put(
        commands,
        under,
        (
            Mesh3d(meshes.add(Cylinder::new(fx(v.radius), h).mesh().resolution(24).build())),
            MeshMaterial3d(materials.standard.add(translucent(draft, 0.10))),
            Transform::from_xyz(x, floor + h * 0.5, z),
            bevy::pbr::NotShadowCaster,
        ),
    );
    put(
        commands,
        under,
        (
            Mesh3d(meshes.add(Annulus::new(fx(v.radius) * 0.8, fx(v.radius)))),
            MeshMaterial3d(materials.standard.add(translucent(draft, 0.35))),
            Transform::from_xyz(x, floor + 0.05, z)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            bevy::pbr::NotShadowCaster,
        ),
    );
}

/// **Each frame**: a seam's light shows its gate -- lit or dark.
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
    for (mut b, material, mut shown) in &mut beacons {
        // A room is a creature's own arena too, and a hunt picked from the
        // menu is not on a journey: its exit leads nowhere, so nothing marks
        // it. `set_if_neq`, so an unchanged beacon is not re-sent.
        shown.set_if_neq(if w.valley.on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        let Some(seam) = valley::place(b.place).and_then(|p| p.seams.get(b.seam)) else {
            continue;
        };
        let lit = seam.gate.lit(w.valley.beaten);
        if b.shown == Some(lit) {
            continue;
        }
        b.shown = Some(lit);
        let rgb = if lit { b.lit } else { b.dark };
        if let Some(m) = materials.get_mut(&material.0) {
            m.base_color = Color::linear_rgba(rgb[0], rgb[1], rgb[2], b.strength);
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
    // Where the first fighter stands: during a chase the world's frame is
    // the nearest place with an origin of its own, not always the one you
    // are in (exploration 0023).
    let feet = w.players[0].pos.add(w.map_origin());
    let here = sim::atlas::valley()
        .place_for(feet)
        .map_or(w.arena, |p| p.arena);
    let Some(place) = valley::place(here).or(valley::place(w.arena)) else {
        return String::new();
    };
    let name = place.arena.get().name;
    let den = w.den_id();
    let mut out = match place.kind {
        _ if w.hunting() && den != place.arena => {
            let who = w
                .hunted()
                .into_iter()
                .flatten()
                .next()
                .map_or("", |s| s.get().name);
            let from = den.get().name;
            if w.valley.homing {
                format!("The {name} -- the {who} of the {from} gave up, and is going home")
            } else {
                format!("The {name} -- the {who} of the {from} is after you")
            }
        }
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
            _ => out.push_str(&format!("On to {}", seam.says)),
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

/// Rewritten only when what it says changes. In dev mode a second line says
/// how much of the map is loaded (`crate::stream`).
pub fn update_text(
    sim: Res<crate::Sim>,
    stream: Res<crate::stream::Stream>,
    mut text: Query<&mut Text, With<ValleyText>>,
    mut shown: Local<String>,
) {
    let mut now = self::text(&sim.cur);
    if crate::dev_mode() && sim.cur.valley.on {
        let (places, boxes, tiles) = stream.loaded();
        let atlas = sim::atlas::valley();
        now.push_str(&format!(
            "\nloaded: {places} of {} places, {boxes} of {} boxes, {tiles} tiles of land",
            atlas.places.len(),
            atlas.solids.len()
        ));
    }
    if *shown == now {
        return;
    }
    if let Ok(mut t) = text.single_mut() {
        *t = Text::new(now.clone());
    }
    *shown = now;
}
