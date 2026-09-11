//! A PNG writer, in about a hundred lines and with no dependencies.
//!
//! It exists so that an animation can be **looked at** without running the
//! game. Judging motion from a table of numbers is not possible, and building
//! the whole Bevy app to see whether a walk cycle skates is a minute a change
//! -- which in practice means the change does not get made. A contact sheet
//! renders in milliseconds and can be opened by anything.
//!
//! Compression is deflate's *stored* mode: no compression at all, just framing.
//! A real deflate implementation would be several hundred lines to save a few
//! hundred kilobytes on files nobody checks in.

/// An RGB canvas.
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pixels: Vec<u8>,
}

impl Canvas {
    pub fn new(width: usize, height: usize, fill: [u8; 3]) -> Canvas {
        let mut pixels = Vec::with_capacity(width * height * 3);
        for _ in 0..width * height {
            pixels.extend_from_slice(&fill);
        }
        Canvas {
            width,
            height,
            pixels,
        }
    }

    pub fn set(&mut self, x: i32, y: i32, colour: [u8; 3]) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        let i = (y as usize * self.width + x as usize) * 3;
        self.pixels[i..i + 3].copy_from_slice(&colour);
    }

    /// Blend a colour in, for anti-aliasing and for ghost trails.
    pub fn blend(&mut self, x: i32, y: i32, colour: [u8; 3], alpha: f32) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        let a = alpha.clamp(0.0, 1.0);
        let i = (y as usize * self.width + x as usize) * 3;
        for c in 0..3 {
            let old = self.pixels[i + c] as f32;
            self.pixels[i + c] = (old + (colour[c] as f32 - old) * a).round() as u8;
        }
    }

    pub fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, colour: [u8; 3]) {
        for y in y0..y1 {
            for x in x0..x1 {
                self.set(x, y, colour);
            }
        }
    }

    /// A line with a bit of thickness, drawn by walking the long axis.
    pub fn line(&mut self, a: (f32, f32), b: (f32, f32), colour: [u8; 3], width: f32) {
        let steps = ((b.0 - a.0).abs().max((b.1 - a.1).abs()) * 2.0)
            .ceil()
            .max(1.0) as i32;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = a.0 + (b.0 - a.0) * t;
            let y = a.1 + (b.1 - a.1) * t;
            self.dot((x, y), colour, width * 0.5, 1.0);
        }
    }

    /// A soft round dot, which is what keeps a stick figure from looking like a
    /// staircase at this size.
    pub fn dot(&mut self, at: (f32, f32), colour: [u8; 3], radius: f32, alpha: f32) {
        let r = radius.max(0.5);
        let x0 = (at.0 - r - 1.0).floor() as i32;
        let x1 = (at.0 + r + 1.0).ceil() as i32;
        let y0 = (at.1 - r - 1.0).floor() as i32;
        let y1 = (at.1 + r + 1.0).ceil() as i32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = x as f32 + 0.5 - at.0;
                let dy = y as f32 + 0.5 - at.1;
                let d = (dx * dx + dy * dy).sqrt();
                let cover = (r + 0.5 - d).clamp(0.0, 1.0);
                if cover > 0.0 {
                    self.blend(x, y, colour, cover * alpha);
                }
            }
        }
    }

    pub fn write(&self, path: &std::path::Path) -> std::io::Result<()> {
        std::fs::write(path, self.encode())
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut raw = Vec::with_capacity(self.height * (1 + self.width * 3));
        for y in 0..self.height {
            raw.push(0); // filter: none
            let row = y * self.width * 3;
            raw.extend_from_slice(&self.pixels[row..row + self.width * 3]);
        }

        let mut out = Vec::new();
        out.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]);

        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&(self.width as u32).to_be_bytes());
        ihdr.extend_from_slice(&(self.height as u32).to_be_bytes());
        ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8-bit, truecolour
        chunk(&mut out, b"IHDR", &ihdr);
        chunk(&mut out, b"IDAT", &zlib_stored(&raw));
        chunk(&mut out, b"IEND", &[]);
        out
    }
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut crc = Crc::new();
    crc.update(kind);
    crc.update(data);
    out.extend_from_slice(&crc.finish().to_be_bytes());
}

/// A zlib stream made entirely of stored deflate blocks.
fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut rest = data;
    loop {
        let take = rest.len().min(65535);
        let last = take == rest.len();
        out.push(if last { 1 } else { 0 });
        out.extend_from_slice(&(take as u16).to_le_bytes());
        out.extend_from_slice(&(!(take as u16)).to_le_bytes());
        out.extend_from_slice(&rest[..take]);
        rest = &rest[take..];
        if last {
            break;
        }
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for byte in data {
        a = (a + *byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

struct Crc(u32);

impl Crc {
    fn new() -> Crc {
        Crc(0xffff_ffff)
    }

    fn update(&mut self, data: &[u8]) {
        for byte in data {
            let mut c = (self.0 ^ *byte as u32) & 0xff;
            for _ in 0..8 {
                c = if c & 1 != 0 {
                    0xedb8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
            }
            self.0 = c ^ (self.0 >> 8);
        }
    }

    fn finish(self) -> u32 {
        self.0 ^ 0xffff_ffff
    }
}
