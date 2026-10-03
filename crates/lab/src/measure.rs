//! What a body is checked against: where its feet are, whether its gait
//! skates, whether its blows come from its body, whether anybody can climb it,
//! and how the fight against it goes.

use crate::{f, rig_of};
use sim::beast::{Pose, Rig};
use sim::monster::{Doing, Monster};
use sim::species::Species;
use sim::{Class, V3};

/// The creature's whole pose -- the baked clip and every layer and hook the
/// game puts over it (the lame layer, head tracking, `FightDecl::repose`).
fn posed(m: &Monster) -> Pose {
    m.pose()
}

/// Standing, at a point in its breath.
pub fn standing(s: &'static Species, at: f32) -> Monster {
    let mut m = Monster::new(s.id);
    m.beat = (at * 65536.0) as u16;
    m
}

/// Walking (or galloping) at a point in its stride.
pub fn striding(s: &'static Species, at: f32, gallop: bool) -> Monster {
    let mut m = Monster::new(s.id);
    m.speed = if gallop { s.gallop() } else { s.walk() };
    m.stride = (at * 65536.0) as u16;
    m
}

/// Partway through one phase of a move: 0 windup, 1 out, 2 recovery.
pub fn moving(s: &'static Species, kind: u8, phase: u8, at: f32) -> Monster {
    let a = s.attack(kind);
    let mut m = Monster::new(s.id);
    let of = |n: u16| ((n as f32 * (1.0 - at)).round() as u16).clamp(1, n.max(1));
    m.doing = match phase {
        0 => Doing::Startup {
            kind,
            left: of(a.startup),
        },
        1 => Doing::Active {
            kind,
            left: of(a.active),
        },
        _ => Doing::Recovery {
            kind,
            left: of(a.recovery),
        },
    };
    m
}

/// Points spread over a part's box, in the world: an `n`-per-side grid.
pub fn box_points(rig: &Rig, s: &Species, part: usize, n: usize) -> Vec<V3> {
    let sh = s.shape(part);
    let mut out = Vec::with_capacity(n * n * n);
    let t = |i: usize| {
        if n < 2 {
            0.5
        } else {
            i as f32 / (n - 1) as f32
        }
    };
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let lerp = |a: sim::Fx, b: sim::Fx, u: f32| crate::fx(f(a) + (f(b) - f(a)) * u);
                let local = V3::new(
                    lerp(sh.min.x, sh.max.x, t(i)),
                    lerp(sh.min.y, sh.max.y, t(j)),
                    lerp(sh.min.z, sh.max.z, t(k)),
                );
                out.push(rig.part_to_world(part, local));
            }
        }
    }
    out
}

fn foot_parts(s: &Species) -> Vec<usize> {
    if s.legs.is_empty() {
        (0..s.parts.len()).collect()
    } else {
        s.legs.iter().map(|l| l.foot).collect()
    }
}

/// The lowest point of any foot in a pose; of any part for a legless body.
pub fn lowest_feet(s: &'static Species, pose: &Pose) -> f32 {
    let rig = rig_of(s, pose);
    foot_parts(s)
        .into_iter()
        .flat_map(|p| box_points(&rig, s, p, 2))
        .map(|p| f(p.y))
        .fold(f32::MAX, f32::min)
}

/// The lowest point of the whole body in a pose.
pub fn lowest_body(s: &'static Species, pose: &Pose) -> f32 {
    let rig = rig_of(s, pose);
    (0..s.parts.len())
        .flat_map(|p| box_points(&rig, s, p, 2))
        .map(|p| f(p.y))
        .fold(f32::MAX, f32::min)
}

/// Where the feet go, across the clips that keep the animal standing.
#[derive(Debug, Default, Clone)]
pub struct Feet {
    /// Lowest sole over the idle, in metres: zero is standing on the floor,
    /// below zero is in it, above is hovering.
    pub idle: f32,
    pub idle_high: f32,
    /// Lowest sole over the walk and the gallop.
    pub gait: f32,
    /// Lowest sole over every attack's three phases, and which attack.
    pub moves: f32,
    pub worst_move: String,
    /// Lowest point of the body over the idle (a belly through the floor).
    pub belly: f32,
}

