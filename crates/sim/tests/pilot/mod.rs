//! **A stand-in player for the valley's climbs** (2026-10-16), shared by the
//! tests that walk them: one hop jumped the way a person tries it, landing or
//! catching the edge and climbing; and a gully crossed by kicking from wall
//! to wall. Each try stands the fighter on the hop's start, so a route's hops
//! are judged one at a time and a miss names the hop.
//!
//! Everything here is in the world's own coordinates: a caller with map
//! boxes takes the map's origin off first ([`local`]).

#![allow(dead_code)]

use sim::arena::Solid;
use sim::{Fx, Input, V3, World};

pub fn cm(v: i32) -> Fx {
    sim::arena::cm(v)
}

/// A map box in the world's coordinates.
pub fn local(w: &World, s: Solid) -> Solid {
    let o = w.map_origin();
    Solid {
        min: s.min.sub(o),
        max: s.max.sub(o),
        material: s.material,
        shape: s.shape,
    }
}

fn mid(a: Fx, b: Fx) -> Fx {
    Fx::from_raw(a.raw() / 2 + b.raw() / 2)
}

/// How far a body at `p` can go along the flat direction `d` before leaving
/// the top of `s`.
fn to_the_edge(s: &Solid, p: V3, d: (Fx, Fx)) -> Fx {
    let mut t = Fx::from_int(1000);
    for (pos, dir, lo, hi) in [(p.x, d.0, s.min.x, s.max.x), (p.z, d.1, s.min.z, s.max.z)] {
        if dir.raw() > 0 {
            t = t.min(hi.sub(pos).div(dir));
        } else if dir.raw() < 0 {
            t = t.min(lo.sub(pos).div(dir));
        }
    }
    t
}

/// The point of `s`'s top nearest `to`, kept `inset` inside its edges.
fn nearest_on(s: &Solid, to: V3, inset: Fx) -> (Fx, Fx) {
    let clamp = |v: Fx, lo: Fx, hi: Fx| {
        let (lo, hi) = (lo.add(inset), hi.sub(inset));
        if lo.raw() > hi.raw() {
            mid(lo, hi)
        } else {
            v.max(lo).min(hi)
        }
    };
    (clamp(to.x, s.min.x, s.max.x), clamp(to.z, s.min.z, s.max.z))
}

/// Stand fighter one at a point, still, facing `d`.
fn stand(w: &mut World, at: V3, d: V3) {
    let p = &mut w.players[0];
    p.pos = at;
    p.vel = V3::ZERO;
    p.grounded = true;
    p.facing = d;
    p.fall_over = at.y;
    p.ledge = sim::state::Ledge::NONE;
    p.health = p.health.max(1000);
}

/// A box given in the frame whose map origin was `o`, in the world's frame
/// now.
fn moved(w: &World, o: V3, s: Solid) -> Solid {
    let d = w.map_origin().sub(o);
    Solid {
        min: s.min.sub(d),
        max: s.max.sub(d),
        material: s.material,
        shape: s.shape,
    }
}

/// Fighter one's feet in the frame a try started in (`o`, its map origin
/// then): the world moves into the next place's coordinates when a hop
/// crosses into it, and a try is judged in the frame it was set up in.
fn feet(w: &World, o: V3) -> V3 {
    w.players[0].pos.add(w.map_origin()).sub(o)
}

/// Is fighter one standing on `to`'s top -- its edge counts, a body is a
/// metre across, and so does a cairn on it?
fn on_top(w: &World, o: V3, to: &Solid) -> bool {
    let at = feet(w, o);
    w.players[0].grounded
        && to.over(at.x, at.z, Fx::ratio(1, 2))
        && at.y.sub(to.max.y).raw() > Fx::ratio(-1, 10).raw()
        && at.y.sub(to.max.y).raw() < Fx::ratio(1, 2).raw()
}

