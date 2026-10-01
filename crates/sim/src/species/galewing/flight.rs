//! **How the Galewing flies** (bestiary P6): a steering controller in three
//! dimensions, and what each thing it does asks of it.
//!
//! The body is a point with a heading, a horizontal speed and a vertical
//! one. It steers toward a target point under three limits -- how hard it
//! can turn (`AirTurn`, scaled down toward a broken wing), how fast it climbs
//! (`ClimbRate`) and how fast it sinks (`SinkRate`, faster in a stoop) -- and
//! its turn has **mass**: the turn rate itself is limited in how fast it can
//! change (`AirTurnAccel`), so a hard turn overshoots, which is the window a
//! player cutting across its lane gets (§5). Its bank is the turn it is
//! making, as a share of its fullest, times `BankMax`; its pitch is its climb
//! over its speed. Both go onto the root bone by [`repose`], so the rig, the
//! hit test and the riders see the banked body.
//!
//! **Some moves arrive rather than steer**: a dive onto a point, a hover over
//! one, the landing on the perch. Those place the body on a straight line
//! that reaches its mark on the frame the move says it does
//! ([`Flight::arrive`]) -- the hit is where the telegraph says, on the frame
//! it says, whatever the controller would have made of it.
//!
//! The flight state is the hunt's lore (`fight::word::POS`..`YAW_RATE`). The
//! shared step walks the body as though it were on the ground and the frame
//! hook throws that away: while the bird is in the air, its position and
//! heading are this file's.

use crate::DT;
use crate::beast::{self, Pose};
use crate::fixed::Fx;
use crate::lore::Lore;
use crate::math::{self, V3};
use crate::monster::{Doing, Monster};
use crate::state::World;

use super::fight::{self, flag, ride, word};
use super::{
    BUFFET, CARRY, Clip, DOWNWASH, HOP, Knob, LIFT, PERCH, ROLL, SCREECH, SPECIES, STOOP, TALON,
    VOLLEY, bones,
};

/// The body in the air.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flight {
    /// Where its feet would be: the point the rig is built from.
    pub pos: V3,
    /// Along its heading, metres a second.
    pub speed: Fx,
    /// Up, metres a second.
    pub vy: Fx,
    /// Its heading, in turns.
    pub yaw: Fx,
    /// Turns a second.
    pub yaw_rate: Fx,
}

/// What limits a steer.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Turns a second at full bank.
    pub turn: Fx,
    /// The share of that it can turn toward each side: left, right.
    pub sides: (Fx, Fx),
    pub climb: Fx,
    pub sink: Fx,
}

impl Flight {
    pub fn load(lore: &Lore) -> Flight {
        let f = |i| Fx::from_raw(lore.int(i));
        Flight {
            pos: V3::new(f(word::POS), f(word::POS + 1), f(word::POS + 2)),
            speed: f(word::SPEED),
            vy: f(word::VY),
            yaw: f(word::YAW),
            yaw_rate: f(word::YAW_RATE),
        }
    }

    pub fn save(&self, lore: &mut Lore) {
        lore.set_int(word::POS, self.pos.x.raw());
        lore.set_int(word::POS + 1, self.pos.y.raw());
        lore.set_int(word::POS + 2, self.pos.z.raw());
        lore.set_int(word::SPEED, self.speed.raw());
        lore.set_int(word::VY, self.vy.raw());
        lore.set_int(word::YAW, self.yaw.raw());
        lore.set_int(word::YAW_RATE, self.yaw_rate.raw());
    }

