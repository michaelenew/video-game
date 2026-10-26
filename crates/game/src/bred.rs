//! **A bred creature grown** (2026-10-24, exploration 0031): the body a
//! genome builds to (`sim::breed::build`) handed to the same growing the
//! cast is drawn with (`crate::grown`), so a crossbreed is a creature with
//! flesh, plates, feathers or chitin rather than a stack of boxes.
//!
//! A genome says what its body *is* in discrete genes -- a hide, a jaw, a
//! crown, what each limb ends in -- and how much of it there is in sliders.
//! This is the one place those genes are read as the growing's knobs
//! ([`build_of`]): feathers from a feathered hide, a beak from a beak, a
//! paw, talons or a tarsus from what the walking limbs end in, a scythe from
//! a bladed raptorial pair, armour from plates, rings from chitin. Every
//! length is scaled by the body's own length, so a small cross grows small
//! plates.
//!
//! **The colour is not a gene.** It is blended from the cast's own colours
//! by how near the genome is to each of them ([`affinity`]), so a line that
//! walks from the Ridgeback to the Galewing goes from stone-brown to slate
//! by degrees, while the feathers arrive the generation the hide does.

use bevy::prelude::*;
use sim::beast::{Leg, Part};
use sim::breed::build::{self, Body};
use sim::breed::{Build as Limb, Crown, Genome, Hide, Jaw, Role, Tip};
use sim::species::SpeciesId;

use crate::anatomy::Role as Part_;
use crate::contour::{Contour, Kind};
use crate::grown::Anatomy;
use crate::species::{Build, Foot};

fn fx3(v: sim::V3) -> Vec3 {
    Vec3::new(
        v.x.to_f32_for_render(),
        v.y.to_f32_for_render(),
        v.z.to_f32_for_render(),
    )
}

fn f(x: sim::Fx) -> f32 {
    x.to_f32_for_render()
}

/// A genome's body as something to grow: standing as its posture says, or,
/// with wings, in its flight -- wings spread, the Galewing's own rest pose,
/// and the one the feather rule reads. Folded, a bred wing is rods and
/// planes along the flank with its elbow ahead of the shoulder.
pub fn anatomy(g: &Genome) -> Anatomy {
    crate::touch::touched(g, &[])
}

