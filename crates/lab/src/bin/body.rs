//! Every topology experiment in `docs/design/exploration/0005_body_plans.md`,
//! run in one go. Prints a Markdown report and writes contact sheets to
//! `target/lab/`.
//!
//!     CARGO_TARGET_DIR=target/lab cargo run --release \
//!         --manifest-path crates/lab/Cargo.toml --bin body > target/lab/body.md

use std::fmt::Write as _;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;

use lab::measure::{self, Fight, SEEDS};
use lab::{Body, f, fx, keep_feet_on_floor, restore, roles, standing_sole, stretch};
use sim::Class;
use sim::oven::MonsterField as F;
use sim::species::{self, Species, SpeciesId};

static PANIC: Mutex<String> = Mutex::new(String::new());

fn bone(s: &Species, name: &str) -> usize {
    s.bones
        .iter()
        .position(|b| b.name == name)
        .unwrap_or_else(|| panic!("{} has no bone {name}", s.name))
}

/// Run one experiment; a panic is a result, not the end of the run.
fn trial<T>(what: &str, out: &mut String, run: impl FnOnce() -> T) -> Option<T> {
    PANIC.lock().unwrap().clear();
    let r = catch_unwind(AssertUnwindSafe(run));
    restore();
    match r {
        Ok(v) => Some(v),
        Err(_) => {
            let msg = PANIC.lock().unwrap().clone();
            let _ = writeln!(
                out,
                "| {what} | **panicked**: `{}` |",
                msg.replace('|', "/").replace('\n', " ")
            );
            None
        }
    }
}

fn wins(fight: &Fight) -> String {
    fight
        .won
        .iter()
        .map(|(c, n)| format!("{}{}", &c.name()[..2], n))
        .collect::<Vec<_>>()
        .join(" ")
}

fn total_wins(fight: &Fight) -> u32 {
    fight.won.iter().map(|(_, n)| n).sum()
}

fn broken(fight: &Fight) -> String {
    if fight.broken.is_empty() {
        "none".into()
    } else {
        fight.broken.join(", ")
    }
}

struct Statics {
    feet: measure::Feet,
    skate: f32,
    back: Option<f32>,
    climb: usize,
    strikes: Vec<measure::Strike>,
}

fn statics(s: &'static Species, apex: &[(Class, f32)]) -> Statics {
    Statics {
        feet: measure::feet(s),
        skate: measure::skate(s),
        back: measure::lowest_back(s),
        climb: measure::climbable_by(s, apex),
        strikes: measure::strikes(s),
    }
}

/// Body blows whose volume is more than this far from every part of the body
/// on the frame they come out.
const DETACHED: f32 = 0.5;

fn detached(st: &Statics) -> String {
    let far: Vec<String> = st
        .strikes
        .iter()
        .filter(|k| k.kind == "body" && k.gap > DETACHED)
        .map(|k| format!("{} {:.1}", k.name, k.gap))
        .collect();
    if far.is_empty() {
        "none".into()
    } else {
        far.join("; ")
    }
}

fn statics_row(name: &str, st: &Statics) -> String {
    format!(
        "| {name} | {:+.2} / {:+.2} | {:+.2} | {:+.2} ({}) | {:+.2} | {:.2} | {} | {} | {} |",
        st.feet.idle,
        st.feet.idle_high,
        st.feet.gait,
        st.feet.moves,
        st.feet.worst_move,
        st.feet.belly,
        st.skate,
        st.back.map_or("--".into(), |b| format!("{b:.1}")),
        st.climb,
        detached(st),
    )
}

const STATICS_HEAD: &str = "| Body | Idle sole, low / high (m) | Gait sole | Lowest sole in a move | Belly | Skate | Back (m) | Classes up from the floor | Body blows off the body (m) |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- |";

fn fight_row(name: &str, fight: &Fight) -> String {
    format!(
        "| {name} | {} / {} | {} | {} | {} | {:.0} | {} |",
        total_wins(fight),
        fight.runs,
        wins(fight),
        fight.unanswerable,
        fight.unresolved,
        fight.secs,
        broken(fight),
    )
}

const FIGHT_HEAD: &str = "| Body | Hunts won | By class | Unanswerable | Unresolved | Mean length (s) | Contract clauses broken (Champion) |\n| --- | --- | --- | --- | --- | --- | --- |";

