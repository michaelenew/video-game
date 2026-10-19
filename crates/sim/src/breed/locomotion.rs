//! Locomotion: which gait a body uses at a speed, and where every foot is
//! at each moment of it.
//!
//! **The Froude number** decides the gait. It is a speed squared over
//! gravity times hip height, `v^2 / (g h)` -- a speed with the body's size
//! taken out -- and animals of every size change gait at about the same
//! values of it: a walk below a half, a trot to about two and a half, a
//! gallop above (Alexander and Jayes, 1983). So a gnawer and the Ridgeback
//! at the same Froude number move alike, the gnawer simply faster in its
//! cadence; and a crossbred body picks its gait from its own legs, without
//! a table per species.
//!
//! **Stride and cadence follow.** Stride length is about `2.3 h Fr^0.3`
//! across mammals (Alexander, 1976), and the cadence is speed over stride.
//! A big body takes long, slow strides and a small one short, quick ones,
//! at the same Froude number, which is what makes a giant read as heavy.
//!
//! **The pattern is the leg count's**, from the animals that have it:
//!
//! | Legs | Slow | Faster | Fastest |
//! | --- | --- | --- | --- |
//! | 2 | walk (alternating) | run | hop, if its hind legs are long (a hopper) |
//! | 4 | lateral-sequence walk | trot (diagonal pairs) | rotary gallop, or a bound on sprung legs |
//! | 6 | wave (back to front, a side at a time) | tripod | tripod |
//! | 8 | wave | alternating tetrapod (a spider's) | tetrapod |
//! | 0 | slither: a wave down the body | | |
//!
//! In each, a foot on the floor stays where it was put while the body
//! passes over it (its stance, a duty factor of the cycle), then lifts and
//! swings forward. The body bobs twice a cycle in a walk or trot and rocks
//! nose-to-tail once in a gallop or bound; the head is held steady against
//! the body's rock, as every animal's is; the tail swings opposite to the
//! body.

use super::build::{Proportions, proportions};
use super::drive::{Drive, Foot, Wings};
use super::posture::Posture;
use super::{Build, Genome, Role, STATIONS, Stance, Station, fx};
use crate::fixed::{Fx, cos_turns, sin_turns};

/// Gravity, m/s^2: the Froude number's.
pub const G: Fx = fx(981, 100);

/// Frames a second the simulation runs at.
pub const FPS: i32 = 60;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum GaitKind {
    Stand,
    Walk,
    Trot,
    Gallop,
    Bound,
    Run,
    Hop,
    /// Six or eight legs, slow: a wave from the hind legs forward, one
    /// side half a cycle behind the other.
    Wave,
    /// Six legs, fast: three feet down at a time.
    Tripod,
    /// Eight legs, fast: four feet down at a time.
    Tetrapod,
    /// Legs too weak for the body: a belly on the floor, the legs paddling.
    Crawl,
    Slither,
    Fly,
}

impl GaitKind {
    pub const fn name(self) -> &'static str {
        match self {
            GaitKind::Stand => "stand",
            GaitKind::Walk => "walk",
            GaitKind::Trot => "trot",
            GaitKind::Gallop => "gallop",
            GaitKind::Bound => "bound",
            GaitKind::Run => "run",
            GaitKind::Hop => "hop",
            GaitKind::Wave => "wave",
            GaitKind::Tripod => "tripod",
            GaitKind::Tetrapod => "tetrapod",
            GaitKind::Crawl => "crawl",
            GaitKind::Slither => "slither",
            GaitKind::Fly => "fly",
        }
    }

    /// How much of a cycle a foot is on the floor.
    const fn duty(self) -> Fx {
        match self {
            GaitKind::Stand => fx(1, 1),
            GaitKind::Walk => fx(13, 20),
            GaitKind::Trot => fx(1, 2),
            GaitKind::Gallop => fx(2, 5),
            GaitKind::Bound => fx(7, 20),
            GaitKind::Run => fx(7, 20),
            GaitKind::Hop => fx(3, 10),
            GaitKind::Wave => fx(3, 4),
            GaitKind::Tripod => fx(11, 20),
            GaitKind::Tetrapod => fx(3, 5),
            GaitKind::Crawl => fx(7, 10),
            GaitKind::Slither | GaitKind::Fly => fx(1, 1),
        }
    }
}

