//! Two `StandardMaterial` extensions for visualizing a direction vector, plus the plugin
//! that registers them.
//!
//! These are the shader materials behind a "which way / how much" indicator (a velocity
//! arrow, a thrust vector, a wind gauge):
//!
//! - [`DirectionMagnitudeMaterial`] displaces a mesh's vertices upward (+Y in local space) by
//!   an amount that peaks at the local origin and falls off to zero past `radius`, scaled by
//!   `magnitude_input`. On a cone it reads as a needle whose height tracks a magnitude.
//! - [`DirectionSphereMaterial`] tints a mesh by how closely each fragment's normal faces the
//!   mesh's local -Z, raised to `sharpness`. On a sphere it reads as a soft highlight pointing
//!   along the object's forward axis - orient the mesh toward a direction and the highlight
//!   points that way.
//!
//! Both are [`ExtendedMaterial`] extensions over [`StandardMaterial`], so they keep normal PBR
//! shading (base color, alpha blend, textures) and only add their effect. The game drives them
//! by orienting the mesh (e.g. with [`transform`](crate::transform)) and, for the magnitude
//! material, writing `magnitude_input` each frame.
//!
//! Add [`DirectionMaterialsPlugin`] to register both material types and embed their shaders
//! (no wgsl files to ship - they are compiled into the binary). See `examples/16_direction`.

use bevy::{
    asset::embedded_asset,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

pub mod prelude {
    pub use super::{
        DirectionMagnitudeExt, DirectionMagnitudeMaterial, DirectionMaterialsPlugin,
        DirectionSphereExt, DirectionSphereMaterial,
    };
}

/// The full material type to store and mutate: a [`StandardMaterial`] extended with
/// [`DirectionMagnitudeMaterial`]. Spawn it as `MeshMaterial3d<DirectionMagnitudeExt>`.
pub type DirectionMagnitudeExt = ExtendedMaterial<StandardMaterial, DirectionMagnitudeMaterial>;

/// The full material type to store and mutate: a [`StandardMaterial`] extended with
/// [`DirectionSphereMaterial`]. Spawn it as `MeshMaterial3d<DirectionSphereExt>`.
pub type DirectionSphereExt = ExtendedMaterial<StandardMaterial, DirectionSphereMaterial>;

/// Displaces vertices upward (local +Y) by `magnitude_input`, peaking at the local origin and
/// falling off to zero past `radius`, clamped to `max_height`. Drive `magnitude_input` each
/// frame to animate the height.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct DirectionMagnitudeMaterial {
    /// The current magnitude; scales the vertex displacement. Write this each frame.
    #[uniform(100)]
    pub magnitude_input: f32,
    /// Radius (in local space) over which the displacement falls off to zero.
    #[uniform(100)]
    pub radius: f32,
    /// Maximum displacement height, so a large magnitude does not shoot off unbounded.
    #[uniform(100)]
    pub max_height: f32,
    #[cfg(target_arch = "wasm32")]
    #[uniform(100)]
    _webgl2_padding_16b: u32,
}

impl DirectionMagnitudeMaterial {
    /// Set the falloff [`radius`](Self::radius).
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Set the [`max_height`](Self::max_height) clamp.
    pub fn with_max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }
}

impl MaterialExtension for DirectionMagnitudeMaterial {
    fn vertex_shader() -> ShaderRef {
        "embedded://bevy_common_systems/material/shaders/directional_magnitude.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_common_systems/material/shaders/directional_magnitude.wgsl".into()
    }
}

/// Tints a mesh by `dot(normal, local_forward)^sharpness`, where `local_forward` is the mesh's
/// local -Z in world space: fragments facing the object's forward axis stay bright, the rest
/// fade out. Orient the mesh toward a direction to point the highlight that way.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct DirectionSphereMaterial {
    /// Nominal radius of the sphere the material is applied to (kept for parity with the
    /// magnitude material and future use by the shader).
    #[uniform(100)]
    pub radius: f32,
    /// Falloff exponent: higher values give a tighter, sharper highlight.
    #[uniform(100)]
    pub sharpness: f32,
    #[cfg(target_arch = "wasm32")]
    #[uniform(100)]
    _webgl2_padding_16b1: u32,
    #[cfg(target_arch = "wasm32")]
    #[uniform(100)]
    _webgl2_padding_16b2: u32,
}

impl DirectionSphereMaterial {
    /// Set the [`radius`](Self::radius).
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Set the [`sharpness`](Self::sharpness) falloff exponent.
    pub fn with_sharpness(mut self, sharpness: f32) -> Self {
        self.sharpness = sharpness;
        self
    }
}

impl MaterialExtension for DirectionSphereMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_common_systems/material/shaders/directional_sphere.wgsl".into()
    }
}

/// Registers [`DirectionMagnitudeMaterial`] and [`DirectionSphereMaterial`] (as
/// [`ExtendedMaterial`]s over [`StandardMaterial`]) and embeds their shaders into the binary.
#[derive(Default)]
pub struct DirectionMaterialsPlugin;

impl Plugin for DirectionMaterialsPlugin {
    fn build(&self, app: &mut App) {
        debug!("DirectionMaterialsPlugin: build");

        // Compile the shaders into the binary so consumers ship no wgsl files.
        embedded_asset!(app, "shaders/directional_magnitude.wgsl");
        embedded_asset!(app, "shaders/directional_sphere.wgsl");

        app.add_plugins(MaterialPlugin::<DirectionMagnitudeExt>::default());
        app.add_plugins(MaterialPlugin::<DirectionSphereExt>::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magnitude_builders_set_fields() {
        let m = DirectionMagnitudeMaterial::default()
            .with_radius(0.2)
            .with_max_height(1.0);
        assert_eq!(m.radius, 0.2);
        assert_eq!(m.max_height, 1.0);
        assert_eq!(m.magnitude_input, 0.0);
    }

    #[test]
    fn sphere_builders_set_fields() {
        let m = DirectionSphereMaterial::default()
            .with_radius(5.0)
            .with_sharpness(10.0);
        assert_eq!(m.radius, 5.0);
        assert_eq!(m.sharpness, 10.0);
    }
}