/// A built body of a genome, in whatever pose it was built in.
pub fn anatomy_of(g: &Genome, body: &Body) -> Anatomy {
    let mut bones = body.bones().to_vec();
    // The foot bone of a walking limb is the second joint's: it stays flat
    // on the floor (`sim::beast::Bone::paw`), and the growing hangs a paw,
    // talons or a tarsus off it.
    for b in bones.iter_mut() {
        if b.name.ends_with(".foot") {
            b.paw = true;
        }
    }
    let mut parts: Vec<Part> = body.parts().to_vec();
    let length = f(body.length).max(0.3);
    let build = build_of(g, length);
    for p in parts.iter_mut() {
        // The torso's boxes meet end to end; grown so, each is a rounded
        // sausage with a waist between. Overlapped, they are one body.
        if bones[p.shape.bone].side == 0
            && matches!(
                p.name,
                "haunch" | "barrel" | "shoulders" | "segment" | "rump"
            )
        {
            let c = p.shape.min.x.add(p.shape.max.x).mul(sim::Fx::ratio(1, 2));
            let h = p.shape.max.x.sub(p.shape.min.x).mul(sim::Fx::ratio(13, 20));
            p.shape.min.x = c.sub(h);
            p.shape.max.x = c.add(h);
        }
        // A feathered leg's upper segment is a bird's thigh: thick with
        // feathers, the shin below it bare and thin.
        if g.minors.hide == Hide::Feathers {
            if let Some((station, 0)) = limb_of(bones[p.shape.bone].name) {
                if g.limbs[station].is_some_and(|l| l.role.grounded()) {
                    let k = sim::Fx::ratio(11, 5);
                    for v in [&mut p.shape.min, &mut p.shape.max] {
                        v.y = v.y.mul(k);
                        v.z = v.z.mul(k);
                    }
                }
            }
        }
        // A grown foot -- a paw, talons, a tarsus -- stands in for the box
        // under it.
        if build.claws > 0 && bones[p.shape.bone].name.ends_with(".foot") {
            if let Some((station, _)) = limb_of(bones[p.shape.bone].name) {
                if g.limbs[station].is_some_and(|l| l.role.grounded()) {
                    p.shape.hollow = true;
                }
            }
        }
    }
    let legs: Vec<Leg> = body.legs().to_vec();
    let rest: Vec<Mat4> = (0..bones.len())
        .map(|b| Mat4::from_translation(fx3(body.at(b))))
        .collect();
    let frames: Vec<Mat4> = (0..parts.len())
        .map(|i| {
            let u = body.axes[i];
            let (v, w) = build::frame(u);
            Mat4::from_cols(
                fx3(u).extend(0.0),
                fx3(v).extend(0.0),
                fx3(w).extend(0.0),
                fx3(body.at(parts[i].shape.bone)).extend(1.0),
            )
        })
        .collect();
    let roles = (0..parts.len())
        .map(|i| role_of(g, &bones, &parts[i]))
        .collect();
    let shapes = (0..parts.len())
        .map(|i| contour_of(g, &bones, &parts[i]))
        .collect();
    let head = parts.iter().position(|p| p.name == "head").unwrap_or(0);
    let mut look = *crate::species::look(nearest(g));
    look.armour.rgb = colour(g);
    look.head = head;
    look.spikes = None;
    look.horns = None;
    look.stance = None;
    look.critters = &[];
    look.no_knuckles = &[];
    Anatomy {
        capsules: true,
        long: vec![Some(0); parts.len()],
        roles,
        shapes,
        bones,
        parts,
        legs,
        rest,
        frames,
        build,
        look,
    }
}

/// Which limb a bone is on, and which of its three segments: `fore.l`,
/// `fore.l.lower`, `fore.l.foot`.
/// Which limb a bone is on, and which of its three segments: `fore.l`,
/// `fore.l.lower`, `fore.l.foot`.
fn limb_of(name: &str) -> Option<(usize, usize)> {
    let station = ["fore.", "midfore.", "midhind.", "hind."]
        .iter()
        .position(|s| name.starts_with(s))?;
    let seg = if name.ends_with(".foot") {
        2
    } else if name.ends_with(".lower") {
        1
    } else {
        0
    };
    Some((station, seg))
}

/// What a bred part is, where the genome knows better than its box: a
/// bladed limb's last segment is a scythe though nothing breaks it.
fn role_of(g: &Genome, bones: &[sim::beast::Bone], p: &Part) -> Option<Part_> {
    let (station, seg) = limb_of(bones[p.shape.bone].name)?;
    let pair = g.limbs[station]?;
    if seg == 2 && pair.tip == Tip::Blade && !pair.role.grounded() {
        return Some(Part_::Blade);
    }
    // A wing is read off its box: spread flat it is a wing, which the
    // feather rule lays its flight feathers on.
    if pair.role == Role::Wing {
        return None;
    }
    // Everything else on a limb is flesh: a wing folded along the flank is
    // not the spread wing the feather rule lays feathers across.
    Some(Part_::Body)
}

