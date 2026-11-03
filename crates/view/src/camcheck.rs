//! **The camera check**: walk a fighter along a route through a place, with
//! the fight's own camera following, and say where the camera misbehaves --
//! before anybody has to load the place to find out
//! (`docs/design/level-design.md`, "The play check").
//!
//! The camera is [`CameraRig`], the same rig the game draws from, settling
//! frame by frame at sixty a second. Along each leg the fighter walks at a
//! run and looks the way they are going; at each stop they stand and turn the
//! view once all the way round, the way a player does on arriving somewhere.
//! Four things are flagged:
//!
//! - **A snap-in**: the arm pulled in sharply in one frame because the view
//!   swung into something solid, or let out again as fast once it swung
//!   clear -- the camera's designed answer to a wall or a pier at its back,
//!   which keeps it out of them. Some is unavoidable beside anything solid;
//!   a place where it happens often is cramped.
//! - **A jump**: the eye moved, relative to the fighter, by much more in one
//!   frame than turning the view accounts for, and not by pulling in. What a
//!   player reports as "the camera jumped".
//! - **A wall cut away**: the line from the eye to the fighter passes
//!   through something wall-sized (three metres tall and four broad, or
//!   more) that the camera does not keep out of
//!   ([`sim::arena::Solid::holds_the_camera`]), so the game draws it
//!   see-through round the fighter. Columns, posts and platforms are cut
//!   away by design and are not counted.
//! - **Cramped**: the eye is within a metre of the fighter's head -- the arm
//!   pulled all the way in, nearly first person.
//! - **In the land**: the eye is under the ground there.
//!
//! It is a tool for building places, not for fights: no creature is
//! considered, and nothing here is per frame in the game.

use crate::camera::{RigConfig, Zones};
use crate::{CameraRig, Surroundings};
use sim::arena::Terrain;
use sim::{Fx, V3};

/// How a route is walked, and where the lines are drawn.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// Walking speed along a leg, metres a second: a run.
    pub speed: f32,
    /// How long the turn on the spot at each stop takes, seconds.
    pub spin: f32,
    /// How far the eye may move relative to the fighter in one frame, past
    /// what the turn of the view accounts for, metres.
    pub jump: f32,
    /// Closer than this from eye to head is cramped, metres.
    pub cramped: f32,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            speed: 7.0,
            spin: 3.0,
            jump: 0.35,
            cramped: 1.0,
        }
    }
}

/// What went wrong.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Jump,
    SnapIn,
    CutAway,
    Cramped,
    InLand,
}

impl Kind {
    pub const ALL: [Kind; 5] = [
        Kind::Jump,
        Kind::SnapIn,
        Kind::CutAway,
        Kind::Cramped,
        Kind::InLand,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Kind::Jump => "camera jumped",
            Kind::SnapIn => "snapped in",
            Kind::CutAway => "wall cut away",
            Kind::Cramped => "cramped",
            Kind::InLand => "eye in the land",
        }
    }
}

/// One flagged frame.
#[derive(Clone, Copy, Debug)]
pub struct Flag {
    pub kind: Kind,
    /// Which leg (0 is the first stop to the second) or, past the last, which
    /// stop's turn: see [`Report::place`].
    pub step: usize,
    /// Whether this was the turn on the spot rather than the walk.
    pub turning: bool,
    /// Where the fighter's feet were.
    pub feet: [f32; 3],
    pub eye: [f32; 3],
    /// How bad: metres for a jump, how near for cramped, how deep for in
    /// the land, how tall the wall for a cut.
    pub size: f32,
}

/// What a walk found.
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub frames: usize,
    pub flags: Vec<Flag>,
    /// Steps up or down the walk took that were taller than a stair (half a
    /// metre in a frame): a route drawn through a ledge rather than round
    /// it. The check is blind for a moment after each one.
    pub steps: usize,
    /// Frames the route put the fighter inside the side of a solid, which a
    /// body cannot do: not judged, and a sign the route wants another stop.
    pub inside: usize,
}

impl Report {
    pub fn count(&self, kind: Kind) -> usize {
        self.flags.iter().filter(|f| f.kind == kind).count()
    }

    /// The share of frames with this kind of flag.
    pub fn share(&self, kind: Kind) -> f32 {
        self.count(kind) as f32 / self.frames.max(1) as f32
    }

    /// **The bar** (`docs/design/delivery.md`, D3): no jumps, a snap-in on
    /// under one frame in a hundred, the eye never in the land, a wall cut
    /// away on under one frame in fifty, and cramped on under one in twenty.
    pub fn passes(&self) -> bool {
        self.count(Kind::Jump) == 0
            && self.share(Kind::SnapIn) < 0.01
            && self.count(Kind::InLand) == 0
            && self.share(Kind::CutAway) < 0.02
            && self.share(Kind::Cramped) < 0.05
    }

