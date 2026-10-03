//! **Numbers as differences in kind** -- the sweep behind
//! `docs/design/exploration/0006_differences_in_kind.md`.
//!
//!     cargo run --release -p hunt --bin spread -- --regime base --seeds 240
//!     cargo run --release -p hunt --bin spread -- --regime mutate --genomes 400 --shard 0/4
//!     cargo run --release -p hunt --bin spread -- --regime wild --genomes 200 --shard 1/4
//!     cargo run --release -p hunt --bin spread -- --regime oat --shard 2/4
//!
//! A *setting* is a genome: a value for some of a species' Oven knobs, the
//! rest as baked. Every setting is hunted by the scripted hunter of every
//! class over the same seeds, so two settings differ only in the genome. One
//! row per hunt goes to `<out>/<regime>.<shard>.hunts.csv`, and the changed
//! knobs of every setting to `<out>/<regime>.<shard>.genomes.csv`;
//! `scripts/spread.py` reads them.
//!
//! The Oven's cells are process-global, so the parallelism is processes:
//! `--shard i/n` takes every n-th setting starting at i. Nothing is baked and
//! nothing is written outside `--out`. The random numbers are this tool's own
//! and are not the simulation's: a genome is drawn here, then the hunt is
//! exactly the deterministic hunt `fight` plays.

use std::fs::File;
use std::io::{BufWriter, Write};

use hunt::report::Threat;
use hunt::{Outcome, Report};
use sim::oven::{Knob, Tunable};
use sim::species::{Common, Species};
use sim::state::MAX_PLAYERS;

