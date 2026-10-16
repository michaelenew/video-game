//! What the books are made of has to fit on the browser's stack.
//!
//! A wasm module's stack is one megabyte, and a value is built on it before
//! `Box::new` moves it to the heap. The tape was `TAPE` worlds held inline,
//! about 590 KB, and building one ran the page off the bottom of its memory:
//! `RuntimeError: memory access out of bounds` at load, in whatever ran next.
//! The desktop's eight-megabyte main thread never noticed.

use regions::{Ledger, Tape};

/// A tape is a handle to its slots, not the slots.
#[test]
fn a_tape_is_a_handle() {
    assert!(
        std::mem::size_of::<Tape>() <= 64,
        "Tape is {} bytes: its slots are inline again, and it is built on the stack",
        std::mem::size_of::<Tape>()
    );
}

/// The ledger is built by value and moved into the game's state, so it has
/// to stay a small fraction of the stack too.
#[test]
fn a_ledger_is_small() {
    assert!(
        std::mem::size_of::<Ledger>() <= 16 * 1024,
        "Ledger is {} bytes; box what grew",
        std::mem::size_of::<Ledger>()
    );
}
