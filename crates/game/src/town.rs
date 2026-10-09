//! **The town, drawn** (`sim::arena::hearth`): houses of plaster and timber
//! under tiled roofs, the inn, the hall, the armoury, the barn, the bell
//! tower's spire, market stalls under awnings, barrels, crates, a cart, hay,
//! lamps along the street, the well, banners at the gates, flagstones.
//!
//! **Drawn from the town's own tables**, the ones its collision is made from
//! (`hearth::BUILDINGS`, `PROPS`, `TREES`), so a house is drawn over exactly
//! the boxes a body bumps into: its walls to the eaves, its roof's steps
//! inside the slopes drawn over them. Those boxes are not drawn again by what
//! they are made of ([`whole`]). The roof's eaves overhang the walls a hand
//! or two, above anybody's head; everything else stays in its box.
//!
//! Colours are the palette's (`look::Palette::roof`, `plaster`, `framing`,
//! `cloth`, ...); nothing here decides one.

use bevy::prelude::*;
use look::Palette;
use sim::arena::hearth::{self, Building, Prop, Use};
use sim::arena::{ArenaId, Material};

use crate::arenas::{Under, put};
use crate::forms::Kit;
use crate::shapes::hash01;

fn m(v: i32) -> f32 {
    v as f32 / 100.0
}

fn vary(rgb: [f32; 3], k: f32, by: f32) -> [f32; 3] {
    let s = 1.0 + by * (k - 0.5) * 2.0;
    [rgb[0] * s, rgb[1] * s, rgb[2] * s]
}

/// **Is a box one the town draws whole** -- a building's, a prop's, a
/// tree's -- and not to be drawn again by its material? The box in the
/// town's own coordinates, metres.
pub fn whole(arena: ArenaId, min: Vec3, max: Vec3) -> bool {
    if arena != ArenaId::HEARTH {
        return false;
    }
    let f = |v: sim::Fx| v.to_f32_for_render();
    (hearth::BUILDINGS_FROM..hearth::TOWERS_FROM).any(|k| {
        let s = &hearth::ARENA.solids[k];
        (Vec3::new(f(s.min.x), f(s.min.y), f(s.min.z)) - min)
            .abs()
            .max_element()
            < 0.01
            && (Vec3::new(f(s.max.x), f(s.max.y), f(s.max.z)) - max)
                .abs()
                .max_element()
                < 0.01
    })
}