    /// **Steer toward a point** at a speed, under the limits: the turn
    /// toward the point's bearing (rate-limited, and the rate itself
    /// acceleration-limited), the speed eased toward `speed`, the climb or
    /// sink eased toward the height.
    pub fn steer(&mut self, to: V3, speed: Fx, lim: &Limits) {
        let flat = V3::new(to.x.sub(self.pos.x), Fx::ZERO, to.z.sub(self.pos.z));
        if math::wide_flat_len(flat).raw() > 0 {
            let want = math::atan2_turns(flat.z, flat.x);
            let error = math::wrap_turns(want.sub(self.yaw));
            // Increasing yaw turns toward +z, its right.
            let side = if error.raw() > 0 {
                lim.sides.1
            } else {
                lim.sides.0
            };
            let cap = lim.turn.mul(side);
            let desired = error.mul(Knob::AirTurnGain.fx()).clamp(cap.neg(), cap);
            let budget = Knob::AirTurnAccel.fx().mul(DT);
            let change = desired.sub(self.yaw_rate).clamp(budget.neg(), budget);
            self.yaw_rate = self.yaw_rate.add(change);
        } else {
            // Nowhere to turn to: the turn eases off at the same rate it
            // can change.
            let budget = Knob::AirTurnAccel.fx().mul(DT);
            self.yaw_rate = self.yaw_rate.sub(self.yaw_rate.clamp(budget.neg(), budget));
        }
        self.yaw = math::wrap_turns(self.yaw.add(self.yaw_rate.mul(DT)));
        let gain = Knob::AirAccel.fx().mul(DT);
        self.speed = self
            .speed
            .add(speed.sub(self.speed).clamp(gain.neg(), gain))
            .max(Fx::ZERO);
        let want_vy =
            to.y.sub(self.pos.y)
                .mul(Knob::VertGain.fx())
                .clamp(lim.sink.neg(), lim.climb);
        let vgain = Knob::VertAccel.fx().mul(DT);
        self.vy = self.vy.add(want_vy.sub(self.vy).clamp(vgain.neg(), vgain));
        let ahead = V3::from_turns(self.yaw).scale(self.speed.mul(DT));
        self.pos = V3::new(
            self.pos.x.add(ahead.x),
            self.pos.y.add(self.vy.mul(DT)),
            self.pos.z.add(ahead.z),
        );
    }

    /// **Arrive at `to` exactly `left` frames from now**, on a straight line,
    /// turning the heading toward the way it is going at no more than
    /// `turn` turns a frame.
    pub fn arrive(&mut self, to: V3, left: u16, turn: Fx) {
        let n = Fx::from_int(left.max(1) as i32);
        let d = to.sub(self.pos);
        let step = V3::new(d.x.div(n), d.y.div(n), d.z.div(n));
        let rate = Fx::ONE.div(DT);
        let flat = V3::new(step.x, Fx::ZERO, step.z);
        let along = math::wide_flat_len(flat);
        if along.raw() > 0 {
            let want = math::atan2_turns(step.z, step.x);
            let error = math::wrap_turns(want.sub(self.yaw));
            let turned = error.clamp(turn.neg(), turn);
            self.yaw = math::wrap_turns(self.yaw.add(turned));
            self.yaw_rate = turned.mul(rate);
        } else {
            self.yaw_rate = Fx::ZERO;
        }
        self.speed = along.mul(rate);
        self.vy = step.y.mul(rate);
        self.pos = self.pos.add(step);
    }

    /// Hold still in the air or on the ground: no speed, no turn.
    pub fn hold(&mut self) {
        self.speed = Fx::ZERO;
        self.vy = Fx::ZERO;
        self.yaw_rate = Fx::ZERO;
    }
}

/// The limits it flies under, with its broken wing taken into account.
pub fn limits(m: &Monster) -> Limits {
    let turn = Knob::AirTurn.fx();
    let hurt = Knob::BrokenTurn.fx();
    let left = if fight::broken(m, 0) { hurt } else { Fx::ONE };
    let right = if fight::broken(m, 1) { hurt } else { Fx::ONE };
    let climb = if fight::broken_wings(m) > 0 {
        Knob::ClimbRate.fx().mul(Knob::BrokenClimb.fx())
    } else {
        Knob::ClimbRate.fx()
    };
    Limits {
        turn,
        sides: (left, right),
        climb,
        sink: Knob::SinkRate.fx(),
    }
}

/// The highest it may fly, over `base`: no higher than `BrokenCeiling` with
/// a wing broken.
fn ceiling(m: &Monster, base: Fx, want: Fx) -> Fx {
    if fight::broken_wings(m) > 0 {
        want.min(base.add(Knob::BrokenCeiling.fx()))
    } else {
        want
    }
}

/// The height it circles at, over `base`: lower when desperate.
fn cruise(m: &Monster, base: Fx) -> Fx {
    let alt = if fight::below(m, Knob::DesperateHealth.fx()) {
        Knob::DesperateAlt.fx()
    } else {
        Knob::CruiseAlt.fx()
    };
    ceiling(m, base, base.add(alt))
}

