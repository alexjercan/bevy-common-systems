//! JSON-authored `EventHandler` construction for the modding event bus.
//!
//! [`events`](super::events) can already carry event payloads as
//! `serde_json::Value` across a scripting boundary, but its `EventHandler`
//! filter and action trait objects can only be built from Rust. This module
//! closes that gap: register the event kinds, filters and actions your game
//! understands under string names, then build handlers from data.
//!
//! ```
//! # use bevy::prelude::*;
//! # use serde::Deserialize;
//! # use bevy_common_systems::prelude::*;
//! #[derive(Resource, Default, Debug, Clone)]
//! struct MyWorld {
//!     score: i32,
//! }
//! impl EventWorld for MyWorld {
//!     fn world_to_state_system(_: &mut World) {}
//!     fn state_to_world_system(_: &mut World) {}
//! }
//!
//! #[derive(Clone, EventKind)]
//! #[event_name("score")]
//! struct ScoreEvent;
//!
//! #[derive(Deserialize)]
//! struct AddScore {
//!     amount: i32,
//! }
//! impl EventAction<MyWorld> for AddScore {
//!     fn action(&self, world: &mut MyWorld, _: &GameEventInfo) {
//!         world.score += self.amount;
//!     }
//! }
//!
//! let mut registry = EventHandlerRegistry::<MyWorld>::new();
//! registry.register_event::<ScoreEvent>();
//! registry.register_action_de::<AddScore>("add_score");
//!
//! let handlers = registry
//!     .parse_handlers(
//!         r#"[{ "event": "score", "actions": [
//!             { "type": "add_score", "params": { "amount": 10 } }
//!         ] }]"#,
//!     )
//!     .unwrap();
//! assert_eq!(handlers.len(), 1);
//! ```

use std::{collections::HashMap, fmt, sync::Arc};

use bevy::prelude::*;
use serde::Deserialize;

use super::events::{EventAction, EventFilter, EventHandler, EventKind, EventWorld};

pub mod prelude {
    pub use super::{
        parse_specs, EventHandlerRegistry, HandlerComponentSpec, HandlerSpec, RegistryError,
    };
}

/// Constructor that turns a params JSON value into a shared filter.
type FilterCtor<W> =
    Box<dyn Fn(&serde_json::Value) -> Result<Arc<dyn EventFilter<W>>, String> + Send + Sync>;

/// Constructor that turns a params JSON value into a shared action.
type ActionCtor<W> =
    Box<dyn Fn(&serde_json::Value) -> Result<Arc<dyn EventAction<W>>, String> + Send + Sync>;

/// Data description of a single filter or action inside a [`HandlerSpec`].
///
/// The `type` field selects a registered constructor by name; `params` is the
/// (optional) JSON handed to that constructor. A missing `params` is `null`.
/// Unknown fields are rejected so a typo in a mod file is a parse error rather
/// than a silent no-op.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandlerComponentSpec {
    /// The registered name of the filter or action to construct.
    #[serde(rename = "type")]
    pub kind: String,
    /// Constructor parameters, passed through verbatim. Defaults to `null`.
    #[serde(default)]
    pub params: serde_json::Value,
}

/// Data description of one [`EventHandler`], deserializable from JSON.
///
/// ```json
/// {
///   "name": "OnUpdate Handler",
///   "event": "onupdate",
///   "filters": [{ "type": "min_value", "params": { "min_value": 0.5 } }],
///   "actions": [{ "type": "increment_counter" }]
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandlerSpec {
    /// Optional human-readable name, e.g. for a Bevy `Name` component. The
    /// registry does not use it; it is carried through for the caller.
    #[serde(default)]
    pub name: Option<String>,
    /// The registered event name this handler reacts to.
    pub event: String,
    /// Filters that must all pass before the actions run.
    #[serde(default)]
    pub filters: Vec<HandlerComponentSpec>,
    /// Actions to run when the event fires and the filters pass.
    #[serde(default)]
    pub actions: Vec<HandlerComponentSpec>,
}

