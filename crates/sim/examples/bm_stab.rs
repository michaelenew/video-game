use sim::monster::{Doing, Monster};
use sim::species::SpeciesId;
use sim::species::broodmother::{self as bm, fight, legs};
use sim::{Fx, V3};
fn f(v: Fx) -> f32 {
    v.to_f32_for_render()
}
fn main() {
    let mut m = Monster::new(SpeciesId::BROODMOTHER);
    let a = bm::SPECIES.attack(bm::STAB);
    for leg in [0usize, 3, 6] {
        let disc = {
            let rig = m.rig();
            let h = legs::foot(&m, &rig, leg);
            V3::new(h.x.mul(Fx::ratio(3, 5)), Fx::ZERO, h.z.mul(Fx::ratio(3, 5)))
        };
        m.aim_at(disc);
        m.own[fight::body::STAB] = leg as i32 + 1;
        println!("leg {leg} disc ({:.2},{:.2})", f(disc.x), f(disc.z));
        for (label, d) in [
            (
                "start",
                Doing::Startup {
                    kind: bm::STAB,
                    left: a.startup,
                },
            ),
            (
                "mid",
                Doing::Startup {
                    kind: bm::STAB,
                    left: a.startup / 2,
                },
            ),
            (
                "hit0",
                Doing::Active {
                    kind: bm::STAB,
                    left: a.active,
                },
            ),
            (
                "hit-end",
                Doing::Active {
                    kind: bm::STAB,
                    left: 0,
                },
            ),
            (
                "rec",
                Doing::Recovery {
                    kind: bm::STAB,
                    left: a.recovery / 2,
                },
            ),
            (
                "end",
                Doing::Recovery {
                    kind: bm::STAB,
                    left: 0,
                },
            ),
        ] {
            m.doing = d;
            let rig = m.rig();
            let ft = legs::foot(&m, &rig, leg);
            let tel = m
                .telegraph()
                .map(|t| (f(t.anchor.x), f(t.anchor.z), f(t.radius)));
            println!(
                "  {label:8} foot ({:.2},{:.2},{:.2}) telegraph {:?}",
                f(ft.x),
                f(ft.y),
                f(ft.z),
                tel
            );
        }
    }
    // the list
    let mut m = Monster::new(SpeciesId::BROODMOTHER);
    for leg in [3usize, 5] {
        let s = bm::SPECIES.break_slot(bm::shin_part(leg)).unwrap();
        m.breaks[s] = 0;
    }
    println!("list {}", legs::list(&m));
    let rig = m.rig();
    for leg in 0..8 {
        let ft = legs::foot(&m, &rig, leg);
        println!(
            "  foot {leg} ({:.2},{:.2},{:.2})",
            f(ft.x),
            f(ft.y),
            f(ft.z)
        );
    }
    for i in 0..6 {
        let c = fight::sac_middle(&m, i);
        println!("  sac {i} y {:.2}", f(c.y));
    }
}