fn sheet(s: &'static Species, name: &str) {
    let path = format!("target/lab/{name}.png");
    let _ = anim::beast::sheet::states(s).write(std::path::Path::new(&path));
}

/// A donor move's row of knobs, read for a recipient: distances scaled by the
/// ratio of hip heights, the bone it rides found by role.
fn row_for(
    donor: &'static Species,
    kind: usize,
    to: &mut Body,
    to_sp: &'static Species,
) -> (Vec<i32>, String) {
    let mut row: Vec<i32> = F::ALL
        .iter()
        .map(|fld| sim::oven::species_raw(donor.id, donor.move_index(kind, *fld)))
        .collect();
    let ratio = roles::hip(to_sp) / roles::hip(donor);
    for fld in [
        F::HitX,
        F::HitZ,
        F::HitRadius,
        F::HitLow,
        F::HitHigh,
        F::IdealRange,
        F::RangeSpan,
        F::Advance,
        F::Travel,
    ] {
        let i = fld as usize;
        row[i] = (row[i] as f32 * ratio).round() as i32;
    }
    let follows = row[F::Follows as usize] as usize;
    let donor_bone = donor.follow_bone(follows as u8);
    let back = roles::map(to_sp, donor);
    let note;
    let to_bone = match back[donor_bone] {
        Some(b) => {
            note = format!(
                "{} rides {}",
                donor.bones[donor_bone].name, to_sp.bones[b].name
            );
            b
        }
        None => {
            note = format!(
                "{} has no counterpart; rides the body",
                donor.bones[donor_bone].name
            );
            to.follows[0]
        }
    };
    let k = match to.follows.iter().position(|b| *b == to_bone) {
        Some(k) => k,
        None => {
            to.follows.push(to_bone);
            to.follows.len() - 1
        }
    };
    row[F::Follows as usize] = k as i32;
    (row, note)
}

/// Give `to` one of `donor`'s moves, clip read across by role.
fn transplant(
    donor: &'static Species,
    kind: usize,
    to: &mut Body,
    to_sp: &'static Species,
) -> String {
    let decl = donor.moves[kind];
    let rows = roles::transplant_rows(donor, decl.clip, to, to_sp);
    let (row, note) = row_for(donor, kind, to, to_sp);
    to.add_move(decl, donor.clips[decl.clip], rows, row);
    note
}

/// A species' senses row, as its store holds it.
fn senses_of(s: &'static Species) -> Vec<i32> {
    sim::oven::species_knobs(s)
        .into_iter()
        .filter(|k| matches!(k, sim::oven::Knob::Species(_, sim::oven::Tunable::Fight(_))))
        .map(|k| k.raw())
        .collect()
}