/// **Where its circle is**: drifting at `CircleDrift` toward the circle that
/// passes over its target -- the target on its rim, on the side away from
/// its own `circle` site -- and kept inside the arena. So every lap comes
/// over whoever it is hunting, and *when* it can come at them is where they
/// stand against that circle (§5): ahead of it in its line-up arc, at the
/// range a move wants.
fn drift(w: &mut World, m: &Monster, home: V3) -> V3 {
    let r = Knob::CircleRadius.fx();
    let was = w.lore.word(word::CIRCLE_AT);
    let now = if was == 0 { home } else { fight::point(was) };
    let target = V3::new(m.brain.seen.x, Fx::ZERO, m.brain.seen.z);
    // Inside the bounds by its radius and a little: the circle stays over
    // the arena.
    let b = w.arena().bounds;
    let keep = r.add(Knob::CircleKeep.fx());
    let clamp = |v: Fx, lo: Fx, hi: Fx| {
        let lo = lo.add(keep);
        let hi = hi.sub(keep);
        if lo.raw() > hi.raw() {
            math::half(lo.add(hi))
        } else {
            v.clamp(lo, hi)
        }
    };
    // Of the circles with the target on their rim, eight ways round, the
    // one that stays inside the arena and nearest its own site.
    let mut best: Option<(V3, Fx)> = None;
    for k in 0..BEARINGS as i32 {
        let dir = V3::from_turns(Fx::ratio(k, BEARINGS as i32));
        let want = target.add(dir.scale(r));
        let kept = V3::new(
            clamp(want.x, b.lo_x, b.hi_x),
            Fx::ZERO,
            clamp(want.z, b.lo_z, b.hi_z),
        );
        let off = math::wide_flat_dist(kept, want);
        let cost = off
            .mul(Fx::from_int(BEARINGS as i32))
            .add(math::wide_flat_dist(kept, home));
        if best.is_none_or(|(_, c)| cost.raw() < c.raw()) {
            best = Some((kept, cost));
        }
    }
    let want = best.map_or(home, |(at, _)| at);
    let gap = V3::new(want.x.sub(now.x), Fx::ZERO, want.z.sub(now.z));
    let far = math::wide_flat_len(gap);
    let step = Knob::CircleDrift.fx().mul(DT).min(far);
    let next = if far.raw() > 0 {
        now.add(math::wide_normalized(gap).scale(step))
    } else {
        now
    };
    w.lore.set_word(word::CIRCLE_AT, fight::word_of(next));
    next
}

/// How many ways round a target it tries a circle: a count, not an angle.
const BEARINGS: usize = 8;

/// **The point it chases round its circle**: on the circle, `CircleLead`
/// of a turn on from where it is now, anticlockwise -- so it settles onto
/// the circle from wherever it is, and comes round it at its own speed.
pub fn circle_point(centre: V3, at: V3, height: Fx) -> V3 {
    let out = V3::new(at.x.sub(centre.x), Fx::ZERO, at.z.sub(centre.z));
    let now = if math::wide_flat_len(out).raw() > 0 {
        math::atan2_turns(out.z, out.x)
    } else {
        Fx::ZERO
    };
    let ahead = now.add(Knob::CircleLead.fx());
    let r = Knob::CircleRadius.fx();
    let dir = V3::from_turns(ahead);
    V3::new(
        centre.x.add(dir.x.mul(r)),
        height,
        centre.z.add(dir.z.mul(r)),
    )
}

/// **Round its circle** at a height and a speed -- never lower than
/// `TowerOver` above whatever is under it now or round the circle ahead of
/// it, as far as it flies in `TowerLook` seconds: a bird swooping or gliding
/// low climbs over the rock face rather than through it, beating hard.
fn round(w: &World, f: &mut Flight, centre: V3, h: Fx, speed: Fx, lim: &Limits) {
    let r = Knob::CircleRadius.fx();
    let out = V3::new(f.pos.x.sub(centre.x), Fx::ZERO, f.pos.z.sub(centre.z));
    let now = if math::wide_flat_len(out).raw() > 0 {
        math::atan2_turns(out.z, out.x)
    } else {
        Fx::ZERO
    };
    let arc = f.speed.mul(Knob::TowerLook.fx());
    let round_turns = math::turns_to_radians(r);
    let reach = if round_turns.raw() > 0 {
        arc.div(round_turns)
    } else {
        Fx::ZERO
    };
    let mut floor = fight::ground_at(w, f.pos);
    for k in 1..=LOOKS as i32 {
        let at = now.add(reach.mul(Fx::from_int(k)).div(Fx::from_int(LOOKS as i32)));
        let p = centre.add(V3::from_turns(at).scale(r));
        floor = floor.max(fight::ground_at(w, p));
    }
    let clear = floor.add(Knob::TowerOver.fx());
    if h.raw() < clear.raw() {
        let up = Limits {
            climb: lim.climb.max(Knob::LiftClimb.fx()),
            ..*lim
        };
        f.steer(circle_point(centre, f.pos, clear), speed, &up);
    } else {
        f.steer(circle_point(centre, f.pos, h), speed, lim);
    }
}

