//! `cargo run --release -p art --bin quarry` -- cut the rocks open and look.
//!
//! Every rock in the library, sliced three ways, so the volume can be inspected
//! as a volume. **Three cuts through the same stone, at right angles**, because
//! that is the one view that shows whether it is really three-dimensional: if
//! the three faces of a corner do not agree, it is three pictures rather than a
//! solid, and no amount of looking at one face will tell you.
//!
//! Also cuts two *parallel* slices a little apart, which is the check for the
//! other failure: a pattern that is the same on every parallel cut is a
//! two-dimensional field extruded, not a rock.
//!
//! Writes `quarry.png`, or wherever the first argument points.

use art::stone::{Stone, library};

const CELL: u32 = 220;
const PAD: u32 = 5;
/// How much rock each slice covers, in metres. About the size of a face you
/// would actually stand in front of.
const SPAN: f32 = 1.6;

/// Columns: three orthogonal cuts, a second cut parallel to the first, and a
/// close-up so the grain can be seen at a scale that actually resolves it.
const COLS: u32 = 5;

/// A plane through the volume: two texture coordinates to a point in space.
type Cut = fn(f32, f32) -> [f32; 3];

/// How much rock the close-up covers. Small enough that individual crystals are
/// several pixels across, which is the only honest way to look at them.
const CLOSE: f32 = 0.13;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "quarry.png".into());
    let rocks = library();

    let width = COLS * CELL + (COLS + 1) * PAD;
    let height = rocks.len() as u32 * (CELL + PAD) + PAD;
    let mut canvas = vec![18u8; (width * height * 4) as usize];
    for px in canvas.chunks_mut(4) {
        px[3] = 255;
    }

    for (r, (name, stone)) in rocks.iter().enumerate() {
        let cuts: [(Cut, f32); 5] = [
            (|u, v| [u, v, 0.0], SPAN),
            (|u, v| [u, 0.0, v], SPAN),
            (|u, v| [0.0, u, v], SPAN),
            (|u, v| [u, v, 0.3], SPAN),
            (|u, v| [u, v, 0.0], CLOSE),
        ];
        for (c, (cut, span)) in cuts.iter().enumerate() {
            draw(&mut canvas, width, c as u32, r as u32, stone, *cut, *span);
        }
        println!(
            "{name:>10}  grain {:.0} mm, {:.1} px across at the wide cut",
            stone.grain * 1000.0,
            stone.grain / (SPAN / CELL as f32)
        );
    }

    let png = art::png::encode(width, height, &canvas);
    std::fs::write(&path, &png).expect("could not write the slices");
    println!(
        "\ncolumns: three cuts at right angles, a fourth parallel to the first, then a close-up"
    );
    println!("wrote {path} ({} KB)", png.len() / 1024);
}

fn draw(canvas: &mut [u8], width: u32, col: u32, row: u32, stone: &Stone, cut: Cut, span: f32) {
    let x0 = PAD + col * (CELL + PAD);
    let y0 = PAD + row * (CELL + PAD);
    for y in 0..CELL {
        for x in 0..CELL {
            let u = (x as f32 / CELL as f32 - 0.5) * span;
            let v = (0.5 - y as f32 / CELL as f32) * span;
            // Told how big a pixel is, so grain finer than one fades into the
            // rock's own mean instead of fizzing. See `Stone::at_scale`.
            let s = stone.at_scale(cut(u, v), span / CELL as f32);
            let dst = (((y0 + y) * width + x0 + x) * 4) as usize;
            canvas[dst..dst + 4].copy_from_slice(&[
                byte(s.colour[0]),
                byte(s.colour[1]),
                byte(s.colour[2]),
                255,
            ]);
        }
    }
}

fn byte(linear: f32) -> u8 {
    (art::color::linear_to_srgb(linear.clamp(0.0, 1.0)) * 255.0 + 0.5) as u8
}
