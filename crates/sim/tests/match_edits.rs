//! **The Oven moved in a match is the same game on every machine.** A knob
//! is asked for on a player's input (`Input::tune`), kept in the world
//! (`oven::Edits`) and laid over the Oven before every frame (`oven::adopt`).
//! What that has to hold, through rollback:
//!
//! - the edit changes the rule from the frame that carried it, and not before;
//! - a world loaded from before the edit runs its frames on the old rule, even
//!   though the store last ran a world that had it -- a rollback;
//! - a world's checksum is the same whichever world the store last ran;
//! - a match holds `MATCH_EDITS` knobs, and the next is refused everywhere.
//!
//! One test, in a file of its own: the Oven's store is the process's, and a
//! test running a world beside this one would run on its rules.

use sim::input::Tune;
use sim::oven::{self, Knob, MATCH_EDITS, Scalar};
use sim::{Class, Input, World};

fn walk(frames: u32, w: &mut World, tune_at: Option<(u32, Tune)>) {
    for _ in 0..frames {
        let mut me = Input::aimed(Input::W, 0);
        if let Some((at, tune)) = tune_at
            && w.frame == at
        {
            me = me.tuning(tune);
        }
        w.advance([me, Input::default()]);
    }
}

#[test]
fn a_knob_moved_in_a_match_is_the_same_game_everywhere() {
    let speed = Knob::Scalar(Scalar::MoveSpeed);
    let baked = speed.raw();
    let faster = Tune::set(speed.cell(), baked * 2);
    let start = World::with_classes([Class::Champion, Class::ShadowReaver]);
    let base = oven::hash();

    // Straight through: the edit taken at frame 30.
    let mut tuned = start.clone();
    walk(30, &mut tuned, None);
    let before_edit = tuned.clone();
    walk(30, &mut tuned, Some((30, faster)));
    assert_eq!(tuned.edits.get(speed.cell()), Some(baked * 2));
    assert_eq!(speed.raw(), baked * 2, "the store runs the world's edits");

    // The same thirty frames on the baked rule: the edit changed the fight.
    let mut plain = before_edit.clone();
    walk(30, &mut plain, None);
    assert!(plain.edits.is_empty());
    assert_eq!(speed.raw(), baked, "a world with no edits puts them back");
    assert_eq!(oven::hash(), base);
    assert_ne!(plain.players[0].pos, tuned.players[0].pos);

    // **A rollback**: the store has just run the edited world, and a world
    // from before the edit is loaded and run again. Its frames run on the
    // old rule, and with the edit carried again it reaches the same world.
    let mut again = start.clone();
    walk(60, &mut tuned.clone(), None); // the store on the edited rules
    walk(30, &mut again, None);
    assert_eq!(again, before_edit, "run on the baked rule before the edit");
    walk(30, &mut tuned.clone(), None);
    walk(30, &mut again, Some((30, faster)));
    assert_eq!(again, tuned);
    assert_eq!(again.checksum(), tuned.checksum());

    // The checksum does not ask which world the store last ran.
    let sum = plain.checksum();
    walk(1, &mut tuned.clone(), None);
    assert_eq!(plain.checksum(), sum);
    let sum = tuned.checksum();
    walk(1, &mut plain.clone(), None);
    assert_eq!(tuned.checksum(), sum);
    assert_ne!(tuned.checksum(), plain.checksum());

    // A match holds so many knobs; the next one is refused, and one already
    // moved still moves.
    let mut full = start.clone();
    for k in 0..MATCH_EDITS as u16 {
        let tune = Tune::set(k, oven::cell_raw(k).0);
        full.advance([Input::default().tuning(tune), Input::default()]);
    }
    assert!(full.edits.is_full());
    let next = MATCH_EDITS as u16;
    full.advance([
        Input::default().tuning(Tune::set(next, 1)),
        Input::default(),
    ]);
    assert_eq!(full.edits.get(next), None);
    full.advance([
        Input::default().tuning(Tune::set(0, oven::cell_raw(0).0 + 1)),
        Input::default(),
    ]);
    assert_eq!(full.edits.get(0), Some(oven::cell_raw(0).1 + 1));

    // And nothing is left behind.
    walk(1, &mut start.clone(), None);
    assert_eq!(oven::hash(), base);
}
