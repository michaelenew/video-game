//! The Veilstalker's hunter sees what the screen shows (`veilstalker.md` §9):
//! the strength `World::shown` gives each part, and nothing about where the
//! animal is that the renderer does not draw.

use hunt::plans::veilstalker::{SEE, body_seen};
use sim::aim::Scene;
use sim::input::Input;
use sim::species::SpeciesId;
use sim::species::veilstalker as vs;
use sim::state::MAX_PLAYERS;

#[test]
fn what_is_drawn_is_what_the_hunter_and_the_report_call_visible() {
    // A hunt, and every frame the camera pointed straight at the animal --
    // the one place a hunter who knew where it was would look. Whatever the
    // hunter's eye calls seen must be a part drawn at least faintly enough to
    // see; while nothing is drawn, the animal dead centre on the screen is
    // not seen at all.
    let card = hunt::plans::card(SpeciesId::VEILSTALKER).expect("its card");
    let (mut cloaked_centre, mut seen_frames) = (0u32, 0u32);
    let report = hunt::play_card_in(
        card,
        None,
        0,
        [sim::Class::Champion; MAX_PLAYERS],
        1,
        9_000,
        7,
        |w| {
            let Some(slot) = (0..w.monsters.len()).find(|s| {
                w.monsters[*s].is_some_and(|m| m.species == SpeciesId::VEILSTALKER && m.alive())
            }) else {
                return;
            };
            let m = w.monsters[slot].unwrap();
            let me = w.players[0];
            if me.health <= 0 {
                return;
            }
            let to = m.pos.sub(me.pos);
            let yaw = sim::math::atan2_turns(to.z, to.x);
            let view = Input::aimed(0, (yaw.raw() as u32 & 0xFFFF) as u16);
            let stones = sim::stones::gather(&w.players);
            let ground = w.terrain();
            let scene = Scene {
                stones: &stones,
                players: &w.players,
                effects: &w.effects,
                quarry: &w.monsters,
                critters: &w.critters,
                arena: &ground,
            };
            let drawn = (0..vs::PART_COUNT).any(|p| w.shown(slot, p).raw() >= SEE.raw());
            let seen = body_seen(w, 0, view, &scene).is_some();
            assert!(
                drawn || !seen,
                "frame {}: the hunter saw a body nothing drew",
                w.frame
            );
            if seen {
                seen_frames += 1;
            }
            if (0..vs::PART_COUNT).all(|p| w.shown(slot, p).raw() == 0) {
                cloaked_centre += 1;
            }
        },
    );
    assert!(
        cloaked_centre > 600,
        "only {cloaked_centre} cloaked frames to check"
    );
    assert!(
        seen_frames > 60,
        "only {seen_frames} frames it was seen at all"
    );
    // And what the report calls a blind hit -- a decloak on screen under the
    // reaction floor at commit -- never happened.
    assert_eq!(report.unanswerable, 0, "a blind hit");
}
