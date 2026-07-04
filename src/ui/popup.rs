//! A Bevy plugin for short-lived floating "+N" popup text.
//!
//! ## Overview
//!
//! `PopupPlugin` animates a screen-space UI text label that rises up the screen
//! and fades out over a lifetime, then despawns itself -- the floating "+N"
//! score / "+FUEL" pickup / "-10" damage text that almost every game shows on a
//! pickup or a hit.
//!
//! The popup is a plain Bevy UI `Text` node with a [`Popup`] component; the
//! plugin advances it. Because it is screen-space, the caller decides *where* on
//! screen it appears: for a world event, project the world position to a
//! viewport point yourself (see [`camera::project::world_to_screen`]) and pass
//! that in; for a fixed banner, pass a constant screen point. Keeping the
//! projection in the caller is deliberate -- it varies per game, so the module
//! stays free of a camera handle.
//!
//! [`camera::project::world_to_screen`]: crate::camera::project::world_to_screen
//!
//! [`Popup`] is the config (lifetime, rise speed, base color); inserting it
//! attaches a private `Tween<f32>` that owns the fade timing and the despawn.
//!
//! Spawn the common case with the [`popup`] bundle builder; for a custom layout
//! (a centered banner, a different rise) put a [`Popup`] on your own `Text` /
//! `Node` instead.
//!
//! ## Usage
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! # fn on_pickup(mut commands: Commands, viewport_pos: Vec2) {
//! // A "+10" that rises from a projected screen point and fades out.
//! commands.spawn(popup(viewport_pos, "+10", 28.0, Color::WHITE));
//! # }
//! ```
//!
//! The fade and lifetime are backed by the crate's [`tween`](crate::tween)
//! module: inserting a [`Popup`] attaches a `Tween<f32>` that ramps the alpha to
//! zero and despawns the popup when it completes. The rise stays a plain
//! velocity, since it has no fixed end.

use bevy::prelude::*;

use crate::tween::prelude::*;

pub mod prelude {
    pub use super::{popup, Popup, PopupPlugin, PopupSystems};
}

/// Configuration for a floating popup label on a UI text entity.
///
/// Put this on any UI `Text` node (the [`popup`] builder does it for you) and
/// [`PopupPlugin`] will rise it up the screen, ramp its `TextColor` alpha to
/// zero over `lifetime`, and despawn it when the lifetime is up.
#[derive(Component, Debug, Clone, Reflect)]
pub struct Popup {
    /// Total lifetime in seconds; the popup despawns once its age reaches this.
    pub lifetime: f32,

    /// Upward screen speed in pixels per second. Applied to the node's `top`
    /// (only when `top` is a `Val::Px`).
    pub rise_speed: f32,

    /// Base color. Its alpha is ramped from its starting value down to zero as
    /// the popup ages, so a translucent base color fades from that alpha.
    pub base_color: Color,
}

impl Default for Popup {
    fn default() -> Self {
        Self {
            lifetime: 0.8,
            rise_speed: 70.0,
            base_color: Color::WHITE,
        }
    }
}

/// System set for [`PopupPlugin`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PopupSystems {
    /// Rises the popup and applies the fade `Tween`'s value to its color. Runs
    /// after [`TweenSystems::Advance`], since it reads the tween.
    Animate,
}

/// Plugin that animates and despawns floating [`Popup`] text.
pub struct PopupPlugin;

impl Plugin for PopupPlugin {
    fn build(&self, app: &mut App) {
        debug!("PopupPlugin: build");

        // The fade / despawn ride on a `Tween<f32>`; make sure it is advanced.
        if !app.is_plugin_added::<TweenPlugin>() {
            app.add_plugins(TweenPlugin);
        }

        app.register_type::<Popup>();

        app.add_observer(on_insert_popup);
        app.add_systems(
            Update,
            animate_popups
                .after(TweenSystems::Advance)
                .in_set(PopupSystems::Animate),
        );
    }
}

