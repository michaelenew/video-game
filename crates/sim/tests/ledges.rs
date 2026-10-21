//! **Creatures and the ground's steps** (2026-10-21, from play: "In the high
//! plane the two ridgeback fight is really crazy. The ridgebacks clip into
//! and out of the box they start next to just by walking, their spikes and
//! charge are super inconsistent with how they interact with it.").
//!
//! - a creature never stands inside a solid: it is pushed off it, stands on
//!   it, or is getting up onto it or down off it;
//! - a creature gets up a ledge under its reach and down one, as a move that
//!   takes frames and in which it throws nothing (`sim::climb`);
//! - a ledge over its reach is a wall;
//! - what it throws along the floor stops at a solid.

use sim::Class;
use sim::arena::Terrain;
use sim::fixed::Fx;
use sim::input::Input;
use sim::math::V3;
use sim::monster::{Doing, Monster};
use sim::state::World;
use sim::valley::ranges;

fn range(name: &str) -> &'static ranges::Range {
    ranges::RANGES.iter().find(|r| r.name == name).unwrap()
}

/// The Sky Meadow's two Ridgebacks, woken and roused, with fighter one
/// standing at `at` (the range's coordinates, on the floor or a top).
fn sky_meadow() -> World {
    let r = range("Sky Meadow");
    let mut w = World::arrive([Class::Champion], Default::default(), r.arena, Some(0), 1);
    assert!(w.hunting(), "the Sky Meadow wakes");
    w.valley.roused = true;
    for m in w.monsters.iter_mut().flatten() {
        m.brain.grace = 0;
    }
    w
}

/// The rock beside the Sky Meadow, in the world's coordinates: the solid
/// four metres tall nearest the herd.
fn the_rock(w: &World) -> sim::arena::Solid {
    let ground = w.terrain();
    let m = w.monsters[0].unwrap();
    ground
        .around(m.pos, Fx::from_int(30))
        .filter(|s| s.max.y.sub(s.min.y).raw() > Fx::from_int(3).raw())
        .filter(|s| !ground.hangs(s))
        .min_by_key(|s| {
            let c = V3::new(
                Fx::from_raw(s.min.x.raw() / 2 + s.max.x.raw() / 2),
                Fx::ZERO,
                Fx::from_raw(s.min.z.raw() / 2 + s.max.z.raw() / 2),
            );
            sim::math::wide_flat_dist(c, m.pos).raw()
        })
        .expect("a rock by the Sky Meadow")
}

/// How deep into a solid a creature's body is, flat, if its feet are below
/// the solid's top and its middle inside the box grown by its radius.
fn inside_by(m: &Monster, s: &sim::arena::Solid, ground: &Terrain) -> Option<Fx> {
    let r = sim::climb::footprint(m.sp());
    let feet = m.pos.y;
    if feet.raw() >= s.max.y.sub(Fx::ratio(1, 10)).raw() || ground.hangs(s) {
        return None;
    }
    let ox = (m.pos.x.sub(s.min.x.sub(r))).min(s.max.x.add(r).sub(m.pos.x));
    let oz = (m.pos.z.sub(s.min.z.sub(r))).min(s.max.z.add(r).sub(m.pos.z));
    let deep = ox.min(oz);
    (deep.raw() > Fx::ratio(1, 4).raw()).then_some(deep)
}

fn look(w: &World, i: usize, at: V3) -> Input {
    let p = w.players[i].pos;
    let aim = sim::math::atan2_turns(at.z.sub(p.z), at.x.sub(p.x)).raw() as u16;
    Input::looking_at(0, aim, 0)
}

