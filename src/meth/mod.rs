//! Utilities for vector math and interpolation in 3D space.
//!
//! This module provides helper functions and traits for smooth value interpolation
//! (`LerpSnap`) and spherical coordinate conversions and operations (`sphere`).
//!
//! # Recipe: difficulty ramp over time
//!
//! Every example game ramps difficulty as a run goes on, but via genuinely
//! different idioms -- a continuous time-lerp (`06_fruitninja`), a discrete
//! per-interval `Level` (`07_orbit`, `11_overload`), a per-clear `Wave`
//! (`10_asteroids`), and log-scaled score tiers (`09_reactor`). Only the two
//! time-based ones share a core, and each of those is a one-liner, so they live
//! here as a recipe rather than a `progress` module: what each level or ramp
//! step *does* is game-specific and does not generalize.
//!
//! ## Continuous ramp
//!
//! Normalize elapsed time to `0..=1` over a fixed duration, then interpolate a
//! value from its start to its cap. Wrap the normalized `t` in a Bevy
//! `EaseFunction` sample (`ease.sample_clamped(t)`) for a curved ramp instead of
//! a linear one; the crate's future `tween` easing will wrap the same call.
//!
//! ```rust
//! // Normalized progress through a fixed-duration ramp, clamped to 0..=1.
//! fn ramp_t(elapsed: f32, duration: f32) -> f32 {
//!     (elapsed / duration).clamp(0.0, 1.0)
//! }
//! // Interpolate a value from `start` to `end` across the ramp (linear here;
//! // pass `t` through an `EaseFunction` for a curve).
//! fn ramp(elapsed: f32, duration: f32, start: f32, end: f32) -> f32 {
//!     start + (end - start) * ramp_t(elapsed, duration)
//! }
//!
//! assert_eq!(ramp_t(30.0, 60.0), 0.5);
//! // A spawn interval that speeds up from 1.2s to 0.5s over the first 60s:
//! assert!((ramp(0.0, 60.0, 1.2, 0.5) - 1.2).abs() < 1e-6);
//! assert!((ramp(60.0, 60.0, 1.2, 0.5) - 0.5).abs() < 1e-6);
//! ```
//!
//! ## Level-interval timer
//!
//! For discrete difficulty levels, derive a 1-based level from elapsed time, one
//! level per `period` seconds. Keep the current level in a resource and, each
//! frame, compare the freshly computed level against it: when it grows, that
//! frame is a "level up" -- the moment to ping a sound and rescale the game's
//! knobs (`07_orbit` does exactly this in its `advance_level` system).
//!
//! ```rust
//! // Discrete 1-based level from elapsed time.
//! fn level_at(elapsed: f32, period: f32) -> usize {
//!     1 + (elapsed / period).floor().max(0.0) as usize
//! }
//!
//! assert_eq!(level_at(0.0, 20.0), 1);
//! assert_eq!(level_at(19.9, 20.0), 1);
//! assert_eq!(level_at(20.0, 20.0), 2);
//! assert_eq!(level_at(60.0, 20.0), 4);
//!
//! // The level-up edge: recompute each frame, act when it climbs.
//! let mut level = 1usize;
//! let next = level_at(20.0, 20.0);
//! if next > level {
//!     level = next; // ... ping level-up sound, rescale hazards ...
//! }
//! assert_eq!(level, 2);
//! ```
//!
//! The event-driven `Wave` counter (`10_asteroids`, bumped on clearing a wave,
//! not on a clock) and the log-scaled score tiers (`09_reactor`
//! `tier_for_score`) are not time ramps and stay game-specific.

pub mod lerp;
pub mod sphere;

/// The prelude re-exports the most commonly used math utilities.
///
/// Use `bevy_common_systems::meth::prelude::*` to easily access `LerpSnap` for smooth
/// interpolation and all spherical math functions from `sphere`.
pub mod prelude {
    pub use super::{lerp::LerpSnap, sphere::*};
}
