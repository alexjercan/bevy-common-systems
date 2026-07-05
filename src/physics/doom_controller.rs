//! A Doom-style first-person character controller: mouse-look with a pitch clamp
//! plus WASD-style planar movement for a physics-body character.
//!
//! Named `Doom` on purpose. This is the pragmatic, arena-shooter controller
//! harvested from [`examples/14_breach`] -- flat ground, no jump / crouch / air
//! control / slope handling. The premium `FirstPersonController` / `FpsController`
//! name is deliberately reserved for a more capable controller built later; reach
//! for this when you want a Quake/Doom-simple first-person mover and nothing more.
//!
//! Like the crate's other body drivers ([`pd_controller`](crate::physics::pd_controller),
//! the [`transform`](crate::transform) family) it is **output-only** and takes no
//! physics dependency: it computes a desired horizontal velocity into
//! [`DoomControllerOutput`], and your game writes that into the body's velocity
//! (e.g. avian's `LinearVelocity`), leaving the vertical component to gravity so the
//! solver does collide-and-slide against the level. It also owns the *look*: it
//! integrates a per-frame look delta ([`DoomControllerInput::look`]) into yaw/pitch
//! (clamping pitch), stores them in [`DoomControllerState`], and writes the rotation
//! onto a [`DoomEye`]-marked camera child.
//!
//! The body itself must keep its rotation locked (e.g. `LockedAxes::ROTATION_LOCKED`)
//! and stay axis-aligned: yaw lives in the controller state and drives the movement
//! direction, while the eye child carries the full view rotation. This keeps the
//! physics solver from fighting the camera.
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! # fn spawn_player(mut commands: Commands) {
//! // The body carries the controller; a DoomEye camera child carries the view.
//! commands
//!     .spawn((
//!         DoomController::default(),
//!         Transform::default(),
//!         // ... your physics body here: RigidBody::Dynamic, a capsule Collider,
//!         // and LockedAxes::ROTATION_LOCKED so only the eye rotates.
//!     ))
//!     .with_children(|parent| {
//!         parent.spawn((Camera3d::default(), DoomEye, Transform::from_xyz(0.0, 0.6, 0.0)));
//!     });
//! // Each frame: write DoomControllerInput.look/movement from your input BEFORE the
//! // DoomControllerSystems::Drive set, and copy DoomControllerOutput.velocity into
//! // the body's LinearVelocity (keeping .y) AFTER it.
//! # }
//! ```

use bevy::prelude::*;

pub mod prelude {
    pub use super::{
        doom_move_dir, DoomController, DoomControllerInput, DoomControllerOutput,
        DoomControllerPlugin, DoomControllerState, DoomControllerSystems, DoomEye,
    };
}

/// Configuration for a Doom-style first-person controller. Put it on the physics
/// body; a [`DoomEye`]-marked camera child carries the view. Pulls in
/// [`DoomControllerInput`] / [`DoomControllerState`] / [`DoomControllerOutput`].
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
#[require(DoomControllerInput, DoomControllerState, DoomControllerOutput)]
pub struct DoomController {
    /// Planar move speed, in world units per second.
    pub move_speed: f32,
    /// Radians of view rotation per unit of look delta (mouse pixel / stick).
    pub look_sensitivity: f32,
    /// Lowest allowed pitch, in radians (look-down limit; negative).
    pub pitch_min: f32,
    /// Highest allowed pitch, in radians (look-up limit).
    pub pitch_max: f32,
}

impl Default for DoomController {
    fn default() -> Self {
        Self {
            move_speed: 6.0,
            look_sensitivity: 0.0022,
            // Just under +/- 90deg, so the view cannot flip over the pole.
            pitch_min: -1.54,
            pitch_max: 1.54,
        }
    }
}

/// Per-frame input, written by game code. `look` is a raw look delta (mouse motion
/// or a look stick); `movement` is a `(strafe, forward)` intent (`+x` = right,
/// `+y` = forward), typically in `-1..=1`.
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct DoomControllerInput {
    /// Look delta this frame (x = yaw, y = pitch).
    pub look: Vec2,
    /// Movement intent (x = strafe, y = forward).
    pub movement: Vec2,
}

