//! **The valley from above**: every place on one sheet, as a plan.
//!
//! ```text
//! cargo run -p look --example valley      # target/valley.ppm
//! ```
//!
//! Each place is drawn at half a metre a pixel, looking straight down: the
//! highest surface at each point -- the floor's relief, or the top of a solid
//! -- in its material's colour as the arena's palette has it, lit from the
//! north-west by the height so cliffs and shelves read as steps. Over it,
//! **the ways out** (a box round each seam: gold open, grey behind a
//! waystone), **waystones** (yellow), **cairns** (white), **vines** (green
//! bars) and **updrafts** (pale rings). A place is judged in seconds here,
//! where in the game it is a walk; what this cannot show is how a jump feels,
//! which is what the game is for.

use look::sheet::{byte, text, write};
use look::{palette, skies, tint};
use sim::arena::{Arena, Solid};
use sim::valley::{self, Kind};

const PX: f32 = 0.5;
const PAD: usize = 8;
const LABEL: usize = 22;

fn f(v: sim::Fx) -> f32 {
    v.to_f32_for_render()
}

/// The top surface at a floor point: its height and what it is made of.
fn top_at(a: &Arena, x: f32, z: f32) -> (f32, sim::arena::Material) {
    let fx = |v: f32| sim::Fx::from_raw((v * 65536.0) as i32);
    let mut best = (f(a.relief_at(fx(x), fx(z))), a.floor_at(fx(x), fx(z)));
    for s in a.solids() {
        if x > f(s.min.x)
            && x < f(s.max.x)
            && z > f(s.min.z)
            && z < f(s.max.z)
            && f(s.max.y) > best.0
        {
            best = (f(s.max.y), s.material);
        }
    }
    best
}

fn draw(a: &'static Arena) -> (usize, usize, Vec<[u8; 3]>) {
    let b = a.bounds;
    let (x0, x1, z0, z1) = (
        f(b.lo_x) - 4.0,
        f(b.hi_x) + 4.0,
        f(b.lo_z) - 4.0,
        f(b.hi_z) + 4.0,
    );
    let w = ((x1 - x0) / PX) as usize + PAD * 2;
    let h = ((z1 - z0) / PX) as usize + PAD * 2 + LABEL;
    let mut px = vec![[24u8, 26, 30]; w * h];
    let sky = skies::of(a.id).resolved();
    let pal = palette::Palette::under(&sky);
    // Plan: +x to the right, +z up the page.
    let to_px = |x: f32, z: f32| -> (i32, i32) {
        (
            PAD as i32 + ((x - x0) / PX) as i32,
            (LABEL + PAD) as i32 + ((z1 - z) / PX) as i32,
        )
    };
    let heights: Vec<f32> = (0..(w * h))
        .map(|i| {
            let (i, j) = ((i % w) as f32, (i / w) as f32);
            let x = x0 + (i - PAD as f32) * PX;
            let z = z1 - (j - (LABEL + PAD) as f32) * PX;
            top_at(a, x, z).0
        })
        .collect();
    for j in (LABEL + PAD)..(h - PAD) {
        for i in PAD..(w - PAD) {
            let x = x0 + (i - PAD) as f32 * PX;
            let z = z1 - (j - LABEL - PAD) as f32 * PX;
            let (y, m) = top_at(a, x, z);
            let left = heights[j * w + i - 1];
            let up = heights[(j - 1) * w + i];
            // Lit from the north-west: a step down to the south-east is bright.
            let slope = ((y - left) + (y - up)) * 0.08;
            let lift = 0.82 + (y * 0.006).min(0.25) + slope.clamp(-0.35, 0.35);
            let c = palette::lit(pal.of(m));
            px[j * w + i] = byte([c[0] * lift, c[1] * lift, c[2] * lift]);
        }
    }
    let dot = |px: &mut Vec<[u8; 3]>, x: f32, z: f32, r: i32, c: [u8; 3]| {
        let (cx, cy) = to_px(x, z);
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    let (u, v) = (cx + dx, cy + dy);
                    if u >= 0 && v >= 0 && (u as usize) < w && (v as usize) < h {
                        px[v as usize * w + u as usize] = c;
                    }
                }
            }
        }
    };
    let rect = |px: &mut Vec<[u8; 3]>, lo: (f32, f32), hi: (f32, f32), c: [u8; 3], fill: bool| {
        let (a0, b1) = to_px(lo.0, lo.1);
        let (a1, b0) = to_px(hi.0, hi.1);
        for v in b0.max(0)..=b1.min(h as i32 - 1) {
            for u in a0.max(0)..=a1.min(w as i32 - 1) {
                let edge = u == a0 || u == a1 || v == b0 || v == b1;
                if fill || edge {
                    px[v as usize * w + u as usize] = c;
                }
            }
        }
    };
    if let Some(place) = valley::place(a.id) {
        for (_, s) in place.cairns() {
            dot(
                &mut px,
                (f(s.min.x) + f(s.max.x)) * 0.5,
                (f(s.min.z) + f(s.max.z)) * 0.5,
                3,
                [250, 250, 250],
            );
        }
        for v in place.vines {
            rect(
                &mut px,
                (f(v.min.x), f(v.min.z)),
                (f(v.max.x), f(v.max.z)),
                [60, 200, 90],
                true,
            );
        }
        for v in place.vents {
            let r = (f(v.radius) / PX) as i32;
            dot(&mut px, f(v.at.0), f(v.at.1), r, [200, 225, 245]);
        }
        for seam in place.seams {
            let open = matches!(seam.gate, valley::Gate::Open);
            let c = if open {
                [245, 200, 90]
            } else {
                [150, 160, 175]
            };
            let z = seam.zone;
            rect(
                &mut px,
                (f(z.min.x), f(z.min.z)),
                (f(z.max.x), f(z.max.z)),
                c,
                false,
            );
            if let Some(k) = seam.waystone {
                let s: Solid = a.solids()[k as usize];
                dot(
                    &mut px,
                    (f(s.min.x) + f(s.max.x)) * 0.5,
                    (f(s.min.z) + f(s.max.z)) * 0.5,
                    3,
                    [250, 225, 60],
                );
            }
        }
        let kind = match place.kind {
            Kind::Town => "town",
            Kind::Ring => "ring",
            Kind::Reach => "reach",
            Kind::Room(_) => "room",
        };
        text(
            &mut px,
            w,
            PAD,
            4,
            &format!("{} ({kind})", a.name.to_uppercase()),
        );
    }
    let _ = tint::linear;
    (w, h, px)
}

fn main() {
    let places: Vec<&'static Arena> = valley::all()
        .filter(|p| !matches!(p.kind, Kind::Room(_)))
        .map(|p| p.arena.get())
        .collect();
    let drawn: Vec<(usize, usize, Vec<[u8; 3]>)> = places.iter().map(|a| draw(a)).collect();
    // One column, each place under the last, as the valley runs.
    let w = drawn.iter().map(|d| d.0).max().unwrap_or(1);
    let h: usize = drawn.iter().map(|d| d.1).sum();
    let mut px = vec![[18u8, 19, 22]; w * h];
    let mut top = 0;
    for (dw, dh, d) in &drawn {
        for j in 0..*dh {
            px[(top + j) * w..(top + j) * w + dw].copy_from_slice(&d[j * dw..(j + 1) * dw]);
        }
        top += dh;
    }
    write("target/valley.ppm", w, h, &px);
    println!("target/valley.ppm  {w} x {h}: {} places", places.len());
}
