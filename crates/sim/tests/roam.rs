//! scratch
use sim::Class;
use sim::fixed::Fx;
use sim::input::Input;
use sim::keep::Item;
use sim::math::V3;
use sim::species::SpeciesId;
use sim::state::World;
use sim::valley::{self, ranges};

fn on_map(w: &World, i: usize) -> V3 {
    w.players[i].pos.add(w.map_origin())
}

fn step_to(w: &mut World, to: (Fx, Fx)) -> bool {
    let feet = on_map(w, 0);
    let (dx, dz) = (to.0.sub(feet.x), to.1.sub(feet.z));
    if sim::math::wide_len(V3::new(dx, Fx::ZERO, dz)).raw() < Fx::ratio(3, 2).raw() {
        return true;
    }
    let aim = sim::math::atan2_turns(dz, dx).raw() as u16;
    w.advance([Input::looking_at(Input::W, aim, -1500)]);
    false
}

#[test]
fn scratch_chase() {
    let r = ranges::RANGES
        .iter()
        .find(|r| r.name == "Upper Meadow")
        .unwrap();
    let mut w = World::arrive(
        [Class::Champion],
        valley::Journey::default(),
        r.arena,
        Some(0),
        1,
    );
    assert!(w.hunting());
    let egg = w
        .loose
        .iter()
        .flatten()
        .find(|l| matches!(l.item, Item::Egg(_)))
        .unwrap()
        .at;
    w.players[0].pos = egg;
    w.advance([Input::new(0)]);
    assert!(w.egg_carrier().is_some());
    let road: Vec<(Fx, Fx)> = sim::valley::layout::road()[0].clone();
    // nearest road point
    let feet = on_map(&w, 0);
    let k = (0..road.len())
        .min_by_key(|&k| {
            sim::math::wide_len(V3::new(
                road[k].0.sub(feet.x),
                Fx::ZERO,
                road[k].1.sub(feet.z),
            ))
            .raw()
        })
        .unwrap();
    let mut path: Vec<(Fx, Fx)> = road[..=k].iter().rev().copied().collect();
    path.truncate(40);
    let mut f = 0;
    let mut hp0 = w.players[0].health;
    for to in path {
        for _ in 0..900 {
            f += 1;
            if f % 60 == 0 {
                let m = w
                    .monsters
                    .iter()
                    .flatten()
                    .map(|m| {
                        (
                            m.pos.add(w.map_origin()),
                            m.health,
                            m.doing,
                            m.brain.grace,
                            m.speed,
                            m.brain.seen.add(w.map_origin()),
                        )
                    })
                    .collect::<Vec<_>>();
                eprintln!(
                    "f{f} {} p={:?} hp={} m={:?} den={:?} out={} away={} hom={}",
                    w.arena().name,
                    on_map(&w, 0),
                    w.players[0].health,
                    m,
                    w.valley.den,
                    w.den().out(),
                    w.valley.away,
                    w.valley.homing
                );
            }
            let before = w.players[0].health;
            if step_to(&mut w, to) {
                break;
            }
            if w.players[0].health < before {
                eprintln!("HIT in {} f{f}", w.arena().name);
            }
            w.players[0].health = 1000;
        }
    }
    let _ = hp0;
    hp0 = 0;
    let _ = hp0;
    if let Some(m) = w.monsters[1] {
        eprintln!(
            "M2 yaw={:?} rate={:?} rooted={} slowed={} think={} doing={:?} speed={:?} sm={:?}",
            m.yaw, m.yaw_rate, m.rooted, m.slowed, m.brain.think_left, m.doing, m.speed, m.slow_mul
        );
    }
    for _ in 0..600 {
        w.advance([Input::new(0)]);
        f += 1;
        if f % 60 == 0 {
            let m = w
                .monsters
                .iter()
                .flatten()
                .map(|m| m.pos.add(w.map_origin()))
                .collect::<Vec<_>>();
            eprintln!(
                "wait f{f} {} hp={} m={:?}",
                w.arena().name,
                w.players[0].health,
                m
            );
        }
    }
    let _ = SpeciesId::RIDGEBACK;
}

#[test]
fn scratch_stuck() {
    let atlas = sim::atlas::valley();
    let bank = atlas.placed(sim::arena::ArenaId::BANK).unwrap().at;
    let g = sim::arena::Terrain::placed(atlas, sim::arena::ArenaId::BANK.get());
    let p = V3::new(
        Fx::from_raw((269.831954f64 * 65536.0) as i32),
        Fx::ZERO,
        Fx::from_raw((21.094421f64 * 65536.0) as i32),
    )
    .sub(bank);
    let y = g.relief_at(p.x, p.z);
    eprintln!(
        "y={} land={} steep={:?}",
        y.to_f32_for_render(),
        g.on_land(p.x, p.z),
        g.downhill(p.x, p.z)
    );
    for s in g.around(V3::new(p.x, y, p.z), Fx::from_int(3)) {
        eprintln!(
            "solid {:?}..{:?} {:?}",
            s.min.add(bank),
            s.max.add(bank),
            s.shape
        );
    }
    for k in 0..12 {
        let d = V3::from_turns(Fx::ratio(k, 12));
        let q = p.add(d.scale(Fx::from_int(2)));
        eprintln!(
            "dir{k} -> {:?} y={}",
            sim::valley::roam::step_onto(&g, V3::new(p.x, y, p.z), q, Fx::from_int(2))
                .map(|v| v.to_f32_for_render()),
            g.relief_at(q.x, q.z).to_f32_for_render()
        );
    }
}

