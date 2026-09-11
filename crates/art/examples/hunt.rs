// Throwaway search: lightness and small hue nudges for the four player colours
// and hostile red, maximising the worst pairwise legibility -- among the five
// identities, and between each identity and the arena it is seen against --
// under both ordinary and red-green colour-blind vision.
use art::color::{self, Lch};
use art::materials;
use art::palette::deuteranope_separation;

fn sep(a: [f32; 3], b: [f32; 3]) -> f32 {
    color::difference(a, b).min(deuteranope_separation(a, b))
}

/// Lightnesses the arena actually presents, sampled off the world materials.
fn world() -> Vec<[f32; 3]> {
    let mut v = vec![];
    for m in materials::FIXED {
        if m.role != materials::Role::World {
            continue;
        }
        for k in 0..9 {
            v.push(m.surface.ramp.at(k as f32 / 8.0));
        }
    }
    v
}

/// How well the five identities separate from **each other**. This is the hard
/// task -- five colourful things told apart at a glance -- and it is what the
/// search maximises.
fn identity_score(set: &[Lch]) -> f32 {
    let rgb: Vec<_> = set.iter().map(|c| c.to_linear()).collect();
    let mut w = f32::MAX;
    for i in 0..rgb.len() {
        for j in i + 1..rgb.len() {
            w = w.min(sep(rgb[i], rgb[j]));
        }
    }
    w
}

/// How well the identities separate from the **arena**. A different and much
/// easier task, because the arena has almost no chroma and a coloured thing is
/// therefore far from all of it by construction. Folding the two into one
/// objective is what the first attempt did, and it collapsed every identity
/// onto the same lightness chasing a term that was never in danger. So this is
/// a floor to clear, not a quantity to maximise.
fn arena_floor(set: &[Lch], world: &[[f32; 3]]) -> f32 {
    let mut w = f32::MAX;
    for c in set {
        for b in world {
            w = w.min(sep(c.to_linear(), *b));
        }
    }
    w
}

const ARENA_FLOOR: f32 = 0.12;

fn main() {
    let world = world();
    let base = [0.700f32, 0.190, 0.420, 0.880];
    let mut best = (0.0f32, vec![]);
    let ls: Vec<f32> = (0..11).map(|i| 0.46 + i as f32 * 0.04).collect();
    let hs: Vec<f32> = (0..6).map(|i| 0.48 + i as f32 * 0.035).collect();
    let nudge = [-0.05f32, 0.0, 0.05];

    for &a in &ls {
        for &b in &ls {
            for &c in &ls {
                for &d in &ls {
                    for &h in &hs {
                        for &n0 in &nudge {
                            for &n2 in &nudge {
                                for &n3 in &nudge {
                                    let set = [
                                        Lch::new(a, 0.14, base[0] + n0),
                                        Lch::new(b, 0.14, base[1]),
                                        Lch::new(c, 0.14, base[2] + n2),
                                        Lch::new(d, 0.14, base[3] + n3),
                                        Lch::new(h, 0.19, 0.080),
                                    ];
                                    if arena_floor(&set, &world) < ARENA_FLOOR {
                                        continue;
                                    }
                                    let s = identity_score(&set);
                                    if s > best.0 {
                                        best = (s, vec![a, b, c, d, h, n0, n2, n3]);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let v = &best.1;
    let set = [
        ("blue", Lch::new(v[0], 0.14, base[0] + v[5])),
        ("amber", Lch::new(v[1], 0.14, base[1])),
        ("green", Lch::new(v[2], 0.14, base[2] + v[6])),
        ("violet", Lch::new(v[3], 0.14, base[3] + v[7])),
        ("HOSTILE", Lch::new(v[4], 0.19, 0.080)),
    ];
    println!("worst identity pair {:.3}", best.0);
    for (n, c) in &set {
        println!("  {n:>8}  L {:.2}  C {:.2}  h {:.3}", c.l, c.c, c.h);
    }
    println!("-- pairs --");
    for i in 0..set.len() {
        for j in i + 1..set.len() {
            let (a, b) = (set[i].1.to_linear(), set[j].1.to_linear());
            println!(
                "  {:>8} / {:<8} normal {:.3}  deuter {:.3}",
                set[i].0,
                set[j].0,
                color::difference(a, b),
                deuteranope_separation(a, b)
            );
        }
    }
    let cs: Vec<Lch> = set.iter().map(|x| x.1).collect();
    println!(
        "-- worst identity against the arena: {:.3}",
        arena_floor(&cs, &world)
    );
}