/// **One hop, jumped**, the way a person would try it: from a run-up of
/// nothing to two and a half metres, braking in the air sooner or later or
/// not at all, until one stands on the top -- landed, or caught at the hands
/// and climbed. `Ok` if any does; the last miss if none.
pub fn jump(w: &mut World, from: Solid, to: Solid) -> Result<(), String> {
    let o = w.map_origin();
    let mut last = String::new();
    for run in [0, 50, 100, 150, 250] {
        for coast in [None, Some(0), Some(100), Some(200)] {
            // A try can move the world into another place's frame: the
            // next is set up in the frame the boxes were given in.
            let (from, to) = (moved(w, o, from), moved(w, o, to));
            match try_jump(w, from, to, Fx::ratio(run, 100), coast) {
                Ok(()) => return Ok(()),
                Err(e) => last = e,
            }
        }
    }
    Err(last)
}

/// One try: stood on `from`, `run` back from the point nearest `to` (as far
/// as the top allows), runs at the middle of `to`'s top, jumps at the edge
/// and holds it, steering at the middle -- and, `coast`ing, pulls back once
/// it is over it or within that many centimetres of its middle. Hanging
/// from an edge, it pushes into the wall, which climbs.
fn try_jump(
    w: &mut World,
    from: Solid,
    to: Solid,
    run: Fx,
    coast: Option<i32>,
) -> Result<(), String> {
    let target = V3::new(mid(to.min.x, to.max.x), to.max.y, mid(to.min.z, to.max.z));
    let inset = Fx::ratio(35, 100);
    let near = nearest_on(&from, target, inset);
    let flat = V3::new(target.x.sub(near.0), Fx::ZERO, target.z.sub(near.1));
    let d = if flat.flat_len().raw() == 0 {
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
    } else {
        flat.normalized()
    };
    let back = V3::new(near.0, from.max.y, near.1).sub(d.scale(run));
    let start = nearest_on(&from, back, inset);
    stand(w, V3::new(start.0, from.max.y, start.1), d);
    let o = w.map_origin();
    let mut jumped = false;
    for f in 0..420 {
        let p = &w.players[0];
        let at = feet(w, o);
        let (dx, dz) = (target.x.sub(at.x), target.z.sub(at.z));
        let aim = sim::math::atan2_turns(dz, dx).raw() as u16;
        let edge = to_the_edge(&from, at, (d.x, d.z));
        if !jumped && p.grounded && (edge.raw() <= Fx::ratio(3, 10).raw() || f > 90) {
            jumped = true;
        }
        let off = sim::math::wide_len(V3::new(dx, Fx::ZERO, dz));
        let hanging = p.ledge.is_some();
        let brake = !hanging
            && coast
                .is_some_and(|near| to.over(at.x, at.z, Fx::ZERO) || off.raw() < cm(near).raw());
        let walk = if jumped && brake { Input::S } else { Input::W };
        let bits = walk | if jumped && !hanging { Input::SPACE } else { 0 };
        w.advance([Input::looking_at(bits, aim, 0), Input::new(0)]);
        if std::env::var("PILOT").is_ok() && f % 4 == 0 {
            println!(
                "{f} {:?} g {} ledge {} run {run:?} coast {coast:?}",
                feet(w, o),
                w.players[0].grounded,
                w.players[0].ledge.is_some()
            );
        }
        if w.players[0].health <= 0 {
            return Err("died".into());
        }
        if jumped && f > 20 && w.players[0].grounded {
            // On it -- or on something higher still, which is further up.
            let higher = feet(w, o).y.raw() > to.max.y.add(Fx::ratio(1, 2)).raw();
            return if on_top(w, o, &to) || higher {
                Ok(())
            } else {
                Err(format!(
                    "came down at {:?}, not on the top at {} m",
                    feet(w, o),
                    to.max.y.to_f32_for_render()
                ))
            };
        }
    }
    Err(format!("never came down: at {:?}", feet(w, o)))
}