/// The controller's accumulated orientation. Public so a game can set it directly
/// (to face a direction on spawn, or aim under a test/AI); the plugin integrates
/// [`DoomControllerInput::look`] into it each frame and clamps pitch.
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct DoomControllerState {
    /// Yaw (about world +Y), in radians. Drives both the view and the move basis.
    pub yaw: f32,
    /// Pitch (look up/down), in radians, clamped to the config's range.
    pub pitch: f32,
}

/// The controller's output: the desired horizontal velocity (`y` is always 0). The
/// game writes this into the body's velocity, leaving the vertical component to
/// gravity so the physics solver resolves walls/floor.
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct DoomControllerOutput {
    /// Desired planar velocity (world units per second, `y == 0`).
    pub velocity: Vec3,
}

/// Marker for the eye camera: a child of a [`DoomController`] body. The plugin
/// writes its local rotation from the parent controller's yaw/pitch each frame.
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct DoomEye;

/// Turn a `(strafe, forward)` intent into a world-space, ground-plane move
/// direction for a given yaw. `+forward` is `-Z` at yaw 0, `+strafe` is `+X`; the
/// result is yaw-rotated, flattened to the ground plane and normalized (zero input
/// yields zero).
pub fn doom_move_dir(yaw: f32, movement: Vec2) -> Vec3 {
    let local = Vec3::new(movement.x, 0.0, -movement.y);
    let world = Quat::from_rotation_y(yaw) * local;
    Vec3::new(world.x, 0.0, world.z).normalize_or_zero()
}

/// System sets for [`DoomControllerPlugin`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DoomControllerSystems {
    /// Integrates look into [`DoomControllerState`], writes the [`DoomEye`] child's
    /// rotation, and sets [`DoomControllerOutput::velocity`]. Runs in `Update`. Order
    /// your systems around it: write [`DoomControllerInput`] **before** this set
    /// (`.before(DoomControllerSystems::Drive)`) or the controller reads last frame's
    /// input, and apply [`DoomControllerOutput::velocity`] **after** it
    /// (`.after(DoomControllerSystems::Drive)`).
    Drive,
}

/// Drives [`DoomController`] entities: mouse-look (with pitch clamp) onto the eye
/// child, and a planar velocity output. Output-only -- add this, then copy
/// [`DoomControllerOutput::velocity`] into your body's velocity yourself.
pub struct DoomControllerPlugin;

impl Plugin for DoomControllerPlugin {
    fn build(&self, app: &mut App) {
        debug!("DoomControllerPlugin: build");

        app.register_type::<DoomController>()
            .register_type::<DoomControllerInput>()
            .register_type::<DoomControllerState>()
            .register_type::<DoomControllerOutput>()
            .register_type::<DoomEye>()
            .add_systems(
                Update,
                (drive_controller, orient_eye)
                    .chain()
                    .in_set(DoomControllerSystems::Drive),
            );
    }
}

fn drive_controller(
    mut q: Query<(
        &DoomController,
        &DoomControllerInput,
        &mut DoomControllerState,
        &mut DoomControllerOutput,
    )>,
) {
    for (cfg, input, mut state, mut output) in &mut q {
        trace!(
            "drive_controller: look {:?} move {:?}",
            input.look,
            input.movement
        );
        state.yaw -= input.look.x * cfg.look_sensitivity;
        state.pitch =
            (state.pitch - input.look.y * cfg.look_sensitivity).clamp(cfg.pitch_min, cfg.pitch_max);
        output.velocity = doom_move_dir(state.yaw, input.movement) * cfg.move_speed;
    }
}

