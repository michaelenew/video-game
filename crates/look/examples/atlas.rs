//! **The valley as one map, from above**: every place where `sim::atlas` put
//! it, on one sheet.
//!
//! ```text
//! cargo run -p look --example atlas      # target/atlas.ppm
//! ```
//!
//! A metre a pixel, looking straight down: the highest surface at each point
//! -- whichever place's floor is there, or the top of a box -- in its
//! material's colour as that place's palette has it, lit from the north-west
//! by the height so cliffs and shelves read as steps, and darker the lower it
//! is on the map so the climb reads left to right. Over it, **the doorways**
//! (gold, or grey behind a waystone) and each place's name. Where nothing is,
//! the void, is the background. What the per-place sheet
//! (`--example valley`) cannot show is how the places meet; this is that.

use look::sheet::{byte, text, write};
use look::{palette, skies};
use sim::arena::Material;
use sim::atlas::{self, EXTRA};

const PX: f32 = 1.0;
const PAD: usize = 8;

fn f(v: sim::Fx) -> f32 {
    v.to_f32_for_render()
}

fn fx(v: f32) -> sim::Fx {
    sim::Fx::from_raw((v * 65536.0) as i32)
}

fn main() {
    let a = atlas::valley();
    let (x0, x1) = (f(a.lo.0), f(a.hi.0));
    let (z0, z1) = (f(a.lo.1), f(a.hi.1));
    let w = ((x1 - x0) / PX) as usize + PAD * 2;
    let h = ((z1 - z0) / PX) as usize + PAD * 2;
    let mut px = vec![[20u8, 22, 26]; w * h];
    let palettes: Vec<palette::Palette> = a
        .places
        .iter()
        .map(|p| palette::Palette::under(&skies::of(p.arena).resolved()))
        .collect();
    // The top at a map point: its height, its material, and whose palette.
    let top = |x: f32, z: f32| -> Option<(f32, Material, usize)> {
        let place = a.place_at(fx(x), fx(z));
        let k = place.map(|p| a.index_of(p.arena).unwrap_or(0));
        let mut best = place.map(|p| {
            (
                f(a.relief_at(fx(x), fx(z))),
                a.floor_at(fx(x), fx(z)),
                k.unwrap_or(0),
                p,
            )
        });
        for i in a.near((fx(x), fx(z)), (fx(x), fx(z))) {
            let s = &a.solids[i as usize];
            if x > f(s.min.x)
                && x < f(s.max.x)
                && z > f(s.min.z)
                && z < f(s.max.z)
                && best.is_none_or(|b| f(s.max.y) > b.0)
            {
                let src = a.sources[i as usize].place;
                let k = if src == EXTRA {
                    k.unwrap_or(0)
                } else {
                    src as usize
                };
                best = Some((f(s.max.y), s.material, k, &a.places[k]));
            }
        }
        best.map(|(y, m, k, _)| (y, m, k))
    };
    let at = |i: usize, j: usize| (x0 + (i - PAD) as f32 * PX, z1 - (j - PAD) as f32 * PX);
    let mut heights = vec![0.0f32; w * h];
    for j in PAD..h - PAD {
        for i in PAD..w - PAD {
            let (x, z) = at(i, j);
            heights[j * w + i] = top(x, z).map_or(-100.0, |t| t.0);
        }
    }
    let (lo, hi) = heights
        .iter()
        .filter(|y| **y > -50.0)
        .fold((f32::MAX, f32::MIN), |(lo, hi), y| (lo.min(*y), hi.max(*y)));
    for j in PAD + 1..h - PAD {
        for i in PAD + 1..w - PAD {
            let (x, z) = at(i, j);
            let Some((y, m, k)) = top(x, z) else {
                continue;
            };
            let (left, up) = (heights[j * w + i - 1], heights[(j - 1) * w + i]);
            let slope = ((y - left) + (y - up)) * 0.06;
            let rise = (y - lo) / (hi - lo).max(1.0);
            let lift = 0.6 + 0.45 * rise + slope.clamp(-0.35, 0.35);
            let c = palette::lit(palettes[k].of(m));
            px[j * w + i] = byte([c[0] * lift, c[1] * lift, c[2] * lift]);
        }
    }
    let to_px = |x: f32, z: f32| {
        (
            ((x - x0) / PX) as i32 + PAD as i32,
            ((z1 - z) / PX) as i32 + PAD as i32,
        )
    };
    // The doorways: an outline round each cut.
    for d in &a.doors {
        let open = matches!(d.gate, sim::valley::Gate::Open);
        let c = if open {
            [230, 190, 70]
        } else {
            [150, 150, 160]
        };
        let (u0, v1) = to_px(f(d.cut.min.x), f(d.cut.min.z));
        let (u1, v0) = to_px(f(d.cut.max.x), f(d.cut.max.z));
        for u in u0..=u1 {
            for v in [v0, v1] {
                if u >= 0 && v >= 0 && (u as usize) < w && (v as usize) < h {
                    px[v as usize * w + u as usize] = c;
                }
            }
        }
        for v in v0..=v1 {
            for u in [u0, u1] {
                if u >= 0 && v >= 0 && (u as usize) < w && (v as usize) < h {
                    px[v as usize * w + u as usize] = c;
                }
            }
        }
    }
    for p in &a.places {
        let (u, v) = to_px(f(p.lo.0) + 2.0, f(p.hi.1) - 2.0);
        if u >= 0 && v >= 0 {
            text(&mut px, w, u as usize, v as usize, p.get().name);
        }
    }
    write("target/atlas.ppm", w, h, &px);
    let (tiles, entries) = a.tile_count();
    println!(
        "target/atlas.ppm: {} places, {} doorways, {} boxes in {tiles} tiles ({entries} entries, \
         at most {} in one)",
        a.places.len(),
        a.doors.len(),
        a.solids.len(),
        a.busiest_tile()
    );
}
