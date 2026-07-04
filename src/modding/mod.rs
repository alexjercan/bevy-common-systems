//! Top-level game events module.

pub mod events;
pub mod registry;

/// Re-export commonly used items from the `events` and `registry` modules.
pub mod prelude {
    pub use super::{events::prelude::*, registry::prelude::*};
}
