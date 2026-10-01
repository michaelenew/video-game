//! Scratch: the Broodmother's geometry, standing and through a move.
use sim::monster::{Doing, Monster};
use sim::species::SpeciesId;
use sim::species::broodmother as bm;
use sim::{Fx, V3};
fn f(v: Fx) -> f32 {
    v.to_f32_for_render()
}
fn report(m: &Monster, label: &str) {
    let rig = m.rig();
    let sp = m.sp();
    println!("== {label}");
    for leg in 0..bm::LEG_COUNT {
        let foot = rig.bone[bm::bones::tibia(leg)].local_to_world(V3::new(
            bm::tibia_length(),
            Fx::ZERO,
            Fx::ZERO,
        ));
        let knee = rig.bone[bm::bones::tibia(leg)].at;
        println!(
            "  leg {leg}: knee ({:.2},{:.2},{:.2}) foot ({:.2},{:.2},{:.2})",
            f(knee.x),
            f(knee.y),
            f(knee.z),
            f(foot.x),
            f(foot.y),
            f(foot.z)
        );
    }
    for part in 0..bm::FEMUR0 {
        let sh = sp.shape(part);
        let mut lo = V3::new(Fx::MAX, Fx::MAX, Fx::MAX);
        let mut hi = V3::new(Fx::MIN, Fx::MIN, Fx::MIN);
        for c in 0..8 {
            let p = V3::new(
                if c & 1 == 0 { sh.min.x } else { sh.max.x },
                if c & 2 == 0 { sh.min.y } else { sh.max.y },
                if c & 4 == 0 { sh.min.z } else { sh.max.z },
            );
            let w = rig.part_to_world(part, p);
            lo = V3::new(lo.x.min(w.x), lo.y.min(w.y), lo.z.min(w.z));
            hi = V3::new(hi.x.max(w.x), hi.y.max(w.y), hi.z.max(w.z));
        }
        println!(
            "  {:16} x {:6.2}..{:6.2}  y {:5.2}..{:5.2}  z {:6.2}..{:6.2}",
            sp.parts[part].name,
            f(lo.x),
            f(hi.x),
            f(lo.y),
            f(hi.y),
            f(lo.z),
            f(hi.z)
        );
    }
}
fn main() {
    let mut m = Monster::new(SpeciesId::BROODMOTHER);
    report(&m, "standing");
    let a = m.sp().attack(bm::SLAM);
    m.doing = Doing::Recovery {
        kind: bm::SLAM,
        left: a.recovery - 30,
    };
    report(&m, "slam recovery, 30 in");
    m.doing = Doing::Recovery {
        kind: bm::SLAM,
        left: 25,
    };
    report(&m, "slam recovery, 25 left");
    m.doing = Doing::Toppled { left: 120 };
    report(&m, "collapse");
}
