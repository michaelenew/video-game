//! WebAssembly bindings for the browser sandbox.
//!
//! Raw C ABI, no wasm-bindgen. Every getter returns raw 16.16 fixed point;
//! JavaScript divides by 65536 for rendering. Float conversion happens on the
//! far side of the boundary so the simulation stays integer-only.

use core::cell::UnsafeCell;
use sim::state::{Action, MAX_PLAYERS, move_frames};
use sim::{Input, World};

struct Cell(UnsafeCell<Option<World>>);
// Single-threaded by construction: wasm32-unknown-unknown without threads.
unsafe impl Sync for Cell {}
static WORLD: Cell = Cell(UnsafeCell::new(None));

#[allow(clippy::mut_from_ref)]
fn world() -> &'static mut World {
    // Safety: wasm is single threaded here and every entry point is called
    // from the same JS task.
    unsafe { (*WORLD.0.get()).get_or_insert_with(World::new) }
}

#[unsafe(no_mangle)]
pub extern "C" fn sim_reset() {
    *world() = World::new();
}

#[unsafe(no_mangle)]
pub extern "C" fn sim_advance(a: u32, b: u32) {
    world().advance([Input::new(a as u16), Input::new(b as u16)]);
}

#[unsafe(no_mangle)]
pub extern "C" fn sim_frame() -> u32 {
    world().frame
}

#[unsafe(no_mangle)]
pub extern "C" fn sim_checksum_lo() -> u32 {
    world().checksum() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn sim_checksum_hi() -> u32 {
    (world().checksum() >> 32) as u32
}

fn p(i: u32) -> &'static sim::state::Player {
    &world().players[(i as usize).min(MAX_PLAYERS - 1)]
}

#[unsafe(no_mangle)]
pub extern "C" fn p_x(i: u32) -> i32 {
    p(i).pos.x.raw()
}
#[unsafe(no_mangle)]
pub extern "C" fn p_y(i: u32) -> i32 {
    p(i).pos.y.raw()
}
#[unsafe(no_mangle)]
pub extern "C" fn p_z(i: u32) -> i32 {
    p(i).pos.z.raw()
}
#[unsafe(no_mangle)]
pub extern "C" fn p_fx(i: u32) -> i32 {
    p(i).facing.x.raw()
}
#[unsafe(no_mangle)]
pub extern "C" fn p_fz(i: u32) -> i32 {
    p(i).facing.z.raw()
}
#[unsafe(no_mangle)]
pub extern "C" fn p_health(i: u32) -> i32 {
    p(i).health
}
#[unsafe(no_mangle)]
pub extern "C" fn p_grounded(i: u32) -> u32 {
    p(i).grounded as u32
}
#[unsafe(no_mangle)]
pub extern "C" fn p_action(i: u32) -> u32 {
    p(i).action.tag()
}
#[unsafe(no_mangle)]
pub extern "C" fn p_left(i: u32) -> u32 {
    p(i).action.frames_left() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn p_kind(i: u32) -> u32 {
    match p(i).action {
        Action::Startup { kind, .. }
        | Action::Active { kind, .. }
        | Action::Recovery { kind, .. } => kind as u32,
        _ => u32::MAX,
    }
}

/// The live attack volume during active frames, in raw fixed point: a capsule
/// between two points, of `hit_r`. Radius is zero when nothing is active, which
/// is how JS tests for a hitbox.
///
/// Two ends rather than one centre, because one of the six attacks is a line:
/// the Elementalist's auto is a beam along the crosshair. A swing is the case
/// where both ends are the same point.
#[unsafe(no_mangle)]
pub extern "C" fn hit_x(i: u32) -> i32 {
    hitbox(i).map(|b| b.from.x.raw()).unwrap_or(0)
}
#[unsafe(no_mangle)]
pub extern "C" fn hit_z(i: u32) -> i32 {
    hitbox(i).map(|b| b.from.z.raw()).unwrap_or(0)
}
#[unsafe(no_mangle)]
pub extern "C" fn hit_x2(i: u32) -> i32 {
    hitbox(i).map(|b| b.to.x.raw()).unwrap_or(0)
}
#[unsafe(no_mangle)]
pub extern "C" fn hit_z2(i: u32) -> i32 {
    hitbox(i).map(|b| b.to.z.raw()).unwrap_or(0)
}
#[unsafe(no_mangle)]
pub extern "C" fn hit_r(i: u32) -> i32 {
    hitbox(i).map(|b| b.radius.raw()).unwrap_or(0)
}

/// Straight from the simulation, so the browser tool cannot drift from the
/// game the way a reconstruction from the move table did -- it drew every
/// attack a fixed reach ahead of the body, which has not been true of an aimed
/// move for a while and was never true of a beam or of a weapon that sweeps.
fn hitbox(i: u32) -> Option<sim::state::Hitbox> {
    sim::state::hitbox(p(i))
}

/// Frame data for the debug overlay: startup, active, recovery.
#[unsafe(no_mangle)]
pub extern "C" fn move_startup(kind: u32) -> u32 {
    move_frames(world().players[0].class, kind as u8).0 as u32
}
#[unsafe(no_mangle)]
pub extern "C" fn move_active(kind: u32) -> u32 {
    move_frames(world().players[0].class, kind as u8).1 as u32
}
#[unsafe(no_mangle)]
pub extern "C" fn move_recovery(kind: u32) -> u32 {
    move_frames(world().players[0].class, kind as u8).2 as u32
}
#[unsafe(no_mangle)]
pub extern "C" fn parry_window() -> u32 {
    sim::state::parry_window() as u32
}
