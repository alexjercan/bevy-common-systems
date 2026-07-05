//! Player input helpers.
//!
//! This module collects small, game-agnostic input building blocks:
//! - [`pointer`] - a unified [`pointer::UnifiedPointer`] resource that collapses
//!   mouse, touch and cursor into one per-frame abstraction.
//! - [`cursor`] - lock / release the mouse cursor for mouse-look.
//!
//! The `prelude` re-exports the commonly used types:
//!
//! ```rust
//! use bevy_common_systems::input::prelude::*;
//! ```

pub mod cursor;
pub mod pointer;
pub mod state;

/// Re-exports the commonly used input types for convenience.
///
/// ```rust
/// use bevy_common_systems::input::prelude::*;
/// ```
pub mod prelude {
    pub use super::{cursor::prelude::*, pointer::prelude::*, state::prelude::*};
}
