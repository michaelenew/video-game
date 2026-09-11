//! `cargo run -p art --bin sheet` — every material, baked to a PNG you can look at.
//!
//! The numbers in the test suite say whether stone is anisotropic and whether
//! the palette separates. They do not say whether stone looks like stone, and
//! the gap between those two is exactly where procedural materials go wrong.
//! So there is a sheet, and looking at it is part of the loop.
//!
//! Writes `art-sheet.png` in the working directory, or wherever the first
//! argument points.

use art::bake;
use art::color::Lch;
use art::materials::{self, Material};
use art::palette;

/// One row per material, one column per map. Seeing albedo, normal, roughness
/// and emission side by side is what makes a wrong one obvious -- a flat
/// normal map next to a lively albedo says the relief is zero, which is not
/// visible in the albedo alone.
const CELL: u32 = materials::BAKE_SIZE;
const PAD: u32 = 6;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "art-sheet.png".into());

    let tint = Lch::new(0.62, 0.13, palette::PLAYERS[0].h);
    let mut rows: Vec<Material> = materials::FIXED.to_vec();
    rows.push(materials::cloth(tint));
    rows.push(materials::armour(tint));
    rows.push(materials::arcane(palette::PLAYERS[3]));

    let cols = 4;
    let width = cols * CELL + (cols + 1) * PAD;
    let height = rows.len() as u32 * (CELL + PAD) + PAD;
    let mut canvas = vec![24u8; (width * height * 4) as usize];
    for px in canvas.chunks_mut(4) {
        px[3] = 255;
    }

    for (r, m) in rows.iter().enumerate() {
        // Each material knows how it wants to be baked -- tiled or not, and
        // along which axes. A stone previewed flat across X and Z would never
        // cross a single one of its strata and would look like a bug in the
        // material rather than a bug in the preview.
        let maps = bake::bake(&m.surface, m.plan(CELL));
        let strip = [
            &maps.albedo,
            &maps.normal,
            &maps.metallic_roughness,
            &maps.emissive,
        ];
        for (c, tex) in strip.iter().enumerate() {
            let x0 = PAD + c as u32 * (CELL + PAD);
            let y0 = PAD + r as u32 * (CELL + PAD);
            for y in 0..CELL {
                for x in 0..CELL {
                    let src = ((y * tex.width + x) * 4) as usize;
                    let dst = (((y0 + y) * width + x0 + x) * 4) as usize;
                    canvas[dst..dst + 4].copy_from_slice(&tex.pixels[src..src + 4]);
                }
            }
        }
        println!(
            "{:>8}  {:?}  {:.1} m tile  {:.1} texels/cycle  emission x{:.2}",
            m.name,
            m.role,
            m.extent,
            m.texels_per_cycle(CELL),
            maps.emissive_strength
        );
    }

    let png = art::png::encode(width, height, &canvas);
    std::fs::write(&path, &png).expect("could not write the sheet");
    println!("\nalbedo / normal / roughness+metallic / emission");
    println!("wrote {path} ({} KB)", png.len() / 1024);
}
