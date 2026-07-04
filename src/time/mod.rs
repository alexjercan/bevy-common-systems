//! Timing helpers.
//!
//! - [`cooldown`] - a [`cooldown::Cooldown`] countdown for fire gates and
//!   invulnerability windows.
//!
//! ```rust
//! use bevy_common_systems::time::prelude::*;
//! ```
//!
//! # Recipe: timed spawner
//!
//! A "spawn something every N seconds" cadence does not need a crate type: Bevy's
//! [`Timer`](bevy::time::Timer) in `Repeating` mode already is the primitive, and
//! a ramping cadence is just `set_duration`. A `Spawner { interval, jitter }`
//! component would be a thin wrapper -- and no example needs *temporal* jitter
//! (their spawn randomness is in position, not the interval), so it stays a
//! documented pattern rather than a module.
//!
//! ```rust
//! # use bevy::prelude::*;
//! #[derive(Resource)]
//! struct SpawnTimer(Timer);
//!
//! fn setup(mut commands: Commands) {
//!     commands.insert_resource(SpawnTimer(Timer::from_seconds(0.9, TimerMode::Repeating)));
//! }
//!
//! fn spawn_things(time: Res<Time>, mut timer: ResMut<SpawnTimer>) {
//!     if !timer.0.tick(time.delta()).just_finished() {
//!         return;
//!     }
//!     // Ramp the next interval (difficulty scaling) by setting a new duration.
//!     timer
//!         .0
//!         .set_duration(std::time::Duration::from_secs_f32(0.5));
//!     // ... spawn the game-specific entity ...
//! }
//! ```
//!
//! See `examples/06_fruitninja.rs` (`spawn_projectile`) for the worked version,
//! including the difficulty-ramped interval.

pub mod cooldown;

/// Re-exports the commonly used timing types for convenience.
///
/// ```rust
/// use bevy_common_systems::time::prelude::*;
/// ```
pub mod prelude {
    pub use super::cooldown::prelude::*;
}
