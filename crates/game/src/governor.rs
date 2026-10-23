//! **The governor**: the picture gives way before the controls do.
//!
//! A painted look's brush (`wash.wgsl` `smear`) is worked out per pixel,
//! and what a machine can afford of it is not known until it runs. So the
//! brush is drawn at a share of the screen's resolution, and this watches
//! the frame: half a second of frames running over a sixtieth of a second
//! and the share steps down; a few seconds clear and it tries a step up,
//! waiting twice as long before the next try each time a step up has to be
//! taken back. A slow machine's painting is laid with a coarser brush;
//! its frame stays a sixtieth.
//!
//! The median of the window, not the mean, so one hitch (a pipeline, a
//! tile streaming in) is not read as a machine that cannot keep up.
//!
//! `--smear <share>` fixes the share and the governor stays out of it.

use bevy::prelude::*;

/// The shares of the screen's resolution the brush may be drawn at, each
/// costing about half the one before (the cost goes with the area).
pub const STEPS: [f32; 5] = [1.0, 0.75, 0.5, 0.35, 0.25];

/// A frame is a sixtieth of a second, whatever the display could show.
const FRAME: f32 = 1.0 / 60.0;
/// A window's median past this is running over: a vsynced frame that
/// misses comes in at two sixtieths, never a little over one.
const OVER: f32 = FRAME * 1.08;
/// And under this it is clear.
const CLEAR: f32 = FRAME * 1.04;
/// A judgement is made every half second, over the frames in it -- at most
/// this many, so a display faster than sixty is judged as often.
/// And over at least so many, so one long hitch is never a window alone.
const SPAN: f32 = 0.5;
const WINDOW: usize = 64;
const FEWEST: usize = 5;
/// How long clear before a step up is tried, at first; and the most it
/// waits after steps up that had to be taken back.
const FIRST_WAIT: f32 = 3.0;
const LONGEST_WAIT: f32 = 64.0;
/// A step down this soon after a step up is that step up taken back.
const TAKEN_BACK: f32 = 2.0;

#[derive(Resource, Debug, Clone)]
pub struct Governor {
    step: usize,
    fixed: Option<f32>,
    frames: [f32; WINDOW],
    seen: usize,
    clear_for: f32,
    wait: f32,
    since_up: f32,
}

impl Default for Governor {
    fn default() -> Self {
        let fixed = crate::platform::value("--smear")
            .and_then(|v| v.parse::<f32>().ok())
            .map(|v| v.clamp(0.1, 1.0));
        Governor::new(fixed)
    }
}

impl Governor {
    pub fn new(fixed: Option<f32>) -> Self {
        Governor {
            step: 0,
            fixed,
            frames: [0.0; WINDOW],
            seen: 0,
            clear_for: 0.0,
            wait: FIRST_WAIT,
            since_up: f32::MAX,
        }
    }

    /// The share of the screen's resolution the brush is drawn at.
    pub fn scale(&self) -> f32 {
        self.fixed.unwrap_or(STEPS[self.step])
    }

    /// One frame's length, in seconds. True if the share changed.
    pub fn frame(&mut self, dt: f32) -> bool {
        if self.fixed.is_some() {
            return false;
        }
        self.since_up += dt;
        self.frames[self.seen] = dt;
        self.seen += 1;
        let spanned: f32 = self.frames[..self.seen].iter().sum();
        if (spanned < SPAN || self.seen < FEWEST) && self.seen < WINDOW {
            return false;
        }
        let mut sorted = self.frames;
        let sorted = &mut sorted[..self.seen];
        sorted.sort_by(f32::total_cmp);
        let median = sorted[self.seen / 2];
        self.seen = 0;
        if median > OVER {
            self.clear_for = 0.0;
            if self.since_up < TAKEN_BACK {
                self.wait = (self.wait * 2.0).min(LONGEST_WAIT);
            }
            self.since_up = f32::MAX;
            if self.step + 1 < STEPS.len() {
                self.step += 1;
                return true;
            }
        } else if median < CLEAR {
            self.clear_for += spanned;
            if self.clear_for >= self.wait && self.step > 0 {
                self.clear_for = 0.0;
                self.step -= 1;
                self.since_up = 0.0;
                return true;
            }
        } else {
            self.clear_for = 0.0;
        }
        false
    }
}

/// Every frame, in a painted look: the frame's real length to the governor.
pub fn govern(time: Res<Time<Real>>, mut governor: ResMut<Governor>) {
    if governor.frame(time.delta_secs()) && crate::platform::flag("--profile") {
        crate::platform::say(&format!(
            "brush drawn at {:.2} of the screen",
            governor.scale()
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(g: &mut Governor, dt: f32, seconds: f32) {
        for _ in 0..(seconds / dt) as usize {
            g.frame(dt);
        }
    }

    #[test]
    fn a_machine_that_keeps_up_keeps_the_whole_brush() {
        let mut g = Governor::new(None);
        run(&mut g, FRAME, 30.0);
        assert_eq!(g.scale(), 1.0);
    }

    #[test]
    fn running_over_steps_down_within_a_second_and_bottoms_out() {
        let mut g = Governor::new(None);
        run(&mut g, FRAME * 2.0, 1.1);
        assert!(g.scale() < 1.0);
        run(&mut g, FRAME * 2.0, 10.0);
        assert_eq!(g.scale(), *STEPS.last().unwrap());
    }

    #[test]
    fn a_hitch_is_not_a_slow_machine() {
        let mut g = Governor::new(None);
        for i in 0..600 {
            g.frame(if i % 10 == 0 { 0.2 } else { FRAME });
        }
        assert_eq!(g.scale(), 1.0);
    }

    #[test]
    fn a_long_hitch_is_not_a_slow_machine() {
        let mut g = Governor::new(None);
        for i in 0..1200 {
            g.frame(if i % 300 == 0 { 1.5 } else { FRAME });
        }
        assert_eq!(g.scale(), 1.0);
    }

    #[test]
    fn a_step_up_taken_back_waits_longer_next_time() {
        // A machine that keeps up at the second step and not the first.
        let mut g = Governor::new(None);
        let mut ups = Vec::new();
        let mut t = 0.0;
        while t < 120.0 {
            let dt = if g.step == 0 { FRAME * 2.0 } else { FRAME };
            let before = g.step;
            g.frame(dt);
            if g.step < before {
                ups.push(t);
            }
            t += dt;
        }
        assert!(ups.len() >= 3, "{ups:?}");
        let gaps: Vec<f32> = ups.windows(2).map(|w| w[1] - w[0]).collect();
        assert!(gaps.windows(2).all(|w| w[1] > w[0] * 1.5), "{gaps:?}");
        assert!(g.scale() < 1.0 || g.step == 0);
    }

    #[test]
    fn a_fixed_share_is_left_alone() {
        let mut g = Governor::new(Some(0.5));
        run(&mut g, FRAME * 3.0, 10.0);
        assert_eq!(g.scale(), 0.5);
    }
}
