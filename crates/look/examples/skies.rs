//! Every arena's sky, side by side, without starting the game.
//!
//! ```text
//! cargo run -p look --example skies            # the contact sheet
//! cargo run -p look --example skies -- gulf    # one arena, large
//! ```
//!
//! **This is the loop, not a toy.** There will not be a developer for this
//! game forever, so what matters is how quickly somebody can tell whether a
//! sky is right and change it. Judging one in the game costs a minute --
//! launch, pick the arena, walk somewhere the sky is visible, look up, look
//! down -- and what you are judging is a gradient, which does not need a game
//! to exist. Here all of them are one command and one picture, which is also
//! the only way to see the thing that actually matters: whether they look like
//! one world.
//!
//! Each cell is a pinhole camera at eye level, horizon a little above centre,
//! with the sun off to the left so the glow is in frame. Along the bottom are
//! **five grey blocks at a fifth, two fifths, ... of that sky's reach**, each
//! hazed by exactly the fog the game would apply. That strip is the altitude
//! cue made visible: if the far blocks are still crisp, nothing in that arena
//! will read as far away, and a course sixty metres up will look like a course
//! on the ground.
//!
//! Output is a PPM, which every image tool reads and which needs no
//! dependency; `scripts/screenshot.sh` already shells out to ImageMagick if a
//! PNG is wanted.

use look::sheet::{byte, text, write};
use look::{Sky, skies, tint};

/// One cell of the sheet.
const CELL_W: usize = 200;
const CELL_H: usize = 150;
/// The label strip under each cell.
const LABEL_H: usize = 14;
const COLS: usize = 5;

/// Where the sun is, for the sheet. Low and to the left of the view, so every
/// glow is in frame and comparable. The game's own sun angle is per arena; this
/// is a fixed light for a fair comparison, the way a photographer shoots a
/// colour chart under one lamp.
fn sun() -> [f32; 3] {
    dir_of(-40.0_f32.to_radians(), 18.0_f32.to_radians())
}

/// A unit vector from a yaw (left-right, 0 straight ahead) and a pitch.
fn dir_of(yaw: f32, pitch: f32) -> [f32; 3] {
    let (cp, sp) = (pitch.cos(), pitch.sin());
    [yaw.sin() * cp, sp, yaw.cos() * cp]
}

/// Draw one sky into an `w` by `h` image: a camera view, plus the haze ruler.
fn view(sky: &Sky, w: usize, h: usize) -> Vec<[u8; 3]> {
    // A 62 degree vertical field, horizon at 40% down, so there is more sky
    // than ground in frame but enough ground to see the haze below.
    let fov = 62.0_f32.to_radians();
    let half = (fov * 0.5).tan();
    let aspect = w as f32 / h as f32;
    let horizon_at = 0.40;
    let sun = sun();
    let sky = sky.resolved();

    let mut px = vec![[0u8; 3]; w * h];
    for y in 0..h {
        // Signed screen height, with the horizon where we put it rather than
        // at the centre.
        let sy = (horizon_at - (y as f32 + 0.5) / h as f32) * 2.0 * half;
        for x in 0..w {
            let sx = ((x as f32 + 0.5) / w as f32 - 0.5) * 2.0 * half * aspect;
            let len = (1.0 + sx * sx + sy * sy).sqrt();
            let dir = [sx / len, sy / len, 1.0 / len];
            px[y * w + x] = byte(sky.looking(dir, sun));
        }
    }

    // The haze ruler: five blocks at a fifth of the reach apiece, each faded by
    // the fog the game would put on something that far away.
    let (start, end) = sky.fog();
    let pad = w / 25;
    let block = (w - pad * 6) / 5;
    let size = block.min(h / 7);
    let top = h - size - pad;
    for i in 0..5 {
        let away = end * (i as f32 + 1.0) / 5.0;
        let f = ((away - start) / (end - start)).clamp(0.0, 1.0);
        // Fog blends in linear light, like the renderer's does.
        let (g, hz) = (tint::linear([0.45, 0.45, 0.45]), tint::linear(sky.horizon));
        let c = byte(tint::srgb([
            g[0] + (hz[0] - g[0]) * f,
            g[1] + (hz[1] - g[1]) * f,
            g[2] + (hz[2] - g[2]) * f,
        ]));
        let left = pad + i * (block + pad);
        for y in top..(top + size).min(h) {
            for x in left..(left + size).min(w) {
                px[y * w + x] = c;
            }
        }
    }
    px
}

fn main() {
    let arg = std::env::args().nth(1);
    let arenas: Vec<_> = sim::arena::all().collect();

    if let Some(name) = arg {
        let Some(arena) = sim::arena::named(&name) else {
            eprintln!("no arena called {name:?}. One of:");
            for a in &arenas {
                eprintln!("  {}", a.slug());
            }
            std::process::exit(1);
        };
        let (w, h) = (800, 600);
        let px = view(&skies::of(arena.id), w, h);
        write(&format!("target/sky-{}.ppm", arena.slug()), w, h, &px);
        return;
    }

    let rows = arenas.len().div_ceil(COLS);
    let (w, h) = (COLS * CELL_W, rows * (CELL_H + LABEL_H));
    let mut px = vec![[24u8, 24, 28]; w * h];
    for (i, arena) in arenas.iter().enumerate() {
        let (col, row) = (i % COLS, i / COLS);
        let (ox, oy) = (col * CELL_W, row * (CELL_H + LABEL_H));
        // A hair of margin, so the cells read as separate pictures rather than
        // as one continuous and very confusing panorama.
        let (cw, ch) = (CELL_W - 4, CELL_H - 4);
        let sky = skies::of(arena.id);
        let cell = view(&sky, cw, ch);
        for y in 0..ch {
            for x in 0..cw {
                px[(oy + y + 2) * w + ox + x + 2] = cell[y * cw + x];
            }
        }
        let label = format!("{} {}M", arena.name.to_uppercase(), sky.reach as i32);
        text(&mut px, w, ox + 4, oy + CELL_H + 1, &label);
    }
    write("target/skies.ppm", w, h, &px);
}
