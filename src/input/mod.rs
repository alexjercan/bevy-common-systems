//! Player input helpers.
//!
//! This module collects small, game-agnostic input building blocks:
//! - [`pointer`] - a unified [`pointer::UnifiedPointer`] resource that collapses
//!   mouse, touch and cursor into one per-frame abstraction.
//!
//! The `prelude` re-exports the commonly used types:
//!
//! ```rust
//! use bevy_common_systems::input::prelude::*;
//! ```

pub mod pointer;

/// Re-exports the commonly used input types for convenience.
///
/// ```rust
/// use bevy_common_systems::input::prelude::*;
/// ```
pub mod prelude {
    pub use super::pointer::prelude::*;
}