pub fn feet(s: &'static Species) -> Feet {
    if s.legs.is_empty() {
        return Feet {
            worst_move: "no legs".into(),
            ..Default::default()
        };
    }
    let mut out = Feet {
        idle: f32::MAX,
        idle_high: f32::MIN,
        gait: f32::MAX,
        moves: f32::MAX,
        belly: f32::MAX,
        ..Default::default()
    };
    for k in 0..32 {
        let p = posed(&standing(s, k as f32 / 32.0));
        let lo = lowest_feet(s, &p);
        out.idle = out.idle.min(lo);
        out.idle_high = out.idle_high.max(lo);
        out.belly = out.belly.min(lowest_body(s, &p));
    }
    for gallop in [false, true] {
        for k in 0..64 {
            let p = posed(&striding(s, k as f32 / 64.0, gallop));
            out.gait = out.gait.min(lowest_feet(s, &p));
        }
    }
    for (kind, m) in s.moves.iter().enumerate() {
        if m.unanimated {
            continue;
        }
        let lo = move_feet(s, kind as u8);
        if lo < out.moves {
            out.moves = lo;
            out.worst_move = s.moves[kind].name.to_string();
        }
    }
    out
}

/// The lowest sole through one move, posed by the creature.
pub fn move_feet(s: &'static Species, kind: u8) -> f32 {
    let mut lo = f32::MAX;
    for phase in 0..3u8 {
        for k in 0..=16 {
            let p = posed(&moving(s, kind, phase, k as f32 / 16.0));
            lo = lo.min(lowest_feet(s, &p));
        }
    }
    lo
}

/// **Skating**: how far the stride the walk animates is from the ground it
/// covers. One walk cycle is `GaitStride` metres of ground (the cycle is
/// indexed by ground covered), so a planted foot should sweep backward under
/// the body at `stride / samples` a sample. This is the mismatch between that
/// and how fast each foot actually sweeps back while it is going back, as a
/// share: zero is a foot that stays where it was put, one is a foot that does
/// not sweep at all and is dragged.
pub fn skate(s: &'static Species) -> f32 {
    if s.legs.is_empty() {
        return 0.0;
    }
    let n = 64;
    let stride = f(s.gait_stride());
    let expected = stride / n as f32;
    let mut sweep = 0.0;
    let mut count = 0;
    for leg in s.legs {
        let track: Vec<(f32, f32)> = (0..n)
            .map(|k| {
                let p = posed(&striding(s, k as f32 / n as f32, false));
                let rig = rig_of(s, &p);
                let b = box_points(&rig, s, leg.foot, 2);
                let x = b.iter().map(|q| f(q.x)).sum::<f32>() / b.len() as f32;
                let y = b.iter().map(|q| f(q.y)).fold(f32::MAX, f32::min);
                (x, y)
            })
            .collect();
        let floor = track.iter().map(|t| t.1).fold(f32::MAX, f32::min);
        let low = |t: (f32, f32)| t.1 <= floor + 0.08 * f(s.scale());
        let xs: Vec<f32> = track.iter().map(|t| t.0).collect();
        if std::env::var("LAB_SKATE").is_ok() {
            eprintln!(
                "{} foot {}: {:?}",
                s.name,
                leg.foot,
                xs.iter().map(|x| format!("{x:.2}")).collect::<Vec<_>>()
            );
        }
        let (mut leg_sweep, mut leg_n) = (0.0, 0);
        for k in 0..n {
            let dx = xs[(k + 1) % n] - xs[k];
            if dx < 0.0 && low(track[k]) && low(track[(k + 1) % n]) {
                sweep += -dx;
                count += 1;
                leg_sweep += -dx;
                leg_n += 1;
            }
        }
        if std::env::var("LAB_SKATE").is_ok() {
            eprintln!(
                "{} foot {}: {} stance samples, {:.3} a sample against {:.3}",
                s.name,
                leg.foot,
                leg_n,
                leg_sweep / leg_n.max(1) as f32,
                expected
            );
        }
    }
    if count == 0 {
        return 1.0;
    }
    (sweep / count as f32 - expected).abs() / expected
}

/// What a move's blow is, and how far its volume is from the body throwing it
/// on the frame it comes out.
#[derive(Debug, Clone)]
pub struct Strike {
    pub name: &'static str,
    /// `body` -- the volume should be on the body; `lobbed`, `travels`,
    /// `own` and `harmless` are blows that leave it on purpose or have no
    /// cylinder; `chosen` is false for a move the brain never picks.
    pub kind: &'static str,
    pub gap: f32,
}