/// A part's contour, from what the genome says it is rather than from how
/// its box happens to lean: a bred leg leans, and a leaning box read by
/// its tilt is a torso's barrel -- a pipe.
fn contour_of(g: &Genome, bones: &[sim::beast::Bone], p: &Part) -> Option<Contour> {
    let bone = bones[p.shape.bone].name;
    if let Some((station, seg)) = limb_of(bone) {
        let pair = g.limbs[station]?;
        let kind = match (pair.build, seg) {
            (Limb::Column, _) => Contour::of(Kind::Column).between_const(Kind::Taper, 0.3),
            (Limb::Wing, _) => Contour::of(Kind::Taper),
            (_, 0) => Contour::of(Kind::Taper),
            _ => Contour::of(Kind::Taper).between_const(Kind::Column, 0.3),
        };
        return Some(kind);
    }
    Some(match p.name {
        "thorax" if g.stance == sim::breed::Stance::Upright => Contour::of(Kind::Stalk),
        "abdomen" => Contour::of(Kind::Barrel).between_const(Kind::Whip, 0.55),
        "head" | "jaw" => Contour::of(Kind::Wedge),
        "neck" => Contour::of(Kind::Column).between_const(Kind::Taper, 0.5),
        n if n.starts_with("tail") => Contour::of(Kind::Whip),
        "haunch" | "barrel" | "shoulders" | "segment" | "rump" | "thorax" => {
            if g.minors.hide == Hide::Feathers {
                Contour::of(Kind::Keel).between_const(Kind::Barrel, 0.5)
            } else {
                Contour::of(Kind::Barrel)
            }
        }
        _ => return None,
    })
}

/// The growing's knobs, read off the genes. `length` is the body's, in
/// metres: what every absolute size here is a share of.
pub fn build_of(g: &Genome, length: f32) -> Build {
    let s = &g.sliders;
    let m = &g.minors;
    let mass = f(s.mass);
    // Shares of the cast's own: the Ridgeback is about thirteen metres,
    // the Mantis about three.
    let big = length / 13.0;
    let small = length / 3.2;
    // What the walking limbs end in, from the hindmost grounded pair.
    let walking = g
        .limbs
        .iter()
        .rev()
        .flatten()
        .find(|p| p.role.grounded())
        .copied();
    let (foot, claws) = match (walking.map(|p| p.tip), m.hide) {
        (Some(Tip::Talon), _) => (Foot::Talon, 4),
        (Some(Tip::Claw | Tip::Pad), Hide::Chitin) => (Foot::Tarsus, 2),
        (Some(Tip::Pad | Tip::Claw), _) => (Foot::Paw, 4),
        (Some(Tip::Hoof), _) => (Foot::Paw, 3),
        _ => (Foot::Paw, 0),
    };
    let columns = g
        .limbs
        .iter()
        .flatten()
        .any(|p| p.role.grounded() && p.build == Limb::Column);
    let raptorial = g
        .limbs
        .iter()
        .flatten()
        .any(|p| p.build == Limb::Raptorial || p.tip == Tip::Blade);
    Build {
        fat: 0.3 + 0.4 * mass,
        muscle: 0.3 + 0.5 * mass,
        heavy: mass,
        contours: &[],
        belly: if g.stance == sim::breed::Stance::Level {
            0.8 * mass
        } else {
            0.0
        },
        girth: if columns { 1.0 + 0.3 * mass } else { 1.0 },
        armour: match m.hide {
            Hide::Plates => 0.2 * big,
            Hide::Shell => 0.26 * big,
            _ => 0.0,
        },
        scute: 1.05 * big.max(0.2),
        spines: if m.back == sim::breed::Back::Ridge {
            1.25 * big
        } else {
            0.0
        },
        horns: if m.crown == Crown::Horns {
            1.6 * big
        } else {
            0.0
        },
        claws,
        foot,
        feathers: if m.hide == Hide::Feathers {
            length / 8.0
        } else {
            0.0
        },
        beak: if m.jaw == Jaw::Beak { 0.5 } else { 0.0 },
        crest: if m.crown == Crown::Crest { 1.0 } else { 0.0 },
        chitin: if m.hide == Hide::Chitin {
            0.19 * small
        } else {
            0.0
        },
        grip: if raptorial { 0.16 * small } else { 0.0 },
        antennae: if m.crown == Crown::Feelers {
            1.0 * small
        } else {
            0.0
        },
        eyes: if m.jaw == Jaw::Mandibles {
            0.15 * small
        } else {
            0.0
        },
    }
}