/// A gait at a speed: everything a cycle of it needs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Gait {
    pub kind: GaitKind,
    /// Its Froude number.
    pub froude: Fx,
    /// Metres a second.
    pub speed: Fx,
    /// Metres a cycle.
    pub stride: Fx,
    /// Frames a cycle.
    pub cycle: i32,
    pub duty: Fx,
    /// Each foot's place in the cycle, by station and side.
    pub offset: [[Fx; 2]; STATIONS],
    /// Which feet walk in this gait (a walked-on wing counts).
    pub walks: [bool; STATIONS],
}

/// The pairs that walk, fore to hind: legs, and wings the posture leans on.
fn walking_pairs(g: &Genome, q: &Posture) -> ([usize; STATIONS], usize) {
    let mut out = [0; STATIONS];
    let mut n = 0;
    for st in Station::ALL {
        let i = st.index();
        if q.stations[i].grounded {
            out[n] = i;
            n += 1;
        }
    }
    let _ = g;
    (out, n)
}

/// `x^0.3`, for the stride's law, from a table: there is no logarithm in
/// fixed point and the law is only good to a few percent anyway.
fn pow_three_tenths(x: Fx) -> Fx {
    const KNOTS: [(Fx, Fx); 8] = [
        (fx(5, 100), fx(407, 1000)),
        (fx(1, 10), fx(501, 1000)),
        (fx(2, 10), fx(617, 1000)),
        (fx(5, 10), fx(812, 1000)),
        (fx(1, 1), fx(1, 1)),
        (fx(2, 1), fx(1231, 1000)),
        (fx(3, 1), fx(1390, 1000)),
        (fx(5, 1), fx(1621, 1000)),
    ];
    if x <= KNOTS[0].0 {
        return KNOTS[0].1;
    }
    for w in KNOTS.windows(2) {
        let (a, b) = (w[0], w[1]);
        if x <= b.0 {
            let t = x.sub(a.0).div(b.0.sub(a.0));
            return a.1.add(b.1.sub(a.1).mul(t));
        }
    }
    KNOTS[7].1
}

/// The highest Froude number the body reaches: its speed slider, held
/// back by weight, by a low splayed stance, and to a crawl by legs too
/// weak for it.
pub fn top_froude(g: &Genome, q: &Posture) -> Fx {
    if q.bellied {
        return fx(1, 10);
    }
    let s = &g.sliders;
    let mut top = fx(3, 10)
        .add(s.speed.mul(Fx::from_int(4)))
        .mul(fx(13, 10).sub(s.mass.mul(fx(3, 5))));
    if g.stance == Stance::Lowslung {
        top = top.mul(fx(3, 5));
    }
    // A slither is slow for its height: a snake's best is a jogger's.
    if g.stance == Stance::Limbless {
        top = top.mul(fx(1, 3));
    }
    top.max(fx(1, 5))
}