/// **Draw the town** under `under`, in its own coordinates: every
/// building, prop and decoration; and, alone (off the valley's map, where
/// `crate::stream` draws trees round trunks), its trees.
pub fn draw(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
    white: &Handle<StandardMaterial>,
    under: Under,
) {
    let mut kit = Kit::new();
    for (k, b) in hearth::BUILDINGS.iter().enumerate() {
        building(&mut kit, b, k as u32, palette);
    }
    let mut glow = Kit::new();
    for (k, (what, lo, hi)) in hearth::PROPS.iter().enumerate() {
        let lo = Vec3::new(m(lo[0]), m(lo[1]), m(lo[2]));
        let hi = Vec3::new(m(hi[0]), m(hi[1]), m(hi[2]));
        prop(&mut kit, &mut glow, *what, lo, hi, k as u32, palette);
    }
    spire(&mut kit, palette);
    banners(&mut kit, palette);
    paving(&mut kit, palette);
    put(
        commands,
        under,
        (
            Mesh3d(meshes.add(kit.build())),
            MeshMaterial3d(white.clone()),
            Transform::default(),
        ),
    );
    if !glow.is_empty() {
        let lamp = palette.surface([1.0, 0.82, 0.5]);
        let lit = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            emissive: LinearRgba::rgb(lamp[0] * 4.0, lamp[1] * 3.2, lamp[2] * 2.0),
            ..default()
        });
        put(
            commands,
            under,
            (
                Mesh3d(meshes.add(glow.build())),
                MeshMaterial3d(lit),
                Transform::default(),
                bevy::pbr::NotShadowCaster,
            ),
        );
    }
    if matches!(under, Under::Scenery) {
        let root = put(
            commands,
            under,
            (Transform::default(), Visibility::default()),
        );
        let mut trees = crate::land::Trees::default();
        for k in hearth::TREES_FROM..hearth::TREES_FROM + hearth::TREES.len() {
            let s = hearth::ARENA.solids[k];
            trees.spawn(
                commands,
                meshes,
                materials,
                &s,
                ArenaId::HEARTH,
                palette,
                root,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Buildings
// ---------------------------------------------------------------------------

/// The faces of a footprint: outward normal, a corner it runs from, the
/// direction it runs, its length.
fn sides(lo: Vec3, hi: Vec3) -> [(Vec3, Vec3, Vec3, f32); 4] {
    [
        (
            Vec3::NEG_Z,
            Vec3::new(lo.x, 0.0, lo.z),
            Vec3::X,
            hi.x - lo.x,
        ),
        (
            Vec3::Z,
            Vec3::new(hi.x, 0.0, hi.z),
            Vec3::NEG_X,
            hi.x - lo.x,
        ),
        (
            Vec3::NEG_X,
            Vec3::new(lo.x, 0.0, hi.z),
            Vec3::NEG_Z,
            hi.z - lo.z,
        ),
        (Vec3::X, Vec3::new(hi.x, 0.0, lo.z), Vec3::Z, hi.z - lo.z),
    ]
}

/// A slab on a face: from `a` to `b` along it, `y0` to `y1` up, standing
/// `proud` off the face's plane inward-to-outward from `depth` behind it.
#[allow(clippy::too_many_arguments)]
fn on_face(
    kit: &mut Kit,
    side: &(Vec3, Vec3, Vec3, f32),
    a: f32,
    b: f32,
    y0: f32,
    y1: f32,
    back: f32,
    front: f32,
    rgb: [f32; 3],
) {
    let (out, from, dir, _) = *side;
    let p0 = from + dir * a + out * -back + Vec3::Y * y0;
    let p1 = from + dir * b + out * -front + Vec3::Y * y1;
    kit.cube(p0.min(p1), p0.max(p1), rgb, rgb);
}

/// **A building**: a stone footing, walls of plaster between dark timbers
/// (or stone, or boards), windows with shutters, a door to the street, a
/// tiled roof with a ridge and overhanging eaves, a chimney.
fn building(kit: &mut Kit, b: &Building, seed: u32, p: &Palette) {
    let lo = Vec3::new(m(b.lo.0), 0.0, m(b.lo.1));
    let hi = Vec3::new(m(b.hi.0), m(b.eave), m(b.hi.1));
    let eave = m(b.eave);
    let ridge = m(b.ridge);
    let stone = p.of(Material::Stone);
    let plaster = vary(p.plaster(), hash01(seed, 1, 1), 0.06);
    let frame = p.framing();
    let wood = p.of(Material::Wood);
    let glass = [0.10, 0.11, 0.14];
    let shutter = vary(p.cloth(), hash01(seed, 2, 2), 0.15);
    let foot = 0.6f32.min(eave * 0.2);
    // The walls: stone for the armoury and the hall's ground floor, boards
    // for the barn, plaster everywhere else; set back so the timbers stand
    // proud inside the box.
    let set = 0.07;
    let stone_to = match b.kind {
        Use::Armoury => eave,
        Use::Hall => eave * 0.45,
        _ => foot,
    };
    // The footing and any stone storey: dressed stone, in courses.
    {
        let size = Vec3::new(hi.x - lo.x, stone_to, hi.z - lo.z);
        if let Some(mesh) = crate::forms::build(
            crate::forms::Form::Masonry,
            size,
            seed.wrapping_mul(7919),
            Material::Stone,
            p,
        ) {
            kit.keep(
                &mesh,
                Transform::from_translation(Vec3::new(
                    (lo.x + hi.x) * 0.5,
                    stone_to * 0.5,
                    (lo.z + hi.z) * 0.5,
                )),
            );
        }
    }
    let wall = match b.kind {
        Use::Barn => vary(wood, 0.4, 0.1),
        _ => plaster,
    };
    if stone_to < eave {
        kit.cube(
            Vec3::new(lo.x + set, stone_to, lo.z + set),
            Vec3::new(hi.x - set, eave, hi.z - set),
            wall,
            wall,
        );
    }
    // The street side: whichever long face looks toward the main street (z
    // = 0), or the lane.
    let toward = if (lo.z + hi.z) * 0.5 > 0.0 { 0 } else { 1 };
    let storeys = if eave > 5.0 { 2 } else { 1 };
    let floor_h = (eave - foot) / storeys as f32;
    for (si, side) in sides(lo, hi).iter().enumerate() {
        let (_, _, _, len) = *side;
        if b.kind == Use::Barn {
            // Boards, upright, each a shade off its neighbour.
            let n = (len / 0.3) as u32;
            for k in 0..n {
                let a = len * k as f32 / n as f32;
                let c = vary(wood, hash01(seed, k, si as u32), 0.12);
                on_face(
                    kit,
                    side,
                    a + 0.01,
                    a + len / n as f32 - 0.01,
                    foot,
                    eave,
                    set,
                    0.0,
                    c,
                );
            }
        }
        if b.kind != Use::Armoury && b.kind != Use::Barn {
            // The frame: posts at the corners and every metre and a half,
            // a sill, a plate at each floor and the eaves, braces.
            let from_y = stone_to.max(foot);
            let posts = ((len / 1.5).round() as u32).max(2);
            for k in 0..=posts {
                let a = (len * k as f32 / posts as f32).clamp(0.12, len - 0.12);
                on_face(kit, side, a - 0.12, a + 0.12, from_y, eave, set, 0.0, frame);
            }
            let mut y = from_y;
            while y < eave - 0.1 {
                on_face(kit, side, 0.0, len, y, y + 0.2, set, 0.0, frame);
                y += floor_h;
            }
            on_face(kit, side, 0.0, len, eave - 0.22, eave, set, 0.0, frame);
            // Braces, out from the corners, on the upper storey.
            if eave - from_y > 2.0 {
                let (out, origin, dir, _) = *side;
                let y0 = eave - floor_h + 0.2;
                let y1 = eave - 0.22;
                for (a0, a1) in [
                    (0.12, 1.3f32.min(len * 0.3)),
                    (len - 0.12, len - 1.3f32.min(len * 0.3)),
                ] {
                    let s = origin + dir * a0 + out * -0.035 + Vec3::Y * y0;
                    let e = origin + dir * a1 + out * -0.035 + Vec3::Y * y1;
                    kit.tube(s, e, 0.07, 0.07, 4, frame, None);
                }
            }
        }
        // Windows, each storey, with frames and shutters; and on the street
        // side, the door.
        let street = (si == 0 && toward == 0) || (si == 1 && toward == 1);
        let door_at = len * 0.5;
        let n = ((len / 2.3).floor() as u32).max(1);
        for storey in 0..storeys {
            let y0 = foot + floor_h * storey as f32 + floor_h * 0.35;
            let y1 = (y0 + floor_h * 0.4).min(eave - 0.35);
            if y1 - y0 < 0.4 {
                continue;
            }
            for k in 0..n {
                let a = len * (k as f32 + 0.5) / n as f32;
                if street && storey == 0 && (a - door_at).abs() < 1.2 {
                    continue;
                }
                let w = 0.45;
                on_face(
                    kit,
                    side,
                    a - w,
                    a + w,
                    y0,
                    y1,
                    set + 0.05,
                    set - 0.02,
                    glass,
                );
                // Sill and lintel.
                on_face(
                    kit,
                    side,
                    a - w - 0.08,
                    a + w + 0.08,
                    y0 - 0.1,
                    y0,
                    set + 0.02,
                    0.0,
                    wood,
                );
                on_face(
                    kit,
                    side,
                    a - w - 0.08,
                    a + w + 0.08,
                    y1,
                    y1 + 0.1,
                    set + 0.02,
                    0.0,
                    frame,
                );
                // Shutters, open, either side.
                if b.kind != Use::Barn && b.kind != Use::Armoury {
                    on_face(
                        kit,
                        side,
                        a - w - 0.42,
                        a - w - 0.06,
                        y0,
                        y1,
                        set,
                        0.0,
                        shutter,
                    );
                    on_face(
                        kit,
                        side,
                        a + w + 0.06,
                        a + w + 0.42,
                        y0,
                        y1,
                        set,
                        0.0,
                        shutter,
                    );
                }
            }
        }
        if street {
            let (dw, dh) = match b.kind {
                Use::Barn | Use::Hall => (1.3, 3.0f32.min(eave - 0.6)),
                _ => (0.65, 2.2f32.min(eave - 0.4)),
            };
            on_face(
                kit,
                side,
                door_at - dw,
                door_at + dw,
                0.0,
                dh,
                set + 0.06,
                set - 0.01,
                vary(wood, 0.25, 0.1),
            );
            on_face(
                kit,
                side,
                door_at - dw - 0.12,
                door_at + dw + 0.12,
                dh,
                dh + 0.15,
                set,
                0.0,
                frame,
            );
            on_face(
                kit,
                side,
                door_at - dw - 0.12,
                door_at - dw,
                0.0,
                dh,
                set,
                0.0,
                frame,
            );
            on_face(
                kit,
                side,
                door_at + dw,
                door_at + dw + 0.12,
                0.0,
                dh,
                set,
                0.0,
                frame,
            );
            // The inn's sign, out over the street from beside the door.
            if b.kind == Use::Inn {
                let (out, origin, dir, _) = *side;
                let at = origin + dir * (door_at + dw + 1.0) + Vec3::Y * (dh + 0.6);
                kit.tube(at, at + out * 0.9, 0.04, 0.04, 4, frame, None);
                let board = at + out * 0.6 - Vec3::Y * 0.45;
                let across = dir * 0.02;
                kit.cube(
                    (board - across - out * 0.3 - Vec3::Y * 0.3).min(board + across + out * 0.3),
                    (board - across - out * 0.3).max(board + across + out * 0.3 + Vec3::Y * 0.1),
                    p.cloth(),
                    p.cloth(),
                );
            }
        }
    }
    roof(kit, b, lo, hi, eave, ridge, seed, p);
    // The chimney, through the roof.
    let c = hearth::chimney(b);
    let f = |v: sim::Fx| v.to_f32_for_render();
    let (cl, ch) = (
        Vec3::new(f(c.min.x), f(c.min.y), f(c.min.z)),
        Vec3::new(f(c.max.x), f(c.max.y), f(c.max.z)),
    );
    let brick = vary(stone, 0.35, 0.1);
    kit.cube(
        cl + Vec3::new(0.04, 0.0, 0.04),
        ch - Vec3::new(0.04, 0.12, 0.04),
        brick,
        brick,
    );
    kit.cube(
        Vec3::new(cl.x, ch.y - 0.12, cl.z),
        ch,
        vary(stone, 0.6, 0.1),
        p.mortar(),
    );
}

/// **A tiled roof** over a building: two slopes from the eaves to the
/// ridge, laid in courses that overlap, overhanging the walls; its underside
/// dark; its gables filled; a ridge along the top.
#[allow(clippy::too_many_arguments)]
fn roof(
    kit: &mut Kit,
    b: &Building,
    lo: Vec3,
    hi: Vec3,
    eave: f32,
    ridge: f32,
    seed: u32,
    p: &Palette,
) {
    let tile = p.roof();
    let under = p.framing();
    let over = 0.45;
    // Along the ridge (u) and across it (v), as world axes.
    let (u, v) = if b.along_x {
        (Vec3::X, Vec3::Z)
    } else {
        (Vec3::Z, Vec3::X)
    };
    let mid = (lo + hi) * 0.5;
    let len = (hi - lo).dot(u);
    let half = (hi - lo).dot(v) * 0.5;
    let rise = ridge - eave;
    let centre = Vec3::new(mid.x, 0.0, mid.z);
    // A slope's point: `t` along the ridge (-1..1 over its length plus the
    // overhang), `s` from the ridge (0) to the eaves (1) and past them.
    let at = |side: f32, t: f32, s: f32| {
        centre + u * (t * (len * 0.5 + over)) + v * side * (half * s) + Vec3::Y * (ridge - rise * s)
    };
    let past = 1.0 + over / half;
    let courses = ((half * past / 0.38).ceil() as u32).max(3);
    for side in [-1.0f32, 1.0] {
        let out = v * side + Vec3::Y * (half / rise.max(0.1));
        for k in 0..courses {
            let s0 = past * k as f32 / courses as f32;
            let s1 = past * (k + 1) as f32 / courses as f32;
            // Each course a little proud at its lower edge, over the next.
            let lift = Vec3::Y * 0.05;
            let c = vary(tile, hash01(seed, k, (side > 0.0) as u32 + 3), 0.09);
            kit.poly(
                &[
                    at(side, -1.0, s0),
                    at(side, 1.0, s0),
                    at(side, 1.0, s1) + lift,
                    at(side, -1.0, s1) + lift,
                ],
                out,
                c,
            );
            // The course's lower lip.
            kit.poly(
                &[
                    at(side, -1.0, s1),
                    at(side, 1.0, s1),
                    at(side, 1.0, s1) + lift,
                    at(side, -1.0, s1) + lift,
                ],
                v * side - Vec3::Y * 0.2,
                vary(c, 0.2, 0.2),
            );
        }
        // The underside, dark.
        let drop = Vec3::Y * -0.12;
        kit.poly(
            &[
                at(side, -1.0, 0.0) + drop,
                at(side, 1.0, 0.0) + drop,
                at(side, 1.0, past) + drop,
                at(side, -1.0, past) + drop,
            ],
            -out,
            under,
        );
        // The fascia along the eaves.
        kit.poly(
            &[
                at(side, -1.0, past) + drop,
                at(side, 1.0, past) + drop,
                at(side, 1.0, past),
                at(side, -1.0, past),
            ],
            v * side,
            under,
        );
    }
    // The gables: triangles at each end, in the walls' plane, framed.
    let plaster = p.plaster();
    let frame = p.framing();
    for end in [-1.0f32, 1.0] {
        let e = centre + u * end * len * 0.5;
        let a = e + v * -half + Vec3::Y * eave;
        let bb = e + v * half + Vec3::Y * eave;
        let top = e + Vec3::Y * ridge;
        let fill = if b.kind == Use::Barn {
            p.of(Material::Wood)
        } else {
            plaster
        };
        kit.poly(&[a, bb, top], u * end, fill);
        // A king post and a collar.
        let face = u * end * 0.03;
        kit.tube(
            e + Vec3::Y * eave + face,
            top + face,
            0.08,
            0.08,
            4,
            frame,
            None,
        );
        kit.tube(
            e + v * (-half * 0.5) + Vec3::Y * (eave + rise * 0.5) + face,
            e + v * (half * 0.5) + Vec3::Y * (eave + rise * 0.5) + face,
            0.07,
            0.07,
            4,
            frame,
            None,
        );
    }
    // The ridge.
    kit.tube(
        centre + u * -(len * 0.5 + over) + Vec3::Y * (ridge + 0.04),
        centre + u * (len * 0.5 + over) + Vec3::Y * (ridge + 0.04),
        0.13,
        0.13,
        6,
        vary(tile, 0.15, 0.2),
        Some(vary(tile, 0.15, 0.2)),
    );
}

/// The bell tower's spire: a steep pyramid of slate over its top, with the
/// belfry's openings dark on every face.
fn spire(kit: &mut Kit, p: &Palette) {
    let Some(t) = hearth::TOWERS.iter().find(|t| t.spire) else {
        return;
    };
    let lo = Vec3::new(m(t.lo.0), m(t.high), m(t.lo.1));
    let hi = Vec3::new(m(t.hi.0), m(t.high), m(t.hi.1));
    let mid = (lo + hi) * 0.5;
    let over = 0.35;
    let a = Vec3::new(lo.x - over, lo.y, lo.z - over);
    let b = Vec3::new(hi.x + over, lo.y, lo.z - over);
    let c = Vec3::new(hi.x + over, lo.y, hi.z + over);
    let d = Vec3::new(lo.x - over, lo.y, hi.z + over);
    let apex = mid + Vec3::Y * ((hi.x - lo.x) * 1.6);
    let slate = look::tint::mix(p.roof(), p.of(Material::Rock), 0.55);
    for (q, r) in [(a, b), (b, c), (c, d), (d, a)] {
        let out = ((q + r) * 0.5 - mid).normalize_or(Vec3::X) + Vec3::Y * 0.4;
        kit.poly(&[q, r, apex], out, slate);
    }
    kit.poly(&[a, b, c, d], Vec3::NEG_Y, p.framing());
    kit.tube(
        apex,
        apex + Vec3::Y * 1.2,
        0.05,
        0.02,
        4,
        p.of(Material::Stone),
        None,
    );
    // The belfry: an opening on each face under the spire, a hair proud.
    let y0 = lo.y - 3.0;
    let y1 = lo.y - 0.8;
    let w = (hi.x - lo.x) * 0.22;
    let dark = [0.07, 0.07, 0.09];
    for (n, at) in [
        (Vec3::NEG_Z, Vec3::new(mid.x, 0.0, lo.z - 0.01)),
        (Vec3::Z, Vec3::new(mid.x, 0.0, hi.z + 0.01)),
        (Vec3::NEG_X, Vec3::new(lo.x - 0.01, 0.0, mid.z)),
        (Vec3::X, Vec3::new(hi.x + 0.01, 0.0, mid.z)),
    ] {
        let across = Vec3::new(n.z.abs(), 0.0, n.x.abs()) * w;
        let pts = [
            at - across + Vec3::Y * y0,
            at + across + Vec3::Y * y0,
            at + across + Vec3::Y * y1,
            at + Vec3::Y * (y1 + w * 0.8),
            at - across + Vec3::Y * y1,
        ];
        kit.poly(&pts, n, dark);
    }
}

/// Banners on the inside of the wall either side of each gate, in the
/// town's colour.
fn banners(kit: &mut Kit, p: &Palette) {
    let cloth = p.cloth();
    let trim = p.plaster();
    let wall = m(hearth::WALL);
    // (x, z, which way the inner face looks)
    let at = [
        (34.98, -6.0, Vec3::NEG_X),
        (34.98, 6.0, Vec3::NEG_X),
        (-34.98, -6.0, Vec3::X),
        (-34.98, 6.0, Vec3::X),
        (-6.0, 29.98, Vec3::NEG_Z),
        (6.0, 29.98, Vec3::NEG_Z),
    ];
    for (x, z, n) in at {
        let base = Vec3::new(x, 0.0, z) + n * 0.02;
        let across = Vec3::new(n.z.abs(), 0.0, n.x.abs());
        let top = wall - 0.6;
        let low = wall - 4.6;
        kit.poly(
            &[
                base - across * 0.8 + Vec3::Y * top,
                base + across * 0.8 + Vec3::Y * top,
                base + across * 0.8 + Vec3::Y * (low + 0.5),
                base + Vec3::Y * low,
                base - across * 0.8 + Vec3::Y * (low + 0.5),
            ],
            n,
            cloth,
        );
        kit.poly(
            &[
                base - across * 0.8 + Vec3::Y * (top - 0.5) + n * 0.01,
                base + across * 0.8 + Vec3::Y * (top - 0.5) + n * 0.01,
                base + across * 0.8 + Vec3::Y * (top - 0.65) + n * 0.01,
                base - across * 0.8 + Vec3::Y * (top - 0.65) + n * 0.01,
            ],
            n,
            trim,
        );
        kit.tube(
            base - across + Vec3::Y * (top + 0.05) + n * 0.08,
            base + across + Vec3::Y * (top + 0.05) + n * 0.08,
            0.05,
            0.05,
            5,
            p.framing(),
            None,
        );
    }
}

/// **Flagstones** where the town is paved: the street, the lane, the
/// square. A slab a little over a metre, set a hair proud, its colour a
/// shade off its neighbours'.
fn paving(kit: &mut Kit, p: &Palette) {
    let stone = p.of(Material::Stone);
    let arena = &hearth::ARENA;
    let b = arena.bounds;
    let f = |v: sim::Fx| v.to_f32_for_render();
    let to_fx = |v: f32| sim::Fx::from_raw((v * 65536.0) as i32);
    let cell = 0.8f32;
    let (x0, z0) = (f(b.lo_x), f(b.lo_z));
    let nx = ((f(b.hi_x) - x0) / cell) as i32;
    let nz = ((f(b.hi_z) - z0) / cell) as i32;
    for j in 0..nz {
        // Courses offset by half a slab, as a mason lays them.
        let shift = if j % 2 == 0 { 0.0 } else { cell * 0.5 };
        for i in 0..nx {
            let x = x0 + i as f32 * cell + shift;
            let z = z0 + j as f32 * cell;
            let (cx, cz) = (x + cell * 0.5, z + cell * 0.5);
            if arena.floor_at(to_fx(cx), to_fx(cz)) != Material::Stone {
                continue;
            }
            // Not under anything standing there.
            let at = sim::V3::new(to_fx(cx), sim::Fx::from_int(5), to_fx(cz));
            if f(arena.ground_under(at)) > 0.01 {
                continue;
            }
            // Worn: a shade darker than new stone, each one its own, the odd
            // one gone green, and the long side of each a little short.
            let r = hash01(i as u32, j as u32, 41);
            let worn = look::tint::mix(stone, p.mortar(), 0.25);
            let c = vary(worn, r, 0.08);
            let short = 0.08 * hash01(i as u32, j as u32, 42);
            kit.cube(
                Vec3::new(x + 0.035, 0.0, z + 0.035),
                Vec3::new(x + cell - 0.035 - short, 0.03, z + cell - 0.035),
                c,
                c,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn prop(kit: &mut Kit, glow: &mut Kit, what: Prop, lo: Vec3, hi: Vec3, seed: u32, p: &Palette) {
    let size = hi - lo;
    let mid = (lo + hi) * 0.5;
    let wood = p.of(Material::Wood);
    let frame = p.framing();
    let stone = p.of(Material::Stone);
    match what {
        Prop::Barrel => {
            let r = size.x.min(size.z) * 0.5;
            let rings = [(0.0, 0.85), (0.5, 1.0), (1.0, 0.85)];
            let c = vary(wood, hash01(seed, 1, 1), 0.12);
            for w in rings.windows(2) {
                let (a, b) = (w[0], w[1]);
                kit.tube(
                    Vec3::new(mid.x, lo.y + size.y * a.0, mid.z),
                    Vec3::new(mid.x, lo.y + size.y * b.0, mid.z),
                    r * a.1,
                    r * b.1,
                    12,
                    c,
                    None,
                );
            }
            kit.tube(
                Vec3::new(mid.x, hi.y - 0.02, mid.z),
                Vec3::new(mid.x, hi.y, mid.z),
                r * 0.85,
                r * 0.85,
                12,
                vary(wood, 0.8, 0.1),
                Some(vary(wood, 0.8, 0.1)),
            );
            for t in [0.15, 0.5, 0.85] {
                let y = lo.y + size.y * t;
                let k = if t == 0.5 { 1.0 } else { 0.9 };
                kit.tube(
                    Vec3::new(mid.x, y - 0.03, mid.z),
                    Vec3::new(mid.x, y + 0.03, mid.z),
                    r * k + 0.005,
                    r * k + 0.005,
                    12,
                    [0.22, 0.2, 0.2],
                    None,
                );
            }
        }
        Prop::Crate => {
            let c = vary(wood, hash01(seed, 2, 2), 0.12);
            kit.cube(lo + Vec3::splat(0.03), hi - Vec3::splat(0.03), c, c);
            // Battens round the edges.
            let t = 0.06;
            for (a, b) in [
                (Vec3::new(lo.x, lo.y, lo.z), Vec3::new(hi.x, lo.y + t, hi.z)),
                (Vec3::new(lo.x, hi.y - t, lo.z), Vec3::new(hi.x, hi.y, hi.z)),
                (
                    Vec3::new(lo.x, lo.y, lo.z),
                    Vec3::new(lo.x + t, hi.y, lo.z + t),
                ),
                (
                    Vec3::new(hi.x - t, lo.y, lo.z),
                    Vec3::new(hi.x, hi.y, lo.z + t),
                ),
                (
                    Vec3::new(lo.x, lo.y, hi.z - t),
                    Vec3::new(lo.x + t, hi.y, hi.z),
                ),
                (
                    Vec3::new(hi.x - t, lo.y, hi.z - t),
                    Vec3::new(hi.x, hi.y, hi.z),
                ),
            ] {
                kit.cube(a, b, frame, frame);
            }
        }
        Prop::Stall => {
            // The counter: boards.
            kit.cube(
                lo + Vec3::new(0.05, 0.0, 0.05),
                hi - Vec3::new(0.05, 0.05, 0.05),
                vary(wood, 0.3, 0.1),
                wood,
            );
            kit.cube(
                Vec3::new(lo.x, hi.y - 0.06, lo.z),
                hi,
                vary(wood, 0.7, 0.1),
                vary(wood, 0.7, 0.1),
            );
            // Goods on it: fruit, cloth, pots.
            for k in 0..6u32 {
                let t = (k as f32 + 0.5) / 6.0;
                let at = Vec3::new(
                    lo.x + size.x * t,
                    hi.y,
                    mid.z + (hash01(seed, k, 3) - 0.5) * size.z * 0.5,
                );
                let c = match k % 3 {
                    0 => p.bloom(seed + k),
                    1 => p.cloth(),
                    _ => vary(p.of(Material::Sand), 0.3, 0.2),
                };
                kit.turned(
                    at + Vec3::Y * 0.08,
                    Vec3::new(0.12, 0.08, 0.12),
                    hash01(seed, k, 4) * 3.0,
                    c,
                    c,
                );
            }
            // Posts behind, and an awning out over the front, striped.
            let back = lo.z.min(hi.z);
            let posts = [
                Vec3::new(lo.x + 0.05, 0.0, back),
                Vec3::new(hi.x - 0.05, 0.0, back),
            ];
            for q in posts {
                kit.tube(q, q + Vec3::Y * 2.5, 0.05, 0.05, 5, frame, None);
            }
            let stripes = 6;
            for k in 0..stripes {
                let x0 = lo.x + size.x * k as f32 / stripes as f32;
                let x1 = lo.x + size.x * (k + 1) as f32 / stripes as f32;
                let c = if k % 2 == 0 { p.cloth() } else { p.plaster() };
                kit.poly(
                    &[
                        Vec3::new(x0, 2.5, back),
                        Vec3::new(x1, 2.5, back),
                        Vec3::new(x1, 2.05, hi.z + 0.6),
                        Vec3::new(x0, 2.05, hi.z + 0.6),
                    ],
                    Vec3::new(0.0, 1.0, 0.4),
                    c,
                );
                kit.poly(
                    &[
                        Vec3::new(x0, 2.5, back),
                        Vec3::new(x1, 2.5, back),
                        Vec3::new(x1, 2.05, hi.z + 0.6),
                        Vec3::new(x0, 2.05, hi.z + 0.6),
                    ],
                    Vec3::new(0.0, -1.0, -0.4),
                    vary(c, 0.2, 0.3),
                );
            }
        }
        Prop::Lamp => {
            let iron = [0.16, 0.15, 0.15];
            kit.tube(
                Vec3::new(mid.x, 0.0, mid.z),
                Vec3::new(mid.x, hi.y - 0.4, mid.z),
                0.08,
                0.06,
                6,
                iron,
                None,
            );
            kit.tube(
                Vec3::new(mid.x, 0.0, mid.z),
                Vec3::new(mid.x, 0.25, mid.z),
                0.12,
                0.12,
                6,
                iron,
                Some(iron),
            );
            // The lantern's cage, and its light.
            let at = Vec3::new(mid.x, hi.y - 0.25, mid.z);
            kit.cube(
                at - Vec3::new(0.12, 0.15, 0.12),
                at - Vec3::new(-0.12, 0.13, -0.12),
                iron,
                iron,
            );
            kit.cube(
                at + Vec3::new(-0.13, 0.14, -0.13),
                at + Vec3::new(0.13, 0.2, 0.13),
                iron,
                iron,
            );
            glow.cube(
                at - Vec3::new(0.1, 0.13, 0.1),
                at + Vec3::new(0.1, 0.14, 0.1),
                [1.0, 0.9, 0.7],
                [1.0, 0.9, 0.7],
            );
        }
        Prop::Well => {
            let r = size.x.min(size.z) * 0.5;
            kit.tube(
                Vec3::new(mid.x, 0.0, mid.z),
                Vec3::new(mid.x, hi.y, mid.z),
                r,
                r * 0.97,
                14,
                vary(stone, 0.4, 0.1),
                Some(vary(stone, 0.6, 0.1)),
            );
            // Dark water inside the ring's top.
            kit.tube(
                Vec3::new(mid.x, hi.y, mid.z),
                Vec3::new(mid.x, hi.y + 0.005, mid.z),
                r * 0.75,
                r * 0.75,
                14,
                [0.08, 0.12, 0.16],
                Some([0.08, 0.12, 0.16]),
            );
            // Two posts, a crank, and a little roof over it.
            for side in [-1.0f32, 1.0] {
                let q = Vec3::new(mid.x + side * r * 0.85, hi.y, mid.z);
                kit.tube(q, q + Vec3::Y * 1.5, 0.06, 0.06, 5, frame, None);
            }
            kit.tube(
                Vec3::new(mid.x - r * 0.85, hi.y + 1.1, mid.z),
                Vec3::new(mid.x + r * 0.85, hi.y + 1.1, mid.z),
                0.04,
                0.04,
                5,
                frame,
                None,
            );
            let top = hi.y + 1.5;
            let roof = p.roof();
            for side in [-1.0f32, 1.0] {
                kit.poly(
                    &[
                        Vec3::new(mid.x - r - 0.2, top + 0.6, mid.z),
                        Vec3::new(mid.x + r + 0.2, top + 0.6, mid.z),
                        Vec3::new(mid.x + r + 0.2, top, mid.z + side * (r + 0.2)),
                        Vec3::new(mid.x - r - 0.2, top, mid.z + side * (r + 0.2)),
                    ],
                    Vec3::new(0.0, 1.0, side),
                    roof,
                );
            }
        }
        Prop::Cart => {
            let bed = vary(wood, 0.5, 0.1);
            kit.cube(
                Vec3::new(lo.x + 0.1, 0.55, lo.z + 0.1),
                Vec3::new(hi.x - 0.1, 0.7, hi.z - 0.1),
                bed,
                bed,
            );
            for side in [lo.z + 0.05, hi.z - 0.05] {
                kit.cube(
                    Vec3::new(lo.x + 0.1, 0.7, side - 0.04),
                    Vec3::new(hi.x - 0.1, hi.y, side + 0.04),
                    frame,
                    frame,
                );
            }
            for side in [lo.z + 0.06, hi.z - 0.06] {
                let c = Vec3::new(mid.x, 0.55, side);
                kit.tube(
                    c - Vec3::Z * 0.06,
                    c + Vec3::Z * 0.06,
                    0.55,
                    0.55,
                    12,
                    frame,
                    Some(vary(wood, 0.2, 0.1)),
                );
            }
            // The shafts, resting on the ground.
            for dz in [-0.35f32, 0.35] {
                kit.tube(
                    Vec3::new(hi.x - 0.1, 0.62, mid.z + dz),
                    Vec3::new(hi.x + 1.4, 0.05, mid.z + dz * 0.8),
                    0.04,
                    0.04,
                    4,
                    frame,
                    None,
                );
            }
        }
        Prop::Hay => {
            let hay = look::tint::mix(p.of(Material::Sand), p.of(Material::Grass), 0.25);
            let bale = crate::shapes::soft_box(size, 0.35, 8, None);
            kit.mesh(
                &bale,
                Transform::from_translation(mid),
                vary(hay, hash01(seed, 5, 5), 0.1),
            );
            for t in [0.3f32, 0.7] {
                let x = lo.x + size.x * t;
                kit.cube(
                    Vec3::new(x - 0.02, lo.y + 0.02, lo.z + 0.02),
                    Vec3::new(x + 0.02, hi.y - 0.01, hi.z - 0.02),
                    frame,
                    frame,
                );
            }
        }
    }
}
