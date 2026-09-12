//! Contact sheets: look at an animation without running the game.
//!
//! A baked clip is a table of numbers, and nobody can tell from a table whether
//! a walk cycle skates, whether a knee inverts on frame nine, or whether an
//! overhead's silhouette has changed enough by frame four to be reactable. This
//! draws the whole clip as a strip of stick figures -- side view, front view,
//! and a panel with every frame overlaid so the arcs of the hands and feet are
//! visible as arcs.
//!
//! ```text
//! cargo run -p anim --bin preview -- walk_forward
//! ```
//!
//! Three things it is specifically good at catching, all of which are invisible
//! in a table and obvious in a picture: **foot skate** (the ankle dots in the
//! trails panel bunch up where the foot is planted and spread where it swings --
//! a planted foot whose dots drift is sliding), **silhouette** (the side view
//! at gameplay distance is what an opponent actually reads), and **broken
//! joints** (a knee bent the wrong way looks wrong instantly).

use crate::png::Canvas;
use view::math::V3;
use view::pose::Pose;
use view::skeleton::{self, JOINTS, Joint, Skeleton};

const BACKDROP: [u8; 3] = [22, 24, 28];
const PANEL: [u8; 3] = [30, 33, 38];
const PANEL_ALT: [u8; 3] = [35, 39, 45];
const FLOOR: [u8; 3] = [70, 76, 86];
const INK: [u8; 3] = [232, 236, 242];
const TRUNK: [u8; 3] = [186, 194, 206];
const LEFT: [u8; 3] = [96, 200, 210];
const RIGHT: [u8; 3] = [236, 150, 86];
const LABEL: [u8; 3] = [120, 128, 140];

const CELL_W: usize = 108;
const CELL_H: usize = 148;
const COLS: usize = 12;
const TRAIL_H: usize = 240;

/// Draw a clip. Returns the canvas and the frame indices each column shows.
///
/// `travel` is how far the body moves per frame, in metres, for a clip the game
/// plays by distance. Pass zero for anything else. When it is non-zero the
/// overlay panel walks the character across the page instead of stacking every
/// frame on one spot -- and then a planted foot is a *vertical* pile of dots at
/// a fixed position while the body slides past it, which is the clearest
/// possible picture of whether the feet are sliding.
pub fn contact_sheet(skeleton: &Skeleton, frames: &[Pose], travel: f32) -> (Canvas, Vec<usize>) {
    let picks = sample_indices(frames.len(), COLS * 2);
    let rows = picks.len().div_ceil(COLS);

    let width = COLS * CELL_W;
    let side_top = 0;
    let front_top = side_top + rows * CELL_H;
    let trail_top = front_top + rows * CELL_H;
    let height = trail_top + TRAIL_H;

    let mut c = Canvas::new(width, height, BACKDROP);

    for (panel, top) in [(View::Side, side_top), (View::Front, front_top)] {
        for (n, frame) in picks.iter().enumerate() {
            let (col, row) = (n % COLS, n / COLS);
            let x = col * CELL_W;
            let y = top + row * CELL_H;
            let shade = if (col + row) % 2 == 0 {
                PANEL
            } else {
                PANEL_ALT
            };
            c.rect(
                x as i32,
                y as i32,
                (x + CELL_W) as i32,
                (y + CELL_H) as i32,
                shade,
            );
            let cell = Cell::new(x, y, CELL_W, CELL_H, panel);
            cell.floor(&mut c);
            draw(&mut c, &cell, skeleton, &frames[*frame], 1.0);
            digits(&mut c, x + 4, y + 4, *frame, LABEL);
        }
    }

    // Every frame at once, so the arcs are visible as arcs.
    c.rect(0, trail_top as i32, width as i32, height as i32, PANEL);
    let mut trail = Cell::new(0, trail_top, width, TRAIL_H, View::Side);
    // Three times round, when the clip travels. One cycle of a walk covers about
    // a metre, which is less than the character is tall -- three makes the
    // pattern of footfalls visible, and shows the loop closing twice.
    let cycles = if travel > 0.0 { 3 } else { 1 };
    let n = frames.len().max(1);
    let total = travel * (n * cycles) as f32;
    trail.cx -= total * 0.5 * trail.scale;
    trail.floor(&mut c);

    for i in 0..n * cycles {
        let pose = &frames[i % n];
        trail.shift = travel * i as f32;
        let t = (i % n) as f32 / (n.max(2) - 1) as f32;
        // Sparse ghosts when the figure is walking across the page: every frame
        // drawn on top of a moving body is mud rather than an arc.
        if travel == 0.0 || i % 3 == 0 {
            draw(&mut c, &trail, skeleton, pose, 0.10 + 0.16 * t);
        }
    }
    for i in 0..n * cycles {
        let pose = &frames[i % n];
        trail.shift = travel * i as f32;
        let t = (i % n) as f32 / (n.max(2) - 1) as f32;
        let skin = skeleton::solve(skeleton, pose);
        for (joint, colour) in [
            (Joint::FootL, LEFT),
            (Joint::FootR, RIGHT),
            (Joint::HandL, LEFT),
            (Joint::HandR, RIGHT),
        ] {
            let p = skin.tip(skeleton, joint);
            c.dot(trail.project(p), colour, 1.7, 0.35 + 0.65 * t);
        }
    }

    (c, picks)
}