fn arg(name: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

/// A small generator of the tool's own (xorshift64*), for drawing genomes.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// Uniform in `[0, 1)`.
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// The knobs that are genes: every knob of the species but those that are not
/// a continuous magnitude about the fight (0006 §3).
fn genes(s: &'static Species) -> Vec<Knob> {
    use sim::oven::MonsterField as F;
    sim::oven::species_knobs(s)
        .into_iter()
        .filter(|k| match k {
            Knob::Species(_, Tunable::Common(c)) => !matches!(
                c,
                Common::Margin
                    | Common::Spawn
                    | Common::HunterSpawn
                    | Common::HuntGrace
                    | Common::GaitStride
                    | Common::BreathRate
                    | Common::HeadTrack
            ),
            Knob::Species(_, Tunable::Move(_, f)) => !matches!(f, F::Follows | F::Unblockable),
            _ => true,
        })
        .collect()
}

/// A frame count a move cannot do without.
fn needs_a_frame(k: &Knob) -> bool {
    use sim::oven::MonsterField as F;
    matches!(
        k,
        Knob::Species(_, Tunable::Move(_, F::Startup | F::Active | F::Recovery))
    )
}

fn clamp(k: &Knob, v: i64) -> i32 {
    let (lo, hi) = k.range();
    let lo = if needs_a_frame(k) { lo.max(1) } else { lo };
    v.clamp(lo as i64, hi as i64) as i32
}

struct Setting {
    id: u32,
    /// The temper it is hunted at: zero but in the `temper` regime, the
    /// control -- tempers are pure difficulty by design (world.md §4).
    temper: u8,
    /// Changed knobs: index into the gene list, value.
    genes: Vec<(usize, i32)>,
}

fn settings(regime: &str, genes: &[Knob], count: u32, seed: u64) -> Vec<Setting> {
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let baked: Vec<i32> = genes.iter().map(|k| k.baked_raw()).collect();
    match regime {
        "base" => vec![Setting {
            id: 0,
            temper: 0,
            genes: Vec::new(),
        }],
        "temper" => (0..=sim::temper::HIGHEST)
            .map(|t| Setting {
                id: t as u32,
                temper: t,
                genes: Vec::new(),
            })
            .collect(),
        "mutate" => (0..count)
            .map(|id| {
                let mut out = Vec::new();
                for (i, k) in genes.iter().enumerate() {
                    if rng.unit() < 0.25 {
                        let f = 0.7 + 0.6 * rng.unit();
                        let v = clamp(k, (baked[i] as f64 * f).round() as i64);
                        if v != baked[i] {
                            out.push((i, v));
                        }
                    }
                }
                Setting {
                    id,
                    temper: 0,
                    genes: out,
                }
            })
            .collect(),
        "wild" => (0..count)
            .map(|id| {
                let mut out = Vec::new();
                for (i, k) in genes.iter().enumerate() {
                    let (lo, hi) = k.range();
                    let v = clamp(
                        k,
                        lo as i64 + ((hi - lo) as f64 * rng.unit()).round() as i64,
                    );
                    if v != baked[i] {
                        out.push((i, v));
                    }
                }
                Setting {
                    id,
                    temper: 0,
                    genes: out,
                }
            })
            .collect(),
        "oat" => {
            let only = arg("--knob");
            let mut out = Vec::new();
            let mut id = 0;
            for (i, k) in genes.iter().enumerate() {
                if only.as_deref().is_some_and(|o| o != k.id()) {
                    continue;
                }
                let (lo, hi) = k.range();
                // Seven across the range, five across +-30% of tuned.
                let mut levels: Vec<i32> = (0..7)
                    .map(|n| clamp(k, lo as i64 + (hi - lo) as i64 * n / 6))
                    .collect();
                for f in [0.7, 0.85, 1.0, 1.15, 1.3] {
                    levels.push(clamp(k, (baked[i] as f64 * f).round() as i64));
                }
                for v in levels {
                    out.push(Setting {
                        id,
                        temper: 0,
                        genes: vec![(i, v)],
                    });
                    id += 1;
                }
            }
            out
        }
        other => {
            eprintln!("no regime called {other}: base, temper, mutate, wild or oat");
            std::process::exit(2);
        }
    }
}

fn ratio(a: f32, b: f32) -> f32 {
    if b > 0.0 { a / b } else { 0.0 }
}

fn header(moves: usize) -> String {
    let mut h = String::from(
        "setting,class,seed,outcome,frames,fought,health_left,dealt,taken,hits_taken,\
         reactable,committed,reactable_moves,damaging_moves,openings,open_frames,\
         shortest_opening,idle_frames,longest_repeat,threat0,threat1,threat2,threat3,threat4,\
         ride_frames,rides,longest_ride,thrown,fled,swings,connected,unanswerable,ridge_hits,\
         topples,legs_broken,foot_damage,spread,commit_range,aboard_commits,beats,uses",
    );
    for k in 0..moves {
        h.push_str(&format!(",start{k}"));
    }
    for k in 0..moves {
        h.push_str(&format!(",landed{k}"));
    }
    h
}

fn row(setting: u32, class: sim::Class, seed: u32, r: &Report, moves: usize) -> String {
    let (outcome, frames) = match r.outcome {
        Outcome::Killed(f) => ("won", f),
        Outcome::Died => ("lost", r.frames),
        Outcome::Unresolved => ("unresolved", r.frames),
    };
    // Where the fight was: the mean range at commit, and how many commits came
    // with somebody aboard. From the timeline's move starts, not its hits.
    let starts: Vec<_> = r.timeline.iter().filter(|b| b.hit.is_none()).collect();
    let range: f32 = starts.iter().map(|b| b.range.to_f32_for_render()).sum();
    let aboard = starts.iter().filter(|b| b.aboard).count();
    let uses: Vec<String> = r
        .uses
        .lines(class)
        .iter()
        .map(|(_, n, _)| n.to_string())
        .collect();
    let mut s = format!(
        "{setting},{},{seed},{outcome},{frames},{},{},{},{},{},{},{},{},{},{},{},{},{},{},",
        class.name(),
        r.fought,
        r.health_left,
        r.dealt,
        r.taken,
        r.hits_taken,
        r.reactable,
        r.committed,
        r.reactable_moves(),
        r.damaging_moves(),
        r.openings,
        r.open_frames,
        r.shortest_opening,
        r.idle_frames,
        r.longest_repeat,
    );
    s.push_str(&format!(
        "{},{},{},{},{},",
        r.threat[Threat::Threatening as usize],
        r.threat[Threat::PokeOnly as usize],
        r.threat[Threat::Skilled as usize],
        r.threat[Threat::WalkUp as usize],
        r.threat[Threat::Guarded as usize],
    ));
    s.push_str(&format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{:.3},{:.3},{},{},{}",
        r.ride_frames,
        r.rides,
        r.longest_ride,
        r.thrown,
        r.fled,
        r.swings,
        r.connected,
        r.unanswerable,
        r.ridge_hits,
        r.topples,
        r.legs_broken,
        r.foot_damage,
        r.spread.to_f32_for_render(),
        ratio(range, starts.len() as f32),
        aboard,
        starts.len(),
        uses.join("|"),
    ));
    for k in 0..moves {
        s.push_str(&format!(",{}", r.starts[k]));
    }
    for k in 0..moves {
        s.push_str(&format!(",{}", r.landed[k]));
    }
    s
}

