//! **State as bytes, for the wire.** A region's outcome and a newcomer's
//! snapshot carry simulation state -- a `Player`, a whole `World` -- which
//! `sim`, having no dependencies, has no serialiser for.
//!
//! What crosses here is the struct's own bytes, copied in and read back
//! unaligned. That is sound only between two copies of **the same build**,
//! which is what the meeting's terms already demand of two peers before a
//! frame is played (`meet.rs`: a hello with other terms is never answered).
//! It is the one `unsafe` in the networking, kept to this file, and it is a
//! stopgap: the day `sim` writes its state out field by field, this file is
//! deleted and nothing else changes.
//!
//! Padding bytes travel too. They carry nothing, and reading them back
//! lands in padding, which nothing reads.

use sim::World;
use sim::state::Player;

pub const PLAYER: usize = std::mem::size_of::<Player>();
pub const WORLD: usize = std::mem::size_of::<World>();

/// A fighter's state as bytes.
pub fn player_bytes(p: &Player) -> Vec<u8> {
    let mut out = vec![0u8; PLAYER];
    // SAFETY: `Player` is `Copy` with no pointers, `out` is exactly its size,
    // and the source outlives the copy. Padding is copied as it lies.
    unsafe {
        std::ptr::copy_nonoverlapping(p as *const Player as *const u8, out.as_mut_ptr(), PLAYER);
    }
    out
}

/// A fighter's state back from bytes written by [`player_bytes`] on the same
/// build; `None` if it is not that many bytes.
pub fn player_from(bytes: &[u8]) -> Option<Player> {
    if bytes.len() != PLAYER {
        return None;
    }
    // SAFETY: the bytes are a `Player` written by this build, and an
    // unaligned read makes no alignment demand of the buffer.
    Some(unsafe { std::ptr::read_unaligned(bytes.as_ptr() as *const Player) })
}

/// A whole world as bytes: a newcomer's snapshot.
pub fn world_bytes(w: &World) -> Vec<u8> {
    let mut out = vec![0u8; WORLD];
    // SAFETY: as for a player. `World` holds fixed arrays and scalars only
    // (`sim/tests/budget.rs`: a snapshot is a flat copy).
    unsafe {
        std::ptr::copy_nonoverlapping(w as *const World as *const u8, out.as_mut_ptr(), WORLD);
    }
    out
}

pub fn world_from(bytes: &[u8]) -> Option<World> {
    if bytes.len() != WORLD {
        return None;
    }
    // SAFETY: as for a player.
    Some(unsafe { std::ptr::read_unaligned(bytes.as_ptr() as *const World) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim::class::Class;

    #[test]
    fn a_world_and_its_fighters_come_back_as_they_went() {
        let mut w = World::with_classes([Class::Elementalist, Class::ShadowReaver]);
        for _ in 0..30 {
            w.advance([
                sim::Input::aimed(sim::Input::W | sim::Input::LEFT, 7),
                sim::Input::default(),
            ]);
        }
        let back = world_from(&world_bytes(&w)).unwrap();
        assert_eq!(back, w);
        assert_eq!(back.checksum(), w.checksum());
        let p = player_from(&player_bytes(&w.players[0])).unwrap();
        assert_eq!(p, w.players[0]);
    }
}