#[test]
fn the_sky_meadows_ridgebacks_never_stand_inside_the_rock() {
    let mut w = sky_meadow();
    let rock = the_rock(&w);
    // On the rock's top, in its middle: what they want is up there.
    let top = V3::new(
        Fx::from_raw(rock.min.x.raw() / 2 + rock.max.x.raw() / 2),
        rock.max.y,
        Fx::from_raw(rock.min.z.raw() / 2 + rock.max.z.raw() / 2),
    );
    w.players[0].pos = top;
    let mut climbed = 0;
    let mut worst = Fx::ZERO;
    for f in 0..2400 {
        let input = look(&w, 0, w.monsters[0].unwrap().pos);
        w.advance([input]);
        w.players[0].health = sim::tuning::health_of(Class::Champion);
        let ground = w.terrain();
        for m in w.monsters.iter().flatten() {
            if matches!(m.doing, Doing::Climb { .. }) {
                climbed += 1;
                continue;
            }
            if let Some(d) = inside_by(m, &rock, &ground) {
                if d.raw() > worst.raw() {
                    worst = d;
                }
                assert!(
                    d.raw() <= Fx::ratio(1, 2).raw(),
                    "frame {f}: a Ridgeback {} m inside the rock at {:?} (feet {}, top {})",
                    d.to_f32_for_render(),
                    m.pos,
                    m.pos.y.to_f32_for_render(),
                    rock.max.y.to_f32_for_render()
                );
            }
        }
    }
    assert!(
        climbed > 0,
        "a Ridgeback got up onto the rock after its target"
    );
}

/// The middle of a solid's top.
fn top_of(rock: &sim::arena::Solid) -> V3 {
    V3::new(
        Fx::from_raw(rock.min.x.raw() / 2 + rock.max.x.raw() / 2),
        rock.max.y,
        Fx::from_raw(rock.min.z.raw() / 2 + rock.max.z.raw() / 2),
    )
}

/// Run until some creature is doing what `want` asks, or `frames` pass; the
/// fighter held still and unhurt.
fn until(w: &mut World, frames: u32, want: impl Fn(&Monster) -> bool) -> Option<Monster> {
    for _ in 0..frames {
        let input = look(w, 0, w.monsters[0].unwrap().pos);
        w.advance([input]);
        w.players[0].health = sim::tuning::health_of(Class::Champion);
        if let Some(m) = w.monsters.iter().flatten().find(|m| want(m)) {
            return Some(*m);
        }
    }
    None
}

#[test]
fn getting_up_is_a_window_it_throws_nothing_in() {
    let mut w = sky_meadow();
    let rock = the_rock(&w);
    w.players[0].pos = top_of(&rock);
    let up = until(
        &mut w,
        1200,
        |m| matches!(m.doing, Doing::Climb { rise, .. } if rise > 0),
    )
    .expect("it got up after its target");
    let Doing::Climb { total, rise, .. } = up.doing else {
        unreachable!()
    };
    let rise = sim::lore::from_cm(rise);
    assert!(
        rise.sub(rock.max.y).abs().raw() < Fx::ratio(1, 2).raw(),
        "it got up the rock's height, {} m",
        rise.to_f32_for_render()
    );
    assert_eq!(total, sim::climb::frames(up.sp(), rise));
    // The window: frames in which it throws nothing, as long as the rock
    // is tall -- over a second for four metres.
    assert!(
        total as u32 >= sim::TICK_HZ,
        "getting up four metres took only {total} frames"
    );
    let slot = w
        .monsters
        .iter()
        .position(|m| m.is_some_and(|m| m.doing == up.doing && m.pos == up.pos))
        .unwrap();
    let mut climbed = 0;
    while w.monsters[slot].unwrap().doing.climbing() {
        let m = w.monsters[slot].unwrap();
        assert!(m.doing.open(), "getting up is an opening");
        assert!(m.hit_volume().is_none(), "nothing is out while it gets up");
        w.advance([Input::new(0)]);
        w.players[0].health = sim::tuning::health_of(Class::Champion);
        climbed += 1;
        assert!(climbed <= total as u32 + 1);
    }
    let m = w.monsters[slot].unwrap();
    assert_eq!(m.pos.y, rock.max.y, "and it stands on the rock's top");
}

#[test]
fn it_gets_down_after_somebody_who_jumped_off() {
    let mut w = sky_meadow();
    let rock = the_rock(&w);
    w.players[0].pos = top_of(&rock);
    until(&mut w, 1500, |m| m.pos.y == rock.max.y && m.doing.free()).expect("one got up");
    // Off the rock and well away from it.
    w.players[0].pos = V3::new(rock.max.x.add(Fx::from_int(8)), Fx::ZERO, top_of(&rock).z);
    w.players[0].pos.y = w.terrain().ground_under(w.players[0].pos);
    let down = until(
        &mut w,
        900,
        |m| matches!(m.doing, Doing::Climb { rise, .. } if rise < 0),
    )
    .expect("it got down after them");
    let Doing::Climb { total, rise, .. } = down.doing else {
        unreachable!()
    };
    let rise = sim::lore::from_cm(rise);
    assert!(
        total < sim::climb::frames(down.sp(), rise.neg()),
        "getting down is quicker than getting up the same height"
    );
}

