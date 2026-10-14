//! **The field experiment** (`docs/design/exploration/0018` §7.2 and §7.3):
//! the crowd field is rollback-free, exact, and slow by construction.
//!
//! - **S1 and S2.** Two peers follow the same streams. One is handed every
//!   aggregate the frame it is summed; the other gets each one late, by a
//!   different amount every time, but always inside the lookahead. Both feed
//!   by due frame, so their worlds -- and the field in them -- are the same
//!   bytes at every frame, and the far crowd moved a pack's mood on both on
//!   the same frame.
//! - **The gate.** An aggregate later than the lookahead does not make the
//!   worlds differ: it holds the frame that would read it until it comes.
//! - **S3.** `tau >= lookahead`, at the Oven, and the filtered crowd read
//!   `L` late is within `L / tau` of its target.
//!
//! One test for the first two, because they share a world and the Oven is
//! shared by everything in the process.

use regions::Ledger;
use sim::class::Class;
use sim::crowd::Aggregate;
use sim::input::Input;
use sim::pack::mood;
use sim::region::{Grid, RegionId};
use sim::species::SpeciesId;
use sim::tuning as t;
use sim::{Fx, World};

/// The Hornback's crossing: a calm herd on the road, and nobody near it.
fn calm_herd() -> World {
    let mut w = World::hunt_in(
        [Class::Champion; 2],
        [Some(SpeciesId::HORNBACK), None],
        sim::arena::ArenaId::HORNBACK_CROSSING,
    );
    w.players[1].health = 0;
    for _ in 0..2 {
        w.advance([Input::default(); 2]);
    }
    // Off the road at the far west end, out of the herd's alarm radius.
    w.players[0].pos = sim::math::V3::new(Fx::from_int(-38), Fx::ZERO, Fx::from_int(-16));
    w
}

fn keep_up(w: &mut World) {
    w.players[0].health = w.players[0].full_health();
}

/// One frame of a peer: the gate, the tape, the due aggregates, the books.
/// False if the gate held.
fn step(w: &mut World, ledger: &mut Ledger) -> bool {
    ledger.tick();
    let next = w.frame + 1;
    if !ledger.may_advance(next) {
        return false;
    }
    keep_up(w);
    ledger.record(w, [Input::default(); sim::state::MAX_PLAYERS]);
    let (fed, n) = ledger.due(next);
    w.advance_with([Input::default(); 2], &fed[..n]);
    ledger.confirm_through(w.frame, w);
    true
}

/// A region next to the herd's that nobody is behind -- no fighter is a
/// member of it, so the ledger has been speaking for it -- after `frames`
/// of warm-up. The test then puts a peer behind it.
fn warm_up(w: &mut World, ledger: &mut Ledger, frames: u32) -> RegionId {
    for _ in 0..frames {
        assert!(
            step(w, ledger),
            "held at {} with nobody behind any stream",
            w.frame + 1
        );
    }
    let grid = Grid::tuned();
    let home = grid.home(w.pack.unwrap().home.add(w.map_origin()));
    home.neighbours()
        .into_iter()
        .find(|n| {
            ledger
                .followed()
                .iter()
                .any(|f| f.region == *n && f.simulated)
        })
        .unwrap_or_else(|| {
            panic!(
                "no neighbour of the herd's is spoken for: {:?}",
                ledger.followed()
            )
        })
}

fn crowd_of(region: RegionId, epoch: u16, fighters: u16) -> Aggregate {
    Aggregate {
        region,
        epoch,
        fighters,
        ..Aggregate::NONE
    }
}