/// The gait at a share of top speed (nought standing, one flat out).
pub fn at(g: &Genome, q: &Posture, share: Fx) -> Gait {
    let p = proportions(g);
    let h = if g.stance == Stance::Limbless {
        p.half_w.mul(Fx::from_int(4))
    } else {
        q.root.max(fx(1, 10))
    };
    let froude = top_froude(g, q).mul(super::unit(share));
    let (pairs, n) = walking_pairs(g, q);
    let hopper = q.hop > fx(3, 10);
    let springy = (0..n).all(|k| {
        g.limbs[pairs[k]].is_some_and(|l| l.build == Build::Sprung || l.role == Role::Wing)
    });
    let kind = if share <= Fx::ZERO {
        GaitKind::Stand
    } else if g.stance == Stance::Limbless {
        GaitKind::Slither
    } else if q.bellied {
        GaitKind::Crawl
    } else {
        match n {
            0 => GaitKind::Slither,
            1 => {
                if froude < fx(1, 2) {
                    GaitKind::Walk
                } else if hopper {
                    GaitKind::Hop
                } else {
                    GaitKind::Run
                }
            }
            2 => {
                if froude < fx(1, 2) {
                    GaitKind::Walk
                } else if froude < fx(5, 2) {
                    GaitKind::Trot
                } else if springy || hopper {
                    GaitKind::Bound
                } else {
                    GaitKind::Gallop
                }
            }
            3 => {
                if froude < fx(3, 10) {
                    GaitKind::Wave
                } else {
                    GaitKind::Tripod
                }
            }
            _ => {
                if froude < fx(3, 10) {
                    GaitKind::Wave
                } else {
                    GaitKind::Tetrapod
                }
            }
        }
    };
    let speed = froude.mul(G).mul(h).sqrt();
    // Stride: 2.3 h Fr^0.3; a crawl and a slither a body length a cycle.
    let stride = match kind {
        GaitKind::Slither | GaitKind::Crawl => p.len.mul(fx(2, 5)),
        _ => h
            .mul(fx(23, 10))
            .mul(pow_three_tenths(froude.max(fx(5, 100)))),
    };
    // No longer than the shortest walking leg can cover while its foot is
    // down: past that a planted foot would slide, so the body takes more,
    // shorter steps instead -- a quicker cadence, the same speed.
    let shortest = (0..n)
        .map(|k| p.limb[pairs[k]])
        .fold(Fx::from_int(1000), |a, b| a.min(b));
    let stride = if n > 0 && !matches!(kind, GaitKind::Slither | GaitKind::Crawl) {
        let duty = kind.duty();
        stride.min(MAX_REACH.mul(Fx::from_int(2)).mul(shortest).div(duty))
    } else {
        stride
    };
    let cycle = if speed > fx(1, 100) {
        // frames = stride / speed * FPS
        stride
            .div(speed)
            .mul(Fx::from_int(FPS))
            .to_int()
            .clamp(8, 240)
    } else {
        FPS
    };
    let mut offset = [[Fx::ZERO; 2]; STATIONS];
    let mut walks = [false; STATIONS];
    for &i in &pairs[..n] {
        walks[i] = true;
    }
    // Fore to hind, `k` 0 is the front pair.
    for (k, &i) in pairs[..n].iter().enumerate() {
        let last = n - 1;
        let (l, r) = match (kind, n) {
            (GaitKind::Walk | GaitKind::Crawl, 1) | (GaitKind::Run, 1) => (fx(0, 1), fx(1, 2)),
            (GaitKind::Hop, _) => (Fx::ZERO, Fx::ZERO),
            // Lateral sequence: hind left, fore left, hind right, fore
            // right, a quarter apart. A foot lands at `1 - offset` of the
            // cycle (`foot_at` plants it while phase plus offset is under
            // the duty), so a later landing is a smaller offset.
            (GaitKind::Walk | GaitKind::Crawl, _) if n == 2 => {
                if k == 0 {
                    (fx(3, 4), fx(1, 4))
                } else {
                    (Fx::ZERO, fx(1, 2))
                }
            }
            (GaitKind::Trot, _) => {
                if k == 0 {
                    (Fx::ZERO, fx(1, 2))
                } else {
                    (fx(1, 2), Fx::ZERO)
                }
            }
            // Rotary gallop: the hind pair strikes close together, then
            // the fore pair half a cycle later.
            (GaitKind::Gallop, _) => {
                if k == 0 {
                    (fx(1, 2), fx(3, 5))
                } else {
                    (Fx::ZERO, fx(1, 10))
                }
            }
            (GaitKind::Bound, _) => {
                if k == 0 {
                    (fx(1, 2), fx(1, 2))
                } else {
                    (Fx::ZERO, Fx::ZERO)
                }
            }
            // Three or four feet down at a time, alternating down the
            // body: left front, right middle, left hind...
            (GaitKind::Tripod | GaitKind::Tetrapod, _) => {
                if k % 2 == 0 {
                    (Fx::ZERO, fx(1, 2))
                } else {
                    (fx(1, 2), Fx::ZERO)
                }
            }
            // A wave from the hind pair forward, each pair a share of the
            // cycle ahead of the one behind it, the right side half a
            // cycle after the left.
            _ => {
                let from_back = (last - k) as i32;
                let step = wrap(Fx::ONE.sub(Fx::ratio(from_back, n as i32)));
                (step, wrap(step.add(fx(1, 2))))
            }
        };
        offset[i] = [l, r];
    }
    Gait {
        kind,
        froude,
        speed,
        stride,
        cycle,
        duty: kind.duty(),
        offset,
        walks,
    }
}