/// How near a genome is to each of the cast with a look, nought to one and
/// summing to one: a soft nearest-neighbour over the genes, the majors
/// weighed heaviest.
pub fn affinity(g: &Genome) -> Vec<(SpeciesId, f32)> {
    let mut out: Vec<(SpeciesId, f32)> = (0..sim::species::COUNT as u8)
        .map(SpeciesId)
        .filter_map(|id| sim::breed::archetypes::of_species(id).map(|a| (id, distance(g, a))))
        .map(|(id, d)| (id, (-d / 0.8).exp()))
        .collect();
    let sum: f32 = out.iter().map(|x| x.1).sum::<f32>().max(1e-9);
    for x in out.iter_mut() {
        x.1 /= sum;
    }
    out
}

/// The member of the cast a genome is nearest.
pub fn nearest(g: &Genome) -> SpeciesId {
    affinity(g)
        .into_iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|x| x.0)
        .unwrap_or(SpeciesId::RIDGEBACK)
}

/// The body's colour: the cast's, blended by [`affinity`].
pub fn colour(g: &Genome) -> [f32; 3] {
    let mut c = [0.0f32; 3];
    for (id, w) in affinity(g) {
        let rgb = crate::species::look(id).armour.rgb;
        for k in 0..3 {
            c[k] += rgb[k] * w;
        }
    }
    c
}

/// How unalike two genomes are: sliders by how far apart, every major and
/// minor by whether it differs, the majors heaviest.
pub fn distance(a: &Genome, b: &Genome) -> f32 {
    let mut d = 0.0;
    for s in sim::breed::Slider::ALL {
        d += (f(a.sliders.get(s)) - f(b.sliders.get(s))).abs() * 0.6;
    }
    let ne = |x: bool| if x { 1.0 } else { 0.0 };
    d += 2.0 * ne(a.stance != b.stance);
    d += 0.5 * ne(a.head != b.head);
    d += ne(a.segments.thorax != b.segments.thorax);
    d += ne(a.segments.abdomen != b.segments.abdomen);
    d += 0.5 * ne(a.segments.tail != b.segments.tail);
    for (pa, pb) in a.limbs.iter().zip(&b.limbs) {
        match (pa, pb) {
            (Some(x), Some(y)) => {
                d += 0.7 * ne(x.role != y.role);
                d += 0.5 * ne(x.build != y.build);
                d += 0.3 * ne(x.tip != y.tip);
                d += (f(x.length) - f(y.length)).abs() + (f(x.girth) - f(y.girth)).abs();
            }
            (None, None) => {}
            _ => d += 1.0,
        }
    }
    d += 0.4 * ne(a.minors.jaw != b.minors.jaw);
    d += 0.3 * ne(a.minors.crown != b.minors.crown);
    d += 0.2 * ne(a.minors.tail_tip != b.minors.tail_tip);
    d += ne(a.minors.hide != b.minors.hide);
    d += 0.3 * ne(a.minors.back != b.minors.back);
    d
}

/// A creature whose limbs could be wings, by its genome: for the lines'
/// account of it.
#[allow(dead_code)]
pub fn winged(g: &Genome) -> bool {
    g.limbs.iter().flatten().any(|p| p.role == Role::Wing)
}

/// One stage of a line: the genome, which generation, and what changed.
pub struct Stage {
    pub genome: Genome,
    pub generation: usize,
    pub said: String,
}

/// A crossbreed line: what it started as, what it was bred toward, and how.
pub struct Line {
    pub name: &'static str,
    pub stages: Vec<Stage>,
}

/// How a line is bred.
#[derive(Clone, Copy)]
pub enum How {
    /// A morph fed the donor's trophies one at a time, in the order the
    /// donor drops them (`sim::breed::cross::feed`): a visible step each.
    Fed,
    /// Crossed back into the donor every generation (`cross::breed`), with
    /// the slow rates' mutation.
    Backcrossed,
}

