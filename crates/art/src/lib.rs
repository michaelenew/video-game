//! Art by arithmetic.
//!
//! The game has no artist and will not have one. Everything visible has to
//! come out of a formula, which sounds like a constraint and is mostly a
//! different set of trades: you give up the ability to draw one specific thing
//! and you get the ability to make a thousand variations of it for free.
//!
//! ## The scarce resource is decisions, not assets
//!
//! It is tempting to read "no artist" as "we need cheap textures". That is the
//! wrong reading, and taking it leads straight to a folder of noise shaders
//! that is exactly as unmanageable as a folder of PNGs. Procedural generation
//! makes variations free; it does nothing about the fact that **somebody still
//! has to decide which variation is right**, and on a small team that decision
//! cost is what actually runs out.
//!
//! So the whole crate is organised to spend as few decisions as possible:
//!
//! - **Few parameters.** A material with forty knobs is worse than one with
//!   eight when there is one person turning them. See [`surface::Surface`].
//! - **Parameters derived from things already decided.** Fire's colour comes
//!   from its temperature, not a colour picker ([`color::blackbody`]). A
//!   player's colour comes from their slot, not a preference
//!   ([`palette::PLAYERS`]).
//! - **Rules that can be tested instead of judged.** Whether two players are
//!   still distinguishable to a colour-blind viewer is a computation, so it is
//!   a test rather than an opinion someone has to re-form every time the
//!   palette moves.
//!
//! ## Art is look, not rules
//!
//! Nothing in this crate may ever reach the simulation. Tuning values are
//! folded into `World::checksum` so that two peers running different numbers
//! desync loudly and immediately, which is right for anything deciding what
//! *happens* and wrong for anything deciding what you *see*. Two people
//! playing each other have to be able to run different texture resolutions and
//! still agree on the fight.
//!
//! That is the same line the camera already sits on, and it is why this crate
//! is free to use floating point while `sim` is not.
//!
//! ## The two clocks
//!
//! A consequence of rollback that is easy to get backwards, and expensive when
//! you do.
//!
//! **Anything carrying information runs on the simulation's frame counter.**
//! How big the fire pillar is, how far into its startup a move is, whether a
//! hitbox is live. These are read by the player as facts about the fight, they
//! must be identical on both machines, and they must survive being
//! re-simulated after a rollback.
//!
//! **Anything that is only texture runs on the wall clock.** Flicker, drift,
//! shimmer, sparks. A rollback is one to eight frames, which is sixteen to a
//! hundred and thirty milliseconds; a flame whose flicker hiccups across that
//! window is imperceptible, and paying for it in the snapshot would be absurd.
//!
//! Get it the wrong way round and you do not get a crash, you get a **tell
//! that lies** — a flame that looks bigger than the thing that hits you. Which
//! is why the rule is stated here rather than left to be inferred.
//!
//! ## The map
//!
//! | Module | What it is |
//! | --- | --- |
//! | [`noise`] | Three field shapes, and why there are only three |
//! | [`color`] | Perceptual colour, and why fire's is physics |
//! | [`surface`] | One evaluator. A material is a point in its parameter space |
//! | [`materials`] | Every material in the game, as rows of numbers |
//! | [`palette`] | Who is who, and what will hurt you |
//! | [`sky`] | The whole lighting rig, from how high the sun is |
//! | [`stone`] | Rock as a **volume**: how it formed, how it broke, how it weathered |
//! | [`bake`] | Parameters to textures a renderer already eats |
//! | [`png`] | A dependency-free encoder, so a material can be *looked at* |

pub mod bake;
pub mod color;
pub mod materials;
pub mod noise;
pub mod palette;
pub mod png;
pub mod sky;
pub mod stone;
pub mod surface;
