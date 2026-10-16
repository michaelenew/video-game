//! The square root is worked out the short way and must agree with the long
//! way it always was (`Fx::sqrt`): a root a hair different is a different
//! distance, a different hit, a different checksum, and two machines that
//! disagree about it are a desync.
//!
//! The short way was checked against the long way for every one of the 2^31
//! inputs when it was written (2026-10-16), which takes minutes; this keeps a
//! wide sample of them, and every input near a perfect square -- the only
//! ones where the long way's answer depends on how many steps it took.

use sim::Fx;

#[test]
fn the_short_square_root_is_the_long_one() {
    let mut raw: i64 = 1;
    while raw <= i32::MAX as i64 {
        let x = Fx::from_raw(raw as i32);
        assert_eq!(x.sqrt(), x.sqrt_by_steps(), "sqrt of raw {raw}");
        raw += 1 + raw / 4096;
    }
    // Every raw whose scaled value sits within one of a perfect square.
    let mut m: i64 = 256;
    while m * m <= (i32::MAX as i64) << 16 {
        for n in [m * m - 1, m * m, m * m + 1] {
            let raw = n >> 16;
            if (1..=i32::MAX as i64).contains(&raw) {
                let x = Fx::from_raw(raw as i32);
                assert_eq!(x.sqrt(), x.sqrt_by_steps(), "sqrt of raw {raw}");
            }
        }
        m += 1 + m / 2048;
    }
    for raw in [0, -1, i32::MIN, i32::MAX, 1, 2, 3, 4, 65535, 65536, 65537] {
        let x = Fx::from_raw(raw);
        assert_eq!(x.sqrt(), x.sqrt_by_steps(), "sqrt of raw {raw}");
    }
}