/// Error raised while building a handler out of data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// The JSON string could not be parsed into handler specs.
    Parse(String),
    /// The spec named an event with no registered [`EventKind`].
    UnknownEvent(String),
    /// The spec named a filter with no registered constructor.
    UnknownFilter(String),
    /// The spec named an action with no registered constructor.
    UnknownAction(String),
    /// A constructor rejected its params (e.g. deserialization failed).
    Params {
        /// The `type` name of the offending filter or action.
        component: String,
        /// The constructor's error message.
        message: String,
    },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::Parse(msg) => write!(f, "failed to parse handler specs: {msg}"),
            RegistryError::UnknownEvent(name) => write!(f, "unknown event kind: {name:?}"),
            RegistryError::UnknownFilter(name) => write!(f, "unknown filter: {name:?}"),
            RegistryError::UnknownAction(name) => write!(f, "unknown action: {name:?}"),
            RegistryError::Params { component, message } => {
                write!(f, "bad params for {component:?}: {message}")
            }
        }
    }
}

impl std::error::Error for RegistryError {}

/// Parse a JSON array of handler specs without building anything.
///
/// Useful when the caller wants the [`HandlerSpec::name`] (e.g. to attach a
/// Bevy `Name`) alongside the built handler. This is a free function because
/// deserializing specs does not depend on the world type `W`.
pub fn parse_specs(json: &str) -> Result<Vec<HandlerSpec>, RegistryError> {
    serde_json::from_str(json).map_err(|e| RegistryError::Parse(e.to_string()))
}

/// Maps event / filter / action name strings to registered constructors so
/// [`EventHandler`]s can be authored in JSON and built at runtime.
///
/// Register the event kinds, filters and actions the game understands, then
/// call [`build_handler`](Self::build_handler) or
/// [`parse_handlers`](Self::parse_handlers). The registry is also a Bevy
/// `Resource`: [`GameEventsPlugin`](super::events::GameEventsPlugin) inserts
/// an empty one, so game code can populate it from a startup system.
///
/// Intentionally not `Reflect`: it stores boxed constructor closures
/// (`FilterCtor` / `ActionCtor`), which cannot be reflected, so this resource
/// is invisible to the inspector by design.
#[derive(Resource)]
pub struct EventHandlerRegistry<W: EventWorld> {
    events: HashMap<String, &'static str>,
    filters: HashMap<String, FilterCtor<W>>,
    actions: HashMap<String, ActionCtor<W>>,
}

impl<W: EventWorld> Default for EventHandlerRegistry<W> {
    fn default() -> Self {
        Self {
            events: HashMap::new(),
            filters: HashMap::new(),
            actions: HashMap::new(),
        }
    }
}

impl<W: EventWorld> EventHandlerRegistry<W> {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an [`EventKind`] so handlers can name it.
    ///
    /// A [`HandlerSpec`] is only valid if its `event` string is exactly
    /// `E::name()`; there is no aliasing. Registering the same event twice is a
    /// harmless no-op (last-wins).
    pub fn register_event<E: EventKind>(&mut self) -> &mut Self {
        if self
            .events
            .insert(E::name().to_string(), E::name())
            .is_some()
        {
            trace!("EventHandlerRegistry: re-registered event {:?}", E::name());
        }
        self
    }

    /// Register a filter constructor under `name`.
    ///
    /// The constructor receives the spec's `params` JSON and returns the filter
    /// or an error message. Use [`register_filter_de`](Self::register_filter_de)
    /// when the filter can simply be deserialized from its params. Re-using a
    /// `name` overwrites the previous constructor (last-wins).
    pub fn register_filter<F, C>(&mut self, name: impl Into<String>, ctor: C) -> &mut Self
    where
        F: EventFilter<W> + 'static,
        C: Fn(&serde_json::Value) -> Result<F, String> + Send + Sync + 'static,
    {
        let name = name.into();
        let ctor: FilterCtor<W> =
            Box::new(move |params| ctor(params).map(|f| Arc::new(f) as Arc<dyn EventFilter<W>>));
        if self.filters.insert(name.clone(), ctor).is_some() {
            trace!("EventHandlerRegistry: re-registered filter {name:?}");
        }
        self
    }

    /// Register a filter that is deserialized directly from its params JSON.
    pub fn register_filter_de<F>(&mut self, name: impl Into<String>) -> &mut Self
    where
        F: EventFilter<W> + serde::de::DeserializeOwned + 'static,
    {
        self.register_filter(name, |params: &serde_json::Value| {
            F::deserialize(params).map_err(|e| e.to_string())
        })
    }

