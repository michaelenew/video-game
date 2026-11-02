//! **Every start is clear** (`docs/design/level-design.md`, L5): in every
//! registered arena, each fighter's start stands on something, not inside
//! the side of a solid, and the two fighters' starts are apart. A start
//! inside or under something is the first thing a person sees go wrong.

use sim::arena::{self, Terrain};
use sim::{Class, Fx, V3, World};

/// How far into a solid's footprint a fighter's middle has to be to count
/// as inside it. A start pressed against a solid's side (the proving
/// ground's hunters, against a platform) is touching, not inside: the
/// first frame's resolve stands it clear.
const RADIUS: Fx = Fx::ratio(-1, 10);
const SKIN: Fx = Fx::ratio(5, 100);

fn inside(ground: &Terrain, at: V3) -> bool {
    let lo = at.y.add(SKIN);
    let hi = at.y.add(sim::tuning::body_height());
    ground.around(at, Fx::from_int(3)).any(|s| {
        s.min.y.raw() < hi.raw()
            && s.max.y.raw() > lo.raw()
            && s.covers_at(at.x, at.z, lo.max(s.min.y), RADIUS)
    })
}

fn apart(a: V3, b: V3) -> bool {
    let dx = a.x.sub(b.x);
    let dz = a.z.sub(b.z);
    dx.mul(dx).add(dz.mul(dz)).raw() >= Fx::ratio(9, 4).raw()
}

#[test]
fn every_start_is_clear_and_apart() {
    let mut wrong = Vec::new();
    for a in arena::all() {
        let mut worlds = vec![("versus", World::versus_in([Class::Champion; 2], a.id))];
        if let Some(species) = a.creature {
            worlds.push((
                "hunt",
                World::hunt_in([Class::Champion; 2], [Some(species), None], a.id),
            ));
        }
        for (what, w) in worlds {
            let ground = w.terrain();
            let (p, q) = (w.players[0].pos, w.players[1].pos);
            for (seat, at) in [(1, p), (2, q)] {
                if inside(&ground, at) {
                    wrong.push(format!(
                        "{} ({what}): player {seat} starts inside a solid, at ({:.1}, {:.1}, {:.1})",
                        a.name,
                        at.x.to_f32_for_render(),
                        at.y.to_f32_for_render(),
                        at.z.to_f32_for_render()
                    ));
                }
            }
            if !apart(p, q) {
                wrong.push(format!(
                    "{} ({what}): the two starts are under 1.5 m apart",
                    a.name
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "starts that are not clear:\n  {}",
        wrong.join("\n  ")
    );
}