    /// A line for a step: "leg 2" or "turn at stop 3".
    pub fn place(flag: &Flag) -> String {
        if flag.turning {
            format!("turn at stop {}", flag.step + 1)
        } else {
            format!(
                "leg {} (stop {} to {})",
                flag.step + 1,
                flag.step + 1,
                flag.step + 2
            )
        }
    }

    /// A summary a person reads: a line a kind, and the worst of each kind
    /// on each step.
    pub fn text(&self) -> String {
        let mut out = format!(
            "{} frames walked; {} step(s) up or down taller than a stair; {} frames skipped inside a solid (route through a wall)\n",
            self.frames, self.steps, self.inside
        );
        for kind in Kind::ALL {
            out += &format!(
                "  {:16} {:5} frames ({:.1}%)\n",
                kind.name(),
                self.count(kind),
                100.0 * self.share(kind)
            );
        }
        let mut worst: Vec<Flag> = Vec::new();
        for f in &self.flags {
            match worst
                .iter_mut()
                .find(|w| w.kind == f.kind && w.step == f.step && w.turning == f.turning)
            {
                Some(w) if w.size < f.size => *w = *f,
                Some(_) => {}
                None => worst.push(*f),
            }
        }
        for w in &worst {
            out += &format!(
                "  - {} at {}: feet ({:.1}, {:.1}, {:.1}), eye ({:.1}, {:.1}, {:.1}), {:.2}\n",
                w.kind.name(),
                Report::place(w),
                w.feet[0],
                w.feet[1],
                w.feet[2],
                w.eye[0],
                w.eye[1],
                w.eye[2],
                w.size
            );
        }
        out += if self.passes() { "PASS\n" } else { "FAIL\n" };
        out
    }
}

const DT: f32 = 1.0 / 60.0;

fn fx(v: f32) -> Fx {
    Fx::from_raw((v * 65536.0).round() as i32)
}