/// **A gully, crossed** ([`sim::arena::gully`]): from `from`'s top along the
/// flat axis `dir` (`(0, 1)` is north), to `to`'s. Tried as a person learns
/// it: run along one side of the takeoff or the other, jump at its edge,
/// drift into the wall on that side, and press jump each time a wall is
/// touched -- holding forward, because the stick into a wall is a slide and a
/// slide bleeds the run away. `Ok` once it stands on `to`; the last miss if
/// no try does. Returns the kicks the crossing took.
pub fn kick_across(w: &mut World, from: Solid, to: Solid, dir: (i32, i32)) -> Result<u32, String> {
    let o = w.map_origin();
    let mut last = String::new();
    for side in [-1, 1] {
        for off in [60, 85, 98] {
            for hold in [30, 18, 8] {
                let (from, to) = (moved(w, o, from), moved(w, o, to));
                match try_kicks(w, from, to, dir, side * off, hold) {
                    Ok(k) => return Ok(k),
                    Err(e) => last = e,
                }
            }
        }
    }
    Err(last)
}

fn try_kicks(
    w: &mut World,
    from: Solid,
    to: Solid,
    dir: (i32, i32),
    off: i32,
    hold: u32,
) -> Result<u32, String> {
    let d = V3::new(Fx::from_int(dir.0), Fx::ZERO, Fx::from_int(dir.1));
    // Across the run, to the right of it.
    let right = V3::new(d.z.neg(), Fx::ZERO, d.x);
    let (mx, mz) = (mid(from.min.x, from.max.x), mid(from.min.z, from.max.z));
    // Six metres back from the takeoff's edge, or its far side.
    let reach = to_the_edge(&from, V3::new(mx, from.max.y, mz), (d.x, d.z));
    let back = Fx::from_int(6).min(reach.add(reach).sub(Fx::ratio(1, 2)));
    let edge = V3::new(mx, from.max.y, mz).add(d.scale(reach));
    let start = edge.sub(d.scale(back)).add(right.scale(cm(off)));
    stand(w, start, d);
    let o = w.map_origin();
    let aim = sim::math::atan2_turns(d.z, d.x).raw() as u16;
    // The strafe toward the wall on our side: `D` is to the right.
    let drift = if off > 0 { Input::D } else { Input::A };
    let mut jumped: Option<u32> = None;
    let mut touched = false;
    let mut release = false;
    let mut kicks = 0;
    for f in 0..600u32 {
        let p = &w.players[0];
        let mut bits = Input::W;
        match jumped {
            None => {
                let left = to_the_edge(&from, feet(w, o), (d.x, d.z));
                if p.grounded && left.raw() <= Fx::ratio(3, 10).raw() {
                    jumped = Some(f);
                    bits |= Input::SPACE;
                }
            }
            Some(at) => {
                if p.wall_since == 0 {
                    touched = true;
                }
                if !touched {
                    bits = drift;
                }
                let kick = touched && p.wall_since <= 2 && !p.ledge.is_some();
                if kick && !release {
                    release = true;
                } else if kick {
                    bits |= Input::SPACE;
                    kicks += 1;
                    release = false;
                } else if kicks == 0 && f - at < hold {
                    bits |= Input::SPACE;
                }
            }
        }
        w.advance([Input::looking_at(bits, aim, 0), Input::new(0)]);
        if w.players[0].health <= 0 {
            return Err("died".into());
        }
        if jumped.is_some() && w.players[0].grounded {
            return if on_top(w, o, &to) {
                Ok(kicks)
            } else {
                Err(format!(
                    "came down at {:?} after {kicks} kicks, not on the landing",
                    feet(w, o)
                ))
            };
        }
        if feet(w, o).y.raw() < to.max.y.sub(Fx::from_int(20)).raw() {
            return Err(format!("fell, after {kicks} kicks"));
        }
    }
    Err("never came down".into())
}
