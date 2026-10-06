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

fn byte(c: [f32; 3]) -> [u8; 3] {
    [
        (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
    ]
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

fn write(path: &str, w: usize, h: usize, px: &[[u8; 3]]) {
    let mut out = Vec::with_capacity(px.len() * 3 + 32);
    out.extend_from_slice(format!("P6\n{w} {h}\n255\n").as_bytes());
    for c in px {
        out.extend_from_slice(c);
    }
    std::fs::write(path, out).expect("writing the sheet");
    println!("{path}  ({w}x{h})");
}

/// A 3x5 bitmap font, doubled, so a cell can say which arena it is.
///
/// Hand-rolled because the alternative is a font crate and a font file in a
/// crate whose whole point is that it has no dependencies. Three pixels wide is
/// the narrowest a legible letter gets, and at 2x it reads fine.
fn text(px: &mut [[u8; 3]], w: usize, x0: usize, y0: usize, s: &str) {
    const SCALE: usize = 2;
    for (i, ch) in s.chars().enumerate() {
        let rows = glyph(ch);
        let left = x0 + i * 4 * SCALE;
        for (r, bits) in rows.iter().enumerate() {
            for c in 0..3 {
                if bits & (0b100 >> c) == 0 {
                    continue;
                }
                for dy in 0..SCALE {
                    for dx in 0..SCALE {
                        let (x, y) = (left + c * SCALE + dx, y0 + r * SCALE + dy);
                        let at = y * w + x;
                        if x < w && at < px.len() {
                            px[at] = [235, 235, 240];
                        }
                    }
                }
            }
        }
    }
}

fn glyph(ch: char) -> [u8; 5] {
    match ch.to_ascii_uppercase() {
        'A' => [0b111, 0b101, 0b111, 0b101, 0b101],
        'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
        'C' => [0b111, 0b100, 0b100, 0b100, 0b111],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'E' => [0b111, 0b100, 0b111, 0b100, 0b111],
        'F' => [0b111, 0b100, 0b111, 0b100, 0b100],
        'G' => [0b111, 0b100, 0b101, 0b101, 0b111],
        'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'J' => [0b001, 0b001, 0b001, 0b101, 0b111],
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        'N' => [0b110, 0b101, 0b101, 0b101, 0b101],
        'O' => [0b111, 0b101, 0b101, 0b101, 0b111],
        'P' => [0b111, 0b101, 0b111, 0b100, 0b100],
        'Q' => [0b111, 0b101, 0b101, 0b111, 0b001],
        'R' => [0b111, 0b101, 0b111, 0b110, 0b101],
        'S' => [0b111, 0b100, 0b111, 0b001, 0b111],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
        'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
        'W' => [0b101, 0b101, 0b111, 0b111, 0b101],
        'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
        'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
        'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b001, 0b001, 0b001],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b001],
        '-' => [0b000, 0b000, 0b111, 0b000, 0b000],
        '.' => [0b000, 0b000, 0b000, 0b000, 0b010],
        _ => [0; 5],
    }
}
