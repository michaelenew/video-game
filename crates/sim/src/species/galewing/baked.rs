use super::{CLIP_COUNT, bones};
pub const CHANNELS: usize = crate::beast::channels(bones::COUNT);
pub const ROWS: usize = 0;
pub const SPAN: [(u16, u16); CLIP_COUNT] = [(0, 0); CLIP_COUNT];
pub static FRAMES: [[i32; CHANNELS]; ROWS] = [];
