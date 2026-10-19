//! **Where each morph stands** (2026-10-19): at heel at peace, well back in
//! a fight, caught up when left a long way behind. It was the world's, and
//! nothing in a fight reads it, so it is this game's now, worked out every
//! frame from where the fighters are -- the bag and the morphs moved out of
//! the shared world the same day (`bag`).
//!
//! The rules are the ones the simulation kept, in the Oven's numbers
//! (`KeepFollow`, `KeepFollowFight`, `KeepCatchUp`, `KeepHurry`), and the
//! walk is still its body's own gait (`sim::breed::locomotion`).

use bevy::prelude::*;
use sim::breed::{locomotion, posture};
use sim::state::MAX_PLAYERS;

/// One seat's morph, in the world's coordinates.
#[derive(Clone, Copy, Default, Debug)]
pub struct Heel {
    /// On the floor.
    pub at: Vec3,
    /// Turns from `+x` toward `+z`.
    pub facing: f32,
    /// Where in its stride, nought to one.
    pub phase: f32,
    /// Moving this frame.
    pub going: bool,
    /// Put beside its player yet: false outside the valley.
    pub placed: bool,
}

#[derive(Resource, Default)]
pub struct Heels {
    pub seat: [Heel; MAX_PLAYERS],
    /// Each seat's walk and size, and the body they were worked out for.
    walk: [Option<(u64, Walk)>; MAX_PLAYERS],
    /// Where the world's coordinates were last frame.
    origin: Option<Vec3>,
}

fn v3(v: sim::V3) -> Vec3 {
    Vec3::new(
        v.x.to_f32_for_render(),
        v.y.to_f32_for_render(),
        v.z.to_f32_for_render(),
    )
}

fn fx(v: f32) -> sim::Fx {
    sim::Fx::from_raw((v * 65536.0).round() as i32)
}

fn turns(dir: Vec3) -> f32 {
    dir.z.atan2(dir.x) / std::f32::consts::TAU
}

/// Each seat's morph moved on a frame.
pub fn follow(
    sim: Res<crate::Sim>,
    keeping: Res<crate::bag::Keeping>,
    time: Res<Time>,
    mut heels: ResMut<Heels>,
) {
    let w = &sim.cur;
    let Heels { seat, walk, origin } = &mut *heels;
    // The world moved into the next place's coordinates: the morphs move
    // with it.
    let o = v3(w.map_origin());
    if let Some(last) = origin.replace(o)
        && last != o
    {
        for h in seat.iter_mut() {
            h.at += last - o;
        }
    }
    let seats = (w.seats as usize).clamp(1, MAX_PLAYERS);
    let dt = time.delta_secs().min(0.1);
    let terrain = w.terrain();
    for (i, h) in seat.iter_mut().enumerate() {
        if !w.valley.on || i >= seats {
            h.placed = false;
            continue;
        }
        let p = w.players[i];
        let pos = v3(p.pos);
        if !h.placed {
            *h = Heel {
                at: beside(pos),
                facing: turns(v3(p.facing)),
                placed: true,
                ..default()
            };
        }
        // Its walk, worked out once each time its body changes.
        let m = &keeping.held().morphs[i];
        let key = m.genome.fingerprint() ^ (m.growth.raw() as u64) << 40;
        let gait = match walk[i] {
            Some((k, gait)) if k == key => gait,
            _ => {
                let gait = Walk::of(m);
                walk[i] = Some((key, gait));
                gait
            }
        };
        // **Out of the way in a fight**: the creature nearest you.
        let near = w.hunting().then(|| {
            w.monsters
                .iter()
                .flatten()
                .filter(|m| m.alive())
                .map(|m| v3(m.pos))
                .chain(w.critters.iter().filter(|c| c.alive()).map(|c| v3(c.pos)))
                .min_by(|a, b| a.distance(pos).total_cmp(&b.distance(pos)))
        });
        let ahead = Vec3::new(
            p.facing.x.to_f32_for_render(),
            0.0,
            p.facing.z.to_f32_for_render(),
        )
        .try_normalize()
        .unwrap_or(Vec3::Z);
        h.step(pos, ahead, near.flatten(), gait, dt, &Knobs::tuned());
        // On the floor.
        h.at.y = terrain
            .ground_under(sim::V3::new(fx(h.at.x), fx(h.at.y), fx(h.at.z)))
            .to_f32_for_render();
    }
}

/// Where a morph is put beside its player: a step back and to the side.
fn beside(pos: Vec3) -> Vec3 {
    pos - Vec3::new(1.0, 0.0, 1.0)
}

/// The Oven's numbers for following, in metres.
struct Knobs {
    follow: f32,
    fight: f32,
    catch_up: f32,
    hurry: f32,
}

impl Knobs {
    fn tuned() -> Knobs {
        use sim::tuning as t;
        Knobs {
            follow: t::morph_follow().to_f32_for_render(),
            fight: t::morph_follow_fight().to_f32_for_render(),
            catch_up: t::morph_catch_up().to_f32_for_render(),
            hurry: t::morph_hurry().to_f32_for_render(),
        }
    }
}

/// How a morph walks, and how much room it takes.
#[derive(Clone, Copy, Debug)]
pub struct Walk {
    /// Metres a second.
    speed: f32,
    /// Seconds a stride.
    stride: f32,
    /// How far its body reaches from where it stands, on the flat, metres.
    radius: f32,
}