pub fn strikes(s: &'static Species) -> Vec<Strike> {
    let mut out = Vec::new();
    for (kind, decl) in s.moves.iter().enumerate() {
        let a = s.attack(kind as u8);
        let label = if decl.unanimated {
            "brood"
        } else if decl.own_hit {
            "own"
        } else if a.damage <= 0 && !decl.harmless {
            "harmless"
        } else if decl.lobbed {
            "lobbed"
        } else if a.travel.raw() != 0 {
            "travels"
        } else {
            "body"
        };
        let m = crate::at_contact(s.id, kind as u8);
        let gap = match m.hit_volume() {
            Some((anchor, r, low, high)) => {
                let rig = m.rig();
                let (ax, az, r, low, high) = (f(anchor.x), f(anchor.z), f(r), f(low), f(high));
                (0..s.parts.len())
                    .flat_map(|p| box_points(&rig, s, p, 3))
                    .map(|q| {
                        let dh =
                            (((f(q.x) - ax).powi(2) + (f(q.z) - az).powi(2)).sqrt() - r).max(0.0);
                        let dy = (low - f(q.y)).max(f(q.y) - high).max(0.0);
                        (dh * dh + dy * dy).sqrt()
                    })
                    .fold(f32::MAX, f32::min)
            }
            None => f32::NAN,
        };
        out.push(Strike {
            name: decl.name,
            kind: label,
            gap,
        });
    }
    out
}

/// The lowest top of anything you can stand on, standing.
pub fn lowest_back(s: &'static Species) -> Option<f32> {
    let rig = standing(s, 0.0).rig();
    (0..s.parts.len())
        .filter(|i| s.parts[*i].shape.mountable && rig.boardable(*i))
        .map(|i| {
            box_points(&rig, s, i, 2)
                .iter()
                .map(|q| f(q.y))
                .fold(f32::MIN, f32::max)
        })
        .reduce(f32::min)
}

/// Every class's standing full hop, in metres.
pub fn apexes() -> Vec<(Class, f32)> {
    sim::class::ALL_CLASSES
        .iter()
        .map(|c| (*c, f(hunt::jump_apex(*c))))
        .collect()
}

/// How many classes can hop onto it from the floor, standing.
pub fn climbable_by(s: &'static Species, apex: &[(Class, f32)]) -> usize {
    match lowest_back(s) {
        Some(h) => apex.iter().filter(|(_, a)| *a >= h).count(),
        None => 0,
    }
}

// ---------------------------------------------------------------------------
// The fight
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct Fight {
    pub runs: u32,
    pub won: Vec<(Class, u32)>,
    pub unanswerable: u32,
    pub unresolved: u32,
    pub secs: f32,
    pub thrown: Vec<u32>,
    pub landed: Vec<u32>,
    /// The fight contract's clauses (`crates/hunt/tests/fight.rs`) the
    /// Champion's hunts broke.
    pub broken: Vec<&'static str>,
}

pub const SEEDS: [u32; 6] = [1, 7, 101, 2_222, 60_013, 0x2545_F491];

pub fn fight(s: &'static Species, seeds: &[u32]) -> Fight {
    let card = hunt::plans::card(s.id).expect("no hunter plan for this species");
    let n = s.moves.len();
    let mut out = Fight {
        thrown: vec![0; n],
        landed: vec![0; n],
        ..Default::default()
    };
    let mut champ = Vec::new();
    let mut frames = 0u64;
    for class in sim::class::ALL_CLASSES {
        let mut won = 0;
        for seed in seeds {
            let r = hunt::play_card_in(
                card,
                None,
                0,
                [class; sim::state::MAX_PLAYERS],
                1,
                36_000,
                *seed,
                |_| {},
            );
            out.runs += 1;
            frames += r.frames as u64;
            out.unanswerable += r.unanswerable;
            match r.outcome {
                hunt::Outcome::Killed(_) => won += 1,
                hunt::Outcome::Unresolved => out.unresolved += 1,
                hunt::Outcome::Died => {}
            }
            for k in 0..n {
                out.thrown[k] += r.starts[k];
                out.landed[k] += r.landed[k];
            }
            if class == Class::Champion {
                champ.push(r);
            }
        }
        out.won.push((class, won));
    }
    out.secs = frames as f32 / out.runs.max(1) as f32 / 60.0;
    out.broken = contract(s, &champ);
    out
}

