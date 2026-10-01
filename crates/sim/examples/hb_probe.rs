use sim::species::SpeciesId;
use sim::species::hornback as h;
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, V3, World};
fn main() {
    let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::HORNBACK);
    w.players[1].health = 0;
    for _ in 0..2 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let i = (0..10).find(|k| w.critters[*k].kind == h::COW).unwrap();
    let sp = w.critters.sp();
    w.players[0].mount = sim::critter::mount_of(i);
    w.players[0].local = V3::ZERO;
    w.players[0].pos = w.critters[i].back_point(sp, V3::ZERO);
    w.players[0].grounded = true;
    let mut prev = w.players[0].pos;
    let mut pv = V3::ZERO;
    for f in 0..200 {
        w.players[0].health = 1000;
        w.advance([Input::aimed(Input::CROUCH, 0), Input::default()]);
        let p = w.players[0].pos;
        let v = p.sub(prev).scale(sim::fixed::Fx::from_int(60));
        let a = v.sub(pv).scale(sim::fixed::Fx::from_int(60));
        if f > 130 && f < 175 {
            let c = w.critters[i];
            println!(
                "{:?} f{f} y {:.3} vy {:.2} ay {:.1} ax {:.1} az {:.1} mount {} cow s{} a{} t{} vel({:.2},{:.2})",
                w.players[0].action,
                p.y.to_f32_for_render(),
                v.y.to_f32_for_render(),
                a.y.to_f32_for_render(),
                a.x.to_f32_for_render(),
                a.z.to_f32_for_render(),
                w.players[0].mount,
                c.state,
                c.act,
                c.timer,
                c.vel.x.to_f32_for_render(),
                c.vel.z.to_f32_for_render()
            );
        }
        prev = p;
        pv = v;
    }
}
