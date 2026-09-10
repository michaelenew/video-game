//! WebAssembly bindings for the browser sandbox.
//!
//! Raw C ABI, no wasm-bindgen. Every getter returns raw 16.16 fixed point;
//! JavaScript divides by 65536 for rendering. Float conversion happens on the
//! far side of the boundary so the simulation stays integer-only.

use core::cell::UnsafeCell;
use sim::state::{Action, MAX_PLAYERS, move_frames};
use sim::{Fx, Input, World};

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
    world().advance([Input(a as u16), Input(b as u16)]);
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

/// Hitbox centre and radius during active frames, in raw fixed point.
/// Radius is zero when nothing is active, which is how JS tests for a hitbox.
#[unsafe(no_mangle)]
pub extern "C" fn hit_x(i: u32) -> i32 {
    hitbox(i).map(|(c, _)| c.x.raw()).unwrap_or(0)
}
#[unsafe(no_mangle)]
pub extern "C" fn hit_z(i: u32) -> i32 {
    hitbox(i).map(|(c, _)| c.z.raw()).unwrap_or(0)
}
#[unsafe(no_mangle)]
pub extern "C" fn hit_r(i: u32) -> i32 {
    hitbox(i).map(|(_, r)| r.raw()).unwrap_or(0)
}

fn hitbox(i: u32) -> Option<(sim::V3, Fx)> {
    let pl = p(i);
    let Action::Active { kind, .. } = pl.action else {
        return None;
    };
    // Straight from the move table, so the overlay cannot drift from the
    // simulation the way a duplicated constant would.
    let m = sim::moves::get(pl.class, kind);
    Some((pl.pos.add(pl.facing.scale(m.reach)), m.radius))
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
