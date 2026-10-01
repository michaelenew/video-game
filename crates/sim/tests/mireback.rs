//! The Mireback's rules: the floor it lays, the fire that takes it back, the
//! slag that is the way up it, its coat and its warts, the swallow, and the
//! buck. Sentences that state the rule, as `docs/design/creatures/mireback.md`
//! §10 lists them; a failure here is a bug or a design decision that changed
//! without the document changing with it.

use sim::fixed::Fx;
use sim::hazard::{self, Hazard, HazardField};
use sim::monster::{Doing, Monster};
use sim::species::SpeciesId;
use sim::species::mireback::{self, Knob, fight};
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, V3, World};

fn sp() -> &'static sim::species::Species {
    &mireback::SPECIES
}

fn at(x: i32, z: i32) -> V3 {
    V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(z))
}

fn hunt(class: Class) -> World {
    World::hunt_of([class; MAX_PLAYERS], SpeciesId::MIREBACK)
}

/// The creature parked where it is and taking no notice.
fn quiet(w: &mut World) {
    if let Some(m) = w.monsters[0].as_mut() {
        m.brain.grace = u16::MAX;
        if m.doing.free() {
            m.brain.think_left = u16::MAX;
        }
    }
}

/// A hunt with the toad parked in a corner, quiet: a floor to stand on.
fn floor(class: Class) -> World {
    let mut w = hunt(class);
    if let Some(m) = w.monsters[0].as_mut() {
        m.pos = at(10, -10);
    }
    quiet(&mut w);
    w.players[0].pos = at(-10, 10);
    w.players[1].pos = at(-10, -10);
    w
}

fn run(w: &mut World, frames: u32, bits: u16) {
    for _ in 0..frames {
        quiet(w);
        w.advance([Input::aimed(bits, 0), Input::default()]);
        for p in w.players.iter_mut() {
            p.health = p.full_health();
        }
    }
}

fn place(w: &mut World, h: Hazard) -> usize {
    hazard::place(&mut w.lore, h).expect("room for it")
}

fn tar(x: i32, z: i32, r: i32) -> Hazard {
    Hazard::disc(fight::TAR, at(x, z), Fx::from_int(r))
}

// ---------------------------------------------------------------------------
// The buck
// ---------------------------------------------------------------------------

/// Put fighter zero on a part's top, the way landing on it would.
fn board(w: &mut World, part: usize) {
    let beast = w.monster().copied().expect("a hunt has a creature");
    let shape = sp().shape(part);
    let mid = shape.min.add(shape.max).scale(Fx::ratio(1, 2));
    let spot = V3::new(mid.x, shape.max.y.add(Fx::ratio(1, 8)), mid.z);
    w.players[0].pos = beast.world_of(part, spot);
    w.players[0].vel = V3::new(Fx::ZERO, Fx::ratio(-1, 1), Fx::ZERO);
    w.players[0].grounded = false;
    w.advance([Input::default(); MAX_PLAYERS]);
}

/// Board a part, play a move, and say whether the rider stayed on.
fn rides_out(part: usize, kind: u8, brace: bool) -> bool {
    let mut w = hunt(Class::Champion);
    w.players[1].health = 0;
    if let Some(m) = w.monsters[0].as_mut() {
        m.brain.grace = 0;
    }
    board(&mut w, part);
    assert!(
        w.players[0].aboard(),
        "boarding the {} failed",
        sp().parts[part].name
    );
    let held = if brace {
        Input::new(Input::CROUCH)
    } else {
        Input::default()
    };
    let a = sp().attack(kind);
    if let Some(m) = w.monsters[0].as_mut() {
        m.doing = Doing::Startup {
            kind,
            left: a.startup,
        };
        if sp().moves[kind as usize].lobbed {
            let ahead = m.pos.add(V3::from_turns(m.yaw).scale(a.ideal_range));
            m.aim_at(ahead);
        }
    }
    for _ in 0..a.total() {
        w.advance([held, Input::default()]);
        if !w.players[0].aboard() {
            return false;
        }
    }
    true
}

#[test]
fn the_swell_throws_a_loose_rider_and_a_braced_one_holds() {
    for part in [mireback::DOME, mireback::LEFT_RIM] {
        assert!(
            !rides_out(part, mireback::INFLATE, false),
            "an inflate did nothing to somebody standing loose on the {}",
            sp().parts[part].name
        );
        assert!(
            rides_out(part, mireback::INFLATE, true),
            "bracing on the {} did not hold through the swell",
            sp().parts[part].name
        );
    }
}

