//! Contact sheets for the creature: look at it without running the game.
//!
//! ```text
//! cargo run -p anim --bin preview_beast -- shake
//! cargo run -p anim --bin preview_beast -- --all
//! cargo run -p anim --bin preview_beast -- --species ridgeback --all
//! ```
//!
//! Any species: everything drawn is read off its table.
//!
//! The fighters' sheet draws a stick figure, because a fighter's *silhouette*
//! is what an opponent reads. The creature's parts are boxes and the boxes are
//! what you collide with, stand on and hit, so this draws the boxes -- every
//! one of them, in the world, through the same rig the simulation uses. If a
//! wedge opens between two segments here, it opens in the game.
//!
//! Three things it is specifically good at catching:
//!
//! - **A limb that does not move.** The whole reason the creature was rebuilt.
//!   Four legs drawn over twenty-four frames of a walk either stride or they do
//!   not, and a table of angles will not tell you which.
//! - **Where the mountable surfaces are**, drawn in green, against a scale bar
//!   at the height a full hop reaches. That is the climb, as a picture.
//! - **A pose that steps.** The overlay panel stacks every frame; a clip that
//!   jumps shows as a gap in an otherwise even fan, and a gap is an
//!   arbitrarily large acceleration, which is what throws riders.

use crate::png::Canvas;
use sim::beast;
use sim::fixed::Fx;
use sim::math::V3;
use sim::monster::{Doing, Monster};
use sim::species::Species;

const BACKDROP: [u8; 3] = [22, 24, 28];
const PANEL: [u8; 3] = [30, 33, 38];
const PANEL_ALT: [u8; 3] = [35, 39, 45];
const FLOOR: [u8; 3] = [70, 76, 86];
const ARMOUR: [u8; 3] = [150, 158, 150];
const WEAK: [u8; 3] = [232, 110, 84];
const MOUNTABLE: [u8; 3] = [96, 212, 158];
const LEG: [u8; 3] = [196, 186, 150];
const LABEL: [u8; 3] = [120, 128, 140];
const JUMP: [u8; 3] = [90, 130, 200];

const CELL_W: usize = 190;
const CELL_H: usize = 150;
const COLS: usize = 8;
const TRAIL_H: usize = 300;

/// Where a cell's camera sits: which way it looks, where the creature's origin
/// lands on the page, and how many pixels a metre is.
#[derive(Clone, Copy)]
struct Lens {
    view: View,
    ox: f32,
    oy: f32,
    scale: f32,
}

/// Which way the camera is looking.
#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    /// From the creature's right. The view an animation is read from.
    Side,
    /// From above. Where a tail swing and a body turn are visible and a side
    /// view shows nothing at all.
    Top,
}

/// Every sample of a clip, as poses the rig can be built from.
fn poses(sp: &Species, clip: usize) -> Vec<beast::Pose> {
    let (_, count) = sp.span[clip];
    let count = count.max(1) as usize;
    (0..count)
        .map(|i| {
            let at = Fx::from_int(i as i32).div(Fx::from_int(count.max(2) as i32 - 1));
            if sp.clips[clip].phased {
                let which = (i * 3 / count).min(2) as u8;
                let inside = i % (count / 3).max(1);
                let span = (count / 3).max(2) - 1;
                beast::sample_phase(
                    sp,
                    clip,
                    which,
                    Fx::from_int(inside as i32).div(Fx::from_int(span as i32)),
                )
            } else {
                beast::sample(sp, clip, at)
            }
        })
        .collect()
}

/// The creature posed by one sample, with the layers the game adds on top.
fn beast_at(sp: &'static Species, pose: &beast::Pose) -> beast::Rig {
    beast::Rig::build(sp, V3::ZERO, Fx::ZERO, pose)
}

fn f(v: Fx) -> f32 {
    v.to_f32_for_render()
}

impl Lens {
    /// Project a world point onto the page.
    fn at(&self, p: V3) -> (f32, f32) {
        match self.view {
            View::Side => (self.ox + f(p.x) * self.scale, self.oy - f(p.y) * self.scale),
            View::Top => (self.ox + f(p.x) * self.scale, self.oy - f(p.z) * self.scale),
        }
    }
}