/// Breed `from` toward `to` for `generations`, keeping every stage.
pub fn line(
    name: &'static str,
    from: &Genome,
    to: &Genome,
    how: How,
    generations: usize,
    seed: u64,
) -> Line {
    use sim::breed::cross::{self, Rates, Trophy};
    let mut rng = sim::breed::Rng::new(seed);
    let drops: Vec<_> = sim::breed::archetypes::drops(to)
        .into_iter()
        .flatten()
        .collect();
    let mut g = *from;
    let mut stages = vec![Stage {
        genome: g,
        generation: 0,
        said: "start".to_string(),
    }];
    for k in 0..generations {
        let (next, said) = match how {
            How::Fed => {
                let kind = drops[k % drops.len().max(1)];
                let out = cross::feed(&g, &Trophy { kind, donor: *to }, &mut rng);
                let lines = sim::breed::morph::changes(&g, &out.genome, Some(kind));
                let said: Vec<String> = lines
                    .lines
                    .iter()
                    .flatten()
                    .map(|l| format!("{l:?}"))
                    .collect();
                (out.genome, format!("fed {kind:?}: {}", said.join(", ")))
            }
            How::Backcrossed => {
                let out = cross::breed(&g, to, &Rates::DEFAULT, &mut rng);
                let lines = sim::breed::morph::changes(&g, &out.genome, None);
                let said: Vec<String> = lines
                    .lines
                    .iter()
                    .flatten()
                    .map(|l| format!("{l:?}"))
                    .collect();
                (out.genome, format!("crossed back: {}", said.join(", ")))
            }
        };
        g = next;
        stages.push(Stage {
            genome: g,
            generation: k + 1,
            said,
        });
    }
    Line { name, stages }
}