/// How many points round the circle ahead it looks at the ground under: a
/// count, not a distance.
const LOOKS: usize = 4;

/// Turns a frame a body arriving somewhere swings its heading by, at most.
fn arrive_turn() -> Fx {
    Knob::ArriveTurn.fx().mul(DT)
}

/// **One frame of flight**: where the body is, which way it heads, its bank,
/// pitch and heave, and whether it is in the air -- from what it is doing.
pub fn step(w: &mut World, m: &mut Monster, slot: usize) {
    let mut f = Flight::load(&w.lore);
    let base = fight::base(w);
    let home = fight::circle_centre(w);
    let centre = drift(w, m, home);
    let lim = limits(m);
    let was_aloft = fight::aloft(m);
    let mut aloft = was_aloft;
    let mut beating = false;
    let mut bank_list = Fx::ZERO;
    let shared_yaw = m.yaw;
    let riders = fight::riders_on(w, slot);

    // On the ground for good: the shared walker drives, on the plateau.
    if fight::grounded_for_good(m) && !was_aloft {
        let ground = fight::ground_at(w, m.pos);
        m.pos.y = ground;
        f = Flight {
            pos: m.pos,
            speed: m.speed,
            vy: Fx::ZERO,
            yaw: m.yaw,
            yaw_rate: m.yaw_rate,
        };
        if let Some(kind @ HOP) = m.doing.attacking() {
            hop(w, m, &mut f, kind);
        }
        finish(w, m, f, false, false, Fx::ZERO);
        return;
    }

    let ground_here = fight::ground_at(w, f.pos);
    match m.doing {
        Doing::Dead | Doing::Toppled { .. } => {
            if aloft {
                // **Falling**: tumbling out of the sky onto whatever is under
                // it, the time the fall takes not counted against the crash.
                let fall = Knob::CrashFall.fx();
                let target = V3::new(
                    f.pos.x.add(V3::from_turns(f.yaw).x.mul(f.speed)),
                    ground_here.sub(Knob::CrashFall.fx()),
                    f.pos.z.add(V3::from_turns(f.yaw).z.mul(f.speed)),
                );
                let fall_lim = Limits { sink: fall, ..lim };
                f.steer(target, Fx::ZERO, &fall_lim);
                if f.pos.y.raw() <= ground_here.raw() {
                    f.pos.y = ground_here;
                    f.hold();
                    aloft = false;
                    landed_a_crash(w, m, slot, ground_here);
                } else if let Doing::Toppled { .. } = m.doing {
                    m.doing = Doing::Toppled {
                        left: m.sp().topple_frames(),
                    };
                }
            } else {
                f.pos.y = ground_here;
                f.hold();
            }
        }
        Doing::Startup { kind, left }
        | Doing::Active { kind, left }
        | Doing::Recovery { kind, left } => {
            let phase = match m.doing {
                Doing::Startup { .. } => 0,
                Doing::Active { .. } => 1,
                _ => 2,
            };
            match kind {
                STOOP => {
                    let aim = m.aimed_at();
                    let at = V3::new(aim.x, fight::aim_height(m), aim.z);
                    if phase == 0 {
                        f.arrive(at, left.saturating_add(1), arrive_turn());
                        aloft = true;
                    } else {
                        f.pos = at;
                        f.hold();
                        aloft = false;
                    }
                }
                HOP => {
                    hop(w, m, &mut f, kind);
                    aloft = false;
                }
                TALON => {
                    let (start, along, _, _) = fight::lane(&w.lore);
                    let pass = Knob::PassHeight.fx();
                    let start_g = fight::ground_at(w, start);
                    match phase {
                        0 => {
                            // **The approach**: from wherever it broke the
                            // circle to the lane's start, lined up with it --
                            // first toward a point behind the start, then onto
                            // the start itself.
                            let lead_in = Knob::PassSpeed.fx().mul(Fx::ratio(1, 2));
                            let behind = start.sub(along.scale(lead_in));
                            let mark = V3::new(start.x, start_g.add(pass), start.z);
                            let a = SPECIES.attack(TALON);
                            let half = (a.startup / 3).max(1);
                            if left > half {
                                let via = V3::new(behind.x, start_g.add(pass), behind.z);
                                // Most of the way to the point behind the start
                                // over the first two thirds.
                                f.arrive(via, left - half, arrive_turn());
                            } else {
                                f.arrive(mark, left.saturating_add(1), arrive_turn());
                            }
                            aloft = true;
                        }
                        1 => {
                            let s = fight::front(left);
                            let at = start.add(along.scale(s));
                            let g = fight::ground_at(w, at).max(start_g);
                            let to = V3::new(at.x, g.add(pass), at.z);
                            f.arrive(to, 1, arrive_turn());
                            f.yaw = math::atan2_turns(along.z, along.x);
                            aloft = true;
                        }
                        _ => {
                            let h = cruise(m, base);
                            round(w, &mut f, centre, h, Knob::CruiseSpeed.fx(), &lim);
                            aloft = true;
                        }
                    }
                }
                CARRY => {
                    // Climbing with somebody in its talons, on round its circle.
                    let h = ceiling(m, base, base.add(Knob::CarryHeight.fx()));
                    let climb = Limits {
                        climb: Knob::LiftClimb.fx(),
                        ..lim
                    };
                    round(w, &mut f, centre, h, Knob::PassSpeed.fx(), &climb);
                    aloft = true;
                    beating = true;
                }
                DOWNWASH => {
                    let under = fight::wash_point(w);
                    let over = V3::new(under.x, under.y.add(Knob::HoverHeight.fx()), under.z);
                    match phase {
                        0 => f.arrive(over, left.saturating_add(1), arrive_turn()),
                        1 => {
                            f.pos = over;
                            f.hold();
                        }
                        _ => {
                            let h = cruise(m, base);
                            round(w, &mut f, centre, h, Knob::CruiseSpeed.fx(), &lim);
                        }
                    }
                    aloft = true;
                    beating = phase == 1;
                }
                VOLLEY => {
                    let (start, along, length, _) = fight::rake_lane(&w.lore);
                    let up = Knob::VolleyHeight.fx();
                    let g = fight::ground_at(w, start);
                    let a = SPECIES.attack(VOLLEY);
                    match phase {
                        0 => {
                            let mark = V3::new(start.x, g.add(up), start.z);
                            f.arrive(mark, left.saturating_add(1), arrive_turn());
                        }
                        1 => {
                            let gone = a.active.saturating_sub(left) as i32;
                            let s = length
                                .mul(Fx::from_int(gone))
                                .div(Fx::from_int(a.active.max(1) as i32));
                            let at = start.add(along.scale(s));
                            f.arrive(V3::new(at.x, g.add(up), at.z), 1, arrive_turn());
                            f.yaw = math::atan2_turns(along.z, along.x);
                        }
                        _ => {
                            let h = cruise(m, base);
                            round(w, &mut f, centre, h, Knob::CruiseSpeed.fx(), &lim);
                        }
                    }
                    aloft = true;
                }
                SCREECH | BUFFET => {
                    // On the ground, turning on the spot with the shared
                    // steer: where it stands is held.
                    f.pos.y = fight::ground_at(w, f.pos);
                    f.hold();
                    f.yaw = shared_yaw;
                    aloft = false;
                }
                PERCH => {
                    let top = fight::perch_top(w).unwrap_or(V3::new(
                        centre.x,
                        fight::ground_at(w, centre),
                        centre.z,
                    ));
                    if phase == 0 {
                        // Round toward it, then down onto it over the last
                        // second and a half.
                        let land = 90u16;
                        if left > land {
                            let over = V3::new(top.x, top.y.add(Knob::PerchOver.fx()), top.z);
                            let near = circle_point(top, f.pos, over.y);
                            let gap = math::wide_flat_dist(f.pos, top);
                            let to = if gap.raw() < Knob::PerchNear.fx().raw() {
                                over
                            } else {
                                near
                            };
                            f.steer(to, Knob::CruiseSpeed.fx(), &lim);
                            aloft = true;
                        } else {
                            f.arrive(top, left.saturating_add(1), arrive_turn());
                            aloft = left > 0;
                            beating = true;
                        }
                    } else {
                        f.pos = top;
                        f.hold();
                        aloft = false;
                        if !fight::perched(m) {
                            fight::set_flag(m, flag::PERCHED, true);
                            let desperate = fight::below(m, Knob::DesperateHealth.fx());
                            let rest = if desperate {
                                Knob::PerchRestDesperate.raw()
                            } else {
                                Knob::PerchRest.raw()
                            };
                            w.lore.set_word(word::REST, rest.max(0) as u32);
                        }
                    }
                }
                LIFT => {
                    if phase == 0 {
                        // The gather: on the ground, wings coming in.
                        f.pos.y =
                            fight::ground_at(w, f.pos).max(f.pos.y.min(fight::ground_at(w, f.pos)));
                        f.hold();
                        f.yaw = shared_yaw;
                        aloft = false;
                    } else {
                        // Up: steeply, on round its circle.
                        let h = cruise(m, base);
                        let up = Limits {
                            climb: Knob::LiftClimb.fx(),
                            ..lim
                        };
                        round(w, &mut f, centre, h, Knob::CruiseSpeed.fx(), &up);
                        aloft = true;
                        beating = true;
                    }
                }
                ROLL => {
                    ride_flight(w, m, &mut f, home, base, &lim);
                    aloft = true;
                }
                _ => {}
            }
        }
        Doing::Prowl | Doing::Flinch { .. } | Doing::Stumble { .. } => {
            if !aloft {
                // Landed or perched: held where it stands, turning on the
                // spot with the shared steer.
                f.pos.y = fight::ground_at(w, f.pos);
                f.hold();
                f.yaw = shared_yaw;
            } else if riders > 0 {
                beating = ride_flight(w, m, &mut f, home, base, &lim);
            } else {
                fight::set_ride(&mut w.lore, 0, ride::NONE, 0);
                let glide = w.lore.word(word::GLIDE);
                if glide > 0 {
                    // **Clipped**: a low glide along its circle.
                    w.lore.set_word(word::GLIDE, glide - 1);
                    let h = base.add(Knob::GlideHeight.fx());
                    round(w, &mut f, centre, h, Knob::PassSpeed.fx(), &lim);
                } else {
                    let h = cruise(m, base);
                    round(w, &mut f, centre, h, Knob::CruiseSpeed.fx(), &lim);
                }
            }
        }
    }
    if riders == 0
        && fight::ride_state(&w.lore).1 != ride::NONE
        && !matches!(m.doing.attacking(), Some(ROLL))
    {
        fight::set_ride(&mut w.lore, 0, ride::NONE, 0);
    }
    // A broken wing lists it toward that side.
    if aloft {
        let list = Knob::BrokenList.fx();
        if fight::broken(m, 0) {
            bank_list = bank_list.sub(list);
        }
        if fight::broken(m, 1) {
            bank_list = bank_list.add(list);
        }
    }
    // Never through the floor, never out of the bounds.
    let floor = fight::ground_at(w, f.pos);
    if f.pos.y.raw() < floor.raw() {
        f.pos.y = floor;
        if f.vy.raw() < 0 {
            f.vy = Fx::ZERO;
        }
    }
    let b = w.arena().bounds;
    f.pos.x = f.pos.x.clamp(b.lo_x, b.hi_x);
    f.pos.z = f.pos.z.clamp(b.lo_z, b.hi_z);
    finish(w, m, f, aloft, beating, bank_list);
}

