//! Game-feel feedback effects.
//!
//! This module collects short-lived "juice" effects that give a hit or pickup a
//! visible, visceral kick:
//!
//! - `flash` - a material hit-flash: briefly override an entity's material
//!   emissive / base color and ease it back to the original.
//!
//! Import the commonly used types through the prelude:
//!
//! ```rust
//! use bevy_common_systems::feedback::prelude::*;
//! ```

pub mod flash;

/// Re-exports the commonly used feedback types and plugins.
///
/// ```rust
/// use bevy_common_systems::feedback::prelude::*;
/// ```
pub mod prelude {
    pub use super::flash::prelude::*;
}