/// The lines exploration 0031 shows: each of the three formed creatures fed
/// toward the next, and one bred the slow way.
pub fn lines() -> Vec<Line> {
    use sim::breed::archetypes::{GALEWING, MANTIS, RIDGEBACK};
    vec![
        line(
            "ridgeback_to_galewing",
            &RIDGEBACK,
            &GALEWING,
            How::Fed,
            12,
            1,
        ),
        line("galewing_to_mantis", &GALEWING, &MANTIS, How::Fed, 12, 2),
        line("mantis_to_ridgeback", &MANTIS, &RIDGEBACK, How::Fed, 12, 3),
        line(
            "ridgeback_x_mantis",
            &RIDGEBACK,
            &MANTIS,
            How::Backcrossed,
            8,
            4,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every stage of every line grows a body, in one piece or near it.
    #[test]
    fn every_stage_of_every_line_grows() {
        for l in lines() {
            for st in &l.stages {
                let a = anatomy(&st.genome);
                let g = crate::grown::grow(&a)
                    .unwrap_or_else(|| panic!("{} gen {}: no body", l.name, st.generation));
                assert!(
                    g.mesh.count_vertices() > 500,
                    "{} gen {}",
                    l.name,
                    st.generation
                );
            }
        }
    }

    /// `GROWN_LINES=1 cargo test -p game --bin game grown_lines` (or a part
    /// of a line's name for that line alone): every stage of every line as `target/lines/<line>_<stage>.obj`, coloured,
    /// and `target/lines/lines.tsv` saying what each is.
    /// A grown body as an OBJ whose vertices carry their colours.
    fn obj(g: &crate::grown::Grown, a: &Anatomy) -> String {
        use bevy::render::mesh::{Indices, VertexAttributeValues};
        let colours = crate::grown::colours(g, |_| a.look.armour, a.look.weak.rgb);
        let mut out = String::new();
        let Some(VertexAttributeValues::Float32x3(p)) = g.mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            return out;
        };
        let Some(Indices::U32(t)) = g.mesh.indices() else {
            return out;
        };
        for (v, c) in p.iter().zip(&colours) {
            out.push_str(&format!(
                "v {} {} {} {} {} {}\n",
                v[0], v[1], v[2], c[0], c[1], c[2]
            ));
        }
        for tri in t.chunks_exact(3) {
            out.push_str(&format!("f {} {} {}\n", tri[0] + 1, tri[1] + 1, tri[2] + 1));
        }
        out
    }

    #[test]
    fn grown_lines() {
        let Some(only) = crate::platform::env("GROWN_LINES") else {
            return;
        };
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/lines");
        // `GROWN_STAGES=0,3,6`: those stages alone.
        let stages: Option<Vec<usize>> = crate::platform::env("GROWN_STAGES")
            .map(|s| s.split(',').filter_map(|k| k.trim().parse().ok()).collect());
        let mut tsv = String::from("line\tstage\tgeneration\tlength\tnearest\tdropped\tsaid\n");
        // `1` for every line, or a part of one's name.
        for l in lines()
            .into_iter()
            .filter(|l| only == "1" || l.name.contains(&only))
        {
            for (k, st) in l.stages.iter().enumerate() {
                let grow = stages.as_ref().is_none_or(|s| s.contains(&k));
                let hand = crate::touch::touches(l.name, k);
                let mut dropped = 0;
                // The breeder's body, and where there are hand edits, the
                // hand's beside it: `<line>_<k>_raw.obj` and `<line>_<k>.obj`.
                for (suffix, edits) in [("_raw", &[][..]), ("", &hand[..])] {
                    if !grow || suffix == "_raw" && hand.is_empty() {
                        continue;
                    }
                    let a = crate::touch::touched(&st.genome, edits);
                    let Some(g) = crate::grown::grow(&a) else {
                        continue;
                    };
                    dropped = g.dropped;
                    crate::platform::write_to(
                        &dir.join(format!("{}_{k:02}{suffix}.obj", l.name)),
                        &obj(&g, &a),
                        "a line's stage",
                    );
                }
                let body = build::body(&st.genome);
                tsv.push_str(&format!(
                    "{}\t{k}\t{}\t{:.2}\t{:?}\t{}\t{}\n",
                    l.name,
                    st.generation,
                    f(body.length),
                    nearest(&st.genome),
                    dropped,
                    st.said
                ));
            }
        }
        crate::platform::write_to(&dir.join("lines.tsv"), &tsv, "the lines' account");
    }
}

#[cfg(test)]
mod dump {
    /// `BRED_DUMP=1`: the bred archetypes' parts, for looking at;
    /// `BRED_DUMP=<line>:<stage>` one stage of a line, with its hand edits.
    #[test]
    fn bred_dump() {
        let Some(what) = crate::platform::env("BRED_DUMP") else {
            return;
        };
        let mut bodies = vec![
            (
                "ridgeback".to_string(),
                sim::breed::archetypes::RIDGEBACK,
                vec![],
            ),
            (
                "galewing".to_string(),
                sim::breed::archetypes::GALEWING,
                vec![],
            ),
            ("mantis".to_string(), sim::breed::archetypes::MANTIS, vec![]),
        ];
        if let Some((line, k)) = what.split_once(':') {
            let k: usize = k.parse().unwrap_or(0);
            bodies = super::lines()
                .into_iter()
                .filter(|l| l.name == line)
                .filter_map(|l| {
                    let g = l.stages.get(k)?.genome;
                    Some((format!("{line}:{k}"), g, crate::touch::touches(l.name, k)))
                })
                .collect();
        }
        for (name, g, edits) in bodies {
            let a = crate::touch::touched(&g, &edits);
            eprintln!(
                "== {name}: {} bones {} parts {} legs",
                a.bones.len(),
                a.parts.len(),
                a.legs.len()
            );
            for (i, p) in a.parts.iter().enumerate() {
                let s = p.shape;
                let h = super::fx3(s.max.sub(s.min)) * 0.5;
                let c = a.frames[i]
                    .transform_point3(super::fx3(s.max.add(s.min).scale(sim::Fx::ratio(1, 2))));
                eprintln!(
                    "  {i:2} {:<14} bone {:<14} centre ({:5.2},{:5.2},{:5.2}) half ({:4.2},{:4.2},{:4.2}) axis {:?}",
                    p.name,
                    a.bones[s.bone].name,
                    c.x,
                    c.y,
                    c.z,
                    h.x,
                    h.y,
                    h.z,
                    a.frames[i].x_axis.truncate()
                );
            }
            for l in &a.legs {
                eprintln!(
                    "  leg upper {} foot {} hip {} knee {}",
                    a.parts[l.upper].name,
                    a.parts[l.foot].name,
                    a.bones[l.hip].name,
                    a.bones[l.knee].name
                );
            }
        }
    }
}

#[cfg(test)]
mod gait {
    /// `BRED_GAIT=<line>:<stage>` (or `cast:<Name>`): that stage's body boxes
    /// through a cycle of its slowest gait, `target/gait/<line>_<stage>_<gait>_<k>.obj`,
    /// coloured by what each part is -- for watching it move.
    #[test]
    fn bred_gait() {
        let Some(what) = crate::platform::env("BRED_GAIT") else {
            return;
        };
        use sim::breed::{build, locomotion, posture};
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/gait");
        for item in what.split(',') {
            let Some((line, k)) = item.split_once(':') else {
                continue;
            };
            // `cast:<Name>` is one of the cast's own genomes.
            let cast = sim::breed::archetypes::CAST
                .iter()
                .find(|c| line == "cast" && c.1 == k)
                .map(|c| *c.2);
            let tag = if cast.is_some() {
                format!("cast_{k}")
            } else {
                format!("{line}_{:02}", k.parse::<usize>().unwrap_or(0))
            };
            let k: usize = k.parse().unwrap_or(0);
            let Some(g) = cast.or_else(|| {
                super::lines()
                    .into_iter()
                    .find(|l| l.name == line)
                    .and_then(|l| l.stages.get(k).map(|s| s.genome))
            }) else {
                continue;
            };
            let q = posture::posture(&g);
            // Its slowest gait: how it walks.
            for gait in locomotion::repertoire(&g, &q).iter().flatten().take(1) {
                for f in 0..8 {
                    let phase = sim::Fx::ratio(f, 8);
                    let d = locomotion::drive(&g, &q, gait, phase, sim::Fx::ZERO);
                    let body = build::body_driven(&g, &q, &d);
                    let mut out = String::new();
                    let mut n = 0;
                    for i in 0..body.part_count {
                        let p = &body.parts[i];
                        let bone = body.bones[p.shape.bone].name;
                        let c = colour(p.name, bone, &g);
                        for v in body.corners(i) {
                            let v = super::fx3(v);
                            out.push_str(&format!(
                                "v {} {} {} {} {} {}\n",
                                v.x, v.y, v.z, c[0], c[1], c[2]
                            ));
                        }
                        // The box's six faces, corners numbered as `corners` does.
                        for face in [
                            [0, 1, 3, 2],
                            [4, 6, 7, 5],
                            [0, 4, 5, 1],
                            [2, 3, 7, 6],
                            [0, 2, 6, 4],
                            [1, 5, 7, 3],
                        ] {
                            out.push_str(&format!(
                                "f {} {} {} {}\n",
                                n + face[0] + 1,
                                n + face[1] + 1,
                                n + face[2] + 1,
                                n + face[3] + 1
                            ));
                        }
                        n += 8;
                    }
                    crate::platform::write_to(
                        &dir.join(format!("{tag}_{}_{f}.obj", gait.kind.name())),
                        &out,
                        "a stage's gait",
                    );
                }
            }
        }
    }

    /// `BRED_ATTACK=<line>:<stage>` (or `cast:<Name>`): each damaging move's
    /// body boxes at six points from rest through its windup and strike,
    /// `target/gait/<tag>_<move>_<k>.obj`, the striking parts in yellow --
    /// for judging whether a body could make it, and whether it shows it.
    #[test]
    fn bred_attack() {
        let Some(what) = crate::platform::env("BRED_ATTACK") else {
            return;
        };
        use sim::breed::{posture, strike};
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/gait");
        for item in what.split(',') {
            let Some((line, k)) = item.split_once(':') else {
                continue;
            };
            let cast = sim::breed::archetypes::CAST
                .iter()
                .find(|c| line == "cast" && c.1 == k)
                .map(|c| *c.2);
            let tag = if cast.is_some() {
                format!("cast_{k}")
            } else {
                format!("{line}_{:02}", k.parse::<usize>().unwrap_or(0))
            };
            let k: usize = k.parse().unwrap_or(0);
            let Some(g) = cast.or_else(|| {
                super::lines()
                    .into_iter()
                    .find(|l| l.name == line)
                    .and_then(|l| l.stages.get(k).map(|s| s.genome))
            }) else {
                continue;
            };
            let q = posture::posture(&g);
            let (kit, n) = strike::kit(&g, &q);
            for (spec, m) in kit[..n].iter().flatten() {
                if spec.damage == 0 {
                    continue;
                }
                let frames = [
                    0,
                    spec.startup / 2,
                    spec.startup - 1,
                    spec.startup + spec.active / 2,
                    spec.startup + spec.active - 1,
                ];
                let cry = m.cry.map_or(String::new(), |c| format!("-{}", c.name()));
                for (f, &frame) in frames.iter().enumerate() {
                    let body = strike::body_at(&g, &q, m, spec, frame);
                    let mut out = String::new();
                    let mut v0 = 0;
                    for i in 0..body.part_count {
                        let part = &body.parts[i];
                        let bone = body.bones[part.shape.bone].name;
                        let c = if m
                            .strikers
                            .iter()
                            .any(|s| !s.is_empty() && part.name.starts_with(s))
                        {
                            [0.95, 0.85, 0.2]
                        } else {
                            colour(part.name, bone, &g)
                        };
                        for v in body.corners(i) {
                            let v = super::fx3(v);
                            out.push_str(&format!(
                                "v {} {} {} {} {} {}\n",
                                v.x, v.y, v.z, c[0], c[1], c[2]
                            ));
                        }
                        for face in [
                            [0, 1, 3, 2],
                            [4, 6, 7, 5],
                            [0, 4, 5, 1],
                            [2, 3, 7, 6],
                            [0, 2, 6, 4],
                            [1, 5, 7, 3],
                        ] {
                            out.push_str(&format!(
                                "f {} {} {} {}\n",
                                v0 + face[0] + 1,
                                v0 + face[1] + 1,
                                v0 + face[2] + 1,
                                v0 + face[3] + 1
                            ));
                        }
                        v0 += 8;
                    }
                    let name = spec.kind.name().replace(' ', "-");
                    crate::platform::write_to(
                        &dir.join(format!("{tag}_{name}{cry}_{f}.obj")),
                        &out,
                        "a move's frame",
                    );
                }
            }
        }
    }

    /// Torso grey, walking legs blue, wings violet, striking arms orange,
    /// head red, tail green.
    fn colour(name: &str, bone: &str, g: &sim::breed::Genome) -> [f32; 3] {
        if let Some((station, _)) = super::limb_of(bone) {
            return match g.limbs[station] {
                Some(p) if p.role == sim::breed::Role::Wing => [0.55, 0.4, 0.8],
                Some(p) if p.role.grounded() => [0.3, 0.5, 0.85],
                _ => [0.95, 0.55, 0.15],
            };
        }
        if name.starts_with("head") || name == "jaw" || name == "crest" || name == "horns" {
            [0.85, 0.25, 0.2]
        } else if name.starts_with("tail") {
            [0.35, 0.7, 0.35]
        } else {
            [0.7, 0.68, 0.64]
        }
    }
}
