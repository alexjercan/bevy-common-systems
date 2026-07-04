//! Small `StandardMaterial` helpers.
//!
//! [`glowing_material`] builds the emissive-that-actually-blooms material every
//! game hand-writes for glowing objects (bullets, thruster flames, pickups). It
//! bakes in the footgun: an emissive `StandardMaterial` must NOT be `unlit`, or
//! Bevy skips the lighting pass where emissive is applied and the object never
//! blooms under `camera/post` bloom (see the AGENTS.md note and `bevy_pbr`'s
//! `render/pbr.wgsl`). Building through this helper leaves `unlit` at its `false`
//! default, so the glow is applied.
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! fn spawn_bullet(mut materials: ResMut<Assets<StandardMaterial>>) {
//!     // HDR emissive (values > 1.0) streaks under bloom.
//!     let mat = materials.add(glowing_material(
//!         Color::srgb(0.1, 0.3, 0.5),
//!         LinearRgba::rgb(1.0, 5.0, 8.0),
//!     ));
//!     let _ = mat;
//! }
//! ```
//!
//! Games that need extra fields can spread it:
//! `StandardMaterial { perceptual_roughness: 0.5, ..glowing_material(base, glow) }`.

use bevy::prelude::*;

pub mod prelude {
    pub use super::glowing_material;
}

/// A `StandardMaterial` with `base_color` and an `emissive` glow, left lit so
/// the emissive actually renders and blooms.
///
/// Use an HDR `emissive` (channel values above `1.0`) for a material that
/// streaks under `camera/post` bloom. Never wrap the result in `unlit: true`:
/// that skips the pass that applies emissive.
pub fn glowing_material(base_color: Color, emissive: LinearRgba) -> StandardMaterial {
    StandardMaterial {
        base_color,
        emissive,
        ..default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sets_base_and_emissive_and_stays_lit() {
        let base = Color::srgb(0.1, 0.3, 0.5);
        let glow = LinearRgba::rgb(1.0, 5.0, 8.0);
        let mat = glowing_material(base, glow);
        assert_eq!(mat.base_color, base);
        assert_eq!(mat.emissive, glow);
        // The whole point of the helper: never `unlit`, or the emissive would
        // not be applied and the material would not bloom.
        assert!(!mat.unlit);
    }
}