fn coverage(from: &'static Species, to: &'static Species) -> (usize, usize, usize) {
    let m = roles::map(from, to);
    let driven = m.iter().filter(|d| d.is_some()).count();
    let mut used: Vec<usize> = m.iter().flatten().copied().collect();
    used.sort();
    used.dedup();
    (driven, to.bones.len(), from.bones.len() - used.len())
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let loc = info
            .location()
            .map(|l| format!(" at {}:{}", l.file(), l.line()))
            .unwrap_or_default();
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "?".into()
        };
        *PANIC.lock().unwrap() = format!("{msg}{loc}");
    }));
    std::fs::create_dir_all("target/lab").unwrap();
    let only: Vec<String> = std::env::args().skip(1).collect();
    let want = |e: &str| only.is_empty() || only.iter().any(|o| o == e);
    let mut out = String::new();
    let apex = measure::apexes();
    let rb = &species::ridgeback::SPECIES;

    let _ = writeln!(out, "# Topology lab results\n");
    let _ = writeln!(
        out,
        "Sole heights are metres above the floor (negative is in it). Skate is how far a planted foot slides as a share of how far the body moves (0 planted, 1 dragged). \"Body blows off the body\" lists every move whose hit volume, on its first active frame, is more than {DETACHED} m from every part of the body, with the gap in metres; projectiles, lobs and moves with their own hit test are left out. Class hops: {}.\n",
        apex.iter()
            .map(|(c, a)| format!("{} {a:.1} m", c.name()))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // ---------------------------------------------------------------- E0
    if want("e0") {
        let _ = writeln!(out, "## E0 · Every body as built\n\n{STATICS_HEAD}");
        for s in species::all().filter(|s| s.has_body() && !s.id.is_dev()) {
            let st = statics(s, &apex);
            let _ = writeln!(out, "{}", statics_row(s.name, &st));
            sheet(s, &format!("e0-{}", s.slug()));
        }
    }
    if want("e0f") {
        let _ = writeln!(out, "\n{FIGHT_HEAD}");
        for s in species::all().filter(|s| s.has_body() && !s.id.is_dev()) {
            if hunt::plans::card(s.id).is_some() {
                let fight = measure::fight(s, &SEEDS);
                let _ = writeln!(out, "{}", fight_row(s.name, &fight));
            }
        }
        let _ = writeln!(out);
    }

    // ---------------------------------------------------------------- E1
    if want("e1") {
        let _ = writeln!(
            out,
            "## E1 · Proportions (the Ridgeback)\n\nA segment stretched by `k`: its children sit `k` times further out and its boxes stretch with it. *Naive* leaves the hips where they were; *kept* raises or lowers them so the standing sole is where it was.\n\n{STATICS_HEAD}"
        );
        let base_sole = standing_sole(rb);
        let legs: Vec<usize> = rb.legs.iter().flat_map(|l| [l.hip, l.knee]).collect();
        let neck = vec![bone(rb, "neck"), bone(rb, "neck2")];
        let head = vec![bone(rb, "head")];
        let tail = vec![
            bone(rb, "tail1"),
            bone(rb, "tail2"),
            bone(rb, "tail3"),
            bone(rb, "tail4"),
        ];
        let trunk = vec![bone(rb, "root"), bone(rb, "spine"), bone(rb, "chest")];
        let all = [true; 3];
        let x_only = [true, false, false];
        type Variant = (String, Vec<usize>, f32, [bool; 3], bool);
        let mut variants: Vec<Variant> = Vec::new();
        for k in [0.7, 0.85, 1.15, 1.3, 1.6] {
            variants.push((format!("legs x{k} naive"), legs.clone(), k, all, false));
            variants.push((format!("legs x{k} kept"), legs.clone(), k, all, true));
        }
        for k in [0.6, 1.5] {
            variants.push((format!("neck x{k}"), neck.clone(), k, all, true));
        }
        variants.push(("head x1.5".into(), head.clone(), 1.5, all, true));
        for k in [0.5, 1.6] {
            variants.push((format!("tail x{k}"), tail.clone(), k, all, true));
        }
        for k in [0.8, 1.3] {
            variants.push((
                format!("trunk x{k} (length only)"),
                trunk.clone(),
                k,
                x_only,
                true,
            ));
        }
        let mut fights = Vec::new();
        let mut drifts: Vec<(String, f32, &'static str)> = Vec::new();
        let contacts = |s: &'static Species| {
            (0..s.moves.len())
                .map(|k| measure::contact(s, k as u8))
                .collect::<Vec<_>>()
        };
        let before = contacts(rb);
        let st = statics(rb, &apex);
        let _ = writeln!(out, "{}", statics_row("as built", &st));
        for (name, bones, k, axes, keep) in &variants {
            let mut body = Body::of(rb);
            for b in bones {
                stretch(&mut body, *b, *k, *axes);
            }
            if *keep {
                keep_feet_on_floor(&mut body, base_sole);
            }
            let r = trial(name, &mut out, || {
                let s = body.install();
                let (d, k) = measure::drift(&before, &contacts(s));
                let st = statics(s, &apex);
                let slug = name.replace([' ', '(', ')'], "").replace('.', "_");
                if [
                    "legs x1.6 naive",
                    "legs x1.6 kept",
                    "legs x0.7 kept",
                    "neck x1.5",
                    "tail x1.6",
                    "trunk x1.3 (length only)",
                ]
                .contains(&name.as_str())
                {
                    sheet(s, &format!("e1-{slug}"));
                }
                let fight = if *keep
                    && [
                        "legs x0.7 kept",
                        "legs x1.3 kept",
                        "legs x1.6 kept",
                        "neck x1.5",
                        "tail x1.6",
                        "trunk x1.3 (length only)",
                        "head x1.5",
                    ]
                    .contains(&name.as_str())
                {
                    Some(measure::fight(s, &SEEDS))
                } else {
                    None
                };
                (st, fight, d, s.moves[k].name)
            });
            if let Some((st, fight, d, which)) = r {
                drifts.push((name.clone(), d, which));
                let _ = writeln!(out, "{}", statics_row(name, &st));
                if let Some(fight) = fight {
                    fights.push((name.clone(), fight));
                }
            }
        }
        // Size, the knob that already exists, for comparison.
        for k in [0.6f32, 1.5] {
            let name = format!("Size knob x{k}");
            let r = trial(&name, &mut out, || {
                let mut body = Body::of(rb);
                body.set_common(species::Common::Scale, fx(k).raw());
                let s = body.install();
                let (d, w) = measure::drift(&before, &contacts(s));
                let st = statics(s, &apex);
                sheet(s, &format!("e1-size{}", k.to_string().replace('.', "_")));
                (st, measure::fight(s, &SEEDS), d, s.moves[w].name)
            });
            if let Some((st, fight, d, which)) = r {
                drifts.push((name.clone(), d, which));
                let _ = writeln!(out, "{}", statics_row(&name, &st));
                fights.push((name, fight));
            }
        }
        let _ = writeln!(
            out,
            "\n**Drift** -- how far the bone a move rides moves away from where its blow lands, on the frame it comes out, flat metres; the worst move:\n\n| Body | Drift (m) | Move |\n| --- | --- | --- |"
        );
        for (name, d, which) in &drifts {
            let _ = writeln!(out, "| {name} | {d:.2} | {which} |");
        }
        let _ = writeln!(out, "\n{FIGHT_HEAD}");
        let base = measure::fight(rb, &SEEDS);
        let _ = writeln!(out, "{}", fight_row("as built", &base));
        for (name, fight) in &fights {
            let _ = writeln!(out, "{}", fight_row(name, fight));
        }
        let _ = writeln!(out);
    }

    // ---------------------------------------------------------------- E2
    if want("e2") {
        let _ = writeln!(
            out,
            "## E2 · Part flags (the Ridgeback)\n\nParts as built: {}.\n\n{FIGHT_HEAD}",
            rb.parts
                .iter()
                .map(|p| {
                    let mut flags = Vec::new();
                    if p.weak {
                        flags.push("weak");
                    }
                    if p.shape.mountable {
                        flags.push("mount");
                    }
                    if p.shape.breakable {
                        flags.push("breaks");
                    }
                    if !p.shape.solid {
                        flags.push("soft");
                    }
                    format!("{} ({})", p.name, flags.join(","))
                })
                .collect::<Vec<_>>()
                .join("; ")
        );
        type Edit = fn(&mut Body);
        let edits: Vec<(&str, Edit)> = vec![
            ("no weak points", |b| {
                b.parts.iter_mut().for_each(|p| p.weak = false)
            }),
            ("head weak, back not", |b| {
                b.parts
                    .iter_mut()
                    .for_each(|p| p.weak = p.name.contains("head"))
            }),
            ("nothing mountable", |b| {
                b.parts.iter_mut().for_each(|p| p.shape.mountable = false)
            }),
            ("everything mountable", |b| {
                b.parts
                    .iter_mut()
                    .for_each(|p| p.shape.mountable = p.shape.solid)
            }),
            ("feet do not break", |b| {
                b.parts.iter_mut().for_each(|p| p.shape.breakable = false)
            }),
            ("everything breaks (first twelve)", |b| {
                b.parts.iter_mut().for_each(|p| p.shape.breakable = true)
            }),
        ];
        let base = measure::fight(rb, &SEEDS);
        let _ = writeln!(out, "{}", fight_row("as built", &base));
        for (name, edit) in edits {
            let r = trial(name, &mut out, || {
                let mut body = Body::of(rb);
                edit(&mut body);
                measure::fight(body.install(), &SEEDS)
            });
            if let Some(fight) = r {
                let _ = writeln!(out, "{}", fight_row(name, &fight));
            }
        }
        let _ = writeln!(out);
    }

    // ---------------------------------------------------------------- E3
    if want("e3") {
        let _ = writeln!(
            out,
            "## E3 · A cousin's move, same skeleton\n\nThe Pair and the Veilstalker are built on the Ridgeback's eighteen bones in its order, so their baked rows play on it unchanged. Each row is one donor move added to the Ridgeback as a ninth move, its distances scaled by the ratio of hip heights. *Off the body* is the gap between the move's volume and the Ridgeback's body on its first active frame.\n\n| Move | Kind | Rides | Off the body (m) | Lowest sole (m) | Thrown / landed | Hunts won | Unanswerable | Contract broken | Donor appetite, ideal range |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |"
        );
        let base = measure::fight(rb, &SEEDS);
        let _ = writeln!(
            out,
            "| (as built) | | | | | | {} / {} | {} | {} |",
            total_wins(&base),
            base.runs,
            base.unanswerable,
            broken(&base)
        );
        for donor in [&species::pair::SPECIES, &species::veilstalker::SPECIES] {
            for kind in 0..donor.moves.len() {
                let decl = donor.moves[kind];
                if decl.never_chosen {
                    continue;
                }
                let name = format!("{} · {}", donor.name, decl.name);
                let r = trial(&name, &mut out, || {
                    let mut body = Body::of(rb);
                    let note = transplant(donor, kind, &mut body, rb);
                    let s = body.install();
                    let k = s.moves.len() - 1;
                    let strike = measure::strikes(s).pop().unwrap();
                    let lo = measure::move_feet(s, k as u8);
                    let fight = measure::fight(s, &SEEDS);
                    if kind == 0 || decl.name == "Tail spear" {
                        sheet(
                            s,
                            &format!(
                                "e3-{}-{}",
                                donor.slug(),
                                decl.name.to_lowercase().replace([' ', ','], "")
                            ),
                        );
                    }
                    (note, strike, lo, fight, k)
                });
                let appetite = sim::oven::species_raw(donor.id, donor.move_index(kind, F::Weight));
                let ideal = f(sim::Fx::from_raw(sim::oven::species_raw(
                    donor.id,
                    donor.move_index(kind, F::IdealRange),
                )));
                if let Some((note, strike, lo, fight, k)) = r {
                    let _ = writeln!(
                        out,
                        "| {name} | {} | {note} | {} | {lo:+.2} | {} / {} | {} / {} | {} | {} | {appetite}, {ideal:.1} m |",
                        strike.kind,
                        if strike.gap.is_nan() {
                            "--".into()
                        } else {
                            format!("{:.2}", strike.gap)
                        },
                        fight.thrown[k],
                        fight.landed[k],
                        total_wins(&fight),
                        fight.runs,
                        fight.unanswerable,
                        broken(&fight),
                    );
                }
            }
        }
        // And the other way: the Ridgeback's moves on the Pair, whose fight
        // has hooks that read its own move table.
        let pair = &species::pair::SPECIES;
        for kind in [0usize, 2, 6] {
            let name = format!("Ridgeback · {} on the Pair", rb.moves[kind].name);
            let r = trial(&name, &mut out, || {
                let mut body = Body::of(pair);
                let note = transplant(rb, kind, &mut body, pair);
                let s = body.install();
                let k = s.moves.len() - 1;
                let strike = measure::strikes(s).pop().unwrap();
                let fight = measure::fight(s, &SEEDS);
                (note, strike, fight, k)
            });
            if let Some((note, strike, fight, k)) = r {
                let _ = writeln!(
                    out,
                    "| {name} | {} | {note} | {} | | {} / {} | {} / {} | {} | {} |",
                    strike.kind,
                    if strike.gap.is_nan() {
                        "--".into()
                    } else {
                        format!("{:.2}", strike.gap)
                    },
                    fight.thrown[k],
                    fight.landed[k],
                    total_wins(&fight),
                    fight.runs,
                    fight.unanswerable,
                    broken(&fight),
                );
            }
        }
        let _ = writeln!(out);
    }

    // ---------------------------------------------------------------- E4
    if want("e4") {
        let _ = writeln!(
            out,
            "## E4 · A move across skeletons, by role\n\nThe Ridgeback's moves read onto other skeletons by the role map in `crates/lab/src/roles.rs`: each recipient bone takes the angles of the donor bone that plays its role, and holds its own standing angles where none does.\n\n| Recipient | Recipient bones driven | Donor bones with nowhere to go | Move | Rides | Off the body (m) | Lowest sole in the move (m) | Its own worst (m) |\n| --- | --- | --- | --- | --- | --- | --- | --- |"
        );
        let mut fights = Vec::new();
        for to in [
            &species::mireback::SPECIES,
            &species::galewing::SPECIES,
            &species::mantis::SPECIES,
            &species::broodmother::SPECIES,
            &species::siegeshell::SPECIES,
            &species::sandmaw::SPECIES,
        ] {
            let (driven, n, orphans) = coverage(rb, to);
            let own = measure::feet(to).moves;
            for kind in [0usize, 1, 2, 4] {
                let name = format!("{} · {}", to.name, rb.moves[kind].name);
                let r = trial(&name, &mut out, || {
                    let mut body = Body::of(to);
                    let note = transplant(rb, kind, &mut body, to);
                    let s = body.install();
                    let k = s.moves.len() - 1;
                    let strike = measure::strikes(s).pop().unwrap();
                    let lo = measure::move_feet(s, k as u8);
                    if kind == 0 || kind == 2 {
                        sheet(
                            s,
                            &format!(
                                "e4-{}-{}",
                                to.slug(),
                                rb.moves[kind].name.to_lowercase().replace(' ', "")
                            ),
                        );
                    }
                    let fight = if kind == 0 && hunt::plans::card(to.id).is_some() {
                        Some(measure::fight(s, &SEEDS[..3]))
                    } else {
                        None
                    };
                    (note, strike, lo, fight, k)
                });
                if let Some((note, strike, lo, fight, k)) = r {
                    let _ = writeln!(
                        out,
                        "| {} | {driven} of {n} | {orphans} of {} | {} | {note} | {} | {lo:+.2} | {own:+.2} |",
                        to.name,
                        rb.bones.len(),
                        rb.moves[kind].name,
                        if strike.gap.is_nan() {
                            "--".into()
                        } else {
                            format!("{:.2}", strike.gap)
                        },
                    );
                    if let Some(fight) = fight {
                        fights.push((
                            format!(
                                "{} with the Bite (thrown {}, landed {})",
                                to.name, fight.thrown[k], fight.landed[k]
                            ),
                            fight,
                        ));
                    }
                }
            }
        }
        let _ = writeln!(out, "\nThe role map, bone by bone:\n");
        for to in [
            &species::mireback::SPECIES,
            &species::galewing::SPECIES,
            &species::mantis::SPECIES,
            &species::broodmother::SPECIES,
        ] {
            let m = roles::map(rb, to);
            let pairs: Vec<String> = m
                .iter()
                .enumerate()
                .map(|(b, d)| {
                    format!(
                        "{} <- {}",
                        to.bones[b].name,
                        d.map_or("(own)".to_string(), |d| rb.bones[d].name.to_string())
                    )
                })
                .collect();
            let _ = writeln!(out, "- **{}**: {}", to.name, pairs.join(", "));
        }
        let _ = writeln!(out, "\n{FIGHT_HEAD}");
        for (name, fight) in &fights {
            let _ = writeln!(out, "{}", fight_row(name, fight));
        }
        let _ = writeln!(out);
    }

    // ---------------------------------------------------------------- E5
    if want("e5") {
        let _ = writeln!(out, "## E5 · Traits (the Ridgeback)\n\n{FIGHT_HEAD}");
        let sandmaw = &species::sandmaw::SPECIES;
        let mireback = &species::mireback::SPECIES;
        let pair = &species::pair::SPECIES;
        type Edit = Box<dyn Fn(&mut Body)>;
        let edits: Vec<(&str, Edit)> = vec![
            (
                "hears (its own lore, no noise cells)",
                Box::new(|b| b.fight.hears = true),
            ),
            (
                "hears, with the Sandmaw's lore layout",
                Box::new(move |b| {
                    b.fight.hears = true;
                    b.fight.layout = sandmaw.fight.layout;
                }),
            ),
            (
                "sees nobody",
                Box::new(|b| b.fight.perceives = sim::perception::sees_nobody),
            ),
            (
                "sees nobody, hears, Sandmaw layout",
                Box::new(move |b| {
                    b.fight.perceives = sim::perception::sees_nobody;
                    b.fight.hears = true;
                    b.fight.layout = sandmaw.fight.layout;
                }),
            ),
            (
                "sees nobody, hears, Sandmaw layout, **and the senses row**",
                Box::new(move |b| {
                    b.fight.perceives = sim::perception::sees_nobody;
                    b.fight.hears = true;
                    b.fight.layout = sandmaw.fight.layout;
                    b.fight.row = true;
                    b.tuned.extend(senses_of(sandmaw));
                }),
            ),
            (
                "hears, Sandmaw layout, and the senses row",
                Box::new(move |b| {
                    b.fight.hears = true;
                    b.fight.layout = sandmaw.fight.layout;
                    b.fight.row = true;
                    b.tuned.extend(senses_of(sandmaw));
                }),
            ),
            (
                "collides with solids",
                Box::new(|b| b.fight.collides = true),
            ),
            (
                "lands on bodies",
                Box::new(|b| b.fight.lands_on_bodies = true),
            ),
            ("rolls over", Box::new(|b| b.fight.rolls_over = true)),
            (
                "the Mireback's hazards, declared",
                Box::new(move |b| {
                    b.fight.hazards = mireback.fight.hazards;
                    b.fight.layout = mireback.fight.layout;
                }),
            ),
            (
                "the Mireback's whole fight",
                Box::new(move |b| b.fight = *mireback.fight),
            ),
            (
                "the Sandmaw's whole fight",
                Box::new(move |b| b.fight = *sandmaw.fight),
            ),
            (
                "the Pair's whole fight",
                Box::new(move |b| b.fight = *pair.fight),
            ),
        ];
        let base = measure::fight(rb, &SEEDS);
        let _ = writeln!(out, "{}", fight_row("as built", &base));
        for (name, edit) in &edits {
            let r = trial(name, &mut out, || {
                let mut body = Body::of(rb);
                edit(&mut body);
                measure::fight(body.install(), &SEEDS)
            });
            if let Some(fight) = r {
                let _ = writeln!(out, "{}", fight_row(name, &fight));
            }
        }
        let _ = writeln!(out);
    }

    // ---------------------------------------------------------------- E6
    if want("e6") {
        let _ = writeln!(
            out,
            "## E6 · A hybrid\n\nThe Ridgeback's body with legs x1.15 (hips kept), the head as its weak point instead of the back, the Pair's Swat and the Veilstalker's Tail spear, and hearing with the Sandmaw's lore layout.\n"
        );
        let pair = &species::pair::SPECIES;
        let veil = &species::veilstalker::SPECIES;
        let sandmaw = &species::sandmaw::SPECIES;
        let r = trial("hybrid", &mut out, || {
            let base_sole = standing_sole(rb);
            let mut body = Body::of(rb);
            for l in rb.legs {
                stretch(&mut body, l.hip, 1.15, [true; 3]);
                stretch(&mut body, l.knee, 1.15, [true; 3]);
            }
            keep_feet_on_floor(&mut body, base_sole);
            body.parts
                .iter_mut()
                .for_each(|p| p.weak = p.name.contains("head"));
            let swat = pair.moves.iter().position(|m| m.name == "Swat").unwrap();
            let spear = veil
                .moves
                .iter()
                .position(|m| m.name == "Tail spear")
                .unwrap();
            transplant(pair, swat, &mut body, rb);
            transplant(veil, spear, &mut body, rb);
            body.fight.hears = true;
            body.fight.layout = sandmaw.fight.layout;
            let s = body.install();
            sheet(s, "e6-hybrid");
            (
                statics(s, &apex),
                measure::fight(s, &SEEDS),
                s.moves.iter().map(|m| m.name).collect::<Vec<_>>(),
            )
        });
        if let Some((st, fight, names)) = r {
            let _ = writeln!(
                out,
                "{STATICS_HEAD}\n{}\n\n{FIGHT_HEAD}\n{}\n",
                statics_row("hybrid", &st),
                fight_row("hybrid", &fight)
            );
            let moves: Vec<String> = names
                .iter()
                .enumerate()
                .map(|(k, n)| format!("{n} {}/{}", fight.landed[k], fight.thrown[k]))
                .collect();
            let _ = writeln!(out, "Landed / thrown, every move: {}.\n", moves.join(", "));
        }
    }

    let _ = f(sim::Fx::ZERO);
    let _ = SpeciesId::RIDGEBACK;
    print!("{out}");
}