fn wrap(x: Fx) -> Fx {
    let w = x.sub(Fx::from_int(x.to_int()));
    if w < Fx::ZERO { w.add(Fx::ONE) } else { w }
}

fn smooth(t: Fx) -> Fx {
    let t = super::unit(t);
    t.mul(t).mul(Fx::from_int(3).sub(t.mul(Fx::from_int(2))))
}

/// Half a planted foot's travel under the body, as a share of its
/// standing length, and the most a leg allows.
pub fn reach_share(gait: &Gait, standing: Fx) -> Fx {
    gait.stride
        .mul(gait.duty)
        .div(standing.max(fx(1, 100)))
        .mul(fx(1, 2))
        .min(MAX_REACH)
}

/// The furthest a foot goes ahead of or behind its standing spot, as a
/// share of the leg.
pub const MAX_REACH: Fx = fx(9, 20);

/// Where a foot is at a point of the cycle, in shares of its standing
/// length: planted and passing back under the body through its stance,
/// lifted and swung forward through the rest.
pub fn foot_at(gait: &Gait, p: &Proportions, i: usize, side: usize, phase: Fx, turn: Fx) -> Foot {
    if gait.kind == GaitKind::Stand {
        return Foot::default();
    }
    let standing = p.limb[i].max(fx(1, 100));
    // How far a planted foot passes back under the body: as far as the
    // body travels while it is down, a duty's share of the stride, so it
    // stays where it was put. A leg reaches only so far; past that, it
    // slides, and the body would have to take more steps.
    let mut half = reach_share(gait, standing);
    // Turning: the outside feet stride further, the inside ones less.
    let outside = if side == 0 { turn.neg() } else { turn };
    half = half.mul(Fx::ONE.add(outside.mul(fx(3, 10))));
    let lift_peak = match gait.kind {
        GaitKind::Walk | GaitKind::Wave | GaitKind::Crawl => fx(3, 20),
        GaitKind::Trot | GaitKind::Tripod | GaitKind::Tetrapod => fx(1, 5),
        _ => fx(1, 4),
    };
    let pp = wrap(phase.add(gait.offset[i][side]));
    let duty = gait.duty;
    if pp < duty {
        // Planted: from ahead of its spot to behind it.
        Foot {
            forward: half.sub(half.mul(Fx::from_int(2)).mul(pp.div(duty))),
            lift: Fx::ZERO,
        }
    } else {
        let s = pp.sub(duty).div(Fx::ONE.sub(duty));
        Foot {
            forward: half.neg().add(half.mul(Fx::from_int(2)).mul(smooth(s))),
            lift: lift_peak.mul(sin_turns(s.mul(fx(1, 2)))),
        }
    }
}

/// The whole body at a point of a gait: feet, bob, rock, a steady head, a
/// swinging tail, and a turn leaned into.
pub fn drive(g: &Genome, q: &Posture, gait: &Gait, phase: Fx, turn: Fx) -> Drive {
    let p = proportions(g);
    let mut d = Drive::STILL;
    if gait.kind == GaitKind::Stand {
        return rest(g);
    }
    let h = q.root.max(fx(1, 10));
    let two = phase.mul(Fx::from_int(2));
    match gait.kind {
        GaitKind::Walk | GaitKind::Wave | GaitKind::Crawl => {
            d.lift = h.mul(fx(3, 200)).mul(cos_turns(two));
            d.tail_yaw = sin_turns(phase).mul(fx(1, 60));
        }
        GaitKind::Trot | GaitKind::Run | GaitKind::Tripod | GaitKind::Tetrapod => {
            d.lift = h.mul(fx(3, 100)).mul(cos_turns(two));
            d.tail_yaw = sin_turns(phase).mul(fx(1, 80));
        }
        GaitKind::Gallop | GaitKind::Bound | GaitKind::Hop => {
            // Once a cycle: up at push-off, down at landing, and the body
            // rocks nose-to-tail about it -- the spine's flex.
            d.lift = h.mul(fx(1, 20)).mul(sin_turns(phase));
            d.pitch = fx(1, 40).mul(sin_turns(phase.add(fx(1, 4))));
            d.tail_pitch = fx(1, 80).mul(sin_turns(phase));
        }
        GaitKind::Slither => {
            d.wave = Some((phase, fx(1, 12)));
        }
        GaitKind::Fly | GaitKind::Stand => {}
    }
    // The head held steady against the body's rock, and turned into a turn.
    d.neck_pitch = d.pitch.neg();
    d.neck_yaw = turn.mul(fx(1, 16));
    d.tail_yaw = d.tail_yaw.sub(turn.mul(fx(1, 50)));
    for st in Station::ALL {
        let i = st.index();
        if !gait.walks[i] {
            continue;
        }
        for side in 0..2 {
            d.feet[i][side] = foot_at(gait, &p, i, side, phase, turn);
        }
    }
    // A striking pair carries itself: the leading arm a little out.
    d.arms = [fx(3, 10), Fx::ZERO];
    d
}