/// Write the flight back onto the body and the lore.
fn finish(w: &mut World, m: &mut Monster, f: Flight, aloft: bool, beating: bool, list: Fx) {
    m.pos = f.pos;
    m.yaw = f.yaw;
    m.yaw_rate = Fx::ZERO;
    m.speed = if aloft { Fx::ZERO } else { m.speed };
    fight::set_flag(m, flag::ALOFT, aloft);
    // **A beat starts and stops at the bottom of its stroke**, where the
    // heave is nothing: turned on or off mid-beat, the back would jump.
    if beating != (fight::flags(m) & BEATING != 0) && beat_phase(m) == 0 {
        fight::set_flag(m, BEATING, beating);
    }
    // **Low** is measured from the plateau the fight is on -- not from the
    // tower's top, which it may pass a few metres over.
    let low = f.pos.y.sub(fight::base(w)).raw() < Knob::LowBelow.fx().raw();
    fight::set_flag(m, flag::LOW, low);
    // The bank is the turn it is making, as a share of its fullest.
    let turn = Knob::AirTurn.fx();
    let share = if turn.raw() > 0 {
        f.yaw_rate.div(turn).clamp(Fx::ONE.neg(), Fx::ONE)
    } else {
        Fx::ZERO
    };
    let bank = if aloft {
        share.mul(Knob::BankMax.fx()).add(list)
    } else {
        Fx::ZERO
    };
    fight::set_bank(m, bank);
    let pitch = if aloft && f.speed.raw() > Fx::ONE.raw() {
        math::atan2_turns(f.vy, f.speed).clamp(Fx::ratio(-3, 20), Fx::ratio(3, 20))
    } else {
        Fx::ZERO
    };
    fight::set_pitch(m, pitch);
    f.save(&mut w.lore);
}