    /// Register an action constructor under `name`.
    ///
    /// The constructor receives the spec's `params` JSON and returns the action
    /// or an error message. Use [`register_action_de`](Self::register_action_de)
    /// when the action can simply be deserialized from its params. Re-using a
    /// `name` overwrites the previous constructor (last-wins).
    pub fn register_action<A, C>(&mut self, name: impl Into<String>, ctor: C) -> &mut Self
    where
        A: EventAction<W> + 'static,
        C: Fn(&serde_json::Value) -> Result<A, String> + Send + Sync + 'static,
    {
        let name = name.into();
        let ctor: ActionCtor<W> =
            Box::new(move |params| ctor(params).map(|a| Arc::new(a) as Arc<dyn EventAction<W>>));
        if self.actions.insert(name.clone(), ctor).is_some() {
            trace!("EventHandlerRegistry: re-registered action {name:?}");
        }
        self
    }

    /// Register an action that is deserialized directly from its params JSON.
    pub fn register_action_de<A>(&mut self, name: impl Into<String>) -> &mut Self
    where
        A: EventAction<W> + serde::de::DeserializeOwned + 'static,
    {
        self.register_action(name, |params: &serde_json::Value| {
            A::deserialize(params).map_err(|e| e.to_string())
        })
    }

    /// Build one [`EventHandler`] from a spec, resolving every name.
    pub fn build_handler(&self, spec: &HandlerSpec) -> Result<EventHandler<W>, RegistryError> {
        let name = *self
            .events
            .get(&spec.event)
            .ok_or_else(|| RegistryError::UnknownEvent(spec.event.clone()))?;

        let mut handler = EventHandler::<W>::from_event_name(name);

        for component in &spec.filters {
            let ctor = self
                .filters
                .get(&component.kind)
                .ok_or_else(|| RegistryError::UnknownFilter(component.kind.clone()))?;
            let filter = ctor(&component.params).map_err(|message| RegistryError::Params {
                component: component.kind.clone(),
                message,
            })?;
            handler.add_filter_arc(filter);
        }

        for component in &spec.actions {
            let ctor = self
                .actions
                .get(&component.kind)
                .ok_or_else(|| RegistryError::UnknownAction(component.kind.clone()))?;
            let action = ctor(&component.params).map_err(|message| RegistryError::Params {
                component: component.kind.clone(),
                message,
            })?;
            handler.add_action_arc(action);
        }

        Ok(handler)
    }

    /// Parse a JSON array of specs and build every handler.
    ///
    /// Fails on the first spec that cannot be built. When the display names
    /// matter, pair [`parse_specs`] with [`build_handler`](Self::build_handler)
    /// instead.
    pub fn parse_handlers(&self, json: &str) -> Result<Vec<EventHandler<W>>, RegistryError> {
        parse_specs(json)?
            .iter()
            .map(|spec| self.build_handler(spec))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modding::events::GameEventInfo;

    #[derive(Resource, Default, Debug, Clone)]
    struct TestWorld {
        counter: i32,
    }

    impl EventWorld for TestWorld {
        fn world_to_state_system(_: &mut World) {}
        fn state_to_world_system(_: &mut World) {}
    }

    #[derive(Clone)]
    struct TickEvent;

    impl EventKind for TickEvent {
        type Info = ();
        fn name() -> &'static str {
            "tick"
        }
    }

    // An action that carries params deserialized from JSON.
    #[derive(Deserialize)]
    struct AddCounter {
        amount: i32,
    }

    impl EventAction<TestWorld> for AddCounter {
        fn action(&self, world: &mut TestWorld, _: &GameEventInfo) {
            world.counter += self.amount;
        }
    }

    // A filter that reads a threshold from params.
    #[derive(Deserialize)]
    struct MinCounter {
        min: i32,
    }

    impl EventFilter<TestWorld> for MinCounter {
        fn filter(&self, world: &TestWorld, _: &GameEventInfo) -> bool {
            world.counter >= self.min
        }
    }

