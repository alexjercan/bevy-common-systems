//! Screen <-> world projection helpers over a `Camera` + `GlobalTransform`.
//!
//! These are the two small pieces of camera-to-screen glue that every pointer-
//! driven or popup-anchoring game ends up hand-rolling:
//!
//! - [`pointer_on_plane`] casts a viewport-space pointer position into the world
//!   and intersects it with an infinite plane, giving the world point the cursor
//!   is "over". This is the pick used to turn a mouse/touch position into a
//!   gameplay position on a fixed play plane.
//! - [`world_to_screen`] projects a world position back to a viewport pixel
//!   position, returning `None` when the point is off-screen or behind the
//!   camera. This is the anchor used to float UI (a "+N" popup, a marker) over a
//!   world entity.
//!
//! Both are plain functions, not a plugin: they take the `Camera` and its
//! `GlobalTransform` (query them however you like) and do one projection. They
//! are thin wrappers over Bevy's `Camera::viewport_to_world` /
//! `Camera::world_to_viewport`, folding the fallible result into an `Option` and
//! (for [`pointer_on_plane`]) the plane intersection.
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! # fn demo(
//! #     q_camera: Query<(&Camera, &GlobalTransform)>,
//! #     q_target: Query<&GlobalTransform>,
//! #     pointer: Vec2,
//! # ) {
//! let (camera, camera_transform) = q_camera.single().unwrap();
//!
//! // Where is the pointer on the z = 0 play plane?
//! if let Some(world) = pointer_on_plane(
//!     camera,
//!     camera_transform,
//!     pointer,
//!     Vec3::ZERO,
//!     InfinitePlane3d::new(Vec3::Z),
//! ) {
//!     // ... spawn something at `world` ...
//! }
//!
//! // Where on screen is a world entity, if it is visible at all?
//! let target = q_target.iter().next().unwrap();
//! if let Some(screen) = world_to_screen(camera, camera_transform, target.translation()) {
//!     // ... anchor a popup at `screen` ...
//! }
//! # }
//! ```

use bevy::prelude::*;

pub mod prelude {
    pub use super::{pointer_on_plane, world_to_screen};
}

/// Cast a viewport-space pointer position into the world and intersect it with
/// an infinite plane.
///
/// `viewport_pos` is a pixel position in the camera's viewport (a cursor or
/// touch position). The camera turns it into a world-space ray; the ray is then
/// intersected with the plane defined by `plane_origin` (any point on the plane)
/// and `plane` (its orientation). Returns the world point of intersection, or
/// `None` if the pointer is outside the viewport or the ray is parallel to the
/// plane (never hits it).
///
/// Typical use is picking a gameplay position on a fixed play plane from the
/// pointer:
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_common_systems::prelude::*;
/// # fn demo(camera: &Camera, camera_transform: &GlobalTransform, pointer: Vec2) {
/// let world = pointer_on_plane(
///     camera,
///     camera_transform,
///     pointer,
///     Vec3::ZERO,
///     InfinitePlane3d::new(Vec3::Z),
/// );
/// # }
/// ```
pub fn pointer_on_plane(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    viewport_pos: Vec2,
    plane_origin: Vec3,
    plane: InfinitePlane3d,
) -> Option<Vec3> {
    let ray = camera
        .viewport_to_world(camera_transform, viewport_pos)
        .ok()?;
    let distance = ray.intersect_plane(plane_origin, plane)?;
    Some(ray.get_point(distance))
}

/// Project a world position to a viewport-space pixel position.
///
/// Returns the pixel position in the camera's viewport, or `None` when the point
/// is off-screen or behind the camera. Use it to anchor UI (a popup, a marker)
/// over a world entity, skipping the UI when the entity is not visible:
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_common_systems::prelude::*;
/// # fn demo(camera: &Camera, camera_transform: &GlobalTransform, world_pos: Vec3) {
/// if let Some(screen) = world_to_screen(camera, camera_transform, world_pos) {
///     // ... place a UI node at `screen` ...
/// }
/// # }
/// ```
pub fn world_to_screen(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    world_pos: Vec3,
) -> Option<Vec2> {
    camera.world_to_viewport(camera_transform, world_pos).ok()
}
