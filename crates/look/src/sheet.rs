//! Drawing a contact sheet: a picture, a label on it, and a file on disk.
//!
//! Shared by the examples, and in the library rather than beside them because
//! an example cannot import another example. It is thirty lines of pixel
//! pushing with no dependencies, which is the point: the harness that decides
//! whether the art is good must not be the thing that is hard to run.

/// One sRGB colour as bytes, clamped.
pub fn byte(c: [f32; 3]) -> [u8; 3] {
    [
        (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
    ]
}

/// Fill a rectangle, clipped to the image.
pub fn box_at(px: &mut [[u8; 3]], w: usize, at: (usize, usize), size: (usize, usize), c: [u8; 3]) {
    for y in at.1..at.1 + size.1 {
        for x in at.0..at.0 + size.0 {
            let i = y * w + x;
            if x < w && i < px.len() {
                px[i] = c;
            }
        }
    }
}

/// Write a PPM. Every image tool reads one and it needs no dependency;
/// `convert` turns it into a PNG if a PNG is wanted.
pub fn write(path: &str, w: usize, h: usize, px: &[[u8; 3]]) {
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
pub fn text(px: &mut [[u8; 3]], w: usize, x0: usize, y0: usize, s: &str) {
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