/// Draw one part's box as a wireframe.
fn draw_part(c: &mut Canvas, rig: &beast::Rig, part: usize, lens: Lens, alpha: f32) {
    let sh = rig.species.shape(part);
    let colour = colour_of(rig.species, part);
    let corner = |i: usize| {
        let x = if i & 1 == 0 { sh.min.x } else { sh.max.x };
        let y = if i & 2 == 0 { sh.min.y } else { sh.max.y };
        let z = if i & 4 == 0 { sh.min.z } else { sh.max.z };
        lens.at(rig.part_to_world(part, V3::new(x, y, z)))
    };
    // The twelve edges of a box, as pairs of corner indices.
    const EDGES: [(usize, usize); 12] = [
        (0, 1),
        (2, 3),
        (4, 5),
        (6, 7),
        (0, 2),
        (1, 3),
        (4, 6),
        (5, 7),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    for (a, b) in EDGES {
        if alpha >= 1.0 {
            c.line(corner(a), corner(b), colour, 1.0);
        } else {
            // A ghost trail: drawn dot by dot so it can be faded.
            let (x0, y0) = corner(a);
            let (x1, y1) = corner(b);
            let steps = ((x1 - x0).abs().max((y1 - y0).abs()) as i32).max(1);
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                c.blend(
                    (x0 + (x1 - x0) * t).round() as i32,
                    (y0 + (y1 - y0) * t).round() as i32,
                    colour,
                    alpha,
                );
            }
        }
    }
}

fn colour_of(sp: &Species, part: usize) -> [u8; 3] {
    let shape = sp.parts[part].shape;
    if sp.is_weak_point(part) {
        WEAK
    } else if shape.mountable {
        MOUNTABLE
    } else if shape.breakable {
        LEG
    } else {
        ARMOUR
    }
}

/// How much bigger than the Ridgeback's scale a species is drawn: one for
/// anything thirteen metres long or more, and enough to fill the cell for a
/// smaller animal -- a four-metre cat at the Ridgeback's scale is a dozen
/// pixels of boxes nobody can check a pose in.
fn zoom(sp: &'static Species) -> f32 {
    let rig = beast_at(sp, &beast::Pose::rest(sp.bones.len()));
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for part in 0..sp.parts.len() {
        let sh = sp.shape(part);
        for x in [sh.min.x, sh.max.x] {
            let at = rig.part_to_world(part, V3::new(x, Fx::ZERO, Fx::ZERO));
            let at = at.x.to_f32_for_render();
            lo = lo.min(at);
            hi = hi.max(at);
        }
    }
    let extent = (hi - lo).max(0.1);
    (13.0 / extent).max(1.0)
}

fn draw_creature(c: &mut Canvas, sp: &'static Species, pose: &beast::Pose, lens: Lens, alpha: f32) {
    let rig = beast_at(sp, pose);
    for part in 0..sp.parts.len() {
        draw_part(c, &rig, part, lens, alpha);
    }
}

/// Draw a clip as a sheet: a side row, a top-down row, and an overlay of every
/// frame so the arcs are visible as arcs.
pub fn contact_sheet(sp: &'static Species, clip: usize) -> Canvas {
    let frames = poses(sp, clip);
    let picks: Vec<usize> = if frames.len() <= COLS * 2 {
        (0..frames.len()).collect()
    } else {
        (0..COLS * 2)
            .map(|i| i * (frames.len() - 1) / (COLS * 2 - 1))
            .collect()
    };
    let rows = picks.len().div_ceil(COLS);

    let width = COLS * CELL_W;
    let side_top = 0;
    let top_top = side_top + rows * CELL_H;
    let trail_top = top_top + rows * CELL_H;
    let height = trail_top + TRAIL_H;
    let mut c = Canvas::new(width, height, BACKDROP);

    // Metres to pixels, chosen so a thirteen-metre animal fits a cell -- and a
    // smaller one is drawn bigger (`zoom`).
    let zoom = zoom(sp);
    let scale = CELL_W as f32 / 16.0 * zoom;

    for (view, top) in [(View::Side, side_top), (View::Top, top_top)] {
        for (n, frame) in picks.iter().enumerate() {
            let (col, row) = (n % COLS, n / COLS);
            let x0 = col * CELL_W;
            let y0 = top + row * CELL_H;
            let shade = if (col + row) % 2 == 0 {
                PANEL
            } else {
                PANEL_ALT
            };
            c.rect(
                x0 as i32,
                y0 as i32,
                (x0 + CELL_W) as i32,
                (y0 + CELL_H) as i32,
                shade,
            );
            // The floor, and the height a full hop reaches, so the climb is
            // visible rather than asserted.
            let ox = x0 as f32 + CELL_W as f32 * 0.62;
            let oy = y0 as f32 + CELL_H as f32 * 0.90;
            if view == View::Side {
                c.rect(
                    x0 as i32,
                    oy as i32,
                    (x0 + CELL_W) as i32,
                    oy as i32 + 1,
                    FLOOR,
                );
                let apex = oy - 4.14 * scale;
                for x in (x0..x0 + CELL_W).step_by(4) {
                    c.set(x as i32, apex as i32, JUMP);
                }
            }
            draw_creature(
                &mut c,
                sp,
                &frames[*frame],
                Lens {
                    view,
                    ox,
                    oy,
                    scale,
                },
                1.0,
            );
            crate::sheet::digits(&mut c, x0 + 4, y0 + 4, *frame, LABEL);
        }
    }

    // Every frame on one spot, faded. A clip that steps shows as a gap.
    c.rect(
        0,
        trail_top as i32,
        width as i32,
        (trail_top + TRAIL_H) as i32,
        PANEL,
    );
    let ox = width as f32 * 0.45;
    let oy = trail_top as f32 + TRAIL_H as f32 * 0.88;
    let big = TRAIL_H as f32 / 7.5 * zoom.min(2.0);
    c.rect(0, oy as i32, width as i32, oy as i32 + 1, FLOOR);
    let apex = oy - 4.14 * big;
    for x in (0..width).step_by(4) {
        c.set(x as i32, apex as i32, JUMP);
    }
    for (i, pose) in frames.iter().enumerate() {
        let alpha = 0.16 + 0.5 * (i as f32 / frames.len().max(1) as f32);
        let lens = Lens {
            view: View::Side,
            ox,
            oy,
            scale: big,
        };
        draw_creature(&mut c, sp, pose, lens, alpha);
    }
    crate::sheet::digits(&mut c, 6, trail_top + 6, frames.len(), LABEL);
    c
}

