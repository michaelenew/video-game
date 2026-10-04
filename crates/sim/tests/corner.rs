//! **A jump that clips a ledge's near corner is not a second jump.**
//!
//! Found by the movement search (`docs/design/courses.md` §6, round two): a
//! body rising past the near top corner of a ledge was pushed up onto the top
//! by the resolve -- the vertical overlap there is the shallowest -- and
//! counted as standing on it, so a jump button still held from the takeoff
//! fired again, stacked on the speed it was already rising at: about 27 m/s
//! up against a takeoff of 16 to 18. It turned a clipped corner into the
//! highest jump in the game. A body is lifted onto a top it clips while
//! rising, as before; it only *lands* there once it is not rising.

use sim::search::Stage;
use sim::{Class, Fx, Input};

/// The fastest a body rises, running at the ledge and holding space from
/// `jump_at` on.
fn fastest_rise(class: Class, lane: usize, jump_at: u16, back: u16) -> Fx {
    let mut w = Stage::lane(class, lane).world(back);
    let mut top = Fx::ZERO;
    for f in 0..90u16 {
        let space = if f >= jump_at { Input::SPACE } else { 0 };
        w.advance([Input::aimed(Input::W | space, 0), Input::default()]);
        top = top.max(w.players[0].vel.y);
    }
    top
}

#[test]
fn clipping_a_corner_on_the_way_up_does_not_jump_again() {
    // The lab's +1 m and +2 m ledges, half a metre past the runway's edge:
    // a held jump taken anywhere in the last two metres before the edge
    // rises into their corners.
    for class in [
        Class::ShadowReaver,
        Class::BloodMage,
        Class::Champion,
        Class::Elementalist,
        Class::DualMage,
    ] {
        let takeoff = sim::tuning::jump_speed().mul(class.mobility().jump);
        for lane in [4usize, 5] {
            for back in [100u16, 150, 200] {
                for at in 0..20u16 {
                    let rise = fastest_rise(class, lane, at, back);
                    assert!(
                        rise.raw() <= takeoff.add(Fx::ONE).raw(),
                        "{} into lane {lane}'s corner, jumping on frame {at} from {back} cm: \
                         rising at {} m/s against a takeoff of {}",
                        class.name(),
                        rise.to_f32_for_render(),
                        takeoff.to_f32_for_render()
                    );
                }
            }
        }
    }
}
