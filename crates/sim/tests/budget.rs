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
/// Currently a little over 1.8 KiB. Raising this is a real decision -- it means
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

/// The two shapes a match comes in, since the creature is most of the work when
/// it is present and none of it when it is not.
fn scenarios() -> [Scenario; 2] {
    [
        ("versus", |c| World::with_classes([c; MAX_PLAYERS])),
        ("hunt", |c| World::hunt([c; MAX_PLAYERS])),
    ]
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
                }
            })
        })
        .collect()
}