/// The creature in one of the states the fight is built around, side on, big.
///
/// Not a clip: the poses the *simulation* produces, layers and all, which is
/// what a player sees and is not the same thing as a baked sample. A broken leg
/// is a lean the clips know nothing about. Read off the species' table:
/// standing, walking, galloping, each of its moves at its first active frame,
/// down on a knee, toppled, and with its front feet broken.
pub fn states(sp: &'static Species) -> Canvas {
    let with = |doing: Doing| {
        let mut beast = Monster::new(sp.id);
        beast.doing = doing;
        beast
    };
    let mut lame = Monster::new(sp.id);
    for leg in sp.legs.iter().filter(|l| l.front) {
        if let Some(slot) = sp.break_slot(leg.foot) {
            lame.breaks[slot] = 0;
        }
    }
    let mut walking = Monster::new(sp.id);
    walking.stride = 24_000;
    walking.speed = sp.walk();
    let mut running = Monster::new(sp.id);
    running.stride = 40_000;
    running.speed = sp.gallop();
    let mut shown: Vec<(String, Monster)> = vec![
        ("standing".to_string(), Monster::new(sp.id)),
        ("walking".to_string(), walking),
        ("galloping".to_string(), running),
    ];
    for (kind, decl) in sp.moves.iter().enumerate() {
        let left = sp.attack(kind as u8).active;
        shown.push((
            format!("{}, out", decl.name.to_lowercase()),
            with(Doing::Active {
                kind: kind as u8,
                left,
            }),
        ));
    }
    shown.push((
        "stumbling".to_string(),
        with(Doing::Stumble {
            left: sp.stumble_frames() / 2,
            front: true,
        }),
    ));
    shown.push((
        "toppled".to_string(),
        with(Doing::Toppled {
            left: sp.topple_frames() / 2,
        }),
    ));
    shown.push(("front feet broken".to_string(), lame));

    let cols = 3;
    let cell_w = 420;
    let cell_h = 300;
    let rows = shown.len().div_ceil(cols);
    let mut c = Canvas::new(cols * cell_w, rows * cell_h, BACKDROP);
    let scale = cell_w as f32 / 17.0 * zoom(sp);
    for (n, (name, beast)) in shown.iter().enumerate() {
        let (col, row) = (n % cols, n / cols);
        let x0 = col * cell_w;
        let y0 = row * cell_h;
        let shade = if (col + row) % 2 == 0 {
            PANEL
        } else {
            PANEL_ALT
        };
        c.rect(
            x0 as i32,
            y0 as i32,
            (x0 + cell_w) as i32,
            (y0 + cell_h) as i32,
            shade,
        );
        let ox = x0 as f32 + cell_w as f32 * 0.62;
        let oy = y0 as f32 + cell_h as f32 * 0.88;
        c.rect(
            x0 as i32,
            oy as i32,
            (x0 + cell_w) as i32,
            oy as i32 + 1,
            FLOOR,
        );
        let apex = oy - 4.14 * scale;
        for x in (x0..x0 + cell_w).step_by(4) {
            c.set(x as i32, apex as i32, JUMP);
        }
        let rig = beast.rig();
        let lens = Lens {
            view: View::Side,
            ox,
            oy,
            scale,
        };
        for part in 0..sp.parts.len() {
            draw_part(&mut c, &rig, part, lens, 1.0);
        }
        crate::sheet::digits(&mut c, x0 + 6, y0 + 6, n, LABEL);
        let _ = name;
    }
    c
}
