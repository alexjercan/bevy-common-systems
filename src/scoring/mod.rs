//! Scoring helpers.
//!
//! Small, game-agnostic building blocks for scoring. Currently just the decay
//! bookkeeping behind combo/streak mechanics:
//! - [`streak`] - a [`streak::Streak`] counter that grows on each hit and decays
//!   when the player goes quiet.
//!
//! There is deliberately no `Score` type here: a running score is a bare
//! `usize`/`f32` the game already owns, and what a hit is worth is game-specific.
//! This module owns only the parts with real, re-derived logic.
//!
//! ```rust
//! use bevy_common_systems::scoring::prelude::*;
//! ```

pub mod streak;

/// Re-exports the commonly used scoring types for convenience.
///
/// ```rust
/// use bevy_common_systems::scoring::prelude::*;
/// ```
pub mod prelude {
    pub use super::streak::prelude::*;
}