/// The clauses of the fight contract a set of hunts breaks: the twelve in
/// `crates/hunt/tests/fight.rs`, on whatever species they were run against.
pub fn contract(s: &'static Species, hunts: &[hunt::Report]) -> Vec<&'static str> {
    use hunt::report::Threat;
    let mut out = Vec::new();
    if hunts.is_empty() {
        return out;
    }
    let r0 = &hunts[0];
    let (ans, dmg) = (r0.reactable_moves(), r0.damaging_moves());
    if !(ans * 2 > dmg && ans < dmg) {
        out.push("reactable mix");
    }
    let mut ever = vec![0u32; s.moves.len()];
    for r in hunts {
        for (k, e) in ever.iter_mut().enumerate() {
            *e += r.starts[k];
        }
    }
    if s.moves
        .iter()
        .enumerate()
        .any(|(k, m)| !m.never_chosen && !m.unanimated && ever[k] == 0)
    {
        out.push("whole move set");
    }
    if hunts
        .iter()
        .any(|r| r.dominant_share() >= 0.55 || r.longest_repeat > 6)
    {
        out.push("no single move");
    }
    let committed = sim::moves::get(Class::Champion, sim::state::SLOT_COMMITTED);
    let needed = (committed.startup + committed.active) as f32;
    if hunts.iter().any(|r| r.mean_opening() <= needed) {
        out.push("openings punishable");
    }
    if hunts
        .iter()
        .any(|r| r.idle_share() >= 0.35 || r.moves_per_minute() <= 20.0)
    {
        out.push("never idle");
    }
    if hunts.iter().any(|r| {
        let t = r.threat_share(Threat::Threatening);
        let w = r.threat_share(Threat::WalkUp);
        let b = r.threat_share(Threat::PokeOnly) + r.threat_share(Threat::Skilled);
        !(0.35..0.62).contains(&t) || w <= 0.12 || b <= 0.2
    }) {
        out.push("threat bands");
    }
    let rides: u32 = hunts.iter().map(|r| r.rides).sum();
    let thrown: u32 = hunts.iter().map(|r| r.thrown + r.fled).sum();
    let ridge: u32 = hunts.iter().map(|r| r.ridge_hits).sum();
    if rides == 0
        || ridge == 0
        || thrown * 4 <= rides
        || hunts.iter().any(|r| r.ride_share() >= 0.8)
    {
        out.push("back reachable, not safe");
    }
    let poke = sim::moves::get(Class::Champion, sim::state::SLOT_POKE);
    let ride_frames: u32 = hunts.iter().map(|r| r.ride_frames).sum();
    let ride_n: u32 = hunts.iter().map(|r| r.rides.max(1)).sum();
    if ride_frames as f32 / ride_n as f32 <= poke.whiff_cost() as f32 {
        out.push("ride long enough");
    }
    if hunts.iter().map(|r| r.topples).sum::<u32>() == 0 {
        out.push("poise breaks");
    }
    if hunts.iter().any(|r| r.unanswerable > 0) {
        out.push("unanswerable zero");
    }
    if hunts
        .iter()
        .any(|r| matches!(r.outcome, hunt::Outcome::Unresolved))
    {
        out.push("hunt concludes");
    }
    let won = hunts
        .iter()
        .filter(|r| matches!(r.outcome, hunt::Outcome::Killed(_)))
        .count();
    if won == 0 || won == hunts.len() {
        out.push("neither free nor hopeless");
    }
    if hunts.iter().any(|r| f(r.spread) <= 4.0) {
        out.push("uses the arena");
    }
    out
}

/// Where the bone a move rides is, and where its volume is, on the frame its
/// hit comes out. `None` for a move with no cylinder.
pub fn contact(s: &'static Species, kind: u8) -> Option<(V3, V3)> {
    let m = crate::at_contact(s.id, kind);
    let (anchor, ..) = m.hit_volume()?;
    let bone = s.follow_bone(s.attack(kind).follows);
    Some((m.rig().bone[bone].at, anchor))
}

/// **Drift**: how far the bone a move rides has moved, relative to where its
/// blow lands, between two bodies -- flat metres. A blow is authored in body
/// space and carried only by the bone's motion *from rest*, so a longer neck
/// puts the head somewhere the bite's volume does not follow.
pub fn drift(before: &[Option<(V3, V3)>], after: &[Option<(V3, V3)>]) -> (f32, usize) {
    let mut worst = (0.0, 0);
    for (k, (a, b)) in before.iter().zip(after).enumerate() {
        if let (Some((ba, va)), Some((bb, vb))) = (a, b) {
            let dx = (f(bb.x) - f(ba.x)) - (f(vb.x) - f(va.x));
            let dz = (f(bb.z) - f(ba.z)) - (f(vb.z) - f(va.z));
            let d = (dx * dx + dz * dz).sqrt();
            if d > worst.0 {
                worst = (d, k);
            }
        }
    }
    worst
}