impl Walk {
    fn of(m: &sim::breed::morph::Morph) -> Walk {
        let g = m.shown();
        let q = posture::posture(&g);
        let gait = locomotion::at(&g, &q, sim::Fx::ratio(1, 2));
        let body = sim::breed::build::body_in(&g, &q);
        let radius = (0..body.part_count)
            .map(|i| {
                let c = body.centre(i);
                let d = body.parts[i].shape.max.sub(body.parts[i].shape.min);
                let half = d.x.max(d.y).max(d.z).to_f32_for_render() / 2.0;
                Vec2::new(c.x.to_f32_for_render(), c.z.to_f32_for_render()).length() + half
            })
            .fold(0.0, f32::max);
        Walk {
            speed: gait.speed.to_f32_for_render().max(1.0),
            stride: gait.cycle.clamp(8, 240) as f32 / locomotion::FPS as f32,
            radius,
        }
    }
}

impl Heel {
    /// One frame of following a player at `pos`, facing `ahead` (flat), the
    /// creature nearest them at `near` if they are in a fight. On the flat:
    /// the floor is the caller's.
    ///
    /// **It keeps to your side, and further the bigger it is** (2026-10-19:
    /// grown, it stood between the camera and you and you could not see
    /// your fighter). The camera is behind you, so behind you is the one
    /// place a big body hides you: it keeps to whichever side of you it is
    /// on, its own reach plus the follow distance out. In a fight it goes
    /// back and to that side, away from the creature nearest you.
    fn step(&mut self, pos: Vec3, ahead: Vec3, near: Option<Vec3>, walk: Walk, dt: f32, k: &Knobs) {
        let to = Vec3::new(pos.x - self.at.x, 0.0, pos.z - self.at.z);
        // Left a long way behind (a jump, a fall, a seam): it catches up.
        if to.length() > k.catch_up + walk.radius {
            self.at = beside(pos);
            self.going = false;
            return;
        }
        // Its side of you: the one it is on.
        let right = Vec3::new(-ahead.z, 0.0, ahead.x);
        let side = if (self.at - pos).dot(right) < 0.0 {
            -right
        } else {
            right
        };
        let (goal, slack) = match near {
            Some(c) => {
                let away = Vec3::new(pos.x - c.x, 0.0, pos.z - c.z)
                    .try_normalize()
                    .unwrap_or(-ahead);
                let out = (away + side).try_normalize().unwrap_or(side);
                (pos + out * (k.fight + walk.radius), 1.0)
            }
            None => (pos + side * (k.follow + walk.radius), k.follow * 0.5),
        };
        let to_goal = Vec3::new(goal.x - self.at.x, 0.0, goal.z - self.at.z);
        let gap = to_goal.length();
        self.going = gap > slack;
        if self.going {
            // Faster the further off it is, up to its hurry.
            let urge = (1.0 + (gap - slack) / k.follow.max(0.1)).min(k.hurry);
            let dir = to_goal / gap;
            self.at += dir * (walk.speed * urge * dt).min(gap);
            self.facing = turns(dir);
            self.phase = (self.phase + dt / walk.stride * urge).fract();
        } else if to.length() > 0.0 {
            // Standing, it watches you.
            self.facing = turns(to);
        }
    }
}

/// Its stride, as the builder takes it.
pub fn phase(h: &Heel) -> sim::Fx {
    fx(h.phase)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL: Walk = Walk {
        speed: 2.0,
        stride: 0.5,
        radius: 0.3,
    };

    fn run(near: Option<Vec3>, walk: Walk) -> (Heel, Vec3) {
        let k = Knobs::tuned();
        let pos = Vec3::new(10.0, 0.0, 0.0);
        let mut h = Heel {
            at: Vec3::ZERO,
            placed: true,
            ..default()
        };
        for _ in 0..1200 {
            h.step(pos, Vec3::X, near, walk, 1.0 / 60.0, &k);
        }
        (h, pos)
    }

    #[test]
    fn at_peace_it_comes_to_your_side() {
        let k = Knobs::tuned();
        let (h, pos) = run(None, SMALL);
        let gap = h.at.distance(pos);
        assert!(gap <= k.follow + SMALL.radius + k.follow, "it is {gap} off");
        // Beside you, not behind you where the camera looks past you.
        let behind = (pos - h.at).dot(Vec3::X);
        assert!(behind < gap * 0.5, "it is {behind} behind you of {gap}");
    }

    #[test]
    fn in_a_fight_it_keeps_back_on_the_far_side() {
        let k = Knobs::tuned();
        let creature = Vec3::new(20.0, 0.0, 0.0);
        let (h, pos) = run(Some(creature), SMALL);
        let gap = h.at.distance(pos);
        assert!(gap >= k.fight - 2.0, "it is {gap} from you in a fight");
        // Further from the creature than you are: behind you, not between.
        assert!(h.at.distance(creature) > pos.distance(creature));
    }

    #[test]
    fn a_big_one_keeps_further_off() {
        let big = Walk {
            radius: 6.0,
            ..SMALL
        };
        let (small, pos) = run(None, SMALL);
        let (large, _) = run(None, big);
        assert!(large.at.distance(pos) > small.at.distance(pos) + 4.0);
        // And clear of you: its body does not reach you.
        assert!(large.at.distance(pos) > big.radius);
    }

    #[test]
    fn left_far_behind_it_catches_up() {
        let k = Knobs::tuned();
        let mut h = Heel {
            at: Vec3::ZERO,
            placed: true,
            ..default()
        };
        let far = Vec3::new(k.catch_up + 5.0, 0.0, 0.0);
        h.step(far, Vec3::X, None, SMALL, 1.0 / 60.0, &k);
        assert!(h.at.distance(far) < 2.0);
    }
}
