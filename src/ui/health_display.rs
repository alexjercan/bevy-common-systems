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

        **text = format!("Health: {}%", display_percent(health.current, health.max));
    }
}

/// The integer percentage shown for a health pool, guarded against the two ways a
/// naive `(current / max * 100).round()` misreads a pool:
///
/// - a non-positive `max` reads 0% (dead), which also avoids the `NaN`/`inf` a
///   divide-by-zero would otherwise render during a section-less death window,
/// - a strictly-positive fraction below 1% ceils to 1%, so a barely-alive sliver
///   (e.g. 0.4 hp on a 230 max) never reads dead while it is still targetable,
/// - everything else rounds to the nearest whole percent as before.
fn display_percent(current: f32, max: f32) -> i32 {
    if max <= 0.0 {
        return 0;
    }
    let percent = current / max * 100.0;
    if percent <= 0.0 {
        0
    } else if percent < 1.0 {
        1
    } else {
        percent.round() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::display_percent;

    #[test]
    fn full_and_zero_read_as_expected() {
        assert_eq!(display_percent(230.0, 230.0), 100);
        assert_eq!(display_percent(0.0, 230.0), 0);
        assert_eq!(display_percent(115.0, 230.0), 50);
    }

    #[test]
    fn living_sliver_ceils_to_one_percent() {
        // 0.4 / 230 = 0.17%, which would round to 0% - a living ship must not read dead.
        assert_eq!(display_percent(0.4, 230.0), 1);
        // Just under 1% still ceils up.
        assert_eq!(display_percent(2.29, 230.0), 1);
        // At/above 1% rounds normally.
        assert_eq!(display_percent(2.3, 230.0), 1);
        assert_eq!(display_percent(3.45, 230.0), 2);
    }

    #[test]
    fn non_positive_max_reads_zero_not_nan() {
        // A section-less root aggregate writes Health { current: 0, max: 0 };
        // dividing would render "NaN%" - it must read 0% instead.
        assert_eq!(display_percent(0.0, 0.0), 0);
        assert_eq!(display_percent(5.0, 0.0), 0);
        assert_eq!(display_percent(1.0, -10.0), 0);
    }

    #[test]
    fn negative_current_reads_zero() {
        assert_eq!(display_percent(-0.5, 230.0), 0);
    }
}