/// On [`Popup`] insert, attach the alpha-fade tween: ramp the color's alpha from
/// its base value to zero over the lifetime, then despawn the whole popup. Uses
/// `On<Insert>` so re-inserting a `Popup` (a game overriding the feel) rebuilds
/// the tween from the new config.
fn on_insert_popup(insert: On<Insert, Popup>, q_popup: Query<&Popup>, mut commands: Commands) {
    let entity = insert.entity;
    let Ok(popup) = q_popup.get(entity) else {
        return;
    };
    trace!("on_insert_popup: entity {:?}", entity);

    commands.entity(entity).insert(
        Tween::new(
            popup.base_color.alpha(),
            0.0,
            popup.lifetime,
            EaseFunction::Linear,
        )
        .with_on_complete(TweenOnComplete::Despawn),
    );
}

/// Rise floating popups up the screen and fade them by writing the fade tween's
/// current alpha into their `TextColor`. Ageing and despawn are owned by the
/// tween ([`TweenPlugin`]); `Node` / `TextColor` are optional so a popup missing
/// either still rides its tween to the despawn.
fn animate_popups(
    time: Res<Time>,
    mut q_popup: Query<(
        &Popup,
        &Tween<f32>,
        Option<&mut Node>,
        Option<&mut TextColor>,
    )>,
) {
    let dt = time.delta_secs();

    for (popup, fade, node, text_color) in q_popup.iter_mut() {
        if let Some(mut node) = node {
            if let Val::Px(top) = node.top {
                node.top = Val::Px(top - popup.rise_speed * dt);
            }
        }

        if let Some(mut text_color) = text_color {
            text_color.0 = popup.base_color.with_alpha(fade.value());
        }
    }
}

/// Bundle builder for the common floating "+N" popup: a screen-space `Text`
/// label anchored at an absolute viewport `position` (logical px), sized and
/// colored, that rises and fades with the default [`Popup`] feel.
///
/// Chain scoping onto the spawn, e.g.
/// `commands.spawn(popup(pos, "+10", 28.0, color)).insert(DespawnOnExit(state))`.
pub fn popup(position: Vec2, text: impl Into<String>, font_size: f32, color: Color) -> impl Bundle {
    (
        Name::new("Popup"),
        Popup {
            base_color: color,
            ..default()
        },
        Text::new(text.into()),
        TextFont {
            font_size: FontSize::Px(font_size),
            ..default()
        },
        TextColor(color),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(position.x),
            top: Val::Px(position.y),
            ..default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(app: &mut App, dt_ms: u64) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(dt_ms));
        app.update();
    }

    #[test]
    fn popup_rises_fades_and_despawns() {
        let mut app = App::new();
        app.init_resource::<Time>();
        app.add_plugins(PopupPlugin);

        let start_top = 200.0;
        let ent = app
            .world_mut()
            .spawn((
                Popup {
                    lifetime: 0.5,
                    rise_speed: 100.0,
                    base_color: Color::WHITE,
                },
                Node {
                    top: Val::Px(start_top),
                    ..default()
                },
                TextColor(Color::WHITE),
            ))
            .id();

        // One 100ms step: the popup rose (top decreased) and faded (alpha < 1).
        step(&mut app, 100);
        let node = app.world().get::<Node>(ent).unwrap();
        let Val::Px(top) = node.top else {
            panic!("top should be Px");
        };
        assert!(
            top < start_top,
            "popup should rise (top {top} < {start_top})"
        );
        let alpha = app.world().get::<TextColor>(ent).unwrap().0.alpha();
        assert!(
            alpha < 1.0 && alpha > 0.0,
            "alpha should be fading: {alpha}"
        );

        // Past the lifetime the popup despawns itself.
        for _ in 0..6 {
            step(&mut app, 100);
        }
        assert!(
            app.world().get_entity(ent).is_err(),
            "popup should have despawned after its lifetime"
        );
    }

    #[test]
    fn bare_popup_without_node_or_text_despawns() {
        // The animate system queries Node/TextColor optionally so a Popup with
        // neither still ages and despawns (rather than leaking).
        let mut app = App::new();
        app.init_resource::<Time>();
        app.add_plugins(PopupPlugin);

        let ent = app
            .world_mut()
            .spawn(Popup {
                lifetime: 0.3,
                ..default()
            })
            .id();

        for _ in 0..5 {
            step(&mut app, 100);
        }
        assert!(
            app.world().get_entity(ent).is_err(),
            "a Node/TextColor-less popup should still despawn after its lifetime"
        );
    }
}