/// The body in flight at a point of the wingbeat: wings beating, the body
/// riding a little opposite to them, legs trailing. A body whose wings
/// cannot lift it flares them where it stands.
pub fn flight(g: &Genome, q: &Posture, phase: Fx) -> Drive {
    let mut d = rest(g);
    let flies = super::plan::assess(g).caps.flies;
    d.wings = if flies { Wings::Aloft } else { Wings::Flared };
    if flies {
        d.beat = sin_turns(phase).mul(fx(1, 8));
        d.lift = q.root.mul(fx(1, 20)).mul(sin_turns(phase.add(fx(1, 2))));
    }
    d
}

/// How long a wingbeat is, in frames: a long wing beats slowly.
pub fn wingbeat(g: &Genome) -> i32 {
    let p = proportions(g);
    let wing = g
        .pairs()
        .filter(|(_, l)| l.role == Role::Wing)
        .map(|(st, _)| p.limb[st.index()])
        .fold(Fx::ZERO, |a, b| a.max(b));
    // About 3 Hz for a 5 m wing, slower as the square root of length.
    let hz = fx(3, 1).mul(fx(5, 1).div(wing.max(fx(1, 2))).sqrt());
    Fx::from_int(FPS)
        .div(hz.max(fx(1, 2)))
        .to_int()
        .clamp(6, 120)
}

/// Standing, at ease: a little weight on one side, the head turned a
/// hair, the tail not quite straight. Rolled once from the genome, so each
/// creature has its own.
pub fn rest(g: &Genome) -> Drive {
    let mut rng = super::Rng::new(g.fingerprint() ^ 0x5e1f);
    let lead = if rng.chance(fx(1, 2)) {
        Fx::ONE
    } else {
        Fx::ONE.neg()
    };
    let mut d = Drive::STILL;
    d.neck_yaw = rng.unit().mul(fx(1, 30)).mul(lead);
    d.tail_yaw = rng.unit().mul(fx(1, 40)).mul(lead.neg());
    for i in 0..STATIONS {
        d.feet[i][0].forward = fx(-1, 16).mul(lead);
        d.feet[i][1].forward = fx(1, 16).mul(lead);
    }
    d
}

/// Every gait the body has, slowest first: a walk, and what it changes up
/// to as it goes faster.
pub fn repertoire(g: &Genome, q: &Posture) -> [Option<Gait>; 4] {
    let mut out = [None; 4];
    let mut n = 0;
    for k in 1..=20 {
        let gait = at(g, q, Fx::ratio(k, 20));
        if n == 0 || out[n - 1].map(|o: Gait| o.kind) != Some(gait.kind) {
            if n < 4 {
                out[n] = Some(gait);
                n += 1;
            }
        } else {
            // Keep the fastest example of each kind.
            out[n - 1] = Some(gait);
        }
    }
    out
}

/// The gait it walks in: about a quarter on the Froude number, where
/// every walker walks, or its slowest if it cannot get so fast.
pub fn walking(g: &Genome, q: &Posture) -> Gait {
    let share = fx(1, 4).div(top_froude(g, q)).min(Fx::ONE);
    at(g, q, share)
}