/// The flag on the body that says its wings are beating hard: the ride's
/// climb, the hover, the lift. What the heave reads.
pub const BEATING: u32 = 128;

/// **The hop**, grounded for good: crouch, then up and onto its circle.
fn hop(w: &World, m: &mut Monster, f: &mut Flight, kind: u8) {
    let a = SPECIES.attack(kind);
    if let Doing::Startup { left, .. } = m.doing {
        let flight = (a.startup / 2).max(1);
        if left <= flight {
            let aim = m.aimed_at();
            let at = V3::new(aim.x, fight::ground_at(w, aim), aim.z);
            f.arrive(at, left.saturating_add(1), arrive_turn());
        }
    }
    m.pos = f.pos;
}

/// **The ride**: climb round its circle beating its wings, a lap, the roll,
/// the swoop over the far side, and up again four metres higher. Returns
/// whether its wings are beating.
fn ride_flight(
    w: &mut World,
    m: &mut Monster,
    f: &mut Flight,
    centre: V3,
    base: Fx,
    lim: &Limits,
) -> bool {
    let (mut lap, mut phase, mut frames) = fight::ride_state(&w.lore);
    if phase == ride::NONE {
        phase = ride::CLIMB;
        lap = 0;
        frames = 0;
    }
    frames = frames.saturating_add(1);
    let speed = Knob::RideSpeed.fx();
    let mut beating = false;
    match phase {
        ride::CLIMB => {
            let h = ceiling(
                m,
                base,
                base.add(Knob::RideClimb.fx())
                    .add(Knob::RideStep.fx().mul(Fx::from_int(lap as i32))),
            );
            round(w, f, centre, h, speed, lim);
            beating = true;
            let there = f.pos.y.sub(h).abs().raw() < Fx::ONE.raw();
            if frames as i32 >= Knob::RideLap.raw() && there && m.doing.free() {
                super::fight::start(m, ROLL);
                phase = ride::ROLL;
                frames = 0;
            }
        }
        ride::ROLL => {
            round(w, f, centre, f.pos.y, speed, lim);
            if !matches!(m.doing.attacking(), Some(ROLL)) {
                phase = ride::SWOOP;
                frames = 0;
            }
        }
        _ => {
            let h = base.add(Knob::SwoopHeight.fx());
            let swoop = Limits {
                sink: Knob::SinkRate.fx(),
                ..*lim
            };
            round(w, f, centre, h, speed, &swoop);
            let low = f.pos.y.sub(h).raw() < Fx::ratio(1, 2).raw();
            if !low {
                frames = 0;
            }
            if low && frames as i32 >= Knob::SwoopFrames.raw() {
                phase = ride::CLIMB;
                lap = lap.saturating_add(1);
                frames = 0;
            }
        }
    }
    fight::set_ride(&mut w.lore, lap, phase, frames);
    beating
}

