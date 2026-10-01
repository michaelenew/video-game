use sim::monster::Doing;
use sim::reachcheck::*;
use sim::species::SpeciesId;
use sim::species::broodmother as bm;
use sim::{Class, Fx, Input, V3};
fn f(v: Fx) -> f32 {
    v.to_f32_for_render()
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let class = match a[1].as_str() {
        "bulwark" => Class::Bulwark,
        "champion" => Class::Champion,
        "reaver" => Class::ShadowReaver,
        "blood" => Class::BloodMage,
        "dual" => Class::DualMage,
        _ => Class::Elementalist,
    };
    let site: usize = a[2].parse().unwrap();
    let x: f32 = a[3].parse().unwrap();
    let z: f32 = a[4].parse().unwrap();
    let button = if a[5] == "L" {
        Input::LEFT
    } else {
        Input::RIGHT
    };
    let doing = Doing::Prowl;
    let part = bm::sac_part(site);
    let mut base = stage(SpeciesId::BROODMOTHER, class);
    let at = centre(&base);
    pin(&mut base, doing, at);
    let target = middle(&base, part);
    let mut w = base.clone();
    pin(&mut w, doing, at);
    w.players[0].pos = at.add(V3::new(
        Fx::from_raw((x * 65536.0) as i32),
        Fx::ZERO,
        Fx::from_raw((z * 65536.0) as i32),
    ));
    let mut pressed = false;
    let mut rising = false;
    for frame in 0..90 {
        pin(&mut w, doing, at);
        let p = &w.players[0];
        let d = target.sub(p.pos);
        let aim = (sim::math::atan2_turns(d.z, d.x).raw() & 0xFFFF) as u16;
        let pitch = sim::aim::look_onto_closely(p.pos, aim, p.aloft, target);
        let mut bits = Input::W;
        if !pressed && (frame < 4 || p.vel.y.raw() > 0) {
            bits |= Input::SPACE;
        }
        if p.vel.y.raw() > 0 {
            rising = true;
        }
        if rising && p.vel.y.raw() <= 0 && !pressed {
            bits |= button;
            pressed = true;
        }
        w.advance([Input::looking_at(bits, aim, pitch), Input::default()]);
        let p = &w.players[0];
        let hb = sim::state::hitbox(p);
        let rel = p.pos.sub(at);
        println!(
            "f{frame} pos ({:.2},{:.2},{:.2}) hb {:?} sac {}",
            f(rel.x),
            f(rel.y),
            f(rel.z),
            hb.map(|h| (
                f(h.from.x.sub(at.x)),
                f(h.from.y),
                f(h.from.z.sub(at.z)),
                f(h.to.x.sub(at.x)),
                f(h.to.y),
                f(h.to.z.sub(at.z)),
                f(h.radius)
            )),
            w.monsters[0].unwrap().breaks[site]
        );
    }
}
