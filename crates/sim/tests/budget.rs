//! Guard test: a simulation frame has to fit inside a frame, several times over.
//!
//! The simulation is not the expensive part of this game and it must stay that
//! way, because it is the part that runs *more than once* per picture. A
//! rendered frame draws once; a rollback re-simulates a whole prediction window
//! -- `net::MAX_ROLLBACK_FRAMES`, eight at the time of writing -- inside that
//! same 16.7 ms, and saves a snapshot for every one of them. So a cost that
//! looks harmless per frame arrives multiplied, and it arrives exactly when the
//! player is already having a bad time: mid-rollback, which is to say mid-lag.
//!
//! That is what makes a performance rule here a *relationship* rather than a
//! number, in the same sense as `feel.rs`: the simulation may spend a quarter
//! of a frame on a whole rollback burst, and nothing more. Divided across the
//! window, that is a thirty-second of a frame for one advance.
//!
//! Three tests, and the two that do not look at a clock are the load-bearing
//! ones. A wall clock on a shared machine can only be trusted with a wide
//! margin, so [`a_frame_of_simulation_fits_its_budget`] is set to catch a
//! catastrophe -- an accidental quadratic, an unbounded scan, a blocking call
//! -- rather than a slow drift. The drift is caught deterministically instead,
//! by counting what a frame *allocates* and measuring what a snapshot *costs to
//! copy*, neither of which varies with how busy the machine is.

use sim::class::{ALL_CLASSES, Class};
use sim::monster::MAX_MONSTERS;
use sim::species::SpeciesId;
use sim::state::MAX_PLAYERS;
use sim::{Input, TICK_HZ, World};

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::time::Instant;

// ---------------------------------------------------------------------------
// The budgets
// ---------------------------------------------------------------------------

/// How long one simulated frame lasts, in nanoseconds. The whole budget.
const FRAME_NS: u128 = 1_000_000_000 / TICK_HZ as u128;

/// What one call to `World::advance` may cost.
///
/// A quarter of the frame for the simulation, divided by the eight frames a
/// rollback may re-simulate inside one -- see the note on this file. Roughly
/// half a millisecond, against a measured worst case of around fifty
/// microseconds for the heaviest class in a hunt, so there is an order of
/// magnitude in hand. That margin is deliberate and is not slack to be spent:
/// it is what lets this run on a loaded laptop without crying wolf.
const ADVANCE_BUDGET_NS: u128 = FRAME_NS / 32;

/// The most a `World` may weigh.
///
/// Rollback copies one every single frame and keeps a ring of nine, so the
/// snapshot is the one structure in the game whose *size* is a running cost
/// rather than a one-off. Four kilobytes keeps the whole ring inside a core's
/// L2 cache, which is what makes a save a memcpy nobody notices.
///
/// About 2.8 KiB since the world gained a second creature slot for the Pair
/// (2026-10-01: 2,856 bytes, from 2,680), and 3,520 bytes since it gained ten
/// critter slots and a pack brain (bestiary P3, 2026-10-01; see
/// `docs/design/critters.md`). Raising this is a real decision -- it means
/// every frame and every rollback got heavier -- so change it deliberately, the
/// way `knobs.rs` wants a reason, rather than to make a red test green.
const SNAPSHOT_CAP: usize = 4096;

/// Frames per timed run. Ten seconds of play, so a move's whole lifecycle --
/// startup, active, recovery, and whatever it leaves on the field -- is inside
/// the measurement many times over.
const FRAMES: u32 = 600;

/// How many times each scenario is timed. The best run wins: noise on a shared
/// machine only ever makes something look slower, never faster, so the minimum
/// is the closest thing to the true cost that a wall clock can offer.
const REPEATS: usize = 3;

// ---------------------------------------------------------------------------
// Counting what a frame allocates
// ---------------------------------------------------------------------------

thread_local! {
    /// Allocations on *this* thread. Thread-local rather than global because
    /// the test harness runs these in parallel, and a counter shared with
    /// whatever else is running would report someone else's work.
    static ALLOCATIONS: Cell<u64> = const { Cell::new(0) };
}

/// The system allocator, with a tally.
struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        bump();
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        bump();
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

