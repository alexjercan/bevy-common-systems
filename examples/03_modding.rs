use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use clap::Parser;

#[derive(Parser)]
#[command(name = "03_modding")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how to create basic custom events", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(PhysicsPlugins::default());

    #[cfg(feature = "debug")]
    app.add_plugins(InspectorDebugPlugin);

    app.add_plugins(custom_plugin);

    app.run();
}

fn custom_plugin(app: &mut App) {
    app.add_plugins(GameEventsPlugin::<CustomEventWorld>::default());
    app.insert_resource(Time::<Fixed>::from_hz(1.0));

    app.init_resource::<SomeCounter>();

    app.add_systems(Startup, setup_handlers);
    app.add_systems(FixedUpdate, (print_counter_system, update_system));
}

// The handlers are authored as data, not Rust. In a real mod this JSON would be
// loaded from a file or received over a scripting boundary; here it is inlined
// so the example stays a single file. Each entry names an `event`, the
// `filters` that must pass, and the `actions` to run -- all by the string names
// registered on the `EventHandlerRegistry` in `setup_handlers` below.
const HANDLERS_JSON: &str = r#"[
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
]"#;

#[derive(Resource, Default, Debug, Clone, Deref, DerefMut)]
pub struct SomeCounter(pub u32);

fn print_counter_system(counter: Res<SomeCounter>) {
    println!("print_counter_system: counter {}", **counter);
}

#[derive(Resource, Default, Debug, Clone)]
pub struct CustomEventWorld {
    pub counter: u32,
}

impl EventWorld for CustomEventWorld {
    fn world_to_state_system(world: &mut World) {
        let counter = **world.resource::<SomeCounter>();
        let mut resource = world.resource_mut::<Self>();
        resource.counter = counter;
    }

    fn state_to_world_system(world: &mut World) {
        let new_counter = world.resource::<Self>().counter;
        let mut counter = world.resource_mut::<SomeCounter>();
        **counter = new_counter;
    }
}

#[derive(Debug, Clone, EventKind)]
#[event_name("onupdate")]
#[event_info(OnUpdateEventInfo)]
pub struct OnUpdateEvent;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OnUpdateEventInfo {
    value: f32,
}

// Attribute-less derive: exercises the default `event_name` (the lowercased
// struct name, "ontick") and the default `Info` type (`()`, meaning no
// payload), so a regression in the macro's generated defaults fails to
// compile here.
#[derive(Debug, Clone, EventKind)]
pub struct OnTick;

#[derive(Debug, Clone)]
struct IncrementCounterAction;

impl EventAction<CustomEventWorld> for IncrementCounterAction {
    fn action(&self, world: &mut CustomEventWorld, _: &GameEventInfo) {
        world.counter += 1;
        println!("IncrementCounterAction: counter {}", world.counter);
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct MinValueFilter {
    min_value: f32,
}

impl EventFilter<CustomEventWorld> for MinValueFilter {
    fn filter(&self, _: &CustomEventWorld, info: &GameEventInfo) -> bool {
        let Some(data) = &info.data else {
            return false;
        };

        let Some(value) = data.get("value").and_then(|v| v.as_f64()) else {
            return false;
        };

        println!("MinValueFilter: value {}", value);
        (value as f32) >= self.min_value
    }
}

fn setup_handlers(
    mut commands: Commands,
    mut registry: ResMut<EventHandlerRegistry<CustomEventWorld>>,
) {
    // Teach the registry the names the JSON is allowed to use: the event kinds,
    // the filter (deserialized straight from its params) and the action (a
    // custom constructor that takes no params). `GameEventsPlugin` already
    // inserted the empty registry resource.
    registry
        .register_event::<OnUpdateEvent>()
        .register_event::<OnTick>()
        .register_filter_de::<MinValueFilter>("min_value")
        .register_action("increment_counter", |_| {
            Ok::<_, String>(IncrementCounterAction)
        });

    // Build a handler entity from each spec. `parse_specs` keeps the optional
    // display name so it can become a Bevy `Name`.
    let specs = parse_specs(HANDLERS_JSON).expect("HANDLERS_JSON should be valid handler specs");
    for spec in &specs {
        let handler = registry
            .build_handler(spec)
            .expect("every event/filter/action name should be registered");
        let name = spec.name.clone().unwrap_or_else(|| spec.event.clone());
        commands.spawn((Name::new(name), handler));
    }
}

fn update_system(mut commands: Commands) {
    commands.fire::<OnUpdateEvent>(OnUpdateEventInfo {
        value: rand::random(),
    });
    commands.fire::<OnTick>(());
}
