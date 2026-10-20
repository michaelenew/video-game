//! **What the camera passes through, drawn see-through round the line to the
//! fighter.**
//!
//! The camera keeps out of large things only (`sim::arena::Solid::holds_the_camera`):
//! a wall, a crag, a terrace. Everything smaller -- a platform, a step, a
//! post, a branch, a trunk, and a grown giant's wood -- it passes through,
//! because a camera shoved in by every small thing behind it jumps as you
//! step on and off blocks (2026-10-20, from play). So those are drawn in a
//! paint with a window in it ([`Paint::window`]), cut along the line from the
//! eye to the fighter, and drawn a second time over the window as a faint
//! ghost of what was cut ([`Paint::ghost`]), so the window reads as the
//! camera's and not as a bite out of the thing.
//!
//! This holds the paints and moves the window every frame. A ghost is a
//! second draw, and blended, so each one is shown only while the line from
//! the eye to the fighter comes within the window's width of it ([`Ghost`]).

use bevy::prelude::*;

use crate::paint::Paint;

/// How wide the window is where it reaches the fighter, metres
/// (`Paint::cut`'s `w`), and half that by the camera: six metres across.
/// Two and a half until 2026-10-16, when seeing what was going on in the
/// Wood was a fight of its own; the forest is not meant to be one about
/// seeing. The ghost is thinnest in the middle and thickens to solid at its
/// edge.
pub const WIDTH: f32 = 3.0;

/// The paints with a window in them, and their ghosts: every one of them is
/// told where the window is each frame.
#[derive(Resource, Default)]
pub struct Windows {
    /// [`Paint::window`] and [`Paint::ghost`] of the white paint, for every
    /// mesh that carries its own colour.
    white: Option<(Handle<Paint>, Handle<Paint>)>,
    /// Every windowed paint and ghost there is.
    painted: Vec<Handle<Paint>>,
}

impl Windows {
    /// The white paint with a window in it, and its ghost.
    pub fn white(&mut self, paints: &mut Assets<Paint>) -> (Handle<Paint>, Handle<Paint>) {
        if let Some(w) = &self.white {
            return w.clone();
        }
        let w = self.of(paints, Paint::white());
        self.white = Some(w.clone());
        w
    }

    /// `base` with a window in it, and its ghost; told where the window is
    /// from now on.
    pub fn of(
        &mut self,
        paints: &mut Assets<Paint>,
        base: Paint,
    ) -> (Handle<Paint>, Handle<Paint>) {
        let solid = paints.add(base.clone().window());
        let ghost = paints.add(base.ghost());
        self.painted.push(solid.clone());
        self.painted.push(ghost.clone());
        (solid, ghost)
    }
}

/// **A ghost**, and the footprint of what it is the ghost of, flat, in the
/// map's coordinates (the picture's off the valley): shown only while the
/// line from the eye to the fighter passes within [`WIDTH`] of it.
#[derive(Component, Clone, Copy)]
pub struct Ghost {
    pub lo: Vec2,
    pub hi: Vec2,
}

impl Ghost {
    /// The footprint of a box, `lo` to `hi`.
    pub fn over(lo: Vec3, hi: Vec3) -> Ghost {
        Ghost {
            lo: lo.xz(),
            hi: hi.xz(),
        }
    }

    /// Does the flat segment `a`-`b` come within `pad` of the footprint:
    /// the slab test, against the footprint grown by `pad`.
    fn near(&self, a: Vec2, b: Vec2, pad: f32) -> bool {
        let (lo, hi) = (self.lo - pad, self.hi + pad);
        let d = b - a;
        let (mut t0, mut t1) = (0.0_f32, 1.0_f32);
        for i in 0..2 {
            if d[i].abs() < 1e-6 {
                if a[i] < lo[i] || a[i] > hi[i] {
                    return false;
                }
                continue;
            }
            let (u, v) = ((lo[i] - a[i]) / d[i], (hi[i] - a[i]) / d[i]);
            t0 = t0.max(u.min(v));
            t1 = t1.min(u.max(v));
        }
        t0 <= t1
    }
}

/// **Move the window** to the line from where the camera now is to the
/// fighter it follows, in every windowed paint and in the outline (which
/// draws no ring round it); show the ghosts that line comes near.
#[allow(clippy::too_many_arguments)] // A Bevy system: one argument per resource it reads.
pub fn follow(
    rig: Res<crate::Rig>,
    stream: Res<crate::stream::Stream>,
    windows: Res<Windows>,
    mut paints: ResMut<Assets<Paint>>,
    mut outlines: Query<&mut crate::outline::Outline>,
    camera: Query<&Transform, With<crate::MainCamera>>,
    mut ghosts: Query<(&Ghost, &mut Visibility)>,
) {
    let f = rig.0.focus();
    let cut = Vec3::new(f[0], f[1], f[2]).extend(WIDTH);
    for h in &windows.painted {
        if let Some(p) = paints.get_mut(h) {
            p.cut = cut;
        }
    }
    for mut o in &mut outlines {
        o.cut = cut;
    }
    let Ok(eye) = camera.single() else {
        return;
    };
    // The ghosts' footprints are the map's; the picture is the map moved by
    // its origin.
    let origin = stream.origin().unwrap_or(Vec3::ZERO);
    let (a, b) = (
        (eye.translation + origin).xz(),
        (cut.truncate() + origin).xz(),
    );
    for (g, mut v) in &mut ghosts {
        v.set_if_neq(if g.near(a, b, WIDTH) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ghost_is_near_a_line_that_crosses_or_passes_it() {
        let g = Ghost::over(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 1.0, 2.0));
        // Straight through.
        assert!(g.near(Vec2::new(-5.0, 1.0), Vec2::new(5.0, 1.0), 0.0));
        // Past it, inside the pad.
        assert!(g.near(Vec2::new(-5.0, 4.0), Vec2::new(5.0, 4.0), 3.0));
        // Past it, outside the pad.
        assert!(!g.near(Vec2::new(-5.0, 6.0), Vec2::new(5.0, 6.0), 3.0));
        // Short of it.
        assert!(!g.near(Vec2::new(-9.0, 1.0), Vec2::new(-6.0, 1.0), 3.0));
        // A line of no length, on it.
        assert!(g.near(Vec2::new(1.0, 1.0), Vec2::new(1.0, 1.0), 0.0));
    }
}