fn orient_eye(
    controllers: Query<&DoomControllerState>,
    mut eyes: Query<(&ChildOf, &mut Transform), With<DoomEye>>,
) {
    for (child_of, mut transform) in &mut eyes {
        if let Ok(state) = controllers.get(child_of.parent()) {
            transform.rotation = Quat::from_euler(EulerRot::YXZ, state.yaw, state.pitch, 0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1e-4
    }

    #[test]
    fn forward_is_negative_z_at_zero_yaw() {
        assert!(approx(
            doom_move_dir(0.0, Vec2::new(0.0, 1.0)),
            Vec3::new(0.0, 0.0, -1.0)
        ));
    }

    #[test]
    fn strafe_is_positive_x_at_zero_yaw() {
        assert!(approx(
            doom_move_dir(0.0, Vec2::new(1.0, 0.0)),
            Vec3::new(1.0, 0.0, 0.0)
        ));
    }

    #[test]
    fn yaw_rotates_forward() {
        let d = doom_move_dir(std::f32::consts::FRAC_PI_2, Vec2::new(0.0, 1.0));
        assert!(approx(d, Vec3::new(-1.0, 0.0, 0.0)));
    }

    #[test]
    fn no_input_is_zero() {
        assert_eq!(doom_move_dir(1.2, Vec2::ZERO), Vec3::ZERO);
    }

    #[test]
    fn drive_integrates_look_and_outputs_velocity() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DoomControllerPlugin));
        let body = app
            .world_mut()
            .spawn(DoomController {
                move_speed: 10.0,
                look_sensitivity: 0.01,
                pitch_min: -1.0,
                pitch_max: 1.0,
            })
            .id();
        {
            let mut input = app
                .world_mut()
                .get_mut::<DoomControllerInput>(body)
                .expect("require inserts the input");
            input.look = Vec2::new(50.0, 0.0);
            input.movement = Vec2::new(0.0, 1.0);
        }
        app.update();

        let state = app.world().get::<DoomControllerState>(body).unwrap();
        // yaw -= 50 * 0.01 = -0.5
        assert!((state.yaw - (-0.5)).abs() < 1e-4);

        let out = app.world().get::<DoomControllerOutput>(body).unwrap();
        let expected = doom_move_dir(-0.5, Vec2::new(0.0, 1.0)) * 10.0;
        assert!((out.velocity - expected).length() < 1e-3);
    }

    #[test]
    fn pitch_is_clamped() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DoomControllerPlugin));
        let body = app
            .world_mut()
            .spawn(DoomController {
                move_speed: 1.0,
                look_sensitivity: 0.1,
                pitch_min: -0.3,
                pitch_max: 0.3,
            })
            .id();
        {
            let mut input = app
                .world_mut()
                .get_mut::<DoomControllerInput>(body)
                .unwrap();
            // pitch -= look.y * 0.1; a big negative look.y drives pitch up past the cap.
            input.look = Vec2::new(0.0, -100.0);
        }
        app.update();
        let state = app.world().get::<DoomControllerState>(body).unwrap();
        assert_eq!(state.pitch, 0.3);
    }

    #[test]
    fn orient_eye_writes_each_eye_from_its_own_controller() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DoomControllerPlugin));

        // Two independent body+eye pairs with different orientations.
        let spawn_pair = |app: &mut App, yaw: f32, pitch: f32| -> Entity {
            let body = app.world_mut().spawn(DoomController::default()).id();
            {
                let mut state = app
                    .world_mut()
                    .get_mut::<DoomControllerState>(body)
                    .unwrap();
                state.yaw = yaw;
                state.pitch = pitch;
            }
            let eye = app.world_mut().spawn((DoomEye, Transform::default())).id();
            app.world_mut().entity_mut(body).add_child(eye);
            eye
        };
        let eye_a = spawn_pair(&mut app, 0.5, 0.2);
        let eye_b = spawn_pair(&mut app, -1.0, -0.3);

        app.update();

        let expect = |yaw, pitch| Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        let rot_a = app.world().get::<Transform>(eye_a).unwrap().rotation;
        let rot_b = app.world().get::<Transform>(eye_b).unwrap().rotation;
        // Each eye takes ITS OWN controller's orientation, not the other's.
        assert!(rot_a.angle_between(expect(0.5, 0.2)) < 1e-4);
        assert!(rot_b.angle_between(expect(-1.0, -0.3)) < 1e-4);
    }
}
