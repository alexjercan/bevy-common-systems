//! 16_direction - the `material/direction` shader materials as a direction+magnitude gauge.
//!
//! A direction vector spins slowly around the origin while its magnitude pulses. Two meshes
//! visualize it, each with one of the promoted materials:
//!
//! - a translucent sphere with [`DirectionSphereMaterial`]: the sphere is oriented so its
//!   local -Z faces the current direction, and the material brightens the fragments that face
//!   that way - a soft highlight that points where the vector points;
//! - a white cone with [`DirectionMagnitudeMaterial`]: oriented so its local +Y points along
//!   the direction, and the material pushes its tip out by the current magnitude - a needle
//!   whose length tracks "how much".
//!
//! The shaders are embedded by [`DirectionMaterialsPlugin`], so this example ships no wgsl.

use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use clap::Parser;

#[derive(Parser)]
#[command(name = "16_direction")]
#[command(version = "1.0.0")]
#[command(
    about = "A direction+magnitude gauge built from the direction shader materials.",
    long_about = None
)]
struct Cli;

/// Radius of the highlight sphere.
const SPHERE_RADIUS: f32 = 3.0;
/// How fast the direction spins, in radians per second.
const SPIN_SPEED: f32 = 0.7;

fn main() {
    let _ = Cli::parse();
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);

    #[cfg(feature = "debug")]
    app.add_plugins(InspectorDebugPlugin);

    app.add_plugins(bevy_enhanced_input::EnhancedInputPlugin);
    app.add_plugins(WASDCameraPlugin);
    app.add_plugins(WASDCameraControllerPlugin);

    // Registers both material types and embeds their shaders.
    app.add_plugins(DirectionMaterialsPlugin);

    app.add_systems(Startup, setup);
    app.add_systems(Update, drive_direction);

    app.run();
}

/// The sphere that shows the directional highlight.
#[derive(Component)]
struct HighlightSphere;

/// The cone whose needle length tracks the magnitude.
#[derive(Component)]
struct MagnitudeNeedle;

/// Handle to the cone's material so we can write `magnitude_input` each frame.
#[derive(Resource)]
struct NeedleMaterial(Handle<DirectionMagnitudeExt>);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sphere_materials: ResMut<Assets<DirectionSphereExt>>,
    mut needle_materials: ResMut<Assets<DirectionMagnitudeExt>>,
) {
    // The highlight sphere. A smooth octahedron sphere gives clean normals for the falloff.
    let sphere_mesh = meshes.add(TriangleMeshBuilder::new_octahedron(6).build());
    let sphere_material = sphere_materials.add(DirectionSphereExt {
        base: StandardMaterial {
            base_color: Color::srgba(0.1, 0.5, 1.0, 0.25),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            double_sided: true,
            perceptual_roughness: 1.0,
            ..default()
        },
        extension: DirectionSphereMaterial::default()
            .with_radius(SPHERE_RADIUS)
            .with_sharpness(8.0),
    });
    commands.spawn((
        Name::new("Highlight Sphere"),
        HighlightSphere,
        Mesh3d(sphere_mesh),
        MeshMaterial3d(sphere_material),
        Transform::from_scale(Vec3::splat(SPHERE_RADIUS)),
    ));

    // The magnitude needle: a cone that points along its local +Y, pushed out by the material.
    let needle_material = needle_materials.add(DirectionMagnitudeExt {
        base: StandardMaterial {
            base_color: Color::srgb(1.0, 0.9, 0.4),
            perceptual_roughness: 1.0,
            ..default()
        },
        extension: DirectionMagnitudeMaterial::default()
            .with_radius(0.4)
            .with_max_height(2.5),
    });
    commands.insert_resource(NeedleMaterial(needle_material.clone()));
    commands.spawn((
        Name::new("Magnitude Needle"),
        MagnitudeNeedle,
        Mesh3d(meshes.add(Cone::new(0.35, 0.2))),
        MeshMaterial3d(needle_material),
        Transform::default(),
    ));

    commands.spawn((
        Name::new("Main Camera"),
        Camera3d::default(),
        WASDCameraController,
        Transform::from_xyz(0.0, 3.0, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.5, 0.0)),
    ));
}

/// Spin the direction and pulse the magnitude, then point both meshes at it and feed the
/// needle its magnitude.
fn drive_direction(
    time: Res<Time>,
    needle_material: Res<NeedleMaterial>,
    mut needle_materials: ResMut<Assets<DirectionMagnitudeExt>>,
    mut q_sphere: Query<&mut Transform, (With<HighlightSphere>, Without<MagnitudeNeedle>)>,
    mut q_needle: Query<&mut Transform, (With<MagnitudeNeedle>, Without<HighlightSphere>)>,
) {
    let t = time.elapsed_secs();
    let dir = spin_direction(t * SPIN_SPEED);
    // Magnitude pulses 0..1.
    let magnitude = 0.5 + 0.5 * (t * 1.3).sin();

    // Orient the sphere so its local -Z faces the direction: the highlight points that way.
    for mut transform in &mut q_sphere {
        transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, dir);
    }

    // Orient the needle so its local +Y points along the direction, and feed it the magnitude.
    for mut transform in &mut q_needle {
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, dir);
    }
    if let Some(mut material) = needle_materials.get_mut(&needle_material.0) {
        material.extension.magnitude_input = magnitude;
    }
}

/// A unit direction spinning in the XZ plane at angle `angle` (radians), tilted slightly up so
/// the effect is visible from the default camera.
fn spin_direction(angle: f32) -> Vec3 {
    Vec3::new(angle.cos(), 0.35, angle.sin()).normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spin_direction_is_a_unit_vector() {
        for i in 0..8 {
            let angle = i as f32;
            assert!((spin_direction(angle).length() - 1.0).abs() < 1e-5);
        }
    }
}