/// **A crash comes down**: riders aboard take their share of the fall, and
/// it is counted.
fn landed_a_crash(w: &mut World, m: &Monster, slot: usize, ground: Fx) {
    let from = Fx::from_raw(w.lore.int(word::FELL_FROM));
    let height = from.sub(ground).max(Fx::ZERO);
    let past = height.sub(crate::tuning::fall_free()).max(Fx::ZERO);
    let damage = past
        .mul(Fx::from_int(crate::tuning::fall_per_metre()))
        .mul(Knob::CrashRideShare.fx())
        .to_int();
    for p in w.players.iter_mut() {
        if p.health > 0 && p.aboard() && crate::monster::mount_slot(p.mount) == slot && damage > 0 {
            p.wound(damage);
        }
    }
    let _ = m;
}

// ---------------------------------------------------------------------------
// The pose
// ---------------------------------------------------------------------------

/// **What it looks like in the air**: a wingbeat or a glide while it flies
/// free; the shared choice otherwise.
pub fn clip(m: &Monster) -> Option<Pose> {
    if !fight::aloft(m) {
        return None;
    }
    let sp = &SPECIES;
    match m.doing {
        Doing::Prowl | Doing::Flinch { .. } | Doing::Stumble { .. } => {
            let phase = Fx::from_raw(m.beat as i32);
            if fight::flags(m) & BEATING != 0 {
                Some(beast::sample(sp, Clip::Fly as usize, phase))
            } else {
                let glide = beast::sample(sp, Clip::Glide as usize, phase);
                Some(glide)
            }
        }
        _ => None,
    }
}