#[derive(Clone, Copy, PartialEq)]
enum View {
    Side,
    Front,
}

struct Cell {
    cx: f32,
    floor_y: f32,
    scale: f32,
    view: View,
    x0: usize,
    x1: usize,
    /// Metres to slide this drawing along, for the overlay panel.
    shift: f32,
}

impl Cell {
    fn new(x: usize, y: usize, w: usize, h: usize, view: View) -> Cell {
        // Two and a bit metres of headroom, so a jump or an overhead does not
        // go off the top of its own cell.
        let scale = (h as f32 - 26.0) / 2.25;
        Cell {
            cx: x as f32 + w as f32 * 0.5,
            floor_y: (y + h) as f32 - 14.0,
            scale,
            view,
            x0: x,
            x1: x + w,
            shift: 0.0,
        }
    }

    fn project(&self, p: V3) -> (f32, f32) {
        let across = match self.view {
            // Side view looks along the character's left, so forward is right.
            View::Side => p[2],
            // Front view looks at the character, so its right hand is on ours.
            View::Front => -p[0],
        };
        (
            self.cx + (across + self.shift) * self.scale,
            self.floor_y - p[1] * self.scale,
        )
    }

    fn floor(&self, c: &mut Canvas) {
        for x in self.x0..self.x1 {
            c.blend(x as i32, self.floor_y as i32, FLOOR, 0.8);
        }
    }
}

fn draw(c: &mut Canvas, cell: &Cell, skeleton: &Skeleton, pose: &Pose, alpha: f32) {
    let skin = skeleton::solve(skeleton, pose);
    for j in JOINTS {
        let colour = match j {
            Joint::ArmL | Joint::ForearmL | Joint::HandL => LEFT,
            Joint::ThighL | Joint::ShinL | Joint::FootL => LEFT,
            Joint::ArmR | Joint::ForearmR | Joint::HandR => RIGHT,
            Joint::ThighR | Joint::ShinR | Joint::FootR => RIGHT,
            Joint::Head => INK,
            _ => TRUNK,
        };
        let a = cell.project(skin.origin[j.index()]);
        let b = cell.project(skin.tip(skeleton, j));
        if j == Joint::Head {
            // Drawn as a head rather than as a bone, because a line from the
            // neck to the crown reads as a long white tube and makes every
            // pose look like it is craning.
            let mid = ((a.0 + b.0) * 0.5, (a.1 + b.1) * 0.5);
            let r = 0.115 * skeleton.scale * cell.scale;
            if alpha >= 0.99 {
                c.dot(mid, INK, r, 1.0);
            } else {
                c.dot(mid, INK, r * 0.5, alpha);
            }
            continue;
        }
        let width = match j {
            Joint::Root | Joint::Spine | Joint::Chest => 5.0,
            Joint::Head => 7.0,
            Joint::HandL | Joint::HandR | Joint::FootL | Joint::FootR => 3.5,
            _ => 3.0,
        };
        if alpha >= 0.99 {
            c.line(a, b, colour, width);
        } else {
            // Ghosts are drawn as a thin trace, or the overlay turns to mud.
            let steps = 8;
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let p = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
                c.dot(p, colour, width * 0.3, alpha);
            }
        }
    }
    if alpha >= 0.99 {
        // A dot on each joint reads as an articulation rather than a crease.
        for j in JOINTS {
            c.dot(cell.project(skin.origin[j.index()]), INK, 1.4, 0.55);
        }
    }
}

/// Up to `most` frames, evenly spread, always including the first and last.
fn sample_indices(len: usize, most: usize) -> Vec<usize> {
    if len == 0 {
        return vec![];
    }
    if len <= most {
        return (0..len).collect();
    }
    (0..most)
        .map(|i| i * (len - 1) / (most - 1))
        .collect::<Vec<_>>()
}

// ---------------------------------------------------------------------------
// A three-pixel-wide font, for frame numbers
// ---------------------------------------------------------------------------

const GLYPHS: [[u8; 5]; 10] = [
    [0b111, 0b101, 0b101, 0b101, 0b111],
    [0b010, 0b110, 0b010, 0b010, 0b111],
    [0b111, 0b001, 0b111, 0b100, 0b111],
    [0b111, 0b001, 0b111, 0b001, 0b111],
    [0b101, 0b101, 0b111, 0b001, 0b001],
    [0b111, 0b100, 0b111, 0b001, 0b111],
    [0b111, 0b100, 0b111, 0b101, 0b111],
    [0b111, 0b001, 0b001, 0b001, 0b001],
    [0b111, 0b101, 0b111, 0b101, 0b111],
    [0b111, 0b101, 0b111, 0b001, 0b111],
];

/// The frame number, at double size so it is legible next to the figure.
fn digits(c: &mut Canvas, x: usize, y: usize, value: usize, colour: [u8; 3]) {
    let text = value.to_string();
    for (n, ch) in text.bytes().enumerate() {
        let glyph = GLYPHS[(ch - b'0') as usize];
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..3 {
                if bits & (0b100 >> col) != 0 {
                    let px = (x + n * 8 + col * 2) as i32;
                    let py = (y + row * 2) as i32;
                    c.rect(px, py, px + 2, py + 2, colour);
                }
            }
        }
    }
}
