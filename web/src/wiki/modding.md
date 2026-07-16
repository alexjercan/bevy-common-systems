# modding

A generic, serde-friendly event bus for Bevy games, plus a JSON-authored
registry so mods can add game behavior without recompiling. Events carry their
payload as `serde_json::Value`, which lets handler filters and actions be
written in Rust and then wired together from data.

Everything below is available via the crate prelude:

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
```

## The event bus

The bus is generic over a world type you provide. Implement [`EventWorld`] for
your game-state resource, then add [`GameEventsPlugin`] for it:

```rust
#[derive(Resource, Default, Debug, Clone)]
struct CustomEventWorld {
    counter: u32,
}

impl EventWorld for CustomEventWorld {
    // Sync your Bevy resources into the world before events run...
    fn world_to_state_system(_: &mut World) {}
    // ...and back out again after they run.
    fn state_to_world_system(_: &mut World) {}
}

fn build(app: &mut App) {
    app.add_plugins(GameEventsPlugin::<CustomEventWorld>::default());
}
```

`GameEventsPlugin` inserts a `GameEventQueue<W>`, an `EventHandlerIndex<W>`, an
empty [`EventHandlerRegistry`] resource, and an observer that queues fired
events. In `PostUpdate` it runs `world_to_state_system`, drains the queue
through matching handlers, then runs `state_to_world_system`.

To fire an event from anywhere with `Commands`, use `CommandsGameEventExt::fire`:

```rust
fn update_system(mut commands: Commands) {
    commands.fire::<OnUpdateEvent>(OnUpdateEventInfo { value: 0.7 });
    commands.fire::<OnTick>(()); // an event with no payload
}
```

Each fired event becomes a [`GameEvent`] carrying a name and a [`GameEventInfo`]
(a `GameEventInfo { data: Option<serde_json::Value> }`). Dispatch is routed by
event name through the `EventHandlerIndex`, so an event only visits the handlers
registered for its name.

## EventKind and the derive

An event kind names an event and declares its payload type. Implement
[`EventKind`] by hand, or derive it. The `#[derive(EventKind)]` macro lives in
`bevy_common_systems_macros` and is re-exported through the prelude:

```rust
#[derive(Debug, Clone, EventKind)]
#[event_name("onupdate")]
#[event_info(OnUpdateEventInfo)]
pub struct OnUpdateEvent;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OnUpdateEventInfo {
    value: f32,
}
```

With no attributes the derive defaults the name to the lowercased struct name
and the `Info` payload type to `()` (no data):

```rust
// name() == "ontick", Info == ()
#[derive(Debug, Clone, EventKind)]
pub struct OnTick;
```

`#[event_name("...")]` overrides the name; `#[event_info(MyPayload)]` overrides
the payload type. The payload must be `Serialize + Default + Clone + Debug`.

## Handlers

An [`EventHandler`] reacts to one event name. It holds a list of
[`EventFilter`]s (all must pass) and a list of [`EventAction`]s (run in order
when the filters pass). Both traits are generic over your `EventWorld`:

```rust
struct IncrementCounterAction;
impl EventAction<CustomEventWorld> for IncrementCounterAction {
    fn action(&self, world: &mut CustomEventWorld, _: &GameEventInfo) {
        world.counter += 1;
    }
}

#[derive(serde::Deserialize)]
struct MinValueFilter {
    min_value: f32,
}
impl EventFilter<CustomEventWorld> for MinValueFilter {
    fn filter(&self, _: &CustomEventWorld, info: &GameEventInfo) -> bool {
        info.data
            .as_ref()
            .and_then(|d| d.get("value"))
            .and_then(|v| v.as_f64())
            .map_or(false, |v| v as f32 >= self.min_value)
    }
}
```

You can build a handler in Rust with the builder methods and spawn it as an
entity:

```rust
let handler = EventHandler::<CustomEventWorld>::new::<OnUpdateEvent>()
    .with_filter(MinValueFilter { min_value: 0.5 })
    .with_action(IncrementCounterAction);
commands.spawn(handler);
```

The dispatcher picks up spawned handlers each frame and prunes despawned ones,
so you can add or remove behavior at runtime.

## The JSON registry

The point of the module: author handlers as data. An [`EventHandlerRegistry`]
maps string names to registered event kinds, filter constructors, and action
constructors, so a mod file can wire them together. `GameEventsPlugin` inserts
an empty registry; populate it from a startup system.

Register the names the JSON is allowed to use. Filters/actions that just
deserialize from their params use `register_filter_de` / `register_action_de`;
anything else takes a custom constructor closure:

```rust
fn setup_handlers(
    mut commands: Commands,
    mut registry: ResMut<EventHandlerRegistry<CustomEventWorld>>,
) {
    registry
        .register_event::<OnUpdateEvent>()
        .register_event::<OnTick>()
        .register_filter_de::<MinValueFilter>("min_value")
        .register_action("increment_counter", |_| {
            Ok::<_, String>(IncrementCounterAction)
        });

    for spec in parse_specs(HANDLERS_JSON).unwrap() {
        let handler = registry.build_handler(&spec).unwrap();
        let name = spec.name.clone().unwrap_or_else(|| spec.event.clone());
        commands.spawn((Name::new(name), handler));
    }
}
```

The JSON is an array of handler specs. Each names an `event`, its optional
`filters`, and its `actions`; a filter/action entry has a `type` (the
registered name) and optional `params` handed to the constructor:

```json
[
    {
        "name": "OnUpdate Handler",
        "event": "onupdate",
        "filters": [{ "type": "min_value", "params": { "min_value": 0.5 } }],
        "actions": [{ "type": "increment_counter" }]
    },
    {
        "name": "OnTick Handler",
        "event": "ontick",
        "actions": [{ "type": "increment_counter" }]
    }
]
```

Unknown JSON fields are rejected, so a typo is a parse error rather than a
silent no-op. Building fails with a [`RegistryError`] naming the offending
`event`, `filter`, `action`, or bad `params`. Use `parse_specs` +
`build_handler` when you want the spec's display `name`; use `parse_handlers`
to build every handler in one call. See the `03_modding` and `09_reactor`
[examples](../examples/) for full games driven this way, and
[persist](../persist/) for saving world state across runs.