/// How far through the roll it has turned, in turns: an S-curve, so the
/// roll starts and stops without a jolt and is fastest in the middle.
pub fn rolled(m: &Monster) -> Fx {
    let Doing::Active { kind: ROLL, left } = m.doing else {
        return Fx::ZERO;
    };
    let a = SPECIES.attack(ROLL);
    let t = fight::through(left, a.active);
    // t - sin(2 pi t) / (2 pi): zero turn rate at both ends.
    let wave = crate::fixed::sin_turns(t).div(math::turns_to_radians(Fx::ONE));
    Knob::RollTurns.fx().mul(t.sub(wave))
}

/// **The wingbeat's heave**: a downstroke kicks the back up at `BeatKick`
/// and eases it to the top over `BeatDown` frames; the upstroke lets it
/// sink back over the rest of the beat. A beat is one breath of its clock
/// (`BreathRate`), so this is a function of the body.
pub fn heave(m: &Monster) -> Fx {
    if fight::flags(m) & BEATING == 0 {
        return Fx::ZERO;
    }
    let period = beat_period(m);
    let k = beat_phase(m);
    let down = Knob::BeatDown.raw().clamp(1, period - 1);
    let top = heave_top_over(down);
    if k < down {
        let u = Fx::ONE.sub(Fx::ratio(k, down));
        top.mul(Fx::ONE.sub(u.mul(u)))
    } else {
        top.mul(Fx::ONE.sub(Fx::ratio(k - down, period - down)))
    }
}

/// **The highest a wingbeat heaves the back**: kicked up at `BeatKick`,
/// slowing evenly to nothing over the downstroke -- half the kick times its
/// time.
pub fn heave_top() -> Fx {
    heave_top_over(Knob::BeatDown.raw().max(1))
}

fn heave_top_over(down: i32) -> Fx {
    math::half(Knob::BeatKick.fx().mul(DT).mul(Fx::from_int(down)))
}

/// Frames in one wingbeat: one breath of its clock.
pub fn beat_period(m: &Monster) -> i32 {
    let rate = m.sp().breath_rate().max(1) as i32;
    (65536 / rate).max(2)
}

/// How many frames into its wingbeat it is.
pub fn beat_phase(m: &Monster) -> i32 {
    let rate = m.sp().breath_rate().max(1) as i32;
    (m.beat as i32 / rate).rem_euclid(beat_period(m))
}

/// **The last word on the pose**: its bank and the roll on the root bone's
/// roll, its pitch on the root's pitch, and the wingbeat's heave on the
/// hips. What the rig -- and so the hit test and the riders -- is built
/// from.
pub fn repose(m: &Monster, mut p: Pose) -> Pose {
    let roll = fight::bank(m).add(rolled(m));
    p.bone[bones::ROOT].z = p.bone[bones::ROOT].z.add(roll);
    p.bone[bones::ROOT].x = p.bone[bones::ROOT].x.add(fight::pitch(m));
    p.hips.y = p.hips.y.add(heave(m));
    p
}
