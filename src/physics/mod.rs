//! Physics helpers built on avian3d.
//!
//! - [`pd_controller`] - a PD attitude controller that torques a rigid body
//!   toward a target rotation.
//!
//! The `prelude` re-exports the commonly used types:
//!
//! ```rust
//! use bevy_common_systems::physics::prelude::*;
//! ```
//!
//! # Recipe: radial ("point") gravity
//!
//! avian3d's global `Gravity` resource is a single uniform field, so pulling
//! bodies toward a point (a planet at the origin, a star) is not something the
//! engine ships as a component -- but it is a one-liner you apply yourself, so
//! it stays a documented recipe here rather than a module. Only one example
//! needs it (`examples/08_dropzone.rs`, the lunar lander), and it fuses gravity
//! with a wind term in the same acceleration channel -- which a fixed
//! `RadialGravity` component that owned that channel would only get in the way
//! of.
//!
//! The idiom has three parts:
//!
//! 1. Disable the global field so it does not fight the radial one:
//!    `app.insert_resource(Gravity(Vec3::ZERO));`
//! 2. Each frame, point an acceleration at the centre. For a centre at the
//!    origin the direction is just the body's position, normalized and negated,
//!    scaled by the gravity strength. This is the whole of the math, and it is
//!    unit-testable without a physics world:
//!
//! ```rust
//! # use bevy::prelude::*;
//! // Radial gravity acceleration pulling a body at `position` toward the origin.
//! fn radial_gravity(position: Vec3, strength: f32) -> Vec3 {
//!     -position.normalize_or(Vec3::Y) * strength
//! }
//!
//! // A body straight "above" the origin is pulled straight down toward it.
//! assert_eq!(radial_gravity(Vec3::new(0.0, 10.0, 0.0), 5.5), Vec3::new(0.0, -5.5, 0.0));
//! // `normalize_or` keeps a body exactly at the centre from producing a NaN.
//! assert_eq!(radial_gravity(Vec3::ZERO, 5.5), Vec3::new(0.0, -5.5, 0.0));
//! ```
//!
//! For a centre `c` other than the origin, aim at it instead:
//! `(c - position).normalize_or(Vec3::Y) * strength`.
//!
//! 3. Write that acceleration onto each body every frame through an avian force
//!    channel -- a `ConstantLinearAcceleration` (world-space and
//!    mass-independent, exactly like gravity) is the natural fit. Because it is
//!    a plain acceleration, other world-space accelerations (wind, an updraft)
//!    are just extra terms added into the same write:
//!
//! ```ignore
//! app.insert_resource(Gravity(Vec3::ZERO));
//!
//! fn apply_radial_gravity(
//!     wind: Res<Wind>,
//!     mut bodies: Query<(&Position, &mut ConstantLinearAcceleration)>,
//! ) {
//!     for (position, mut accel) in &mut bodies {
//!         accel.0 = -position.0.normalize_or(Vec3::Y) * GRAVITY + wind.accel;
//!     }
//! }
//! ```
//!
//! See `examples/08_dropzone.rs` (`apply_ship_forces`) for the worked version:
//! radial gravity plus wind on the lander, with thrust applied through a
//! separate local-space channel and torque from [`pd_controller`].

pub mod doom_controller;
pub mod pd_controller;

pub mod prelude {
    pub use super::{doom_controller::prelude::*, pd_controller::prelude::*};
}