/// An open floor sixty metres across with one block on it, `TALL` high, for
/// the rules that need a ledge of a given height and nothing else.
static BLOCK: sim::arena::Arena = sim::arena::Arena {
    id: sim::arena::ArenaId::PROVING_GROUND,
    name: "a block",
    creature: None,
    bounds: sim::arena::Bounds::cm((-3000, 3000), (-3000, 3000)),
    floor: sim::arena::Material::Ground,
    regions: &[],
    solids: &[sim::arena::Solid::cm(
        [-800, 0, -800],
        [800, 700, 800],
        sim::arena::Material::Stone,
    )],
    spawns: sim::arena::Spawns {
        versus: [
            sim::arena::Mark::cm(-400, 0, (400, 0)),
            sim::arena::Mark::cm(400, 0, (-400, 0)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

#[test]
fn a_ledge_over_its_shoulders_is_a_wall() {
    // Seven metres: over the Ridgeback's shoulder of four and a half.
    let ground = Terrain::bare(&BLOCK);
    let block = BLOCK.solids[0];
    let mut m = Monster::new(sim::species::SpeciesId::RIDGEBACK);
    assert!(block.max.y.raw() > sim::climb::reach(m.sp()).raw());
    m.pos = V3::new(Fx::from_int(-20), Fx::ZERO, Fx::ZERO);
    m.brain.cooldown = [u16::MAX; sim::species::MAX_MOVES];
    let on_top = sim::monster::Quarry {
        pos: V3::new(Fx::ZERO, block.max.y, Fx::ZERO),
        vel: V3::ZERO,
        alive: true,
        aboard: false,
        stunned: false,
    };
    let mut worst = Fx::ZERO;
    for f in 0..900 {
        m.step_in(&[on_top], &sim::monster::Senses::ALL, &ground);
        assert!(
            !m.doing.climbing(),
            "frame {f}: it started getting up seven metres"
        );
        if let Some(d) = inside_by(&m, &block, &ground) {
            worst = worst.max(d);
        }
    }
    assert!(
        worst.raw() <= Fx::ratio(1, 2).raw(),
        "it pressed {} m into the block",
        worst.to_f32_for_render()
    );
    assert_eq!(m.pos.y, Fx::ZERO, "and it is still on the floor");
}

#[test]
fn its_spikes_stop_at_the_rock() {
    // A Ridgeback on the floor facing a fighter on the far side of the rock,
    // and its spray thrown: the rock is cover.
    let mut w = sky_meadow();
    let rock = the_rock(&w);
    let mid = top_of(&rock);
    w.monsters[1] = None;
    let m = w.monsters[0].as_mut().unwrap();
    m.pos = V3::new(rock.max.x.add(Fx::from_int(4)), Fx::ZERO, mid.z);
    m.yaw = Fx::ratio(1, 2);
    m.brain.grace = 0;
    let spray = sim::species::ridgeback::SPECIES.attack(sim::species::ridgeback::SPRAY);
    m.doing = Doing::Startup {
        kind: sim::species::ridgeback::SPRAY,
        left: spray.startup,
    };
    // Behind the rock, at the spray's own range, on the floor.
    w.players[0].pos = V3::new(rock.min.x.sub(Fx::from_int(2)), Fx::ZERO, mid.z);
    w.players[0].pos.y = w.terrain().ground_under(w.players[0].pos);
    let before = w.players[0].health;
    let mut reached = Fx::MAX;
    for _ in 0..spray.total() as u32 {
        w.monsters[0].as_mut().unwrap().brain.think_left = u16::MAX;
        w.advance([Input::new(0)]);
        if let Some((at, r, _, _)) = w.monsters[0].unwrap().hit_volume() {
            reached = reached.min(at.x.sub(r));
        }
    }
    assert_eq!(
        w.players[0].health, before,
        "the spray went through the rock"
    );
    assert!(
        reached.raw() >= rock.max.x.sub(Fx::ratio(1, 10)).raw(),
        "its volume reached {} m, into a rock whose face is at {} m",
        reached.to_f32_for_render(),
        rock.max.x.to_f32_for_render()
    );
}

#[test]
fn it_does_not_charge_a_rock_or_slam_through_one() {
    // A fighter on the rock: nothing the Ridgeback on the floor has reaches up
    // there, so it chooses none of it -- it gets up instead.
    let mut w = sky_meadow();
    let rock = the_rock(&w);
    w.players[0].pos = top_of(&rock);
    for _ in 0..600 {
        let input = look(&w, 0, w.monsters[0].unwrap().pos);
        w.advance([input]);
        w.players[0].health = sim::tuning::health_of(Class::Champion);
        for m in w.monsters.iter().flatten() {
            let below = m.pos.y.raw() < rock.max.y.sub(Fx::ONE).raw();
            if below && let Some(kind) = m.doing.attacking() {
                let a = m.sp().attack(kind);
                let top = m.pos.y.add(a.hit_high);
                assert!(
                    a.damage <= 0 || top.raw() >= rock.max.y.raw(),
                    "from the floor it threw {}, which reaches {} m, at somebody on a {} m rock",
                    a.name,
                    top.to_f32_for_render(),
                    rock.max.y.to_f32_for_render()
                );
            }
        }
    }
}

#[test]
fn the_proving_grounds_platforms_are_got_up_too() {
    // A step the Ridgeback does not walk over (a metre and a half, over its
    // one-metre step) and is well under its shoulders: a short window.
    let mut w = World::hunt([Class::Champion; 2]);
    let ground = w.terrain();
    let platform = ground
        .solids()
        .filter(|s| s.min.x.raw() < 0 && s.max.z.raw() < Fx::from_int(10).raw())
        .find(|s| s.max.x.sub(s.min.x).raw() < Fx::from_int(6).raw())
        .expect("the west platform");
    w.players[0].pos = top_of(&platform);
    w.players[1].health = 0;
    w.monsters[0].as_mut().unwrap().brain.grace = 0;
    let up = until(
        &mut w,
        2400,
        |m| matches!(m.doing, Doing::Climb { rise, .. } if rise > 0),
    )
    .expect("it got up onto the platform after them");
    let Doing::Climb { total, .. } = up.doing else {
        unreachable!()
    };
    assert!(total < sim::climb::frames(up.sp(), Fx::from_int(4)));
}

/// Where a foot's sole is, as a creature stands posed.
fn sole(m: &Monster, foot: usize) -> V3 {
    let sh = m.sp().shape(foot);
    let mid = V3::new(
        sim::math::half(sh.min.x.add(sh.max.x)),
        sh.min.y,
        sim::math::half(sh.min.z.add(sh.max.z)),
    );
    m.rig().part_to_world(foot, mid)
}

#[test]
fn every_body_with_legs_gets_up_with_its_feet_where_the_motion_puts_them() {
    let mut wrong = Vec::new();
    for id in (0..sim::species::COUNT as u8).map(sim::species::SpeciesId) {
        let Some(sp) = sim::species::lookup(id) else {
            continue;
        };
        if !sp.has_body() || sp.legs.is_empty() {
            continue;
        }
        let b = sim::climb::build(sp);
        let rise = b.shoulder.mul(Fx::ratio(3, 4));
        let total = 60u16;
        for (k, (u, what)) in [(0, "starting"), (30, "halfway"), (60, "up")]
            .iter()
            .enumerate()
        {
            let mut m = Monster::new(id);
            let left = total - *u as u16;
            m.doing = Doing::Climb {
                left,
                total,
                rise: sim::lore::to_cm(rise),
            };
            let at = Fx::from_int(*u).div(Fx::from_int(total as i32));
            let ph = sim::climb::phase_of(sp, at, rise);
            m.pos.y = rise.mul(ph.body);
            // Against where the foot is standing: a toad sits on its hind
            // feet folded, and its sole is not at nought.
            let standing = Monster::new(id);
            for leg in sp.legs {
                let want = if leg.front {
                    ph.front_foot
                } else {
                    ph.rear_foot
                };
                let got = sole(&m, leg.foot).y.sub(sole(&standing, leg.foot).y);
                let off = got.sub(want);
                // A foot never goes into what it stands on, and where the
                // motion has it planted it is planted -- the start and the
                // top -- within a tenth of its shoulder.
                let slack = b.shoulder.div(Fx::from_int(10));
                let end = if leg.front { "fore" } else { "hind" };
                if off.raw() < slack.neg().raw() {
                    wrong.push(format!(
                        "{}: {what}, a {end} foot {} m under where it should be",
                        sp.name,
                        off.neg().to_f32_for_render()
                    ));
                }
                if k != 1 && off.raw() > slack.raw() {
                    wrong.push(format!(
                        "{}: {what}, a {end} foot {} m over where it should be",
                        sp.name,
                        off.to_f32_for_render()
                    ));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn a_bred_body_gets_up_the_same_way() {
    // The morph is not a species: the same motion, read into its drive
    // (`sim::breed::climb`). Its feet start on the floor and end on the top,
    // and none goes into the floor on the way.
    use sim::breed::{archetypes, build, climb, locomotion, posture};
    let rates = climb::Rates::tuned();
    let mut wrong = Vec::new();
    for (name, g) in [
        ("Ridgeback", archetypes::RIDGEBACK),
        ("Hornback", archetypes::HORNBACK),
        ("Pair", archetypes::PAIR),
        ("Mireback", archetypes::MIREBACK),
        ("Broodmother", archetypes::BROODMOTHER),
    ] {
        let q = posture::posture(&g);
        let b = climb::build(&g, &q);
        let rise = b.shoulder.mul(Fx::ratio(3, 4));
        let rest = build::body_driven(&g, &q, &locomotion::rest(&g));
        for (u, what) in [(0, "starting"), (50, "halfway"), (100, "up")] {
            let at = Fx::from_int(u).div(Fx::from_int(100));
            let d = climb::drive(&g, &q, locomotion::rest(&g), at, rise, &rates);
            let body = build::body_driven(&g, &q, &d);
            let ph = sim::climb::profile(at, rise, b.shoulder, rates.clear, rates.dip);
            let slack = b.shoulder.div(Fx::from_int(10));
            for (k, leg) in body.legs().iter().enumerate() {
                let got = body.lowest(leg.foot).sub(rest.lowest(rest.legs()[k].foot));
                let want = if leg.front {
                    ph.front_foot
                } else {
                    ph.rear_foot
                };
                let off = got.sub(want);
                if off.raw() < slack.neg().raw() || (u != 50 && off.raw() > slack.raw()) {
                    wrong.push(format!(
                        "{name}: {what}, a {} foot {} m off where it should be",
                        if leg.front { "fore" } else { "hind" },
                        off.to_f32_for_render()
                    ));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Run a hunt of `species` with fighter one standing on the top of the
/// nearest solid it could be got up after (or on the floor where there is
/// none), and report the most any creature's middle was inside a solid it was
/// not standing on or getting up, and whether one got up.
fn walk_its_own_arena(species: sim::species::SpeciesId, frames: u32) -> (Fx, bool) {
    let mut w = World::hunt_of([Class::Champion; 2], species);
    w.players[1].health = 0;
    for m in w.monsters.iter_mut().flatten() {
        m.brain.grace = 0;
    }
    let ground = w.terrain();
    let first = w.monsters.iter().flatten().next().copied().unwrap();
    let step = first.sp().fight_fx(sim::species::FightField::StepOver);
    let reach = sim::climb::reach(first.sp());
    // A top it can get up, standing on the floor, broad enough to be
    // footing: where fighter one waits.
    let wide = sim::climb::footprint(first.sp()).add(sim::climb::footprint(first.sp()));
    let ledge = ground
        .solids()
        .filter(|s| !ground.hangs(s) && s.min.y.raw() <= Fx::ratio(1, 10).raw())
        .filter(|s| s.max.y.raw() > step.raw() && s.max.y.raw() <= reach.raw())
        .filter(|s| {
            s.max.x.sub(s.min.x).raw() >= wide.raw() && s.max.z.sub(s.min.z).raw() >= wide.raw()
        })
        .min_by_key(|s| sim::math::wide_flat_dist(top_of(s), first.pos).raw());
    if let Some(l) = ledge {
        w.players[0].pos = top_of(&l);
    }
    let mut worst = Fx::ZERO;
    let mut climbed = false;
    for _ in 0..frames {
        let input = look(&w, 0, w.monsters.iter().flatten().next().unwrap().pos);
        w.advance([input, Input::new(0)]);
        w.players[0].health = sim::tuning::health_of(Class::Champion);
        let ground = w.terrain();
        for m in w.monsters.iter().flatten() {
            if m.doing.climbing() {
                climbed = true;
                continue;
            }
            for s in ground.around(m.pos, Fx::from_int(4)) {
                if s.max.y.sub(m.pos.y).raw() <= step.raw() {
                    continue;
                }
                if let Some(d) = inside_by(m, &s, &ground) {
                    worst = worst.max(d);
                }
            }
        }
    }
    (worst, climbed)
}

#[test]
fn the_mireback_broodmother_and_siegeshell_keep_out_of_their_arenas_solids() {
    use sim::species::SpeciesId;
    let mut wrong = Vec::new();
    for id in [
        SpeciesId::MIREBACK,
        SpeciesId::BROODMOTHER,
        SpeciesId::SIEGESHELL,
    ] {
        let (worst, _) = walk_its_own_arena(id, 1800);
        if worst.raw() > Fx::ratio(1, 2).raw() {
            wrong.push(format!(
                "{}: its middle {} m inside a solid",
                id.get().name,
                worst.to_f32_for_render()
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// An open floor a hundred metres across with one block on it, eighteen
/// metres square and `$cm` tall: a ledge for a body of any size.
macro_rules! block {
    ($name:ident, $cm:expr) => {
        static $name: sim::arena::Arena = sim::arena::Arena {
            id: sim::arena::ArenaId::PROVING_GROUND,
            name: "a block",
            creature: None,
            bounds: sim::arena::Bounds::cm((-5000, 5000), (-5000, 5000)),
            floor: sim::arena::Material::Ground,
            regions: &[],
            solids: &[sim::arena::Solid::cm(
                [-900, 0, -900],
                [900, $cm, 900],
                sim::arena::Material::Stone,
            )],
            spawns: sim::arena::Spawns {
                versus: [
                    sim::arena::Mark::cm(-400, 0, (400, 0)),
                    sim::arena::Mark::cm(400, 0, (-400, 0)),
                ],
                hunt: None,
            },
            sites: &[],
            rim: None,
        };
    };
}

// Three quarters of each one's reach (`climb::reach`): the Ridgeback's 4.5 m,
// the Mireback's 1.2, the Broodmother's 3.6, the Siegeshell's 6.5.
block!(RIDGEBACK_BLOCK, 340);
block!(MIREBACK_BLOCK, 90);
block!(BROODMOTHER_BLOCK, 270);
block!(SIEGESHELL_BLOCK, 490);

#[test]
fn every_body_that_collides_gets_up_after_somebody_on_a_block() {
    use sim::species::SpeciesId;
    let mut wrong = Vec::new();
    for (id, arena) in [
        (SpeciesId::RIDGEBACK, &RIDGEBACK_BLOCK),
        (SpeciesId::MIREBACK, &MIREBACK_BLOCK),
        (SpeciesId::BROODMOTHER, &BROODMOTHER_BLOCK),
        (SpeciesId::SIEGESHELL, &SIEGESHELL_BLOCK),
    ] {
        let ground = Terrain::bare(arena);
        let block = arena.solids[0];
        let mut m = Monster::new(id);
        assert!(m.climber(), "{} keeps its own footing", m.sp().name);
        m.pos = V3::new(Fx::from_int(-35), Fx::ZERO, Fx::ZERO);
        m.brain.cooldown = [u16::MAX; sim::species::MAX_MOVES];
        let on_top = sim::monster::Quarry {
            pos: V3::new(Fx::ZERO, block.max.y, Fx::ZERO),
            vel: V3::ZERO,
            alive: true,
            aboard: false,
            stunned: false,
        };
        let mut climbed = false;
        for _ in 0..2400 {
            m.step_in(&[on_top], &sim::monster::Senses::ALL, &ground);
            // The Siegeshell's shared walk is held: its march is its own
            // frame hook, which walks it and hands it back to the ground
            // (`Monster::keep_to_the_ground`). Marched straight at the block.
            if id == SpeciesId::SIEGESHELL && !m.doing.climbing() {
                let was = m.pos;
                let forward = V3::from_turns(m.yaw);
                m.pos = m.pos.add(forward.scale(m.sp().walk().mul(sim::DT)));
                m.keep_to_the_ground(&ground, was, Some(on_top.pos));
            }
            climbed |= m.doing.climbing();
        }
        if !climbed || m.pos.y != block.max.y {
            wrong.push(format!(
                "{}: got up {climbed}, standing at {} m on a {} m block",
                m.sp().name,
                m.pos.y.to_f32_for_render(),
                block.max.y.to_f32_for_render()
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
