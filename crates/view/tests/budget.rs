//! Guard test: the work between the snapshot and the screen fits in a frame.
//!
//! `view` is everything the renderer does that is not drawing -- blending the
//! two most recent snapshots, placing the camera, and posing the skeletons --
//! and it runs once per *rendered* frame rather than once per simulated one.
//! That makes its budget a plain share of 16.7 ms rather than the divided one
//! `sim`'s budget is (see `sim/tests/budget.rs`), but it also makes it the part
//! a player feels immediately: a simulation that is late can be caught up, a
//! camera that is late is a camera that stutters.
//!
//! The thing actually being guarded is that all three stay *bounded*. The
//! camera marches a segment through the arena's geometry to keep walls out of
//! the shot, and the pose solver runs IK per fighter; both are the kind of code
//! where a new piece of geometry or a solver that stops converging turns a
//! fixed cost into one that grows with the scene. As in the simulation's
//! budget, the clock is set to catch that rather than a slow drift, and the
//! allocation count -- which is the same on every machine -- is what catches
//! the drift.

use sim::state::MAX_PLAYERS;
use sim::{Class, Input, World};
use view::camera::RigConfig;
use view::{CameraRig, PoseInput, Surroundings, interpolate, pose_for};

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::time::Instant;

// ---------------------------------------------------------------------------
// The budget
// ---------------------------------------------------------------------------

/// How long one frame lasts at 60 Hz, in nanoseconds.
const FRAME_NS: u128 = 1_000_000_000 / sim::TICK_HZ as u128;

/// What one rendered frame's worth of `view` may cost.
///
/// An eighth of the frame, which leaves seven eighths for the thing that
/// actually talks to the GPU. Against a measured worst case of around fifty
/// microseconds that is a margin of roughly forty times, and it is meant to be:
/// this is a tripwire for a cost that has changed *kind* -- become unbounded,
/// started allocating, started scanning the arena -- not a stopwatch to tune
/// against.
const VIEW_BUDGET_NS: u128 = FRAME_NS / 8;

/// Rendered frames per timed run.
const FRAMES: u32 = 600;

/// Timed runs per scenario; the best one wins, because noise only ever adds.
const REPEATS: usize = 3;

/// A rendered frame at 60 Hz, in seconds, for the rig's own smoothing.
const DT: f32 = 1.0 / 60.0;

// ---------------------------------------------------------------------------
// Counting what a frame allocates
// ---------------------------------------------------------------------------

thread_local! {
    /// Allocations on this thread only -- the harness runs tests in parallel,
    /// and a shared counter would report somebody else's work.
    static ALLOCATIONS: Cell<u64> = const { Cell::new(0) };
}

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

/// `try_with` rather than `with`: a thread being torn down has already dropped
/// its locals, and panicking inside the allocator there would be a mystery.
fn bump() {
    let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

fn allocations_during(body: impl FnOnce()) -> u64 {
    let before = ALLOCATIONS.with(Cell::get);
    body();
    ALLOCATIONS.with(Cell::get) - before
}

// ---------------------------------------------------------------------------
// The tests
// ---------------------------------------------------------------------------

/// Everything between the snapshot and the screen fits in its share of a frame.
#[test]
fn a_rendered_frame_of_view_work_fits_its_budget() {
    let mut worst: Option<(String, u128)> = None;

    for class in [Class::Bulwark, Class::ShadowReaver, Class::Elementalist] {
        for (scenario, with_beast) in [("versus", false), ("hunt", true)] {
            let mut best = u128::MAX;
            for _ in 0..REPEATS {
                let (prev, cur) = pair(class, with_beast);
                let mut rig = CameraRig::new(RigConfig::default());
                let started = Instant::now();
                for n in 0..FRAMES {
                    draw_one(&mut rig, &prev, &cur, n);
                }
                best = best.min(started.elapsed().as_nanos() / FRAMES as u128);
            }
            if worst.as_ref().is_none_or(|(_, ns)| best > *ns) {
                worst = Some((format!("{class:?} in a {scenario}"), best));
            }
        }
    }

    let (where_, ns) = worst.expect("there is at least one scenario to measure");
    assert!(
        ns <= VIEW_BUDGET_NS,
        "a rendered frame of view work costs {ns} ns ({where_}), over the {VIEW_BUDGET_NS} ns \
         budget -- an eighth of a {FRAME_NS} ns frame. Interpolation, the camera and the pose \
         solver all run once per picture, so this is time the renderer no longer has."
    );
}

/// None of it goes to the heap.
///
/// Interpolation returns a `Frame` of fixed arrays and a `Pose` is `[f32;
/// CHANNELS]`, so the per-frame path is allocation-free by construction today.
/// Pinning that is what stops a `Vec` appearing in the pose solver or the
/// camera's geometry march, where it would cost an allocation every frame
/// forever and show up as an occasional dropped one.
#[test]
fn a_rendered_frame_of_view_work_allocates_nothing() {
    for class in [Class::Bulwark, Class::ShadowReaver, Class::Elementalist] {
        for (scenario, with_beast) in [("versus", false), ("hunt", true)] {
            let (prev, cur) = pair(class, with_beast);
            let mut rig = CameraRig::new(RigConfig::default());
            // Once outside the count: the first frame settles the rig, and any
            // one-off setup it does is not what this is about.
            draw_one(&mut rig, &prev, &cur, 0);

            let allocations = allocations_during(|| {
                for n in 0..FRAMES {
                    draw_one(&mut rig, &prev, &cur, n);
                }
            });
            assert_eq!(
                allocations, 0,
                "{class:?} in a {scenario} went to the heap {allocations} times over {FRAMES} \
                 rendered frames. The path from snapshot to screen must not allocate."
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The workload
// ---------------------------------------------------------------------------

/// One rendered frame's worth of `view`: blend the snapshots, place the camera,
/// pose both fighters.
///
/// Swept rather than static -- the blend fraction, the yaw and the pitch all
/// move -- because a camera measured at one angle is a camera measured in one
/// branch. The pitch in particular decides whether the rig is in the floor
/// zone, the neutral zone or the first-person handover, and they do different
/// amounts of work.
fn draw_one(rig: &mut CameraRig, prev: &World, cur: &World, n: u32) {
    let sweep = n as f32 / FRAMES as f32;
    let frame = interpolate(prev, cur, sweep.fract());
    let framing = rig.update_around(
        DT,
        frame.players[0].pos,
        sweep * std::f32::consts::TAU,
        // The whole legal range, floor to sky, over the run.
        (sweep - 0.5) * std::f32::consts::PI * 0.9,
        Surroundings {
            beast: cur.monster.as_ref(),
            aboard: cur.players[0].aboard(),
        },
    );
    std::hint::black_box(framing);
    for i in 0..MAX_PLAYERS {
        let input = PoseInput::of(&frame.players[i], cur.players[i].class, &frame);
        std::hint::black_box(pose_for(input));
    }
}

/// Two consecutive snapshots of a match already in progress, which is what the
/// renderer is always handed: fighters mid-move, with whatever they have put on
/// the field still there.
fn pair(class: Class, with_beast: bool) -> (World, World) {
    let mut world = if with_beast {
        World::hunt([class; MAX_PLAYERS])
    } else {
        World::with_classes([class; MAX_PLAYERS])
    };
    let script = input_script(240);
    for inputs in &script {
        world.advance(*inputs);
    }
    let prev = world.clone();
    world.advance(script[0]);
    (prev, world)
}

/// Deterministic inputs, seeded rather than clocked. The same generator the
/// soak and the simulation's own budget test use.
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
