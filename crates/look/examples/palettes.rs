//! Every arena's palette, side by side, without starting the game.
//!
//! ```text
//! cargo run -p look --example palettes
//! ```
//!
//! One row per arena: the **light** it is under, its **accent**, and then its
//! ten surfaces, each shown twice -- plain, and with the accent at the strength
//! an edge gets it. The last column is the falloff itself, across the whole
//! distance the accent reaches. The surfaces are drawn **lit**
//! (`palette::lit`), not as raw albedo, so what is on the sheet is what will be
//! on the screen; judging albedo is how the first version of this palette came
//! out as a white wash in the game while looking fine here. The second half of each swatch is the
//! whole technique in one strip: unrelated surfaces, all touched by one bright
//! colour that describes none of them, and therefore all reading as one place.
//!
//! What to look for, in order:
//!
//! - **Mud.** Anything noticeably darker than its neighbours is a hole in the
//!   picture. The band in `look::palette` exists to stop this and the test
//!   `nothing_anywhere_comes_out_as_mud_or_as_neon` pins it.
//! - **Sameness.** Ten swatches that have become one colour means the pull
//!   toward the light has gone too far, and a player can no longer read the
//!   floor at a glance.
//! - **A beige accent.** If the accent chip looks like more of the light, the
//!   edges in that arena will do nothing at all.

use look::edge::EDGE;
use look::palette::{self, lit, local};
use look::sheet::{box_at, byte, text, write};
use sim::arena::Material;

const EVERY: [Material; 10] = [
    Material::Grass,
    Material::Ground,
    Material::Rock,
    Material::Stone,
    Material::Sand,
    Material::Peat,
    Material::Ash,
    Material::Snow,
    Material::Water,
    Material::Wood,
];

/// How wide the falloff strip is drawn, in pixels. It spans `EDGE.reach`
/// metres, so one strip is the whole gradient a player will see on an edge.
const FALLOFF: usize = 44;

const LABEL: usize = 104;
const CHIP: usize = 30;
const ROW: usize = 34;

fn main() {
    let arenas: Vec<_> = sim::arena::all().collect();
    // Label, light, accent, a gap, then the ten surfaces in two halves each.
    let w = LABEL + CHIP * 2 + 10 + EVERY.len() * (CHIP + 2) + FALLOFF + 10;
    let h = ROW * arenas.len() + 18;
    let mut px = vec![[20u8, 20, 24]; w * h];

    text(&mut px, w, 6, 5, "ARENA");
    text(&mut px, w, LABEL, 5, "LIGHT ACCENT");
    text(
        &mut px,
        w,
        LABEL + CHIP * 2 + 14,
        5,
        "SURFACES  PLAIN / ACCENTED",
    );

    for (row, arena) in arenas.iter().enumerate() {
        let p = palette::of(arena.id);
        let y = 18 + row * ROW;
        text(&mut px, w, 6, y + 10, &short(arena.name));
        box_at(&mut px, w, (LABEL, y), (CHIP, CHIP), byte(p.light));
        box_at(&mut px, w, (LABEL + CHIP, y), (CHIP, CHIP), byte(p.accent));
        for (i, m) in EVERY.iter().enumerate() {
            let x = LABEL + CHIP * 2 + 10 + i * (CHIP + 2);
            // Lit, because albedo is not what anybody sees.
            let c = p.of(*m);
            box_at(&mut px, w, (x, y), (CHIP / 2, CHIP), byte(lit(c)));
            box_at(
                &mut px,
                w,
                (x + CHIP / 2, y),
                (CHIP / 2, CHIP),
                byte(lit(p.accented(c, EDGE.rim))),
            );
        }
        // The gradient itself: a strip across `EDGE.reach` metres of a top
        // face, from the edge inward. What to look for is a band with a
        // visible middle it does not reach -- an edge that covers the whole
        // strip is a surface colour, which is how the first version of this
        // rule came out as *everything is purple*.
        let ground = p.of(Material::Ground);
        for i in 0..FALLOFF {
            let metres = EDGE.reach * i as f32 / (FALLOFF - 1) as f32;
            let c = p.accented(ground, EDGE.at(metres, 1.0, 2.0));
            box_at(
                &mut px,
                w,
                (w - FALLOFF - 4 + i, y),
                (1, CHIP),
                byte(lit(c)),
            );
        }
    }
    write("target/palettes.ppm", w, h, &px);

    // And the ten materials before any arena has had an opinion, so a change to
    // `local` can be judged on its own.
    let w2 = EVERY.len() * (CHIP + 2) + 8;
    let mut raw = vec![[20u8, 20, 24]; w2 * (CHIP + 8)];
    for (i, m) in EVERY.iter().enumerate() {
        box_at(
            &mut raw,
            w2,
            (4 + i * (CHIP + 2), 4),
            (CHIP, CHIP),
            byte(lit(local(*m))),
        );
    }
    write("target/materials.ppm", w2, CHIP + 8, &raw);
}

/// A name the label column has room for.
fn short(name: &str) -> String {
    name.to_uppercase().chars().take(11).collect()
}
