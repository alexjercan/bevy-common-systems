//! A one-line text readout of an entity's [`Health`] as a percentage.
//!
//! Spawn the [`health_display`] bundle pointing at a target entity and
//! [`HealthDisplayPlugin`] keeps its text in sync with that entity's health each frame. It is
//! the turnkey counterpart to wiring a [`status_bar`](crate::ui::status) item by hand when all
//! you want is "Health: N%".
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! # fn spawn_hud(mut commands: Commands, player: Entity) {
//! commands.spawn(health_display(HealthDisplayConfig { target: Some(player) }));
//! # }
//! ```

use bevy::prelude::*;

use crate::health::prelude::*;

pub mod prelude {
    pub use super::{
        health_display, HealthDisplayConfig, HealthDisplayMarker, HealthDisplayPlugin,
        HealthDisplayPluginSystems, HealthDisplayTarget,
    };
}

/// Marker for a health display text node.
#[derive(Component, Debug, Clone, Reflect)]
pub struct HealthDisplayMarker;

/// Configuration for [`health_display`].
#[derive(Clone, Debug, Default)]
pub struct HealthDisplayConfig {
    /// The entity whose [`Health`] is shown. `None` reads as 0%.
    pub target: Option<Entity>,
}

/// The entity a [`HealthDisplayMarker`] node tracks. Update it to retarget the readout.
#[derive(Component, Debug, Clone, Deref, DerefMut, Reflect)]
pub struct HealthDisplayTarget(pub Option<Entity>);

/// Bundle for a bottom-right "Health: N%" readout. Spawn it and drive it with
/// [`HealthDisplayPlugin`]. Override the [`Node`] afterwards to reposition it.
pub fn health_display(config: HealthDisplayConfig) -> impl Bundle {
    debug!("health_display: config {:?}", config);

    (
        Name::new("HealthDisplay"),
        HealthDisplayMarker,
        HealthDisplayTarget(config.target),
        Text::new("Health: 100%"),
        TextShadow::default(),
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
    )
}

/// System set for the health display update, so games can order around it.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum HealthDisplayPluginSystems {
    /// Refreshes each display's text from its target's health.
    Sync,
}

/// Plugin that keeps every [`HealthDisplayMarker`] text in sync with its target's [`Health`].
#[derive(Default)]
pub struct HealthDisplayPlugin;

impl Plugin for HealthDisplayPlugin {
    fn build(&self, app: &mut App) {
        debug!("HealthDisplayPlugin: build");
        app.add_systems(Update, update_text.in_set(HealthDisplayPluginSystems::Sync));
    }
}

fn update_text(
    mut q_hud: Query<(&mut Text, &HealthDisplayTarget), With<HealthDisplayMarker>>,
    q_target: Query<&Health>,
) {
    for (mut text, target) in &mut q_hud {
        let Some(target) = **target else {
            **text = "Health: 0%".to_string();
            continue;
        };

        let Ok(health) = q_target.get(target) else {
            **text = "Health: 0%".to_string();
            continue;
        };

        let health_percent = (health.current / health.max * 100.0).round();
        **text = format!("Health: {}%", health_percent);
    }
}