    fn registry() -> EventHandlerRegistry<TestWorld> {
        let mut registry = EventHandlerRegistry::<TestWorld>::new();
        registry.register_event::<TickEvent>();
        registry.register_filter_de::<MinCounter>("min_counter");
        registry.register_action_de::<AddCounter>("add_counter");
        registry
    }

    #[test]
    fn builds_handler_with_filter_and_action() {
        let registry = registry();
        let handlers = registry
            .parse_handlers(
                r#"[{
                    "name": "tick handler",
                    "event": "tick",
                    "filters": [{ "type": "min_counter", "params": { "min": 3 } }],
                    "actions": [{ "type": "add_counter", "params": { "amount": 5 } }]
                }]"#,
            )
            .expect("should build");

        assert_eq!(handlers.len(), 1);
        let handler = &handlers[0];

        // The filter's `min: 3` param blocks below the threshold and passes at
        // or above it.
        let mut world = TestWorld { counter: 0 };
        assert!(!handler.filter(&world, &GameEventInfo::default()));
        world.counter = 3;
        assert!(handler.filter(&world, &GameEventInfo::default()));

        // The action's `amount: 5` param actually drives behaviour: running the
        // built actions advances the counter by the JSON-supplied amount.
        // (`actions` is `pub(super)`, reachable from this child module.)
        for action in &handler.actions {
            action.action(&mut world, &GameEventInfo::default());
        }
        assert_eq!(world.counter, 8);
    }

    #[test]
    fn unknown_json_fields_are_rejected() {
        let registry = registry();
        // A typo in a field name ("acions") must be a parse error, not a silent
        // no-op handler.
        let err = registry
            .parse_handlers(r#"[{ "event": "tick", "acions": [] }]"#)
            .err()
            .unwrap();
        assert!(matches!(err, RegistryError::Parse(_)));
    }

    // `EventHandler` holds trait objects and is not `Debug`, so the tests below
    // use `.err().unwrap()` (no `Debug` bound on the `Ok` value) rather than
    // `Result::unwrap_err`.

    #[test]
    fn unknown_event_is_reported() {
        let registry = registry();
        let err = registry
            .build_handler(&HandlerSpec {
                name: None,
                event: "nope".to_string(),
                filters: vec![],
                actions: vec![],
            })
            .err()
            .unwrap();
        assert_eq!(err, RegistryError::UnknownEvent("nope".to_string()));
    }

    #[test]
    fn unknown_filter_and_action_are_reported() {
        let registry = registry();
        let err = registry
            .parse_handlers(r#"[{ "event": "tick", "filters": [{ "type": "ghost" }] }]"#)
            .err()
            .unwrap();
        assert_eq!(err, RegistryError::UnknownFilter("ghost".to_string()));

        let err = registry
            .parse_handlers(r#"[{ "event": "tick", "actions": [{ "type": "ghost" }] }]"#)
            .err()
            .unwrap();
        assert_eq!(err, RegistryError::UnknownAction("ghost".to_string()));
    }

    #[test]
    fn bad_params_are_reported() {
        let registry = registry();
        // `amount` should be an integer; a string fails deserialization.
        let err = registry
            .parse_handlers(
                r#"[{ "event": "tick", "actions": [
                    { "type": "add_counter", "params": { "amount": "lots" } }
                ] }]"#,
            )
            .err()
            .unwrap();
        match err {
            RegistryError::Params { component, .. } => assert_eq!(component, "add_counter"),
            other => panic!("expected Params error, got {other:?}"),
        }
    }

    #[test]
    fn invalid_json_is_a_parse_error() {
        let registry = registry();
        let err = registry.parse_handlers("not json").err().unwrap();
        assert!(matches!(err, RegistryError::Parse(_)));
    }

    #[test]
    fn custom_constructor_can_ignore_params() {
        let mut registry = EventHandlerRegistry::<TestWorld>::new();
        registry.register_event::<TickEvent>();
        // A constructor that takes no params at all.
        registry.register_action("noop", |_| Ok::<_, String>(AddCounter { amount: 0 }));

        let handlers = registry
            .parse_handlers(r#"[{ "event": "tick", "actions": [{ "type": "noop" }] }]"#)
            .expect("should build");
        assert_eq!(handlers.len(), 1);
    }
}
