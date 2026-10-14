//! **A set of small indices**: one bit per fighter, stone, critter or effect
//! slot, as many words wide as the thing it counts.
//!
//! Masks used to be whatever integer happened to be big enough on the day --
//! a `u8` of stones while there were six, a bit of seat while there were two
//! fighters -- and each one silently became wrong the day the thing it
//! counted grew past it: two indices sharing a bit, so a disc that had
//! shoved one stone skipped another, or a mage in seat two healed seat zero.
//! A mask sized from the capacity it counts cannot fall behind it:
//!
//! ```ignore
//! pub type Stones = Bits<{ bits::words_for(MAX_STONES) }>;
//! ```
//!
//! Plain words, so it is `Copy`, hashes word by word and costs the snapshot
//! nothing beyond its bits.

/// How many 32-bit words hold `bits` bits.
pub const fn words_for(bits: usize) -> usize {
    bits.div_ceil(32)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Bits<const WORDS: usize>(pub [u32; WORDS]);

impl<const WORDS: usize> Default for Bits<WORDS> {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl<const WORDS: usize> Bits<WORDS> {
    pub const EMPTY: Self = Bits([0; WORDS]);
    /// How many indices it holds.
    pub const CAPACITY: usize = WORDS * 32;

    /// Is `i` in the set? An index past the end never is.
    pub const fn has(&self, i: usize) -> bool {
        i < Self::CAPACITY && (self.0[i / 32] >> (i % 32)) & 1 != 0
    }

    /// Put `i` in the set. An index past the end is a mask sized for less
    /// than it counts, which is the bug this type exists to rule out.
    pub fn set(&mut self, i: usize) {
        debug_assert!(
            i < Self::CAPACITY,
            "index {i} past a {}-bit mask",
            Self::CAPACITY
        );
        if i < Self::CAPACITY {
            self.0[i / 32] |= 1 << (i % 32);
        }
    }

    /// Take `i` out of the set.
    pub fn unset(&mut self, i: usize) {
        if i < Self::CAPACITY {
            self.0[i / 32] &= !(1 << (i % 32));
        }
    }

    /// Put `i` in, and say whether it was new.
    pub fn insert(&mut self, i: usize) -> bool {
        let new = !self.has(i);
        self.set(i);
        new
    }

    /// Empty it.
    pub fn clear(&mut self) {
        *self = Self::EMPTY;
    }

    /// Anything at all.
    pub fn any(&self) -> bool {
        self.0.iter().any(|&w| w != 0)
    }

    /// How many are in it.
    pub fn count(&self) -> u32 {
        self.0.iter().map(|w| w.count_ones()).sum()
    }

    /// The words, for the checksum.
    pub fn words(&self) -> &[u32; WORDS] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_index_has_its_own_bit() {
        let mut b = Bits::<{ words_for(70) }>::EMPTY;
        assert_eq!(Bits::<{ words_for(70) }>::CAPACITY, 96);
        for i in (0..70).step_by(3) {
            b.set(i);
        }
        for i in 0..70 {
            assert_eq!(b.has(i), i % 3 == 0, "{i}");
        }
        assert_eq!(b.count(), 24);
        b.unset(33);
        assert!(!b.has(33));
        assert!(!b.has(1000));
    }
}