#[test]
fn two_peers_fed_the_same_aggregates_at_different_times_agree_and_hear_the_same_crowd() {
    let (e, l) = (t::crowd_epoch(), t::crowd_lookahead());
    let start = calm_herd();
    let mut early = (start.clone(), Ledger::new(&start, false));
    let mut late = (start.clone(), Ledger::new(&start, false));
    let mut alone = start;
    let region = warm_up(&mut early.0, &mut early.1, e);
    assert_eq!(region, warm_up(&mut late.0, &mut late.1, e));
    for _ in 0..e {
        keep_up(&mut alone);
        alone.advance([Input::default(); 2]);
    }
    // A peer is behind the stream now, on both: its aggregates come from
    // the wire -- here, from this test -- every epoch from the next one.
    early.1.peer_behind(region, true);
    late.1.peer_behind(region, true);
    let mut roused = None;
    let mut rng = 0x2545_f491u32;
    // What arrived late and is still to be offered: (arrives at frame, it).
    let mut pending: Vec<(u32, Aggregate)> = Vec::new();
    let frames = l + 6 * e;
    for _ in 0..frames {
        let next = early.0.frame + 1;
        // Summed at the epoch's frame on the early peer; the late one gets
        // it after a trip of up to all but one frame of the lookahead.
        if next.is_multiple_of(e) {
            let a = crowd_of(region, (next / e) as u16, 5);
            early.1.offer(a);
            rng ^= rng << 13;
            rng ^= rng >> 17;
            rng ^= rng << 5;
            let trip = rng % l.max(1);
            pending.push((next + trip, a));
        }
        pending.retain(|(at, a)| {
            if *at <= next {
                late.1.offer(*a);
                false
            } else {
                true
            }
        });
        for (w, ledger) in [&mut early, &mut late] {
            assert!(step(w, ledger), "frame {next} held");
        }
        keep_up(&mut alone);
        alone.advance([Input::default(); 2]);
        assert_eq!(
            early.0.checksum(),
            late.0.checksum(),
            "the peers differ at frame {next} ({} late offers outstanding)",
            pending.len()
        );
        let heard = early.0.crowd.stream(region).map_or(Fx::ZERO, |s| s.crowd);
        if roused.is_none() && early.0.pack.unwrap().mood == mood::HUNTING {
            roused = Some((next, heard));
        }
    }
    let (at, heard) = roused.expect("the herd heard the valley fill next door");
    assert!(
        at > e + l,
        "it heard nothing before the lookahead: roused at {at}"
    );
    assert!(
        heard.raw() >= t::crowd_hears().raw(),
        "roused at {at} hearing {heard:?}"
    );
    assert_eq!(
        alone.pack.unwrap().mood,
        mood::CALM,
        "with nobody next door the herd stays calm, as it always did"
    );
    assert!(
        alone
            .crowd
            .streams
            .iter()
            .all(|s| s.last == Aggregate::NONE),
        "a world alone has heard nothing"
    );
}

#[test]
fn a_stream_with_a_peer_behind_it_holds_the_gate_until_its_aggregate_comes() {
    let (e, l) = (t::crowd_epoch(), t::crowd_lookahead());
    let start = calm_herd();
    let mut w = start.clone();
    let mut ledger = Ledger::new(&start, false);
    let region = warm_up(&mut w, &mut ledger, l + 2 * e);
    // A peer behind it now, silent: the frame that reads the next epoch's
    // aggregate is held, and only that frame -- until the aggregate comes.
    ledger.peer_behind(region, true);
    let from = w.frame;
    let mut held_at = None;
    // The ledger had spoken for every epoch its clock had reached, and the
    // gate reads an epoch a lookahead late: the first it has not heard is
    // a lookahead on.
    for _ in 0..(l + 3 * e) {
        if !step(&mut w, &mut ledger) {
            held_at = Some(w.frame + 1);
            break;
        }
    }
    let held_at = held_at.expect("a silent peer holds the gate");
    let needed = (held_at - l) / e;
    assert!(
        needed * e > from.saturating_sub(l),
        "held for an epoch the ledger had spoken for"
    );
    assert!(
        ledger
            .followed()
            .iter()
            .any(|f| f.region == region && !f.simulated)
    );
    // Still held a few ticks on, then released by the aggregate it waits for.
    for _ in 0..5 {
        assert!(!step(&mut w, &mut ledger));
    }
    ledger.offer(crowd_of(region, needed as u16, 5));
    assert!(
        step(&mut w, &mut ledger),
        "the aggregate it waited for did not release it"
    );
    assert_eq!(w.frame, held_at);
}

#[test]
fn the_time_constant_covers_the_lookahead_and_a_late_read_is_nearly_current() {
    let (l, tau) = (t::crowd_lookahead(), t::crowd_tau());
    assert!(
        tau >= l,
        "tau {tau} must cover the lookahead {l}: a value read that late would not read as current"
    );
    assert!(t::crowd_heat_tau() >= l);
    // A step from nothing to five fighters: after `l` frames the filtered
    // value has moved at most `l / tau` of the way, so the value read `l`
    // late is within that of the one read now.
    let mut s = sim::crowd::Stream {
        on: true,
        last: crowd_of(RegionId::default(), 1, 5),
        ..sim::crowd::Stream::NONE
    };
    let mut field = sim::crowd::Field::NONE;
    field.streams[0] = s;
    let mut at_l = Fx::ZERO;
    for f in 1..=tau {
        field.feed(&[]);
        if f == l {
            at_l = field.streams[0].crowd;
        }
    }
    s = field.streams[0];
    let bound = Fx::from_int(5)
        .mul(Fx::from_int(l as i32))
        .div(Fx::from_int(tau as i32));
    assert!(
        s.crowd.sub(at_l).raw() <= bound.raw(),
        "moved {:?} in the lookahead, bound {:?}",
        s.crowd.sub(at_l),
        bound
    );
}