fn main() {
    let species = arg("--species").unwrap_or_else(|| "ridgeback".into());
    let Some(s) = sim::species::named(&species) else {
        eprintln!("no species called {species}");
        std::process::exit(2);
    };
    let Some(card) = hunt::plans::card(s.id) else {
        eprintln!("{} has no hunter plan", s.name);
        std::process::exit(2);
    };
    let regime = arg("--regime").unwrap_or_else(|| "mutate".into());
    let count: u32 = arg("--genomes").and_then(|n| n.parse().ok()).unwrap_or(400);
    let seeds: u32 = arg("--seeds").and_then(|n| n.parse().ok()).unwrap_or(12);
    let limit: u32 = arg("--frames")
        .and_then(|n| n.parse().ok())
        .unwrap_or(36_000);
    let draw: u64 = arg("--seed").and_then(|n| n.parse().ok()).unwrap_or(1);
    let (shard, shards) = arg("--shard")
        .and_then(|s| {
            let (a, b) = s.split_once('/')?;
            Some((a.parse::<usize>().ok()?, b.parse::<usize>().ok()?))
        })
        .unwrap_or((0, 1));
    let out = arg("--out").unwrap_or_else(|| "target/spread".into());
    let classes: Vec<sim::Class> = match arg("--classes").as_deref() {
        None | Some("all") => sim::class::ALL_CLASSES.to_vec(),
        Some(list) => list
            .split(',')
            .filter_map(|n| {
                sim::class::ALL_CLASSES
                    .iter()
                    .copied()
                    .find(|c| c.name().to_lowercase().contains(&n.to_lowercase()))
            })
            .collect(),
    };

    let genes = genes(s);
    let all = settings(&regime, &genes, count, draw);
    std::fs::create_dir_all(&out).expect("cannot make the output directory");
    let stem = format!("{out}/{regime}.{shard}");
    let mut hunts = BufWriter::new(File::create(format!("{stem}.hunts.csv")).unwrap());
    let mut genomes = BufWriter::new(File::create(format!("{stem}.genomes.csv")).unwrap());
    let moves = s.moves.len();
    writeln!(hunts, "{}", header(moves)).unwrap();
    writeln!(genomes, "setting,knob,unit,value,baked,lo,hi").unwrap();

    // The same seeds for every setting: two genomes differ only in the genome.
    let seed_list: Vec<u32> = (0..seeds)
        .map(|n| 0x2545_F491u32.wrapping_add(n.wrapping_mul(0x9E37_79B9)))
        .collect();

    let mine: Vec<&Setting> = all.iter().skip(shard).step_by(shards.max(1)).collect();
    let started = std::time::Instant::now();
    for (done, setting) in mine.iter().enumerate() {
        sim::oven::reset_to_baked();
        for (i, v) in &setting.genes {
            genes[*i].set_raw(*v);
            let (lo, hi) = genes[*i].range();
            writeln!(
                genomes,
                "{},{},{:?},{},{},{},{}",
                setting.id,
                genes[*i].id().replace(',', ";"),
                genes[*i].unit(),
                v,
                genes[*i].baked_raw(),
                lo,
                hi
            )
            .unwrap();
        }
        if setting.genes.is_empty() {
            writeln!(genomes, "{},,,,,,", setting.id).unwrap();
        }
        for class in &classes {
            for seed in &seed_list {
                let r = hunt::play_card_in(
                    card,
                    None,
                    setting.temper,
                    [*class; MAX_PLAYERS],
                    1,
                    limit,
                    *seed,
                    |_| {},
                );
                writeln!(hunts, "{}", row(setting.id, *class, *seed, &r, moves)).unwrap();
            }
        }
        if done % 10 == 9 || done + 1 == mine.len() {
            hunts.flush().unwrap();
            genomes.flush().unwrap();
            eprintln!(
                "{regime} {shard}/{shards}: {}/{} settings, {:.0}s",
                done + 1,
                mine.len(),
                started.elapsed().as_secs_f32()
            );
        }
    }
    sim::oven::reset_to_baked();
}
