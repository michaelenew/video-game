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

/// **A turn-around, made** (2026-10-22): out of a tunnel along the flat axis
/// `dir`, its floor `from` and its mouth that floor's far edge, against a
/// wall across the way, and back onto `to` -- a top over the tunnel the
/// tunnel's own roof keeps a jump from. Tried as a person learns it: a
/// run-up out of the tunnel, a jump at the mouth held for a while, a press
/// as the wall is touched, and back the way it came, steering for the
/// middle of the near end of `to`. `Ok` once it stands on `to`.
pub fn turn_around(w: &mut World, from: Solid, to: Solid, dir: (i32, i32)) -> Result<(), String> {
    turn_tries(w, from, to, dir, false)
}

/// **The same, without the wall**: out of the mouth and straight back, never
/// kicking -- what a turn-around must not be made by. `Ok` if any try lands.
pub fn turn_without_a_wall(
    w: &mut World,
    from: Solid,
    to: Solid,
    dir: (i32, i32),
) -> Result<(), String> {
    turn_tries(w, from, to, dir, true)
}

fn turn_tries(
    w: &mut World,
    from: Solid,
    to: Solid,
    dir: (i32, i32),
    no_wall: bool,
) -> Result<(), String> {
    let o = w.map_origin();
    let mut last = String::new();
    for back in [0, 150, 300, 500] {
        for hold in [40, 20, 10] {
            for wait in [0, 3, 6, 12, 20] {
                if !no_wall && (wait > 6 || back == 0) {
                    continue;
                }
                let (from, to) = (moved(w, o, from), moved(w, o, to));
                match try_turn(w, from, to, dir, back, hold, wait, no_wall) {
                    Ok(()) => return Ok(()),
                    Err(e) => last = e,
                }
            }
        }
    }
    Err(last)
}

#[allow(clippy::too_many_arguments)]
fn try_turn(
    w: &mut World,
    from: Solid,
    to: Solid,
    dir: (i32, i32),
    back: i32,
    hold: u32,
    wait: u32,
    no_wall: bool,
) -> Result<(), String> {
    let d = V3::new(Fx::from_int(dir.0), Fx::ZERO, Fx::from_int(dir.1));
    let (mx, mz) = (mid(from.min.x, from.max.x), mid(from.min.z, from.max.z));
    let reach = to_the_edge(&from, V3::new(mx, from.max.y, mz), (d.x, d.z));
    let edge = V3::new(mx, from.max.y, mz).add(d.scale(reach));
    // From standing at the very edge, a body's middle on it: a standing jump.
    let back = if back == 0 { -1 } else { back };
    stand(w, edge.sub(d.scale(cm(back.max(0)))), d);
    let o = w.map_origin();
    // Back over the near end of the landing: its edge toward the wall, a
    // metre and a half in.
    let tr = to_the_edge(
        &to,
        V3::new(mid(to.min.x, to.max.x), to.max.y, mid(to.min.z, to.max.z)),
        (d.x, d.z),
    );
    let target = V3::new(mid(to.min.x, to.max.x), to.max.y, mid(to.min.z, to.max.z))
        .add(d.scale(tr.sub(Fx::ratio(3, 2))));
    let out = sim::math::atan2_turns(d.z, d.x).raw() as u16;
    let mut jumped: Option<u32> = None;
    let mut touched: Option<u32> = None;
    let mut kicked = false;
    let mut release = false;
    for f in 0..600u32 {
        let p = &w.players[0];
        let at = feet(w, o);
        let mut aim = out;
        let mut bits = Input::W;
        match jumped {
            None => {
                let left = to_the_edge(&from, at, (d.x, d.z));
                if p.grounded && left.raw() <= Fx::ratio(3, 10).raw() {
                    jumped = Some(f);
                    bits |= Input::SPACE;
                }
            }
            Some(at_f) if no_wall && !kicked => {
                // Back as soon as it has left the mouth, `wait` later, the
                // jump held throughout.
                if f - at_f >= wait {
                    kicked = true;
                }
                bits |= Input::SPACE;
            }
            Some(at_f) => {
                if !kicked {
                    if touched.is_none() && p.wall_since == 0 {
                        touched = Some(f);
                    }
                    let ready = touched.is_some_and(|t| f >= t + wait);
                    if ready && !release {
                        release = true;
                    } else if ready && p.wall_since <= 3 {
                        bits |= Input::SPACE;
                        kicked = true;
                    } else if touched.is_none() && f - at_f < hold {
                        bits |= Input::SPACE;
                    }
                } else {
                    let (dx, dz) = (target.x.sub(at.x), target.z.sub(at.z));
                    aim = sim::math::atan2_turns(dz, dx).raw() as u16;
                    if to.over(at.x, at.z, Fx::ZERO) {
                        bits = Input::S;
                    }
                    bits |= Input::SPACE;
                }
            }
        }
        w.advance([Input::looking_at(bits, aim, 0), Input::new(0)]);
        if std::env::var("PILOT").is_ok() && f % 4 == 0 {
            println!(
                "{f} {:?} g {} wall {} ledge {} back {back} hold {hold} wait {wait}",
                feet(w, o),
                w.players[0].grounded,
                w.players[0].wall_since,
                w.players[0].ledge.is_some()
            );
        }
        if w.players[0].health <= 0 {
            return Err("died".into());
        }
        if jumped.is_some() && f > 10 && w.players[0].grounded {
            return if on_top(w, o, &to) {
                Ok(())
            } else {
                Err(format!(
                    "came down at {:?}, not on the top at {} m (kicked {kicked})",
                    feet(w, o),
                    to.max.y.to_f32_for_render()
                ))
            };
        }
        if feet(w, o).y.raw() < to.max.y.sub(Fx::from_int(30)).raw() {
            return Err(format!("fell (kicked {kicked})"));
        }
    }
    Err("never came down".into())
}