/// One more allocation on this thread.
///
/// `try_with` rather than `with`: a thread being torn down has already dropped
/// its locals, and a panic from inside the allocator at that point would be a
/// mystery with no connection to the thing it is guarding.
fn bump() {
    let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// Run `body` and say how many times it went to the heap.
fn allocations_during(body: impl FnOnce()) -> u64 {
    let before = ALLOCATIONS.with(Cell::get);
    body();
    ALLOCATIONS.with(Cell::get) - before
}

// ---------------------------------------------------------------------------
// The tests
// ---------------------------------------------------------------------------

/// A frame of simulation costs a small fraction of the frame it represents.
///
/// Every class, alone in the arena and in a hunt, because the creature roughly
/// triples the work and is the case a versus-only measurement would miss.
#[test]
fn a_frame_of_simulation_fits_its_budget() {
    let script = input_script(FRAMES);
    let mut worst: Option<(String, u128)> = None;

    for class in ALL_CLASSES {
        for (scenario, build) in scenarios() {
            let mut best = u128::MAX;
            for _ in 0..REPEATS {
                let mut world = build(class);
                let started = Instant::now();
                for inputs in &script {
                    world.advance(*inputs);
                }
                // Kept so the optimiser cannot decide the whole match was
                // pointless and delete the thing being measured.
                std::hint::black_box(world.checksum());
                best = best.min(started.elapsed().as_nanos() / script.len() as u128);
            }
            if worst.as_ref().is_none_or(|(_, ns)| best > *ns) {
                worst = Some((format!("{class:?} in a {scenario}"), best));
            }
        }
    }

    let (where_, ns) = worst.expect("there is at least one class to measure");
    assert!(
        ns <= ADVANCE_BUDGET_NS,
        "a simulation frame costs {ns} ns ({where_}), over the {ADVANCE_BUDGET_NS} ns budget.\n\
         That budget is a quarter of a {FRAME_NS} ns frame, shared out across the eight frames a \
         rollback re-simulates inside one -- so this is not a frame being slightly slow, it is a \
         rollback no longer fitting in the time it has."
    );
}

/// A frame of simulation never goes to the heap.
///
/// `sim` has no dependencies, no I/O and no floating point, and this is the
/// fourth member of that family: no allocation either. It matters for the same
/// reason the others do. An allocator is shared mutable state with a lock and a
/// tail: it is the classic source of a frame that takes a millisecond when the
/// last thousand took ten microseconds, which is exactly the stutter a player
/// reports as framiness rather than as slowness.
///
/// It is also the cheapest possible check, and unlike a clock it gives the same
/// answer on every machine. A `Vec` or a `String` appearing in the step is the
/// thing this catches, on the commit that adds it.
#[test]
fn a_frame_of_simulation_allocates_nothing() {
    let script = input_script(FRAMES);

    for class in ALL_CLASSES {
        for (scenario, build) in scenarios() {
            let mut world = build(class);
            // Built outside the count: constructing a world is allowed to
            // allocate, it is running one that is not.
            let allocations = allocations_during(|| {
                for inputs in &script {
                    world.advance(*inputs);
                }
            });
            assert_eq!(
                allocations, 0,
                "{class:?} in a {scenario} went to the heap {allocations} times over \
                 {FRAMES} frames. A simulation frame must not allocate: see this test's note."
            );
        }
    }
}

/// **The Broodmother's whole fight in the snapshot, run without the heap**: her
/// clocks, the brood at the cap and her floor full are all cells of the
/// `World`, so a copy is flat, it fits, and a minute of it never allocates.
#[test]
fn the_broodmother_fight_fits_the_snapshot_and_does_not_allocate() {
    let script = input_script(FRAMES);
    for class in ALL_CLASSES {
        let mut world = full_hollows(World::hunt_of([class; MAX_PLAYERS], SpeciesId::BROODMOTHER));
        assert!(std::mem::size_of_val(&world) <= SNAPSHOT_CAP);
        let allocations = allocations_during(|| {
            for inputs in &script {
                world.advance(*inputs);
                let copy = world.clone();
                std::hint::black_box(&copy);
            }
        });
        assert_eq!(
            allocations, 0,
            "{class:?} in the Hollows went to the heap {allocations} times"
        );
    }
}

/// Saving and restoring a frame is a copy, and a small one.
///
/// Two claims. The snapshot fits in [`SNAPSHOT_CAP`], and taking one does not
/// allocate -- which together are what make a save cheap enough to do on every
/// frame without thinking about it. A `World` that grew a heap-backed field
/// would still *work*; it would just quietly put an allocation and a pointer
/// chase into the hot path of the netcode, on both the save and the restore.
#[test]
fn a_snapshot_is_a_small_flat_copy() {
    let size = std::mem::size_of::<World>();
    assert!(
        size <= SNAPSHOT_CAP,
        "a World snapshot is {size} bytes, over the {SNAPSHOT_CAP}-byte cap. \
         Rollback copies one every frame and keeps nine of them, so this is a cost paid \
         continuously rather than once -- see the note on SNAPSHOT_CAP before raising it."
    );

    let script = input_script(120);
    let mut world = World::hunt([Class::ShadowReaver; MAX_PLAYERS]);
    for inputs in &script {
        world.advance(*inputs);
    }

    let allocations = allocations_during(|| {
        for _ in 0..64 {
            std::hint::black_box(world.clone());
        }
    });
    assert_eq!(
        allocations, 0,
        "cloning a World allocated {allocations} times, so the snapshot is no longer a flat copy."
    );
}

// ---------------------------------------------------------------------------
// The workload
// ---------------------------------------------------------------------------

/// A way to set up a match, and what to call it when it is the one over budget.
type Scenario = (&'static str, fn(Class) -> World);

/// The shapes a match comes in, since the creature is most of the work when it
/// is present and none of it when it is not -- and two creatures, the most the
/// world holds, are twice that. The last is the same in the range: the biggest
/// arena and the most solids any arena has, which is what every collision,
/// floor and aiming query walks.
fn scenarios() -> [Scenario; 15] {
    [
        ("versus", |c| World::with_classes([c; MAX_PLAYERS])),
        ("hunt", |c| World::hunt([c; MAX_PLAYERS])),
        ("two creatures", |c| {
            World::hunt_with([c; MAX_PLAYERS], [Some(SpeciesId::RIDGEBACK); MAX_MONSTERS])
        }),
        ("two creatures in the range", |c| {
            World::hunt_in(
                [c; MAX_PLAYERS],
                [Some(SpeciesId::RIDGEBACK); MAX_MONSTERS],
                sim::arena::ArenaId::RANGE,
            )
        }),
        // A pack of small bodies (bestiary P3): the dev pack, every critter
        // slot filled, so the ring, the tokens, the separation pairs and every
        // swing tested against ten boxes are all at their most.
        ("full pack", |c| {
            full_pack(World::hunt_of([c; MAX_PLAYERS], SpeciesId::GNATS))
        }),
        // The Gnawers in the Commons, every slot filled (their coop pack is
        // ten): the dart's hand-out, the pile-on's call, the latch, the gnaw
        // and the scramble all run on top of the generic pack.
        ("the gnawers' full pack", |c| {
            full_pack(World::hunt_of([c; MAX_PLAYERS], SpeciesId::GNAWERS))
        }),
        // And a pack with a creature in the fight as well -- the Broodmother's
        // and the Siegeshell's shape -- in the range.
        ("pack and creature in the range", |c| {
            full_pack(World::hunt_in(
                [c; MAX_PLAYERS],
                [Some(SpeciesId::RIDGEBACK), Some(SpeciesId::GNATS)],
                sim::arena::ArenaId::RANGE,
            ))
        }),
        // The Hornback's herd running a stampede on the crossing: every cow
        // steered down its lane round every lee and every fallen fighter, the
        // swept stop asked of the boulders, the cart standing as a solid.
        ("the herd's stampede on the crossing", |c| {
            herd_running(World::hunt_in(
                [c; MAX_PLAYERS],
                [Some(SpeciesId::HORNBACK), None],
                sim::arena::ArenaId::HORNBACK_CROSSING,
            ))
        }),
        // Everything the hunt's lore can hold (bestiary P4 to P7): the dev
        // creature in the range, every hazard slot full where the fight is --
        // tar burning and spreading, smoke, a sinkhole, a strand, a vent on
        // its back -- every noise heard, both defended things standing, and
        // its senses asked of both fighters every frame.
        ("full floor in the range", |c| {
            full_floor(World::hunt_in(
                [c; MAX_PLAYERS],
                [Some(SpeciesId::SENTINEL), None],
                sim::arena::ArenaId::RANGE,
            ))
        }),
        // The Mireback with its whole floor down: every one of its sixteen
        // hazard slots full round the hunters -- tar, fire spreading along
        // it, slag standing, coals -- every brazier and swallow word read.
        ("full mire", |c| {
            full_mire(World::hunt_of([c; MAX_PLAYERS], SpeciesId::MIREBACK))
        }),
        // The Sandmaw in the Pan, loud: both sinkholes open, every cell of
        // the noise ring full, and its spine, its attention and its fences
        // against the islands walked every frame.
        ("the pan, loud", |c| {
            loud_pan(World::hunt_of([c; MAX_PLAYERS], SpeciesId::SANDMAW))
        }),
        // The Pair in the Den, both cats coiled for the twin pounce: two rigs
        // posed, and each posed again for its telegraph, every frame.
        ("the den, both coiled", |c| {
            both_coiled(World::hunt_of([c; MAX_PLAYERS], SpeciesId::PAIR))
        }),
        // The Broodmother at her worst: the brood at its cap round the
        // hunters, every sac held, every hazard slot full -- web patches,
        // strands across the floor, a glob -- and her clocks, guard and legs
        // read every frame.
        ("the hollows, full", |c| {
            full_hollows(World::hunt_of([c; MAX_PLAYERS], SpeciesId::BROODMOTHER))
        }),
        // The Mantis with its guard up between two hunters: its eyes written
        // and read for both every frame, every blow asked of its guard, its
        // memory and its arc drawn.
        ("the shrine, guarded", |c| {
            guarded_shrine(World::hunt_of([c; MAX_PLAYERS], SpeciesId::MANTIS))
        }),
        // The Siegeshell at the siege line: forty-four parts, six legs posed
        // procedurally, its parasites all down, its vents on the plates, a
        // stamp on the legs' channel while the beam winds up, and a hunter
        // on the crown -- every floor and wall query walks its shell.
        ("the valley, at the wall", |c| {
            siege_full(World::hunt_of([c; MAX_PLAYERS], SpeciesId::SIEGESHELL))
        }),
    ]
}

/// The Siegeshell at the siege line with everything in it: [`scenarios`]'.
fn siege_full(mut w: World) -> World {
    use sim::species::siegeshell::{self as ss, fight as f};
    let idle = [Input::default(); MAX_PLAYERS];
    w.advance(idle);
    if let Some(m) = w.monsters[0].as_mut() {
        // Its head twenty metres short of the wall.
        m.pos.x = sim::Fx::from_int(105);
        m.brain.grace = 0;
        m.doing = sim::monster::Doing::Startup {
            kind: ss::BEAM,
            left: ss::SPECIES.attack(ss::BEAM).startup,
        };
        f::leg::set(
            m,
            f::leg::Channel {
                kind: f::leg::STAMP,
                leg: 0,
                phase: f::leg::STARTUP,
                left: 60,
                struck: 0,
            },
        );
    }
    let crown = w.monsters[0]
        .as_ref()
        .map(|m| {
            let top = m.sp().shape(ss::CROWN_PART).max;
            m.rig().part_to_world(
                ss::CROWN_PART,
                sim::V3::new(sim::Fx::ZERO, top.y, sim::Fx::ZERO),
            )
        })
        .unwrap_or_default();
    w.players[0].pos = crown;
    w.advance(idle);
    let at = w.monsters[0].as_ref().map(|m| m.pos).unwrap_or_default();
    if let Some(pack) = w.pack.as_mut() {
        while sim::pack::spawn(pack, &mut w.critters, sim::species::gnawers::GNAWER, at, 0)
            .is_some()
        {}
    }
    w
}

/// **The Siegeshell's whole fight in the snapshot, run without the heap**:
/// its two channels, its anchors and ankles, its vents and its parasites are
/// cells of the `World`, so a copy is flat, it fits, and a minute of it never
/// allocates.
#[test]
fn the_siegeshell_fight_fits_the_snapshot_and_does_not_allocate() {
    let script = input_script(FRAMES);
    for class in ALL_CLASSES {
        let mut world = siege_full(World::hunt_of([class; MAX_PLAYERS], SpeciesId::SIEGESHELL));
        assert!(std::mem::size_of_val(&world) <= SNAPSHOT_CAP);
        let allocations = allocations_during(|| {
            for inputs in &script {
                world.advance(*inputs);
                let copy = world.clone();
                std::hint::black_box(&copy);
                std::hint::black_box(world.marks());
                std::hint::black_box(world.signs());
            }
        });
        assert_eq!(
            allocations, 0,
            "{class:?} in the Last Valley went to the heap {allocations} times"
        );
    }
}

/// The Mantis with its guard up, its prayer over.
fn guarded_shrine(mut w: World) -> World {
    use sim::species::mantis;
    let a = mantis::SPECIES.attack(mantis::GUARD);
    if let Some(m) = w.monsters[0].as_mut() {
        m.brain.grace = 0;
        m.doing = sim::monster::Doing::Active {
            kind: mantis::GUARD,
            left: a.active,
        };
    }
    w
}

/// **The Mantis's whole fight in the snapshot, run without the heap**: its
/// eyes, its memory and its guard are cells of the `World`, so a copy is
/// flat, it fits, and a minute of it never allocates.
#[test]
fn the_mantis_fight_fits_the_snapshot_and_does_not_allocate() {
    let script = input_script(FRAMES);
    for class in ALL_CLASSES {
        let mut world = guarded_shrine(World::hunt_of([class; MAX_PLAYERS], SpeciesId::MANTIS));
        assert!(std::mem::size_of_val(&world) <= SNAPSHOT_CAP);
        let allocations = allocations_during(|| {
            for inputs in &script {
                world.advance(*inputs);
                let copy = world.clone();
                std::hint::black_box(&copy);
                std::hint::black_box(world.marks());
                std::hint::black_box(world.signs());
            }
        });
        assert_eq!(
            allocations, 0,
            "{class:?} in the Shrine went to the heap {allocations} times"
        );
    }
}

/// Both cats winding up the twin pounce at the first fighter.
fn both_coiled(mut w: World) -> World {
    use sim::species::pair;
    let me = w.players[0].pos;
    for m in w.monsters.iter_mut().flatten() {
        m.brain.seen = me;
        m.brain.grace = 0;
        m.doing = sim::monster::Doing::Startup {
            kind: pair::TWIN,
            left: pair::SPECIES.attack(pair::TWIN).startup,
        };
        m.lob(pair::TWIN);
    }
    w
}

/// The Broodmother's fight with everything in it: [`scenarios`]' last.
fn full_hollows(mut w: World) -> World {
    use sim::hazard::{self, Hazard};
    use sim::species::broodmother::{Knob, fight as f};
    let at =
        |x: i32, z: i32| sim::V3::new(sim::Fx::from_int(x), sim::Fx::ZERO, sim::Fx::from_int(z));
    w.advance([Input::default(); MAX_PLAYERS]);
    if let Some(pack) = w.pack.as_mut() {
        let cap = Knob::BroodCap.raw().max(0);
        for k in 0..cap {
            sim::pack::spawn(
                pack,
                &mut w.critters,
                sim::species::gnawers::GNAWER,
                at(-8 + (k % 4) * 2, (k / 4) * 3 - 2),
                0,
            );
        }
    }
    let slots = w.lore.layout().hazards as i32;
    for i in 0..slots {
        let (x, z) = (-12 + (i % 4) * 5, (i / 4) * 5 - 7);
        let placed = match i % 3 {
            0 => Hazard::disc(f::PATCH, at(x, z), sim::Fx::from_int(2)),
            1 => Hazard::strand(f::STRAND, at(x, z), at(x + 4, z), sim::Fx::ratio(1, 4)),
            _ => Hazard::disc(f::GLOB, at(x, z), sim::Fx::ONE),
        };
        hazard::place(&mut w.lore, placed);
    }
    w
}

/// The Hornback's bull bellowing now, so the whole herd is about to run.
fn herd_running(mut w: World) -> World {
    use sim::species::hornback as h;
    let idle = [Input::default(); MAX_PLAYERS];
    for _ in 0..2 {
        w.advance(idle);
    }
    if let Some(pack) = w.pack.as_mut() {
        pack.mood = sim::pack::mood::HUNTING;
        pack.grace = 0;
    }
    if let Some(b) = w.critters.iter().position(|c| c.kind == h::BULL) {
        let bull = &mut w.critters[b];
        bull.state = sim::critter::is::STARTUP;
        bull.act = h::BELLOW;
        bull.timer = 1;
    }
    w
}

/// The Sandmaw's fight with its whole lore in use: two sinkholes and a full
/// ring of noises round the hunters.
fn loud_pan(mut w: World) -> World {
    use sim::hazard::{self, Hazard};
    use sim::noise::{self, NoiseKind};
    use sim::species::sandmaw::fight as f;
    let at =
        |x: i32, z: i32| sim::V3::new(sim::Fx::from_int(x), sim::Fx::ZERO, sim::Fx::from_int(z));
    hazard::place(
        &mut w.lore,
        Hazard::disc(f::SINKHOLE, at(-10, -3), sim::Fx::from_int(3)),
    );
    hazard::place(
        &mut w.lore,
        Hazard::disc(f::SINKHOLE, at(-10, 3), sim::Fx::from_int(3)),
    );
    for i in 0..8 {
        noise::make(
            &mut w.lore,
            NoiseKind::Footfall,
            at(-12 + i, (i % 3) - 1),
            (i % 2) as u8,
            1,
            0,
        );
    }
    w
}

/// Every hazard slot of the Mireback's fight filled round the hunters.
fn full_mire(mut w: World) -> World {
    use sim::hazard::{self, Hazard};
    use sim::species::mireback::fight as f;
    let at =
        |x: i32, z: i32| sim::V3::new(sim::Fx::from_int(x), sim::Fx::ZERO, sim::Fx::from_int(z));
    let r = sim::Fx::from_int(2);
    let kinds = [
        f::TAR,
        f::TAR,
        f::BURNING,
        f::TAR,
        f::SLAG,
        f::TAR,
        f::BURNING,
        f::TAR,
    ];
    for i in 0..16 {
        let kind = kinds[i % kinds.len()];
        let (x, z) = (-16 + (i as i32 % 4) * 4, (i as i32 / 4) * 4 - 6);
        let placed = if i == 15 {
            Hazard::strand(f::COALS, at(x, z), at(x + 4, z), sim::Fx::ONE)
        } else {
            Hazard::disc(kind, at(x, z), r)
        };
        hazard::place(&mut w.lore, placed);
    }
    w
}

/// Every hazard slot of the sentinel's fight filled round the hunters' marks.
fn full_floor(mut w: World) -> World {
    use sim::hazard::{self, Hazard};
    use sim::species::sentinel as s;
    w.lore.set_word(s::word::LAID, 1);
    let at =
        |x: i32, z: i32| sim::V3::new(sim::Fx::from_int(x), sim::Fx::ZERO, sim::Fx::from_int(z));
    let r = sim::Fx::from_int(3);
    let kinds = [
        s::TAR,
        s::BURNING,
        s::TAR,
        s::SMOKE,
        s::SINKHOLE,
        s::SNARE,
        s::TAR,
        s::BURNING,
    ];
    for (i, kind) in kinds.into_iter().enumerate() {
        hazard::place(
            &mut w.lore,
            Hazard::disc(kind, at(-110 + i as i32 * 3, (i as i32 % 3) * 2 - 2), r),
        );
    }
    hazard::place(
        &mut w.lore,
        Hazard::strand(s::STRAND, at(-112, 6), at(-96, 6), sim::Fx::ONE),
    );
    hazard::place(
        &mut w.lore,
        Hazard::on_part(
            s::VENT,
            0,
            sim::species::ridgeback::RIDGE,
            sim::V3::ZERO,
            sim::Fx::ONE,
        ),
    );
    w
}

/// Every critter slot of a pack fight filled, spawned beside the pack's den.
fn full_pack(mut w: World) -> World {
    let Some(pack) = w.pack.as_mut() else {
        return w;
    };
    let home = pack.home;
    while sim::pack::spawn(pack, &mut w.critters, 0, home, 0).is_some() {}
    w
}

/// Deterministic inputs, seeded rather than clocked -- the same script every
/// run, so two measurements are of the same match and not of two different
/// ones. The same generator the soak uses.
///
/// Random buttons rather than a scripted rotation on purpose: what this is
/// timing is the *worst* thing the simulation can be made to do, and a fight
/// where every ability is thrown constantly and interrupted halfway is a
/// heavier load than one a person would actually play.
fn input_script(frames: u32) -> Vec<[Input; MAX_PLAYERS]> {
    let mut rng = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    (0..frames)
        .map(|_| {
            std::array::from_fn(|_| {
                let r = next();
                Input {
                    bits: (r & 0x1ff) as u16,
                    aim: (r >> 16) as u16,
                    pitch: ((r >> 32) as i16) / 8,
                    travel: Default::default(),
                }
            })
        })
        .collect()
}