#[test]
fn scratch_places() {
    let a = sim::atlas::valley();
    eprintln!("map {:?} {:?}", a.lo, a.hi);
    for p in &a.places {
        eprintln!(
            "{:<16} at=({:.0},{:.0},{:.0}) lo=({:.0},{:.0}) hi=({:.0},{:.0}) band={}",
            p.get().name,
            p.at.x.to_f32_for_render(),
            p.at.y.to_f32_for_render(),
            p.at.z.to_f32_for_render(),
            p.lo.0.to_f32_for_render(),
            p.lo.1.to_f32_for_render(),
            p.hi.0.to_f32_for_render(),
            p.hi.1.to_f32_for_render(),
            p.band.is_some()
        );
    }
}

#[test]
fn scratch_frame() {
    use sim::arena::ArenaId;
    let den: ArenaId = match std::env::var("DEN").ok().and_then(|s| s.parse::<u8>().ok()) {
        Some(k) => ArenaId(k),
        None => ArenaId::HIGHLANDS,
    };
    let atlas = sim::atlas::valley();
    let home = atlas.placed(den).copied().unwrap();
    let other = atlas
        .places
        .iter()
        .filter(|p| p.arena != den && p.at != V3::ZERO)
        .min_by_key(|p| sim::math::wide_len(p.at.sub(home.at)).raw())
        .copied()
        .unwrap();
    let mut j = valley::Journey::default();
    j.beaten = u32::MAX;
    let mut w = World::arrive([Class::Champion, Class::DualMage], j, den, Some(0), 2);
    let script = |f: u32| {
        let bits = match f % 80 {
            0..=9 => Input::LEFT,
            10..=29 => Input::W | Input::LEFT,
            30..=34 => Input::SPECIAL,
            35..=49 => Input::W | Input::RIGHT,
            _ => Input::A | Input::LEFT,
        };
        Input::looking_at(bits, (f * 131) as u16, -1500)
    };
    for f in 0..240 {
        w.advance([script(f), script(f + 40)]);
    }
    for f in 240..600 {
        let mut moved = w.clone();
        moved.shift(home.at.sub(other.at));
        moved.arena = other.arena;
        let mut ww = w.clone();
        ww.advance([script(f), script(f + 40)]);
        moved.advance([script(f), script(f + 40)]);
        moved.valley.came = ww.valley.came;
        if moved != ww {
            eprintln!(
                "frame {f} differs: arena {:?} {:?} hunting {}",
                ww.arena,
                moved.arena,
                moved.hunting()
            );
            eprintln!(
                "A {:?}\nB {:?}",
                ww.monsters[0].map(|m| (m.pos, m.yaw, m.speed, m.doing, m.brain.seen)),
                moved.monsters[0].map(|m| (m.pos, m.yaw, m.speed, m.doing, m.brain.seen))
            );
            let fl = |x: String, y: String| {
                let (x, y): (Vec<String>, Vec<String>) = (
                    x.split(", ").map(String::from).collect(),
                    y.split(", ").map(String::from).collect(),
                );
                x.iter()
                    .zip(y.iter())
                    .filter(|(p, q)| p != q)
                    .take(8)
                    .map(|(p, q)| format!("[{p} | {q}]"))
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            eprintln!(
                "players {}",
                fl(format!("{:?}", ww.players), format!("{:?}", moved.players))
            );
            eprintln!(
                "rest {}",
                fl(
                    format!(
                        "{:?}",
                        (
                            ww.effects, ww.bolts, ww.debris, ww.gusts, ww.loose, ww.heel, ww.ground
                        )
                    ),
                    format!(
                        "{:?}",
                        (
                            moved.effects,
                            moved.bolts,
                            moved.debris,
                            moved.gusts,
                            moved.loose,
                            moved.heel,
                            moved.ground
                        )
                    )
                )
            );
            eprintln!(
                "valley {}",
                fl(format!("{:?}", ww.valley), format!("{:?}", moved.valley))
            );
            eprintln!(
                "misc {}",
                fl(
                    format!(
                        "{:?}",
                        (
                            ww.frame, ww.phase, ww.crowd, ww.away, ww.course, ww.paused, ww.seats
                        )
                    ),
                    format!(
                        "{:?}",
                        (
                            moved.frame,
                            moved.phase,
                            moved.crowd,
                            moved.away,
                            moved.course,
                            moved.paused,
                            moved.seats
                        )
                    )
                )
            );
            break;
        }
        w = ww;
    }
}