// ---------------------------------------------------------------------------
// Drifts: boxes that move (`sim::drift`, 2026-10-22)
// ---------------------------------------------------------------------------

use sim::drift::Drift;

/// A map drift's box in the world's frame on the world's frame number.
pub fn drift_now(w: &World, d: &Drift) -> Solid {
    local(w, d.at(w.frame))
}

/// Idle, standing on `on` (a box in the world's frame), until the world's
/// frame number is one `ok` says yes to -- at most a few cycles.
fn idle_until(w: &mut World, on: Option<Solid>, d: &Drift, ok: impl Fn(u32) -> bool) {
    let here = w.players[0].pos;
    for _ in 0..4 * d.period() + 10 {
        if ok(w.frame.wrapping_add(1)) {
            return;
        }
        let at = match on {
            Some(s) => V3::new(mid(s.min.x, s.max.x), s.max.y, mid(s.min.z, s.max.z)),
            None => here,
        };
        stand(w, at, w.players[0].facing);
        w.advance([Input::new(0), Input::new(0)]);
    }
}

/// Is the drift waiting at its first stop on frame `f`, with `left` or more
/// frames of the wait still to go?
fn waiting_at_first(d: &Drift, f: u32, left: u32) -> bool {
    let k = f.wrapping_add(d.phase as u32) % d.period();
    k + left < d.wait as u32
}

