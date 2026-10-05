//! **Can this class clear that hop, and with how much room for error?** The
//! jump courses' instrument (`docs/design/courses.md` §4), on the search
//! (`crate::search`) since round two: hand-written routes underestimated
//! every class.
//!
//! For one hop of a course -- standing on one island, reaching the next, or
//! the one after over a stepping stone -- it searches input programs for a
//! line that lands, twice: with the **shared** blocks every class has (the
//! jump, the airdodge, the clicks, which in the air are aerials that hang her,
//! and the strafe) and with the **whole kit**. A line found is then made as
//! plain as it will go (every input it does not need taken out) and as
//! forgiving as a short climb can make it, and its **window** is measured:
//! how many of the seventeen frames from eight early to eight late its
//! tightest input can be pressed on and still land.
//!
//! Every hop starts fresh, standing in the middle of its island with nothing
//! out: three stones, the shadow at her heel, the Dual mage's bars empty (her
//! tiers are paid for by waiting with nothing to hit, and §0's rule is that
//! reach is not bought by waiting).
//!
//! A search is a **lower bound**: a line it does not find may exist.

use crate::class::Class;
use crate::course::Course;
use crate::search::{self, Act, Kit, Outcome, Program, Rng, Stage};

/// The stage for a hop: from step `from`, `hops` steps on.
pub fn stage(course: &Course, class: Class, from: usize, hops: usize) -> Stage {
    let last = course.route.len() - 1;
    Stage::hop(
        course.arena,
        class,
        course.top(from),
        course.top((from + 1).min(last)),
        course.top((from + hops).min(last)),
    )
}

/// Take out every input a line does not need, and the strafe if it is not
/// needed either.
pub fn simplify(stage: &Stage, prog: &Program) -> Program {
    let mut p = prog.clone();
    let mut i = p.events.len();
    while i > 0 {
        i -= 1;
        let mut q = p.clone();
        q.events.remove(i);
        if search::run(stage, &q).landed {
            p = q;
        }
    }
    if p.strafe > 0 {
        let q = Program {
            strafe: 0,
            ..p.clone()
        };
        if search::run(stage, &q).landed {
            p = q;
        }
    }
    p
}

/// Climb toward a more forgiving line: nudge one input's timing at a time
/// and keep what widens the tightest window. The line and its window.
pub fn forgiving(stage: &Stage, prog: &Program, steps: usize, seed: u64) -> (Program, u32) {
    let mut rng = Rng(seed ^ 0x5DEE_CE66);
    let slack = crate::arena::cm(50);
    let mut best = prog.clone();
    let mut w = search::window(stage, &best, slack).0;
    for _ in 0..steps {
        if best.events.is_empty() || w > 2 * search::reach() as u32 {
            break;
        }
        let i = rng.range(0, best.events.len() as i32 - 1) as usize;
        let q = best.shifted(i, rng.range(-3, 3));
        if !search::run(stage, &q).landed {
            continue;
        }
        let qw = search::window(stage, &q, slack).0;
        if qw > w {
            best = q;
            w = qw;
        }
    }
    (best, w)
}

/// What one class did with one hop, with one kit.
#[derive(Clone, Debug)]
pub struct Line {
    pub program: Program,
    pub outcome: Outcome,
    /// Frames the tightest input can move, of 31.
    pub window: u32,
}

/// Search a hop for `class` with `kit`: the plainest, most forgiving line
/// found, or `None`.
pub fn solve(
    course: &Course,
    class: Class,
    from: usize,
    hops: usize,
    kit: Kit,
    budget: usize,
    seed: u64,
) -> Option<Line> {
    let st = stage(course, class, from, hops);
    // The whole kit starts from the best the shared blocks found, so it is
    // never worse and its tools are tried at the end of that line.
    let extra = if kit == Kit::Full {
        vec![search::search(&st, Kit::Shared, budget / 2, seed).0]
    } else {
        Vec::new()
    };
    let (p, o) = search::search_from(&st, kit, budget, seed, extra);
    if !o.landed {
        return None;
    }
    let p = simplify(&st, &p);
    let (p, window) = forgiving(&st, &p, 12, seed);
    let outcome = search::run(&st, &p);
    Some(Line {
        program: p,
        outcome,
        window,
    })
}

/// The tools a line uses, by name, in the order they first appear.
pub fn uses(p: &Program) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    let mut add = |s: &'static str| {
        if !out.contains(&s) {
            out.push(s);
        }
    };
    for (_, act) in &p.events {
        match act {
            Act::Jump(_) => add("jump"),
            Act::Dodge => add("airdodge"),
            Act::Press(0..=2) => add("click"),
            Act::Press(3) => add("Q"),
            Act::Press(4) => add("F"),
            Act::Press(6) => add("E"),
            Act::Press(_) => add("key"),
            Act::At(1, ..) => add("shadow"),
            Act::At(3, ..) => add("grasp"),
            Act::At(6, ..) => add("stone"),
            Act::At(..) => add("aimed"),
            Act::Dash => add("dash"),
            Act::Vault => add("vault"),
        }
    }
    if p.strafe > 0 {
        add("strafe");
    }
    out
}

/// **The most forgiving line found** for a stage: the search's best few
/// landing lines, from two seeds (and, for the whole kit, from the shared
/// line too), each made plain and as forgiving as a short climb makes it; the
/// one with the widest tightest window. The window is the difficulty measure
/// the courses are built against (`docs/design/courses.md` §4): a hop whose
/// most forgiving line still has a narrow window is hard.
pub fn loosest(stage: &Stage, kit: Kit, budget: usize, seed: u64) -> Option<Line> {
    let mut found: Vec<Program> = Vec::new();
    for s in [seed, !seed] {
        let extra = if kit == Kit::Full {
            search::search_elite(stage, Kit::Shared, budget / 2, s, Vec::new())
                .into_iter()
                .filter(|(_, o)| o.landed)
                .map(|(p, _)| p)
                .collect()
        } else {
            Vec::new()
        };
        for (p, o) in search::search_elite(stage, kit, budget, s, extra) {
            if o.landed && !found.contains(&p) {
                found.push(p);
            }
        }
    }
    let mut best: Option<Line> = None;
    for (i, p) in found.iter().enumerate() {
        let p = simplify(stage, p);
        let (p, window) = forgiving(stage, &p, 8, seed ^ i as u64);
        if best.as_ref().is_none_or(|b| window > b.window) {
            let outcome = search::run(stage, &p);
            best = Some(Line {
                program: p,
                outcome,
                window,
            });
        }
    }
    best
}
