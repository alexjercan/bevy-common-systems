//! A generic on-screen objectives list: a panel that renders one text line per objective.
//!
//! The objectives live in the [`GameObjectives`] resource as `(id, message)` pairs; whenever
//! it changes, [`ObjectivesPlugin`] rebuilds the child text lines under the
//! [`objectives_panel`] node. The `id` is opaque to this module - it exists so game code can
//! address a specific objective (mark it done, update its message) by replacing the list.
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! # fn setup(mut commands: Commands, mut objectives: ResMut<GameObjectives>) {
//! commands.spawn(objectives_panel(ObjectivesPanelConfig::default()));
//! objectives.objectives = vec![
//!     Objective::new("reach_exit", "Reach the exit"),
//!     Objective::new("collect_keys", "Collect 3 keys"),
//! ];
//! # }
//! ```

use bevy::prelude::*;

pub mod prelude {
    pub use super::{
        objectives_panel, GameObjectives, Objective, ObjectiveMarker, ObjectivesPanelConfig,
        ObjectivesPanelMarker, ObjectivesPlugin, ObjectivesPluginSystems,
    };
}

/// A single objective line: an opaque `id` for game code to address, and the `message` shown.
#[derive(Clone, Debug)]
pub struct Objective {
    /// Opaque identifier for game code (not shown).
    pub id: String,
    /// The text shown for this objective.
    pub message: String,
}

impl Objective {
    /// Convenience constructor from string slices.
    pub fn new(id: &str, message: &str) -> Self {
        Self {
            id: id.to_string(),
            message: message.to_string(),
        }
    }
}

/// The current objectives. Replace the `objectives` vec to change what the panel shows.
#[derive(Resource, Clone, Debug, Default)]
pub struct GameObjectives {
    /// The objectives, rendered top to bottom.
    pub objectives: Vec<Objective>,
}

/// Marker for the objectives panel root node.
#[derive(Component, Debug, Clone, Reflect)]
pub struct ObjectivesPanelMarker;

/// Marker for a single objective text line under the panel.
#[derive(Component, Debug, Clone, Reflect)]
pub struct ObjectiveMarker;

/// The `id` of the objective a line was rendered from.
#[derive(Component, Debug, Clone, Deref, DerefMut, Reflect)]
pub struct ObjectiveId(pub String);

/// Configuration for [`objectives_panel`]. Empty for now; a hook for future layout options.
#[derive(Clone, Debug, Default)]
pub struct ObjectivesPanelConfig {}

/// Bundle for the objectives panel root. Spawn one; [`ObjectivesPlugin`] fills it with a text
/// line per [`GameObjectives`] entry. Override the [`Node`] afterwards to reposition it.
pub fn objectives_panel(config: ObjectivesPanelConfig) -> impl Bundle {
    debug!("objectives_panel: config {:?}", config);

    (
        Name::new("ObjectivesPanel"),
        ObjectivesPanelMarker,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            right: Val::Px(5.0),
            ..default()
        },
    )
}

/// System set for the objectives rebuild, so games can order around it.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObjectivesPluginSystems {
    /// Rebuilds the panel's text lines when [`GameObjectives`] changes.
    Sync,
}

/// Plugin that renders [`GameObjectives`] under the [`objectives_panel`] node.
#[derive(Default)]
pub struct ObjectivesPlugin;

impl Plugin for ObjectivesPlugin {
    fn build(&self, app: &mut App) {
        debug!("ObjectivesPlugin: build");
        app.init_resource::<GameObjectives>();

        app.add_systems(
            Update,
            rebuild_lines
                .run_if(resource_changed::<GameObjectives>)
                .in_set(ObjectivesPluginSystems::Sync),
        );
    }
}

fn rebuild_lines(
    mut commands: Commands,
    q_panel: Single<(Entity, Option<&Children>), With<ObjectivesPanelMarker>>,
    objectives: Res<GameObjectives>,
) {
    trace!("rebuild_lines: objectives {:?}", *objectives);
    let (entity, children) = q_panel.into_inner();

    let new_children = objectives
        .objectives
        .iter()
        .map(|objective| {
            commands
                .spawn((
                    Name::new(format!("Objective {}", objective.id)),
                    ObjectiveMarker,
                    ObjectiveId(objective.id.clone()),
                    Text::new(objective.message.clone()),
                    TextShadow::default(),
                    TextLayout::justify(Justify::Center),
                ))
                .id()
        })
        .collect::<Vec<_>>();

    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    commands.entity(entity).replace_children(&new_children);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_renders_one_line_per_objective() {
        let mut app = App::new();
        app.add_plugins(ObjectivesPlugin);
        let panel = app.world_mut().spawn(objectives_panel(default())).id();

        app.world_mut().resource_mut::<GameObjectives>().objectives =
            vec![Objective::new("a", "Alpha"), Objective::new("b", "Bravo")];
        app.update();

        let children = app.world().get::<Children>(panel).expect("panel children");
        assert_eq!(children.len(), 2);

        // Shrinking the list rebuilds down to one line (old lines are despawned).
        app.world_mut().resource_mut::<GameObjectives>().objectives =
            vec![Objective::new("a", "Alpha")];
        app.update();
        assert_eq!(app.world().get::<Children>(panel).unwrap().len(), 1);
    }
}