#[test]
fn rolling_over_and_crashing_throw_everybody() {
    for kind in [mireback::WALLOW, mireback::FLOP] {
        assert!(
            !rides_out(mireback::DOME, kind, true),
            "a braced rider rode out the {}",
            sp().moves[kind as usize].name
        );
    }
}

#[test]
fn the_moves_aimed_at_the_floor_do_not_throw_a_braced_rider() {
    for kind in [
        mireback::SPEW,
        mireback::TONGUE,
        mireback::BELCH,
        mireback::BACKWASH,
    ] {
        for part in [mireback::DOME, mireback::CROWN, mireback::LEFT_RIM] {
            assert!(
                rides_out(part, kind, true),
                "{} threw a braced rider off the {}",
                sp().moves[kind as usize].name,
                sp().parts[part].name
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The floor
// ---------------------------------------------------------------------------

/// How far fighter zero gets walking forward for `frames`.
fn walked(w: &mut World, frames: u32) -> Fx {
    let from = w.players[0].pos.x;
    run(w, frames, Input::W);
    w.players[0].pos.x.sub(from)
}

#[test]
fn tar_slows_the_walk_and_halves_the_dodge_but_keeps_its_invulnerability() {
    let mut clear = floor(Class::Champion);
    let free = walked(&mut clear, 40);
    let mut w = floor(Class::Champion);
    place(&mut w, tar(-6, 10, 6));
    let slowed = walked(&mut w, 40);
    let want = hazard::stat_fx(sp(), fight::TAR, HazardField::Walk);
    assert!(
        slowed.raw() > 0 && slowed.raw() < free.mul(want).add(Fx::ratio(1, 2)).raw(),
        "in tar it walked {slowed:?}, on clear floor {free:?}"
    );

    let dodge = |w: &mut World| {
        let from = w.players[0].pos.x;
        run(w, 1, Input::SHIFT | Input::W);
        let invulnerable = w.players[0].action.invulnerable();
        run(w, 30, 0);
        (w.players[0].pos.x.sub(from), invulnerable)
    };
    let mut clear = floor(Class::Champion);
    run(&mut clear, 30, 0);
    let (far, _) = dodge(&mut clear);
    let mut w = floor(Class::Champion);
    place(&mut w, tar(-10, 10, 3));
    run(&mut w, 30, 0);
    let (near, invulnerable) = dodge(&mut w);
    assert!(invulnerable, "a dodge out of tar is still a dodge");
    let half = hazard::stat_fx(sp(), fight::TAR, HazardField::Dodge);
    assert!(
        near.raw() <= far.mul(half).add(Fx::ratio(1, 2)).raw(),
        "the dodge from tar went {near:?}, from clear floor {far:?}"
    );
}

#[test]
fn the_tar_slow_lingers_after_the_feet_leave_it() {
    let mut w = floor(Class::Champion);
    place(&mut w, tar(-10, 10, 2));
    run(&mut w, 10, 0);
    // Out of it, the far side of the pool.
    w.players[0].pos = at(-5, 10);
    run(&mut w, 1, 0);
    let tail = Knob::TarTail.raw() as u16;
    assert!(
        w.players[0].slowed > 0 && w.players[0].slowed + 2 >= tail,
        "a step out of the tar and the slow was gone at once ({} frames left)",
        w.players[0].slowed
    );
}

#[test]
fn a_pool_landing_on_a_pool_merges_into_it_and_the_areas_add() {
    let mut w = floor(Class::Champion);
    let first = fight::lay_pool(&mut w.lore, at(0, 0), Fx::from_int(3)).unwrap();
    let again = fight::lay_pool(&mut w.lore, at(1, 0), Fx::from_int(3)).unwrap();
    assert_eq!(first, again, "it merged");
    let r = hazard::get(&w.lore, first).radius();
    let want = Fx::from_int(18); // 3 squared, twice
    let got = r.mul(r);
    assert!(
        got.sub(want).abs().raw() < Fx::ratio(1, 2).raw(),
        "two three-metre pools made a pool of radius {r:?}"
    );
    for _ in 0..6 {
        fight::lay_pool(&mut w.lore, at(0, 0), Fx::from_int(3));
    }
    assert_eq!(
        hazard::get(&w.lore, first).radius(),
        Knob::PoolMost.fx(),
        "a pool grows only to its cap"
    );
}

#[test]
fn the_hazard_list_never_holds_more_than_sixteen() {
    let mut w = floor(Class::Champion);
    assert_eq!(hazard::room(&w.lore), 16);
    for i in 0..40 {
        let x = (i % 7) * 5 - 15;
        let z = (i / 7) * 5 - 15;
        fight::lay_pool(&mut w.lore, at(x, z), Fx::from_int(2));
    }
    assert!(hazard::all(&w.lore).count() <= 16);
    assert_eq!(
        hazard::all(&w.lore).count(),
        16,
        "full, and every pool after that merged into the nearest"
    );
}

#[test]
fn what_is_drawn_on_the_floor_is_the_hazard_list() {
    let mut w = floor(Class::Champion);
    fight::lay_pool(&mut w.lore, at(0, 0), Fx::from_int(3));
    fight::lay_pool(&mut w.lore, at(8, 0), Fx::from_int(2));
    let listed: Vec<(usize, Hazard)> = hazard::all(&w.lore).collect();
    let placed: Vec<_> = w.terrain().floor.iter().copied().collect();
    assert_eq!(listed.len(), placed.len());
    for ((slot, h), p) in listed.iter().zip(placed.iter()) {
        assert_eq!(*slot, p.slot as usize);
        assert_eq!(h.index(), p.kind);
        assert_eq!(h.radius(), p.radius);
        assert_eq!(h.centre(), p.a);
    }
}

// ---------------------------------------------------------------------------
// Fire and slag
// ---------------------------------------------------------------------------

fn kind_at(w: &World, slot: usize) -> u8 {
    hazard::get(&w.lore, slot).index()
}

#[test]
fn fire_runs_along_tar_that_touches_and_stops_where_it_does_not() {
    let mut w = floor(Class::Champion);
    let a = place(&mut w, tar(-6, 6, 2));
    let b = place(&mut w, tar(-3, 6, 2));
    let c = place(&mut w, tar(0, 6, 2));
    let apart = place(&mut w, tar(6, 6, 2));
    assert!(hazard::ignite(&mut w.lore, a));
    let every = hazard::stat(sp(), fight::BURNING, HazardField::Spread) as u32;
    run(&mut w, every - 2, 0);
    assert_eq!(kind_at(&w, b), fight::TAR, "fire waits before it runs");
    run(&mut w, 4, 0);
    assert_eq!(kind_at(&w, b), fight::BURNING, "the next pool caught");
    assert_eq!(kind_at(&w, c), fight::TAR, "one step at a time");
    run(&mut w, every + 2, 0);
    assert_eq!(kind_at(&w, c), fight::BURNING);
    run(&mut w, every * 4, 0);
    assert_eq!(
        kind_at(&w, apart),
        fight::TAR,
        "tar that does not touch did not"
    );
}

#[test]
fn a_burnt_pool_leaves_one_slag_mound_at_its_centre() {
    let mut w = floor(Class::Champion);
    let slot = place(&mut w, tar(0, 8, 3));
    hazard::ignite(&mut w.lore, slot);
    let life = hazard::stat(sp(), fight::BURNING, HazardField::Life);
    run(&mut w, life as u32 + 2, 0);
    let slag: Vec<_> = hazard::all(&w.lore)
        .filter(|(_, h)| h.index() == fight::SLAG)
        .collect();
    assert_eq!(slag.len(), 1, "one mound");
    assert_eq!(slag[0].1.centre(), at(0, 8), "at the pool's centre");
    assert_eq!(slag[0].1.state, 1);
    let ground = w.terrain();
    assert_eq!(
        ground.ground_under(at(0, 8)),
        hazard::stat_fx(sp(), fight::SLAG, HazardField::Solid)
    );
    assert_eq!(
        ground.ground_under(at(2, 8)),
        Fx::ZERO,
        "the rest of the disc is clean floor"
    );
    assert!(
        !hazard::all(&w.lore).any(|(_, h)| h.index() == fight::TAR),
        "nothing of the tar is left"
    );
}

/// How high the rim stands, standing: the lowest of its mountable faces.
fn rim(m: &Monster) -> Fx {
    let rig = m.rig();
    [
        mireback::FLANK_L,
        mireback::FLANK_R,
        mireback::RUMP,
        mireback::BROW,
    ]
    .iter()
    .map(|p| {
        let sh = sp().shape(*p);
        let mid = sh.min.add(sh.max).scale(Fx::ratio(1, 2));
        rig.part_to_world(*p, V3::new(mid.x, sh.max.y, mid.z)).y
    })
    .min_by_key(|h| h.raw())
    .unwrap()
}

fn apex(class: Class) -> Fx {
    let mut w = World::with_classes([class; MAX_PLAYERS]);
    for _ in 0..40 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let floor = w.players[0].pos.y;
    let mut best = floor;
    for _ in 0..240 {
        w.advance([Input::new(Input::SPACE), Input::default()]);
        best = best.max(w.players[0].pos.y);
    }
    best.sub(floor)
}

#[test]
fn slag_burnt_again_is_a_step_the_bulwark_can_climb_from() {
    let mut w = floor(Class::Bulwark);
    let mut s = Hazard::disc(fight::SLAG, at(0, 8), Fx::ONE);
    s.state = 1;
    place(&mut w, s);
    // Tar lands on it -- a pool off to one side, its centre not on the mound --
    // and is burnt.
    let pool = fight::lay_pool(&mut w.lore, at(2, 8), Fx::from_int(3)).unwrap();
    hazard::ignite(&mut w.lore, pool);
    let life = hazard::stat(sp(), fight::BURNING, HazardField::Life);
    run(&mut w, life as u32 + 2, 0);
    let top = w.terrain().ground_under(at(0, 8));
    let layer = hazard::stat_fx(sp(), fight::SLAG, HazardField::Solid);
    assert_eq!(top, layer.mul(Fx::from_int(2)), "two layers: three metres");
    let m = Monster::new(SpeciesId::MIREBACK);
    let hop = apex(Class::Bulwark);
    assert!(
        top.add(hop).raw() >= rim(&m).raw(),
        "the Bulwark's {hop:?} from {top:?} does not reach the rim at {:?}",
        rim(&m)
    );
    assert!(
        layer.add(hop).raw() < rim(&m).raw(),
        "one layer was already enough for her, so the second is not a route"
    );
}

#[test]
fn the_rim_is_out_of_a_standing_hop_for_all_but_the_dual_mage() {
    let m = Monster::new(SpeciesId::MIREBACK);
    let rim = rim(&m);
    for class in sim::class::ALL_CLASSES {
        let reach = apex(class);
        if class == Class::DualMage {
            assert!(reach.raw() >= rim.raw(), "the Dual mage cannot hop the rim");
        } else {
            assert!(
                reach.raw() < rim.raw(),
                "{} hops the rim from the floor ({reach:?} against {rim:?})",
                class.name()
            );
            // But from one slag mound, everybody but the Bulwark.
            let slag = hazard::stat_fx(sp(), fight::SLAG, HazardField::Solid);
            assert_eq!(
                class != Class::Bulwark,
                slag.add(reach).raw() >= rim.raw(),
                "{} from one mound",
                class.name()
            );
        }
    }
}

#[test]
fn the_mireback_burns_in_its_own_tar() {
    let mut w = hunt(Class::Champion);
    quiet(&mut w);
    let m = w.monsters[0].unwrap();
    // Four burning pools under its footprint: three of them count.
    for (x, z) in [(-5, 0), (5, 0), (0, 5), (0, -5)] {
        let slot = place(&mut w, tar(m.pos.x.to_int() + x, m.pos.z.to_int() + z, 2));
        hazard::ignite(&mut w.lore, slot);
    }
    let before = w.monsters[0].unwrap().health;
    run(&mut w, 60, 0);
    let after = w.monsters[0].unwrap();
    let ticks = 60 / Knob::BurnSelfEvery.raw();
    let want = Knob::BurnSelf.raw() * Knob::BurnSelfPools.raw() * ticks;
    let took = before - after.health;
    assert!(
        (took - want).abs() <= Knob::BurnSelf.raw() * Knob::BurnSelfPools.raw(),
        "a second over four burning pools took {took}, three pools' worth is {want}"
    );
    assert!(
        fight::coat(&after).raw() < Fx::ONE.raw(),
        "and its coat cracked"
    );
}

#[test]
fn a_brazier_tipped_by_a_blow_spills_coals_that_light_the_tar_they_touch() {
    let mut w = floor(Class::Champion);
    let (i, brazier) = fight::braziers(&w).next().expect("the mire has braziers");
    assert!(fight::brazier_lit(&w.lore, i));
    // The first brazier is on the south bank's plinth. A pool on the floor in
    // front of it, and a fighter on the plinth behind the brazier, facing the
    // arena: the coals spill away from the blow, onto the floor.
    let pool = place(&mut w, tar(brazier.x.to_int(), brazier.z.to_int() + 5, 2));
    w.players[0].pos = V3::new(brazier.x, brazier.y, brazier.z.sub(Fx::ONE));
    w.players[0].vel = V3::ZERO;
    let north = (Fx::ratio(1, 4).raw() & 0xFFFF) as u16;
    for _ in 0..10 {
        quiet(&mut w);
        w.advance([Input::aimed(0, north), Input::default()]);
    }
    for _ in 0..30 {
        quiet(&mut w);
        w.advance([Input::aimed(Input::LEFT, north), Input::default()]);
    }
    assert!(!fight::brazier_lit(&w.lore, i), "the blow tipped it");
    assert!(
        hazard::all(&w.lore).any(|(_, h)| h.index() == fight::COALS),
        "its coals are on the floor"
    );
    run(&mut w, 40, 0);
    assert_ne!(
        kind_at(&w, pool),
        fight::TAR,
        "the coals lit the tar they lie across"
    );
    let relight = Knob::BrazierRelight.raw() as u32;
    run(&mut w, relight + 2, 0);
    assert!(fight::brazier_lit(&w.lore, i), "and it relit");
}

#[test]
fn a_burst_wart_takes_a_quarter_of_its_tar() {
    // A spew's pool before and after a wart goes.
    let pool_after_spew = |burst: usize| -> (Fx, usize) {
        let mut w = hunt(Class::Champion);
        w.players[1].health = 0;
        let m = w.monsters[0].as_mut().unwrap();
        for wart in mireback::WARTS.iter().take(burst) {
            let slot = sp().break_slot(*wart).unwrap();
            m.breaks[slot] = 0;
        }
        m.brain.grace = 0;
        let a = sp().attack(mireback::SPEW);
        m.doing = Doing::Startup {
            kind: mireback::SPEW,
            left: a.startup,
        };
        let ahead = m.pos.add(V3::from_turns(m.yaw).scale(a.ideal_range));
        m.aim_at(ahead);
        for _ in 0..a.startup + 2 {
            w.advance([Input::default(); MAX_PLAYERS]);
        }
        let r = hazard::all(&w.lore)
            .find(|(_, h)| h.index() == fight::TAR)
            .map_or(Fx::ZERO, |(_, h)| h.radius());
        let m = w.monsters[0].unwrap();
        let (_, ring, _) = fight::ring(&m, m.pos, Fx::ONE);
        (r, ring)
    };
    let (whole, ring_whole) = pool_after_spew(0);
    let (less, ring_less) = pool_after_spew(1);
    let (none, _) = pool_after_spew(4);
    let ratio = less.mul(less).div(whole.mul(whole));
    assert!(
        ratio.sub(Fx::ratio(3, 4)).abs().raw() < Fx::ratio(1, 20).raw(),
        "a burst wart left a pool {ratio:?} the area"
    );
    assert_eq!(
        ring_less + 1,
        ring_whole,
        "and a flop's ring one pool fewer"
    );
    assert_eq!(
        none,
        Fx::ZERO,
        "with all four gone it spews no pools at all"
    );
}

#[test]
fn nothing_it_throws_can_reach_its_own_back() {
    for kind in 0..sp().moves.len() as u8 {
        let a = sp().attack(kind);
        let mut m = Monster::new(SpeciesId::MIREBACK);
        let ahead = m.pos.add(V3::from_turns(m.yaw).scale(a.ideal_range));
        m.aim_at(ahead);
        for left in (0..=a.active).rev() {
            m.doing = Doing::Active { kind, left };
            for part in [
                mireback::DOME,
                mireback::CROWN,
                mireback::FLANK_L,
                mireback::RUMP,
                mireback::BROW,
            ] {
                // Only where somebody could be standing: rolled over, its
                // back is underneath it.
                if m.rig().of(part).rot.r[1].y.raw() <= 0 {
                    continue;
                }
                let sh = sp().shape(part);
                let mid = sh.min.add(sh.max).scale(Fx::ratio(1, 2));
                let on = m.world_of(part, V3::new(mid.x, sh.max.y, mid.z));
                assert!(
                    !m.reaches(on, Fx::ratio(18, 10), Fx::ratio(1, 2)),
                    "{} reaches somebody standing on its {}",
                    sp().moves[kind as usize].name,
                    sp().parts[part].name
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The belch, the wallow, the sac
// ---------------------------------------------------------------------------

/// Start a move on the toad, now, where it stands.
fn start(w: &mut World, kind: u8) {
    let m = w.monsters[0].as_mut().unwrap();
    m.brain.grace = 0;
    m.doing = Doing::Startup {
        kind,
        left: sp().attack(kind).startup,
    };
    m.hit_used = false;
}

/// Run frames with the toad left to play out what it is doing, but not
/// starting anything new.
fn play(w: &mut World, frames: u32) {
    for _ in 0..frames {
        if let Some(m) = w.monsters[0].as_mut() {
            if m.doing.free() {
                m.brain.think_left = u16::MAX;
            }
        }
        w.advance([Input::default(); MAX_PLAYERS]);
    }
}

/// A hunt with the toad in the middle, facing west, nobody near it.
fn alone() -> World {
    let mut w = hunt(Class::Champion);
    w.players[0].pos = at(-16, 16);
    w.players[1].health = 0;
    quiet(&mut w);
    w
}

#[test]
fn the_belch_ignites_every_pool_it_marks_and_nothing_it_does_not() {
    let mut w = alone();
    let m = w.monsters[0].unwrap();
    let mouth = fight::mouth(&m);
    let reach = Knob::BelchReach.fx().to_int();
    let near = place(&mut w, tar(mouth.x.to_int() - 4, mouth.z.to_int(), 2));
    let edge = place(
        &mut w,
        tar(mouth.x.to_int() - reach - 1, mouth.z.to_int(), 2),
    );
    let far = place(
        &mut w,
        tar(mouth.x.to_int() - reach - 6, mouth.z.to_int(), 2),
    );
    start(&mut w, mireback::BELCH);
    play(&mut w, 2);
    let marked: Vec<V3> = w
        .marks()
        .iter()
        .filter(|k| k.look == sim::species::MarkLook::Kindling)
        .map(|k| k.at)
        .collect();
    let centre = |slot| hazard::get(&w.lore, slot).centre();
    assert!(marked.contains(&centre(near)) && marked.contains(&centre(edge)));
    assert!(!marked.contains(&centre(far)), "the far pool is not marked");
    play(&mut w, sp().attack(mireback::BELCH).startup as u32 + 2);
    assert_eq!(kind_at(&w, near), fight::BURNING);
    assert_eq!(kind_at(&w, edge), fight::BURNING);
    assert_eq!(kind_at(&w, far), fight::TAR, "nothing it did not mark");
}

#[test]
fn a_belch_broken_through_the_sac_backfires_under_its_own_chin() {
    let mut w = alone();
    let m = w.monsters[0].unwrap();
    let mouth = fight::mouth(&m);
    let chin = place(&mut w, tar(mouth.x.to_int(), mouth.z.to_int(), 2));
    let ahead = place(&mut w, tar(mouth.x.to_int() - 8, mouth.z.to_int(), 2));
    start(&mut w, mireback::BELCH);
    play(&mut w, 20);
    // A burst on the sac, out where the swelling put it.
    let beast = w.monsters[0].as_mut().unwrap();
    let bar = beast.interrupt_bar();
    let dealt = beast.take_hit(mireback::SAC, bar);
    assert!(dealt > 0);
    assert!(matches!(beast.doing, Doing::Flinch { .. }), "staggered");
    play(&mut w, 2);
    assert_eq!(
        kind_at(&w, chin),
        fight::BURNING,
        "the tar under its chin caught"
    );
    assert_eq!(kind_at(&w, ahead), fight::TAR, "and only that");
    let lit = w.lore.word(fight::word::LIT);
    assert_eq!((lit >> 16) & 0xFF, 1, "counted as a backfire");
}

/// Two tar pools under it, and a wallow begun.
fn wallowing() -> World {
    let mut w = alone();
    let m = w.monsters[0].unwrap();
    let (x, z) = (m.pos.x.to_int(), m.pos.z.to_int());
    place(&mut w, tar(x + 2, z, 3));
    place(&mut w, tar(x - 2, z, 3));
    let beast = w.monsters[0].as_mut().unwrap();
    beast.own[fight::body::COAT_LOST] = Fx::ONE.raw();
    start(&mut w, mireback::WALLOW);
    play(&mut w, sp().attack(mireback::WALLOW).startup as u32 + 10);
    w
}

#[test]
fn setting_a_wallow_alight_guts_it() {
    let mut w = wallowing();
    let slot = hazard::all(&w.lore).next().unwrap().0;
    hazard::ignite(&mut w.lore, slot);
    play(&mut w, 2);
    let m = w.monsters[0].unwrap();
    assert!(
        fight::gutted(&m),
        "lit on its back, it is gutted: {:?}",
        m.doing
    );
    assert_eq!(fight::coat(&m), Fx::ZERO, "with its coat gone");
    // And its own burn ticks double.
    let before = m.health;
    play(&mut w, 30);
    let took = before - w.monsters[0].unwrap().health;
    assert!(
        took >= Knob::BurnSelf.raw() * 2 * 2,
        "gutted, a half-second in its own fire took only {took}"
    );
}

#[test]
fn the_coat_comes_back_in_a_wallow_and_a_broken_one_is_half_made() {
    let mut w = wallowing();
    play(&mut w, sp().attack(mireback::WALLOW).active as u32);
    let m = w.monsters[0].unwrap();
    assert!(
        fight::coat(&m).raw() > Fx::ratio(9, 10).raw(),
        "a whole wallow brought the coat back to {:?}",
        fight::coat(&m)
    );

    let mut w = wallowing();
    let beast = w.monsters[0].as_mut().unwrap();
    let bar = Fx::from_int(beast.interrupt_bar())
        .mul(Knob::WallowInterrupt.fx())
        .to_int();
    beast.take_hit(mireback::DOME, bar * 2);
    let m = w.monsters[0].unwrap();
    assert!(
        matches!(m.doing, Doing::Flinch { .. }),
        "broken, it staggers"
    );
    assert_eq!(fight::coat(&m), Fx::ratio(1, 2), "with half a coat");
}

#[test]
fn the_sac_torn_winds_it_and_brings_the_rim_down() {
    let mut w = alone();
    start(&mut w, mireback::INFLATE);
    play(&mut w, sp().attack(mireback::INFLATE).startup as u32 + 4);
    let beast = w.monsters[0].as_mut().unwrap();
    let standing = rim(&Monster::new(SpeciesId::MIREBACK));
    beast.take_hit(mireback::SAC, Knob::SacPoise.raw());
    assert!(fight::winded(beast), "the sac tore: {:?}", beast.doing);
    play(&mut w, sp().stumble_frames() as u32 / 2);
    let m = w.monsters[0].unwrap();
    assert!(fight::winded(&m));
    let low = rim(&m);
    for class in sim::class::ALL_CLASSES {
        let hop = apex(class);
        assert_eq!(
            class != Class::Bulwark,
            hop.raw() >= low.raw(),
            "winded, the rim is at {low:?} and {} hops {hop:?}",
            class.name()
        );
    }
    assert!(low.raw() < standing.raw());
}

#[test]
fn a_knock_up_on_it_is_a_flinch_not_a_winding() {
    let mut w = alone();
    play(&mut w, 2);
    let beast = w.monsters[0].as_mut().unwrap();
    beast.strain = beast.cc_bar() * 2;
    let took = beast.take_control(sim::monster::Control::launching(Fx::from_int(8)));
    assert!(took.stumbled);
    play(&mut w, 1);
    let m = w.monsters[0].unwrap();
    assert!(
        matches!(m.doing, Doing::Flinch { .. }),
        "a launch it shrugged into {:?}",
        m.doing
    );
}

// ---------------------------------------------------------------------------
// The tongue and the swallow
// ---------------------------------------------------------------------------

/// A fighter standing `metres` in front of the toad's mouth, and the tongue
/// thrown at them.
fn tongued(class: Class, metres: i32) -> World {
    let mut w = hunt(class);
    w.players[1].health = 0;
    let m = w.monsters[0].unwrap();
    let ahead = V3::from_turns(m.yaw);
    let spot = m.pos.add(ahead.scale(Fx::from_int(6 + metres)));
    w.players[0].pos = V3::new(spot.x, Fx::ZERO, spot.z);
    w.players[0].vel = V3::ZERO;
    if let Some(m) = w.monsters[0].as_mut() {
        m.brain.seen = spot;
    }
    start(&mut w, mireback::TONGUE);
    w
}

fn swallowed(w: &World) -> bool {
    sim::state::inside(&w.players[0], &w.monsters)
}

#[test]
fn a_swallowed_fighter_is_spat_out_by_time_or_by_damage_whichever_is_first() {
    // By time.
    let mut w = tongued(Class::Champion, 4);
    let mut inside_for = 0;
    let mut was_in = false;
    for _ in 0..400 {
        play(&mut w, 1);
        if swallowed(&w) {
            inside_for += 1;
            was_in = true;
        } else if was_in {
            break;
        }
    }
    assert!(was_in, "the tongue caught and swallowed");
    let held = Knob::SwallowFrames.raw();
    assert!(
        (inside_for - held).abs() <= 2,
        "held {inside_for} frames, against {held}"
    );
    assert!(!w.players[0].aboard(), "and let out");
    let lost = w.players[0].full_health() - w.players[0].health;
    assert!(
        lost > Knob::SpitDamage.raw(),
        "acid and the spit took {lost}"
    );
    assert!(
        w.monsters[0].unwrap().brain.cooldown[mireback::TONGUE as usize] > 0,
        "and the tongue is locked after a swallow"
    );

    // By damage: hits on the stomach cut it short.
    let mut w = tongued(Class::Champion, 4);
    for _ in 0..80 {
        play(&mut w, 1);
        if swallowed(&w) {
            break;
        }
    }
    assert!(swallowed(&w));
    play(&mut w, 10);
    let beast = w.monsters[0].as_mut().unwrap();
    let retch = Knob::SwallowRetch.raw();
    let per = Fx::from_int(retch).div(Knob::VulnStomach.fx()).to_int() + 1;
    beast.take_hit(mireback::STOMACH, per);
    play(&mut w, 2);
    assert!(!swallowed(&w), "the stomach took {retch} and spat them out");
}

#[test]
fn the_tongue_stops_at_the_first_solid_on_its_line() {
    let mut w = tongued(Class::Champion, 6);
    // A slag mound between them.
    let m = w.monsters[0].unwrap();
    let mid = m.pos.add(V3::from_turns(m.yaw).scale(Fx::from_int(8)));
    let mut s = Hazard::disc(fight::SLAG, V3::new(mid.x, Fx::ZERO, mid.z), Fx::ONE);
    s.state = 1;
    place(&mut w, s);
    for _ in 0..60 {
        play(&mut w, 1);
    }
    assert!(!w.players[0].aboard(), "the slag took the tongue");
    let m = w.monsters[0].unwrap();
    assert!(
        sim::math::wide_flat_dist(m.aimed_at(), m.pos).raw() < Fx::from_int(8).raw(),
        "and the line stopped at it"
    );
}

#[test]
fn a_bulwark_blocking_the_tongue_gives_it_the_shield() {
    let mut w = tongued(Class::Bulwark, 4);
    let m = w.monsters[0].unwrap();
    let toward = sim::math::atan2_turns(
        m.pos.z.sub(w.players[0].pos.z),
        m.pos.x.sub(w.players[0].pos.x),
    );
    let aim = (toward.raw() & 0xFFFF) as u16;
    let mut gagged = false;
    for _ in 0..60 {
        if let Some(m) = w.monsters[0].as_mut() {
            if m.doing.free() {
                m.brain.think_left = u16::MAX;
            }
        }
        w.advance([Input::aimed(Input::RIGHT, aim), Input::default()]);
        if w.monsters[0].unwrap().doing.attacking() == Some(mireback::GAG) {
            gagged = true;
            break;
        }
    }
    assert!(gagged, "it choked on the shield");
    assert!(!swallowed(&w), "and swallowed nobody");
    assert!(
        matches!(
            w.players[0].shield(),
            Some(sim::class::Shield::Planted { .. })
        ),
        "the shield is planted where it was spat"
    );
}

#[test]
fn a_dead_toad_is_a_won_hunt_with_its_trophy_and_its_temper() {
    // The trophy is written from `hunt_won` (world W1), and a temper is the
    // same toad fought cleverer (W2): both have to name the Mireback.
    let mut w = hunt(Class::Champion).tempered(2);
    assert_eq!(w.arena().id, sim::arena::ArenaId::MIREBACK);
    assert_eq!(w.hunt_won(), None);
    w.monster_mut().unwrap().health = 0;
    w.advance([Input::default(); MAX_PLAYERS]);
    let (beaten, at) = w.hunt_won().expect("a dead toad is a won hunt");
    assert_eq!(beaten[0], Some(SpeciesId::MIREBACK));
    assert_eq!(at, 2);
}