fn v3(p: [f32; 3]) -> V3 {
    V3::new(fx(p[0]), fx(p[1]), fx(p[2]))
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// **Walk a route.** Each stop is `[x, z, from]`: a point in the place, in
/// metres, and the height to look down from for what the fighter stands on
/// there (as `SHOT_AT`'s third number) -- so a stop on a landing under a
/// roof is the landing, not the roof.
pub fn walk(terrain: &Terrain, route: &[[f32; 3]], opt: &Options) -> Report {
    let mut report = Report::default();
    let mut rig = CameraRig::new(RigConfig::default());
    let around = Surroundings {
        arena: *terrain,
        ..Default::default()
    };
    let pitch = Zones::tuned().start_pitch();
    let head_height = sim::tuning::body_height().to_f32_for_render() * 0.9;
    let stand = |x: f32, z: f32, from: f32| -> [f32; 3] {
        let y = terrain.ground_under(v3([x, from, z])).to_f32_for_render();
        [x, y, z]
    };
    // What the eye was, relative to the point the camera follows (its own
    // smoothed focus, so a fighter stepping off something is not counted as
    // the camera jumping), and the view's yaw then.
    let mut last: Option<([f32; 3], f32)> = None;
    let mut blind = 0usize;
    let mut was_feet: Option<[f32; 3]> = None;
    // One frame: `judge` false settles the camera without counting anything.
    let mut frame = |rig: &mut CameraRig,
                     report: &mut Report,
                     feet: [f32; 3],
                     yaw: f32,
                     step: usize,
                     turning: bool,
                     judge: bool| {
        let framing = rig.update_around(DT, feet, yaw, pitch, around);
        let eye = framing.eye;
        let rel = sub(eye, rig.focus());
        if !judge {
            last = Some((rel, yaw));
            was_feet = Some(feet);
            return;
        }
        let head = [feet[0], feet[1] + head_height, feet[2]];
        // A route drawn through the side of something: the fighter could not
        // stand there, so nothing the camera does there is the place's fault.
        if body_in_solid(terrain, feet, head_height) {
            report.inside += 1;
            last = None;
            blind = 30;
            return;
        }
        report.frames += 1;
        let flag = |kind, size| Flag {
            kind,
            step,
            turning,
            feet,
            eye,
            size,
        };
        if was_feet.is_some_and(|w| (feet[1] - w[1]).abs() > 0.5) {
            report.steps += 1;
            blind = 30;
        }
        if let Some((was, was_yaw)) = last {
            let turned = (yaw - was_yaw).abs();
            let moved = len(sub(rel, was)) - len(was) * turned;
            if blind == 0 && moved > opt.jump {
                // Most of the move was the arm's length changing: pulled in
                // against something, or let out again past it.
                let change = (len(was) - len(rel)).abs();
                let kind = if change > 0.7 * moved {
                    Kind::SnapIn
                } else {
                    Kind::Jump
                };
                report.flags.push(flag(kind, moved));
            }
        }
        blind = blind.saturating_sub(1);
        last = Some((rel, yaw));
        was_feet = Some(feet);
        // From the point the camera follows, which lags a step down as the
        // eye does: measured from the instant head, every step off a ledge
        // read as the arm collapsing.
        let near = len(sub(eye, rig.focus()));
        if near < opt.cramped {
            report.flags.push(flag(Kind::Cramped, near));
        }
        let land = terrain
            .relief_at(fx(eye[0]), fx(eye[2]))
            .to_f32_for_render();
        if eye[1] < land - 0.1 {
            report.flags.push(flag(Kind::InLand, land - eye[1]));
        }
        if let Some(tall) = wall_cut_away(terrain, eye, head) {
            report.flags.push(flag(Kind::CutAway, tall));
        }
    };
    // Settle where the walk starts, looking along the first leg.
    const SETTLE: usize = 180;
    if let (Some(a), Some(b)) = (route.first(), route.get(1)) {
        let yaw = (b[1] - a[1]).atan2(b[0] - a[0]);
        let feet = stand(a[0], a[1], a[2]);
        for _ in 0..SETTLE {
            frame(&mut rig, &mut report, feet, yaw, 0, false, false);
        }
    }
    for (i, leg) in route.windows(2).enumerate() {
        let (a, b) = (leg[0], leg[1]);
        let d = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        let yaw = (b[1] - a[1]).atan2(b[0] - a[0]);
        let n = ((d / (opt.speed * DT)).ceil() as usize).max(1);
        for k in 0..=n {
            let t = k as f32 / n as f32;
            let feet = stand(
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
            );
            frame(&mut rig, &mut report, feet, yaw, i, false, true);
        }
    }
    // The turn at each stop, starting the way the walk arrived: a fresh
    // camera settled there first, so arriving is not counted as a jump.
    for (i, s) in route.iter().enumerate() {
        let feet = stand(s[0], s[1], s[2]);
        let from = if i > 0 {
            let p = route[i - 1];
            (s[1] - p[1]).atan2(s[0] - p[0])
        } else {
            0.0
        };
        rig = CameraRig::new(RigConfig::default());
        for _ in 0..SETTLE {
            frame(&mut rig, &mut report, feet, from, i, true, false);
        }
        let n = (opt.spin / DT) as usize;
        for k in 0..=n {
            let yaw = from + std::f32::consts::TAU * k as f32 / n as f32;
            frame(&mut rig, &mut report, feet, yaw, i, true, true);
        }
    }
    report
}

/// Is a standing body here inside the side of a solid -- its middle, a
/// fighter's half-width round, between its feet and its head?
fn body_in_solid(terrain: &Terrain, feet: [f32; 3], height: f32) -> bool {
    let at = v3(feet);
    let pad = fx(0.3);
    terrain.around(at, fx(2.0)).any(|s| {
        let lo = feet[1] + 0.05;
        let hi = feet[1] + height;
        s.min.y.to_f32_for_render() < hi
            && s.max.y.to_f32_for_render() > lo
            && s.covers_at(at.x, at.z, fx(lo.max(s.min.y.to_f32_for_render())), pad)
    })
}

/// The tallest wall-sized solid the line from the eye to the head passes
/// through that the camera does not keep out of -- which the game then cuts
/// away round the fighter.
fn wall_cut_away(terrain: &Terrain, eye: [f32; 3], head: [f32; 3]) -> Option<f32> {
    let d = sub(head, eye);
    let length = len(d);
    let steps = ((length / 0.25).ceil() as usize).max(1);
    let mut tallest: Option<f32> = None;
    // The walk is short, so the solids round its middle cover all of it.
    let mid = [
        eye[0] + d[0] * 0.5,
        eye[1] + d[1] * 0.5,
        eye[2] + d[2] * 0.5,
    ];
    for s in terrain.around(v3(mid), fx(length * 0.5 + 1.0)) {
        if s.holds_the_camera() {
            continue;
        }
        let w = (s.max.x.sub(s.min.x)).to_f32_for_render();
        let h = (s.max.y.sub(s.min.y)).to_f32_for_render();
        let l = (s.max.z.sub(s.min.z)).to_f32_for_render();
        if h < 3.0 || w.max(l) < 4.0 {
            continue;
        }
        let through = (1..steps).any(|k| {
            let t = k as f32 / steps as f32;
            let p = [eye[0] + d[0] * t, eye[1] + d[1] * t, eye[2] + d[2] * t];
            let y = fx(p[1]);
            y.raw() > s.min.y.raw()
                && y.raw() < s.max.y.raw()
                && s.covers_at(fx(p[0]), fx(p[2]), y, Fx::ZERO)
        });
        if through && tallest.is_none_or(|t| h > t) {
            tallest = Some(h);
        }
    }
    tallest
}