/// **A lift, ridden** (2026-10-22): stood on its deck while it waits at its
/// foot, carried to its head standing still, and walked off on to `to`.
/// `Ok` once standing on `to`.
pub fn ride(w: &mut World, d: &Drift, to: Solid) -> Result<(), String> {
    idle_until(w, None, d, |f| waiting_at_first(d, f, 30));
    let deck = drift_now(w, d);
    let at = V3::new(
        mid(deck.min.x, deck.max.x),
        deck.max.y,
        mid(deck.min.z, deck.max.z),
    );
    stand(w, at, V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
    let o = w.map_origin();
    // Carried, doing nothing, until it has come to rest at its head.
    let mut f = 0;
    loop {
        w.advance([Input::new(0), Input::new(0)]);
        f += 1;
        let k = w.frame.wrapping_add(d.phase as u32) % d.period();
        if k >= (d.wait + d.go) as u32 + 5 {
            break;
        }
        if f > d.period() * 2 {
            return Err("the lift never arrived".into());
        }
    }
    let deck = drift_now(w, d);
    let p = &w.players[0];
    if !(p.grounded
        && deck.over(p.pos.x, p.pos.z, Fx::ZERO)
        && p.pos.y.sub(deck.max.y).abs().raw() < Fx::ratio(1, 4).raw())
    {
        return Err(format!(
            "not on the deck at its head: at {:?}, deck top {}",
            p.pos,
            deck.max.y.to_f32_for_render()
        ));
    }
    // Off it, toward the middle of `to`.
    let target = V3::new(mid(to.min.x, to.max.x), to.max.y, mid(to.min.z, to.max.z));
    for _ in 0..240 {
        let at = feet(w, o);
        let aim = sim::math::atan2_turns(target.z.sub(at.z), target.x.sub(at.x)).raw() as u16;
        w.advance([Input::looking_at(Input::W, aim, 0), Input::new(0)]);
        if on_top(w, o, &to) && to.over(feet(w, o).x, feet(w, o).z, Fx::ZERO.sub(Fx::ratio(1, 2))) {
            return Ok(());
        }
    }
    Err(format!(
        "did not walk off on to the top at {} m: at {:?}",
        to.max.y.to_f32_for_render(),
        feet(w, o)
    ))
}

/// **A jump off a moving stone, at its speed** (2026-10-22): stood at the
/// back of `d`'s deck while it waits at its first stop, the run begun
/// `delay` frames after it sets off, a jump at its front edge, held, and
/// steered at the middle of `to`. Tried for every delay from nothing to
/// most of its way out; `Ok` with the delays that land.
pub fn launch(w: &mut World, d: &Drift, to: Solid) -> Result<Vec<u32>, String> {
    let mut good = Vec::new();
    let mut last = String::new();
    for delay in (0..d.go as i32 + 20).step_by(3) {
        match try_launch(w, d, to, delay - 20) {
            Ok(()) => good.push(delay as u32),
            Err(e) => last = e,
        }
    }
    // Made once more with the first that worked, so it ends on `to`.
    match good.first() {
        Some(&k) => try_launch(w, d, to, k as i32 - 20).map(|()| good),
        None => Err(last),
    }
}

fn try_launch(w: &mut World, d: &Drift, to: Solid, delay: i32) -> Result<(), String> {
    idle_until(w, None, d, |f| waiting_at_first(d, f, 20));
    let o = w.map_origin();
    let target = V3::new(mid(to.min.x, to.max.x), to.max.y, mid(to.min.z, to.max.z));
    let deck = drift_now(w, d);
    let dir = V3::new(
        target.x.sub(mid(deck.min.x, deck.max.x)),
        Fx::ZERO,
        target.z.sub(mid(deck.min.z, deck.max.z)),
    )
    .normalized();
    let back = nearest_on(
        &deck,
        V3::new(
            mid(deck.min.x, deck.max.x),
            deck.max.y,
            mid(deck.min.z, deck.max.z),
        )
        .sub(dir.scale(Fx::from_int(5))),
        Fx::ratio(35, 100),
    );
    stand(w, V3::new(back.0, deck.max.y, back.1), dir);
    let mut jumped = false;
    let mut moving: Option<u32> = None;
    for f in 0..400u32 {
        let at = feet(w, o);
        let aim = sim::math::atan2_turns(target.z.sub(at.z), target.x.sub(at.x)).raw() as u16;
        let mut bits = 0;
        // When it sets off: the first frame it is not at its first stop.
        if moving.is_none()
            && d.share(w.frame.wrapping_add((-delay).max(0) as u32 + 1))
                .raw()
                > 0
        {
            moving = Some(f);
        }
        if moving.is_some_and(|m| f >= m + delay.max(0) as u32) {
            bits = Input::W;
            let deck = drift_now(w, d);
            let edge = to_the_edge(&deck, at, (dir.x, dir.z));
            if !jumped && w.players[0].grounded && edge.raw() <= Fx::ratio(3, 10).raw() {
                jumped = true;
            }
        }
        if jumped {
            bits |= Input::SPACE;
            if to.over(at.x, at.z, Fx::ZERO) {
                bits = (bits & !Input::W) | Input::S;
            }
        }
        w.advance([Input::looking_at(bits, aim, 0), Input::new(0)]);
        if std::env::var("PILOT").is_ok() && f % 4 == 0 {
            println!(
                "L{f} {:?} g {} jumped {jumped} delay {delay}",
                feet(w, o),
                w.players[0].grounded
            );
        }
        if w.players[0].health <= 0 {
            return Err("died".into());
        }
        if jumped && w.players[0].grounded {
            return if on_top(w, o, &to) {
                Ok(())
            } else {
                Err(format!(
                    "came down at {:?}, not on the top at {} m (delay {delay})",
                    feet(w, o),
                    to.max.y.to_f32_for_render()
                ))
            };
        }
        if feet(w, o).y.raw() < to.max.y.sub(Fx::from_int(30)).raw() {
            return Err(format!("fell (delay {delay})"));
        }
    }
    Err(format!("never came down (delay {delay})"))
}

/// **A stone caught going by** (2026-10-22): stood on `from`, a run and a
/// jump at the drift as it passes -- begun on each frame of its way out in
/// turn, aimed where it will be `lead` frames on -- until one lands on its
/// deck. `Ok` with the frames into its way out that worked.
pub fn catch(w: &mut World, from: Solid, d: &Drift) -> Result<Vec<u32>, String> {
    let mut good = Vec::new();
    let mut last = String::new();
    for start in (0..d.go as u32).step_by(3) {
        for lead in [20, 35, 50] {
            match try_catch(w, from, d, start, lead) {
                Ok(()) => {
                    good.push((start, lead));
                    break;
                }
                Err(e) => last = e,
            }
        }
    }
    // Caught once more the first way that worked, so it ends on the deck.
    match good.first() {
        Some(&(start, lead)) => {
            try_catch(w, from, d, start, lead).map(|()| good.iter().map(|g| g.0).collect())
        }
        None => Err(last),
    }
}

fn try_catch(w: &mut World, from: Solid, d: &Drift, start: u32, lead: u32) -> Result<(), String> {
    idle_until(w, Some(from), d, |f| {
        let k = f.wrapping_add(d.phase as u32) % d.period();
        k == d.wait as u32 + start
    });
    let o = w.map_origin();
    // From the point of `from` nearest the deck as it will be.
    let ahead = local(w, d.at(w.frame.wrapping_add(lead)));
    let t = V3::new(
        mid(ahead.min.x, ahead.max.x),
        ahead.max.y,
        mid(ahead.min.z, ahead.max.z),
    );
    let near = nearest_on(&from, t, Fx::ratio(35, 100));
    let dir = V3::new(t.x.sub(near.0), Fx::ZERO, t.z.sub(near.1));
    let dir = if dir.flat_len().raw() == 0 {
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
    } else {
        dir.normalized()
    };
    stand(
        w,
        V3::new(near.0, from.max.y, near.1).sub(dir.scale(Fx::ratio(1, 2))),
        dir,
    );
    let mut jumped = false;
    for f in 0..300u32 {
        let at = feet(w, o);
        let ahead = local(w, d.at(w.frame.wrapping_add(lead.saturating_sub(f))));
        let deck = drift_now(w, d);
        let t = V3::new(
            mid(ahead.min.x, ahead.max.x),
            ahead.max.y,
            mid(ahead.min.z, ahead.max.z),
        );
        let aim = sim::math::atan2_turns(t.z.sub(at.z), t.x.sub(at.x)).raw() as u16;
        let edge = to_the_edge(&from, at, (dir.x, dir.z));
        if !jumped && w.players[0].grounded && edge.raw() <= Fx::ratio(3, 10).raw() {
            jumped = true;
        }
        let mut bits = Input::W;
        if jumped {
            bits |= Input::SPACE;
            // Over it: still, so the deck takes the body on.
            if deck.over(at.x, at.z, Fx::ZERO.sub(Fx::ratio(1, 2))) {
                bits = Input::SPACE;
            }
        }
        w.advance([Input::looking_at(bits, aim, 0), Input::new(0)]);
        if w.players[0].health <= 0 {
            return Err("died".into());
        }
        if jumped && f > 5 && w.players[0].grounded {
            let deck = drift_now(w, d);
            return if sim::drift::attached(&w.players[0], &deck) == Some(sim::drift::Hold::On) {
                Ok(())
            } else {
                Err(format!(
                    "came down at {:?}, not on the deck (start {start}, lead {lead})",
                    feet(w, o)
                ))
            };
        }
        if feet(w, o).y.raw() < from.max.y.sub(Fx::from_int(30)).raw() {
            return Err(format!("fell (start {start}, lead {lead})"));
        }
    }
    Err("never came down".into())
}

/// **Carried on a drift's deck to its far stop**, from standing on it:
/// doing nothing until it has stopped there. `Err` if it put the body
/// anywhere but its deck.
pub fn carried_over(w: &mut World, d: &Drift) -> Result<(), String> {
    for i in 0..d.period() * 2 {
        w.advance([Input::new(0), Input::new(0)]);
        if std::env::var("PILOT").is_ok() && i % 10 == 0 {
            let deck = drift_now(w, d);
            let p = &w.players[0];
            println!(
                "{i} pos {:?} vel {:?} g {} deck top {} x {}..{}",
                p.pos,
                p.vel,
                p.grounded,
                deck.max.y.to_f32_for_render(),
                deck.min.x.to_f32_for_render(),
                deck.max.x.to_f32_for_render()
            );
        }
        let k = w.frame.wrapping_add(d.phase as u32) % d.period();
        if k >= (d.wait + d.go) as u32 + 5 && k < (d.wait + d.go + d.stay) as u32 {
            let deck = drift_now(w, d);
            let p = &w.players[0];
            return if sim::drift::attached(p, &deck) == Some(sim::drift::Hold::On) {
                Ok(())
            } else {
                Err(format!("not on the deck at its far stop: at {:?}", p.pos))
            };
        }
    }
    Err("never got there".into())
}

/// **A riser, ridden** (2026-10-22, `sim::arena::heights::Hop::Riser`):
/// from `from`'s edge along the flat axis `way`, into the slot between two
/// walls `walls` while they wait at their foot; kicked between them, the
/// stick let go through each kick (along a wall it carries you out of the
/// slot) and nudged back to the slot's middle between them; and once they
/// have stopped at the top -- or the feet are over `to` -- out of the
/// slot's far end on to `to`. Tried with a few strafes into the first wall
/// and kick timings; `Ok` once standing on `to`.
pub fn ride_walls(
    w: &mut World,
    from: Solid,
    walls: &[Drift; 2],
    to: Solid,
    way: (i32, i32),
) -> Result<(), String> {
    let o = w.map_origin();
    let mut last = String::new();
    for lead in [30, 20, 10] {
        for side in [-1, 1] {
            let (from, to) = (moved(w, o, from), moved(w, o, to));
            match try_walls(w, from, walls, to, way, lead, side) {
                Ok(()) => return Ok(()),
                Err(e) => last = e,
            }
        }
    }
    Err(last)
}

fn try_walls(
    w: &mut World,
    from: Solid,
    walls: &[Drift; 2],
    to: Solid,
    way: (i32, i32),
    lead: u32,
    side: i32,
) -> Result<(), String> {
    let d = V3::new(Fx::from_int(way.0), Fx::ZERO, Fx::from_int(way.1));
    let wall = &walls[0];
    // Into the slot just before they set off: `lead` frames of their wait
    // left.
    idle_until(w, Some(from), wall, |f| {
        let k = f.wrapping_add(wall.phase as u32) % wall.period();
        k + lead == wall.wait as u32
    });
    let wait = 0;
    let (mx, mz) = (mid(from.min.x, from.max.x), mid(from.min.z, from.max.z));
    let reach = to_the_edge(&from, V3::new(mx, from.max.y, mz), (d.x, d.z));
    let edge = V3::new(mx, from.max.y, mz).add(d.scale(reach));
    stand(w, edge.sub(d.scale(Fx::ratio(1, 2))), d);
    let o = w.map_origin();
    let out = sim::math::atan2_turns(d.z, d.x).raw() as u16;
    let strafe = if side > 0 { Input::D } else { Input::A };
    let mut jumped = false;
    let mut touched: Option<u32> = None;
    let mut release = false;
    let mut leaving = false;
    for f in 0..900u32 {
        let p = &w.players[0];
        let at = feet(w, o);
        let (a, b) = (drift_now(w, &walls[0]), drift_now(w, &walls[1]));
        // Along the slot, from its middle.
        let slot = V3::new(
            mid(mid(a.min.x, a.max.x), mid(b.min.x, b.max.x)),
            Fx::ZERO,
            mid(mid(a.min.z, a.max.z), mid(b.min.z, b.max.z)),
        );
        let along = at.x.sub(slot.x).mul(d.x).add(at.z.sub(slot.z).mul(d.z));
        let top = walls[0].share(w.frame).raw() >= Fx::ONE.raw();
        if top && at.y.raw() >= to.max.y.add(Fx::ONE).raw() {
            leaving = true;
        }
        let mut bits = 0;
        // At the first wall's face, its middle: on its side toward the slot.
        let first = if side > 0 { &b } else { &a };
        // A metre short of its middle, toward the stone: the run carries on.
        let face = V3::new(
            mid(first.min.x, first.max.x),
            Fx::ZERO,
            mid(first.min.z, first.max.z),
        )
        .sub(d.scale(Fx::ONE));
        let into = sim::math::atan2_turns(face.z.sub(at.z), face.x.sub(at.x)).raw() as u16;
        let mut aim = out;
        if !jumped {
            bits = Input::W;
            aim = into;
            let left = to_the_edge(&from, at, (d.x, d.z));
            if p.grounded && left.raw() <= Fx::ratio(3, 10).raw() {
                jumped = true;
                bits |= Input::SPACE;
            }
        } else if p.ledge.is_some() {
            // Hanging from a wall's top: up on to it, into its face, and
            // ride it standing.
            let n = p.ledge.edge.normal;
            aim = sim::math::atan2_turns(n.z.neg(), n.x.neg()).raw() as u16;
            bits = Input::W;
        } else if p.grounded && !leaving {
            // Stood on a wall's top, riding it: still.
        } else if leaving {
            // Out of the slot's end toward `to`: off a wall, the kick's carry
            // the way the stick points.
            let (dx, dz) = (
                mid(to.min.x, to.max.x).sub(at.x),
                mid(to.min.z, to.max.z).sub(at.z),
            );
            aim = sim::math::atan2_turns(dz, dx).raw() as u16;
            bits = Input::W;
            if p.wall_since <= 2 && !release {
                release = true;
            } else if p.wall_since <= 2 {
                bits |= Input::SPACE;
                release = false;
            }
            if to.over(at.x, at.z, Fx::ZERO.sub(Fx::ratio(1, 2))) {
                bits = Input::S;
            }
        } else {
            if touched.is_none() {
                if p.wall_since == 0 {
                    touched = Some(f);
                } else {
                    bits = Input::W | strafe;
                    aim = into;
                }
            }
            // Kicked only while well inside them: not up to their top edge,
            // where the hands would catch it.
            let inside = at.y.raw() < a.max.y.sub(Fx::from_int(3)).raw();
            let kick = touched.is_some_and(|t| f >= t + wait)
                && p.wall_since <= 2
                && !p.ledge.is_some()
                && inside;
            if kick && !release {
                release = true;
            } else if kick {
                bits = Input::SPACE;
                release = false;
            } else if touched.is_some() && p.wall_since > 2 {
                // In the air between them: back toward the slot's middle.
                if along.raw() > Fx::ratio(3, 10).raw() {
                    bits = Input::S;
                } else if along.raw() < Fx::ratio(-3, 10).raw() {
                    bits = Input::W;
                }
            }
        }
        w.advance([Input::looking_at(bits, aim, 0), Input::new(0)]);
        if std::env::var("PILOT").is_ok() && f % 4 == 0 {
            println!(
                "R{f} {:?} g {} wall {} ledge {} along {:.2} walls' top {:.2} leaving {leaving}",
                feet(w, o),
                w.players[0].grounded,
                w.players[0].wall_since,
                w.players[0].ledge.is_some(),
                along.to_f32_for_render(),
                drift_now(w, &walls[0]).max.y.to_f32_for_render(),
            );
        }
        if w.players[0].health <= 0 {
            return Err("died".into());
        }
        let on_a_wall = [&walls[0], &walls[1]].iter().any(|d| {
            sim::drift::attached(&w.players[0], &drift_now(w, d)) == Some(sim::drift::Hold::On)
        });
        if jumped && f > 10 && w.players[0].grounded && !on_a_wall {
            return if on_top(w, o, &to) {
                Ok(())
            } else {
                Err(format!(
                    "came down at {:?}, not on the top at {} m",
                    feet(w, o),
                    to.max.y.to_f32_for_render()
                ))
            };
        }
        if feet(w, o).y.raw() < from.max.y.sub(Fx::from_int(30)).raw() {
            return Err(format!("fell out of the riser (lead {lead}, side {side})"));
        }
    }
    Err("never came out".into())
}

/// Steer for `to` from the air: at its middle, the stick pulled back once
/// over it.
fn steer_for(w: &World, o: V3, to: &Solid) -> (u16, u16) {
    let at = feet(w, o);
    let (dx, dz) = (
        mid(to.min.x, to.max.x).sub(at.x),
        mid(to.min.z, to.max.z).sub(at.z),
    );
    let aim = sim::math::atan2_turns(dz, dx).raw() as u16;
    let over = to.over(at.x, at.z, Fx::ZERO.sub(Fx::ratio(1, 2)));
    (aim, if over { Input::S } else { Input::W })
}

/// Is fighter one down on `to`, or on something as high or higher?
fn down_on(w: &World, o: V3, to: &Solid) -> bool {
    w.players[0].grounded
        && (on_top(w, o, to) || feet(w, o).y.raw() >= to.max.y.sub(Fx::ratio(1, 2)).raw())
}

/// **An updraft, ridden** (2026-10-22, `sim::valley::Vent`): stood on
/// `from` -- the ledge the column rises from -- a jump into the air over
/// it, carried up, and once the feet are over `to`'s top, steered for it.
/// Tried with the steering begun at a few heights; `Ok` once down on `to`.
pub fn updraft(w: &mut World, from: Solid, to: Solid) -> Result<(), String> {
    let o = w.map_origin();
    let mut last = String::new();
    for early in [0, 100, 200, 300] {
        let (from, to) = (moved(w, o, from), moved(w, o, to));
        let at = V3::new(
            mid(from.min.x, from.max.x),
            from.max.y,
            mid(from.min.z, from.max.z),
        );
        stand(w, at, V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
        let o = w.map_origin();
        let mut ok = None;
        for f in 0..900u32 {
            let high = feet(w, o).y.raw() >= to.max.y.sub(cm(early)).raw();
            let (aim, stick) = if high { steer_for(w, o, &to) } else { (0, 0) };
            let space = if f < 10 { Input::SPACE } else { 0 };
            w.advance([Input::looking_at(stick | space, aim, 0), Input::new(0)]);
            if f > 20 && w.players[0].grounded {
                ok = Some(down_on(w, o, &to));
                break;
            }
            if w.players[0].health <= 0 {
                break;
            }
        }
        match ok {
            Some(true) => return Ok(()),
            Some(false) => {
                last = format!(
                    "came down at {:?} (steering from {early} cm under)",
                    feet(w, o)
                )
            }
            None => last = format!("never came down (steering from {early} cm under)"),
        }
    }
    Err(last)
}

/// **A spring, jumped off** (2026-10-22): stood on the drift `d`'s deck
/// while it waits, and -- `delay` frames after it sets off, for every delay
/// through its way up -- a jump, held, steered for `to`. `Ok` with the
/// delays that land on `to` (or higher); played once more with the first,
/// so it ends there.
pub fn spring(w: &mut World, d: &Drift, to: Solid) -> Result<Vec<u32>, String> {
    let mut good = Vec::new();
    let mut last = String::new();
    for delay in (0..d.go as u32 + 10).step_by(4) {
        match try_spring(w, d, to, delay) {
            Ok(()) => good.push(delay),
            Err(e) => last = e,
        }
    }
    match good.first() {
        Some(&k) => try_spring(w, d, to, k).map(|()| good),
        None => Err(last),
    }
}

fn try_spring(w: &mut World, d: &Drift, to: Solid, delay: u32) -> Result<(), String> {
    idle_until(w, None, d, |f| waiting_at_first(d, f, 20));
    let o = w.map_origin();
    let to = moved(w, o, to);
    let deck = drift_now(w, d);
    stand(
        w,
        V3::new(
            mid(deck.min.x, deck.max.x),
            deck.max.y,
            mid(deck.min.z, deck.max.z),
        ),
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
    );
    let mut moving: Option<u32> = None;
    let mut jumped: Option<u32> = None;
    for f in 0..900u32 {
        if moving.is_none() && d.share(w.frame.wrapping_add(1)).raw() > 0 {
            moving = Some(f);
        }
        if jumped.is_none() && moving.is_some_and(|m| f >= m + delay) {
            jumped = Some(f);
        }
        let mut bits = 0;
        let mut aim = 0;
        if let Some(j) = jumped {
            let (a, stick) = steer_for(w, o, &to);
            aim = a;
            bits = stick | if f - j < 40 { Input::SPACE } else { 0 };
        }
        w.advance([Input::looking_at(bits, aim, 0), Input::new(0)]);
        if w.players[0].health <= 0 {
            return Err(format!("died (delay {delay})"));
        }
        if jumped.is_some_and(|j| f > j + 10) && w.players[0].grounded {
            let on_deck = sim::drift::attached(&w.players[0], &drift_now(w, d)).is_some();
            if !on_deck {
                return if down_on(w, o, &to) {
                    Ok(())
                } else {
                    Err(format!("came down at {:?} (delay {delay})", feet(w, o)))
                };
            }
        }
        if jumped.is_some_and(|j| f > j + 400) {
            return Err(format!("never came down off it (delay {delay})"));
        }
    }
    Err(format!("never jumped (delay {delay})"))
}
