//! The Siegeshell's rules, pinned (`docs/design/creatures/siegeshell.md` §10).

use sim::fixed::Fx;
use sim::monster::{Doing, Monster};
use sim::species::SpeciesId;
use sim::species::siegeshell::{self as ss, gait};
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, V3, World};

fn top(m: &Monster, part: usize) -> Fx {
    let sh = m.sp().shape(part);
    let mid = |a: Fx, b: Fx| a.add(b).mul(Fx::ratio(1, 2));
    m.rig()
        .part_to_world(
            part,
            V3::new(mid(sh.min.x, sh.max.x), sh.max.y, mid(sh.min.z, sh.max.z)),
        )
        .y
}

fn hunt() -> World {
    World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::SIEGESHELL)
}

#[test]
fn probe() {
    let mut w = hunt();
    for _ in 0..2 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let m = *w.monster().unwrap();
    for (i, p) in m.sp().parts.iter().enumerate() {
        let rig = m.rig();
        let sh = m.sp().shape(i);
        let lo = rig.part_to_world(i, V3::new(sh.min.x, sh.min.y, sh.min.z));
        let hi = rig.part_to_world(i, V3::new(sh.max.x, sh.max.y, sh.max.z));
        eprintln!(
            "{:20} top {:6.2}  {:?} -> {:?}",
            p.name,
            top(&m, i).to_f32_for_render(),
            (
                lo.x.to_f32_for_render(),
                lo.y.to_f32_for_render(),
                lo.z.to_f32_for_render()
            ),
            (
                hi.x.to_f32_for_render(),
                hi.y.to_f32_for_render(),
                hi.z.to_f32_for_render()
            )
        );
    }
    let mut prev = m.pos;
    for f in 0..600 {
        w.advance([Input::default(); MAX_PLAYERS]);
        if f % 60 == 0 {
            let m = *w.monster().unwrap();
            let feet: Vec<String> = (0..6)
                .map(|l| {
                    let p = gait::foot(&m, l);
                    format!(
                        "({:.1},{:.1},{:.1})",
                        p.x.to_f32_for_render(),
                        p.y.to_f32_for_render(),
                        p.z.to_f32_for_render()
                    )
                })
                .collect();
            eprintln!(
                "f{} pos {:.2} speed {:.2} stride {} yaw {:.4} feet {}",
                f,
                m.pos.x.to_f32_for_render(),
                m.speed.to_f32_for_render(),
                m.stride,
                m.yaw.to_f32_for_render(),
                feet.join(" ")
            );
            prev = m.pos;
        }
    }
    let _ = (prev, Doing::Prowl, ss::LEG_COUNT);
}

fn surface_profile(m: &Monster, label: &str) {
    let rig = m.rig();
    let sp = m.sp();
    for (i, p) in sp.parts.iter().enumerate() {
        if !p.shape.mountable || !rig.boardable(i) {
            continue;
        }
        let sh = sp.shape(i);
        let mut lo = Fx::MAX;
        let mut hi = Fx::MIN;
        for x in [sh.min.x, sh.max.x] {
            for z in [sh.min.z, sh.max.z] {
                let y = rig.part_to_world(i, V3::new(x, sh.max.y, z)).y;
                lo = lo.min(y);
                hi = hi.max(y);
            }
        }
        eprintln!(
            "{label} {:20} top {:6.2}..{:6.2}",
            p.name,
            lo.to_f32_for_render(),
            hi.to_f32_for_render()
        );
    }
}

#[test]
fn probe_stumble() {
    let mut w = hunt();
    for _ in 0..2 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let m = w.monster_mut().unwrap();
    for leg in [0usize, 2] {
        let slot = m.sp().break_slot(ss::ankle_part(leg)).unwrap();
        m.breaks[slot] = 0;
    }
    ss::fight::stumble(m, -1);
    for _ in 0..80 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let m = *w.monster().unwrap();
    surface_profile(&m, "stumble");
    let rig = m.rig();
    for leg in [0usize, 2, 4] {
        let k = rig.bone[ss::bones::shin(leg)].at;
        let h = rig.bone[ss::bones::thigh(leg)].at;
        eprintln!(
            "leg {leg} hip ({:.2},{:.2},{:.2}) knee ({:.2},{:.2},{:.2})",
            h.x.to_f32_for_render(),
            h.y.to_f32_for_render(),
            h.z.to_f32_for_render(),
            k.x.to_f32_for_render(),
            k.y.to_f32_for_render(),
            k.z.to_f32_for_render()
        );
    }
    let mut w = hunt();
    for _ in 0..2 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let m = w.monster_mut().unwrap();
    m.doing = Doing::Toppled {
        left: m.sp().topple_frames(),
    };
    for _ in 0..200 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let m = *w.monster().unwrap();
    surface_profile(&m, "kneel");
}
