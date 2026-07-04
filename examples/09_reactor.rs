//! 09_reactor -- a rules-as-machine incremental where the modding event bus is
//! the game.
//!
//! This is the headline demo of [`modding`](bevy_common_systems::modding): the
//! whole simulation runs on the event bus from `examples/03_modding`, but here
//! the player *builds* the machine at runtime by installing handlers. A reactor
//! world holds three resources -- ENERGY, HEAT and CREDITS -- and the engine
//! `fire`s two kinds of events: a `tick` every half second (the idle heartbeat)
//! and a `click`/`sell` whenever you tap the controls. Every rule that reacts to
//! those events is a JSON-authored [`EventHandler`] entity built through the
//! [`EventHandlerRegistry`]: the built-in "Manual Tap" and "Sell" handlers ship
//! with the reactor, and every machine part you buy from the shop spawns another
//! one. Fuel rods add energy but also heat; heat sinks and coolant pumps bleed
//! heat off; market uplinks turn surplus energy into credits. Compose them into
//! an escalating loop -- but the grid heats up as you climb tiers, so if HEAT
//! ever reaches 100 the reactor melts down and the run ends. Score is the total
//! credits earned.
//!
//! Because the rules ARE the mod system, the "gameplay" and the modding data are
//! the same thing: the shop is a palette of [`HandlerSpec`]s, buying a part is a
//! `build_handler` call, and the reactor is just the event queue draining every
//! frame. It exercises `modding` end to end (events, the JSON registry, filters
//! and actions), plus [`SfxPlugin`] one-shots and a compact [`ui/status`] HUD.
//!
//! Note: within a single tick the handlers run in `Query<&EventHandler>` order
//! (entity spawn / archetype order), which is not specified, so a gated part (a
//! Coolant Pump reading HEAT, a Market Uplink reading ENERGY) may see slightly
//! different intermediate world state depending on the order parts were
//! installed. This is inherent to the event bus and harmless here (the amounts
//! are small relative to a tick), but worth knowing if you copy this pattern.
//!
//! It follows the `06_fruitninja` shape: `States` for menu / playing / game-over
//! and a wasm gallery build. No 3D scene is needed, so it renders with a plain
//! `Camera2d`.
//!
//! Controls: click TAP (or press Space) to generate energy, click SELL (or press
//! Enter) to turn energy into credits, and click a shop card (or press its number
//! key) to install a machine part. Escape gives up.
//!
//! Run it: `cargo run --example 09_reactor` (add `--features debug` for the
//! inspector).

use std::sync::Arc;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use clap::Parser;
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "09_reactor")]
#[command(version = "1.0.0")]
#[command(
    about = "Build a reactor out of modding-bus handlers without letting the heat run away.",
    long_about = None
)]
struct Cli;

// --- Tuning -----------------------------------------------------------------
//
// These were reasoned first and then boot-tested with an autopilot harness. A
// run should bootstrap by hand in the first minute, idle-scale in the middle,
// and eventually lose the heat race once the grid tiers up.

/// Heat reading at which the reactor melts down and the run ends.
const HEAT_MAX: f64 = 100.0;
/// Heat at/above which the console is amber (warning).
const HEAT_AMBER: f64 = 45.0;
/// Heat at/above which the console is red (alarm + meltdown pressure).
const HEAT_RED: f64 = 75.0;

/// Seconds between reactor ticks. The `tick` event drives all idle automation.
const TICK_INTERVAL: f32 = 0.5;

/// Energy a single manual TAP produces (via the built-in Manual Tap handler).
const TAP_ENERGY: f64 = 5.0;
/// Credits produced per unit of energy when you SELL (built-in Sell handler).
const SELL_RATE: f64 = 0.6;

/// Credits the reactor boots with, enough to install a first part by hand.
const STARTING_CREDITS: f64 = 20.0;

/// Seconds between alarm beeps while HEAT is in the red.
const ALARM_INTERVAL: f32 = 0.6;

/// Ambient heat (per second) added per grid tier above zero. Tier 0 is stable;
/// each tier the environment fights your coolers a little harder.
const AMBIENT_PER_TIER: f64 = 1.2;

/// Total credits earned needed to reach grid tier 1.
const TIER_BASE: f64 = 40.0;
/// Each further grid tier needs this multiple of the previous tier's milestone,
/// so the tier ladder (and the ambient heat it drives) is *uncapped* and
/// geometric: heat pressure scales with everything you have ever earned and never
/// plateaus, so there is no set-and-forget equilibrium. Crossing a tier bumps
/// difficulty and plays a chime.
const TIER_GROWTH: f64 = 1.55;

// --- Machine parts ----------------------------------------------------------
//
// The shop palette. Each part is a JSON `HandlerSpec` (parsed at startup and
// built through the registry when bought), plus display and pricing metadata.
// The specs only ever name the events / filters / actions registered in
// `setup_registry`, exactly like a real mod file would.

/// One buyable machine part: its display card, price curve and handler spec.
struct PartDef {
    /// Digit key that also buys this part (1..=6), for keyboard play.
    key: KeyCode,
    /// Short name shown on the shop card.
    name: &'static str,
    /// One-line description of what the installed handler does.
    desc: &'static str,
    /// Price of the first copy; each further copy costs `growth` times more.
    base_cost: f64,
    /// Geometric price growth per copy already owned.
    growth: f64,
    /// The JSON `HandlerSpec` installed when the part is bought.
    spec: &'static str,
}

const PART_COUNT: usize = 6;

const PARTS: [PartDef; PART_COUNT] = [
    PartDef {
        key: KeyCode::Digit1,
        name: "Solar Array",
        desc: "+1.4 energy / tick, no heat",
        base_cost: 8.0,
        growth: 1.15,
        spec: r#"{ "name": "Solar Array", "event": "tick",
            "actions": [{ "type": "add_energy", "params": { "amount": 1.4 } }] }"#,
    },
    PartDef {
        key: KeyCode::Digit2,
        name: "Fuel Rod",
        desc: "+9 energy / tick, but +3.2 heat",
        base_cost: 22.0,
        growth: 1.17,
        spec: r#"{ "name": "Fuel Rod", "event": "tick", "actions": [
            { "type": "add_energy", "params": { "amount": 9.0 } },
            { "type": "add_heat", "params": { "amount": 3.2 } }
        ] }"#,
    },
    PartDef {
        key: KeyCode::Digit3,
        name: "Heat Sink",
        desc: "-2.2 heat / tick, passive",
        base_cost: 18.0,
        growth: 1.16,
        spec: r#"{ "name": "Heat Sink", "event": "tick",
            "actions": [{ "type": "add_heat", "params": { "amount": -2.2 } }] }"#,
    },
    PartDef {
        key: KeyCode::Digit4,
        name: "Coolant Pump",
        desc: "when hot: -8 heat, costs 2 energy",
        base_cost: 40.0,
        growth: 1.2,
        spec: r#"{ "name": "Coolant Pump", "event": "tick",
            "filters": [
                { "type": "min_heat", "params": { "amount": 18.0 } },
                { "type": "min_energy", "params": { "amount": 2.0 } }
            ],
            "actions": [
                { "type": "add_heat", "params": { "amount": -8.0 } },
                { "type": "add_energy", "params": { "amount": -2.0 } }
            ] }"#,
    },
    PartDef {
        key: KeyCode::Digit5,
        name: "Market Uplink",
        desc: "sells 4 energy -> 3 credits / tick",
        base_cost: 28.0,
        growth: 1.2,
        spec: r#"{ "name": "Market Uplink", "event": "tick",
            "filters": [{ "type": "min_energy", "params": { "amount": 4.0 } }],
            "actions": [
                { "type": "add_energy", "params": { "amount": -4.0 } },
                { "type": "add_credits", "params": { "amount": 3.0 } }
            ] }"#,
    },
    PartDef {
        key: KeyCode::Digit6,
        name: "Turbine",
        desc: "when very hot: -5 heat -> +4 energy",
        base_cost: 65.0,
        growth: 1.24,
        spec: r#"{ "name": "Turbine", "event": "tick",
            "filters": [{ "type": "min_heat", "params": { "amount": 35.0 } }],
            "actions": [
                { "type": "add_heat", "params": { "amount": -5.0 } },
                { "type": "add_energy", "params": { "amount": 4.0 } }
            ] }"#,
    },
];

/// The reactor's built-in handlers, installed on every run before the shop. They
/// wire the manual TAP and SELL controls to the bus so even hand-play goes
/// through the event queue. Authored from the tuning constants (via `format!`)
/// so the JSON and the documented values cannot drift apart.
fn builtin_specs() -> [String; 2] {
    [
        format!(
            r#"{{ "name": "Manual Tap", "event": "click",
                "actions": [{{ "type": "add_energy", "params": {{ "amount": {TAP_ENERGY} }} }}] }}"#,
        ),
        format!(
            r#"{{ "name": "Sell", "event": "sell",
                "filters": [{{ "type": "min_energy", "params": {{ "amount": 1.0 }} }}],
                "actions": [{{ "type": "sell_all", "params": {{ "rate": {SELL_RATE} }} }}] }}"#,
        ),
    ]
}

// --- App --------------------------------------------------------------------

fn main() {
    let _ = Cli::parse();
    let mut app = App::new();

    let primary_window = Window {
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#game-canvas".into()),
        #[cfg(target_arch = "wasm32")]
        fit_canvas_to_parent: true,
        ..default()
    };
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(primary_window),
        ..default()
    }));

    // avian is not used for gameplay, but the debug inspector's physics gizmos
    // expect it, so keep it added for a clean `--features debug` boot.
    app.add_plugins(PhysicsPlugins::default());

    #[cfg(feature = "debug")]
    app.add_plugins(InspectorDebugPlugin);

    if !app.is_plugin_added::<bevy::diagnostic::FrameTimeDiagnosticsPlugin>() {
        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
    }

    // The modding event bus that runs the whole simulation.
    app.add_plugins(GameEventsPlugin::<ReactorWorld>::default());
    app.add_plugins(StatusBarPlugin);
    app.add_plugins(SfxPlugin);

    app.insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.08)));
    app.init_resource::<Shop>();
    app.init_resource::<Progress>();
    app.init_resource::<HighScore>();
    app.insert_resource(TickTimer(Timer::from_seconds(
        TICK_INTERVAL,
        TimerMode::Repeating,
    )));

    app.init_state::<GameState>();

    app.add_systems(Startup, (setup_registry, setup).chain());

    // Main menu.
    app.add_systems(OnEnter(GameState::Menu), spawn_menu);
    app.add_systems(
        Update,
        (menu_start, pulse_menu_title).run_if(in_state(GameState::Menu)),
    );

    // Playing.
    app.add_systems(OnEnter(GameState::Playing), (start_run, spawn_hud).chain());
    app.add_systems(
        Update,
        (
            fire_ticks,
            manual_controls,
            buy_input,
            check_meltdown,
            update_readouts,
            update_heat_bar,
            update_shop_cards,
            update_alarm,
            giveup_on_escape,
        )
            .run_if(in_state(GameState::Playing)),
    );

    // Meltdown / game over.
    // `spawn_game_over` reads the still-old best to decide "New best!", then
    // `record_high_score` updates it -- so a run that only ties the best is not
    // announced as a new best.
    app.add_systems(
        OnEnter(GameState::GameOver),
        (spawn_game_over, record_high_score, play_game_over_sfx).chain(),
    );
    app.add_systems(
        Update,
        gameover_dismiss.run_if(in_state(GameState::GameOver)),
    );

    app.run();
}

// --- State ------------------------------------------------------------------

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

// --- The reactor world (an `EventWorld`) ------------------------------------

/// The reactor simulation state. It IS the modding `EventWorld`: the event queue
/// hands it to every handler's filters and actions, and the UI reads it straight
/// back out. Sync is a no-op because the resource is the single source of truth
/// (there is no separate game state to mirror).
#[derive(Resource, Debug, Clone)]
struct ReactorWorld {
    /// Working resource: spent by coolers and sellers, made by generators.
    energy: f64,
    /// Danger resource: generators add it, coolers remove it, 100 melts down.
    heat: f64,
    /// Score currency: made by selling energy, spent installing parts.
    credits: f64,
}

impl Default for ReactorWorld {
    fn default() -> Self {
        Self {
            energy: 0.0,
            heat: 0.0,
            credits: STARTING_CREDITS,
        }
    }
}

impl EventWorld for ReactorWorld {
    fn world_to_state_system(_: &mut World) {}
    fn state_to_world_system(_: &mut World) {}
}

// The three event kinds the reactor fires. None carry a payload, so `Info`
// defaults to `()`; the handlers react purely to the world state.

#[derive(Clone, EventKind)]
#[event_name("tick")]
struct TickEvent;

#[derive(Clone, EventKind)]
#[event_name("click")]
struct ClickEvent;

#[derive(Clone, EventKind)]
#[event_name("sell")]
struct SellEvent;

// --- Filters (world predicates authored by name in JSON) --------------------

/// Passes when the reactor has at least `amount` energy.
#[derive(Deserialize)]
struct MinEnergy {
    amount: f64,
}

impl EventFilter<ReactorWorld> for MinEnergy {
    fn filter(&self, world: &ReactorWorld, _: &GameEventInfo) -> bool {
        world.energy >= self.amount
    }
}

/// Passes when the reactor is at least `amount` hot.
#[derive(Deserialize)]
struct MinHeat {
    amount: f64,
}

impl EventFilter<ReactorWorld> for MinHeat {
    fn filter(&self, world: &ReactorWorld, _: &GameEventInfo) -> bool {
        world.heat >= self.amount
    }
}

// --- Actions (world mutations authored by name in JSON) ---------------------

/// Add `amount` energy (signed; clamped at zero so costs cannot go negative).
#[derive(Deserialize)]
struct AddEnergy {
    amount: f64,
}

impl EventAction<ReactorWorld> for AddEnergy {
    fn action(&self, world: &mut ReactorWorld, _: &GameEventInfo) {
        world.energy = (world.energy + self.amount).max(0.0);
    }
}

/// Add `amount` heat (signed; clamped at zero). Positive from generators,
/// negative from coolers.
#[derive(Deserialize)]
struct AddHeat {
    amount: f64,
}

impl EventAction<ReactorWorld> for AddHeat {
    fn action(&self, world: &mut ReactorWorld, _: &GameEventInfo) {
        world.heat = (world.heat + self.amount).max(0.0);
    }
}

/// Add `amount` credits (clamped at zero).
#[derive(Deserialize)]
struct AddCredits {
    amount: f64,
}

impl EventAction<ReactorWorld> for AddCredits {
    fn action(&self, world: &mut ReactorWorld, _: &GameEventInfo) {
        world.credits = (world.credits + self.amount).max(0.0);
    }
}

/// Convert *all* current energy into credits at `rate`. Used by the built-in
/// manual Sell handler, whose transfer amount is dynamic (unlike the fixed-param
/// Market Uplink), so it is one bespoke action rather than an energy/credit pair.
#[derive(Deserialize)]
struct SellAll {
    rate: f64,
}

impl EventAction<ReactorWorld> for SellAll {
    fn action(&self, world: &mut ReactorWorld, _: &GameEventInfo) {
        world.credits += world.energy * self.rate;
        world.energy = 0.0;
    }
}

/// Register every event / filter / action name the shop JSON is allowed to use.
/// Split out from the startup system so the tests build an identically-populated
/// registry from one source of truth (a divergence would let a spec pass a test
/// but fail in game, or vice versa).
fn register_all(registry: &mut EventHandlerRegistry<ReactorWorld>) {
    registry
        .register_event::<TickEvent>()
        .register_event::<ClickEvent>()
        .register_event::<SellEvent>()
        .register_filter_de::<MinEnergy>("min_energy")
        .register_filter_de::<MinHeat>("min_heat")
        .register_action_de::<AddEnergy>("add_energy")
        .register_action_de::<AddHeat>("add_heat")
        .register_action_de::<AddCredits>("add_credits")
        .register_action_de::<SellAll>("sell_all");
}

/// Populate the registry so it can build handlers from data. `GameEventsPlugin`
/// already inserted the empty registry resource.
fn setup_registry(mut registry: ResMut<EventHandlerRegistry<ReactorWorld>>) {
    register_all(&mut registry);
}

// --- Resources --------------------------------------------------------------

/// Fires the reactor `tick` on a fixed cadence, independent of frame rate.
#[derive(Resource)]
struct TickTimer(Timer);

/// How many of each shop part are installed, plus the credits spent installing
/// them (so total earned = current credits + spent = the score).
#[derive(Resource, Default)]
struct Shop {
    owned: [u32; PART_COUNT],
    spent: f64,
}

impl Shop {
    fn reset(&mut self) {
        self.owned = [0; PART_COUNT];
        self.spent = 0.0;
    }

    /// Number of parts currently installed (shown in the HUD).
    fn total_parts(&self) -> u32 {
        self.owned.iter().sum()
    }
}

/// Per-run progress derived from play: elapsed time, grid tier and alarm timing.
#[derive(Resource, Default)]
struct Progress {
    /// Seconds this run has lasted.
    elapsed: f32,
    /// Grid tier (0-based); raises ambient heat as it climbs.
    tier: u32,
    /// Counts down to the next alarm beep while HEAT is red.
    alarm_timer: f32,
}

impl Progress {
    fn reset(&mut self) {
        self.elapsed = 0.0;
        self.tier = 0;
        self.alarm_timer = 0.0;
    }
}

/// Best score (total credits earned) seen this process.
#[derive(Resource, Default)]
struct HighScore(f64);

/// Handles for the one-shot sound effects, loaded once in `setup`.
#[derive(Resource)]
struct SfxAssets {
    menu_select: Handle<AudioSource>,
    tap: Handle<AudioSource>,
    buy: Handle<AudioSource>,
    alarm: Handle<AudioSource>,
    tier_up: Handle<AudioSource>,
    game_over: Handle<AudioSource>,
}

// --- Components --------------------------------------------------------------

/// A built handler entity representing one installed part (or built-in). Tagged
/// so the whole machine can be cleared when a run ends.
#[derive(Component)]
struct InstalledHandler;

/// Marker for the menu title text (pulses).
#[derive(Component)]
struct MenuTitle;

/// Big header readouts, updated every frame from `ReactorWorld`.
#[derive(Component)]
enum Readout {
    Energy,
    Credits,
}

/// The heat bar's coloured fill (its width tracks HEAT / HEAT_MAX).
#[derive(Component)]
struct HeatBarFill;

/// The numeric "HEAT xx / 100" label under the bar.
#[derive(Component)]
struct HeatLabel;

/// A shop card button for part `idx`; its background shows affordability.
#[derive(Component)]
struct ShopCard(usize);

/// The price / count line on shop card `idx`.
#[derive(Component)]
struct ShopCardStatus(usize);

/// The big TAP button (fires a `click` event).
#[derive(Component)]
struct TapButton;

/// The SELL button (fires a `sell` event).
#[derive(Component)]
struct SellButton;

/// Central alarm banner shown while HEAT is red.
#[derive(Component)]
struct AlarmBanner;

// --- Pure helpers -----------------------------------------------------------

/// Price of the next copy of part `idx` given how many are already owned.
fn part_cost(idx: usize, owned: u32) -> f64 {
    PARTS[idx].base_cost * PARTS[idx].growth.powi(owned as i32)
}

/// Grid tier for a given total-earned score, on an uncapped geometric ladder:
/// tier `k` is reached at `TIER_BASE * TIER_GROWTH^(k-1)`. Below `TIER_BASE` the
/// grid is stable (tier 0); above it the tier keeps climbing forever, so ambient
/// heat never plateaus.
fn tier_for_score(score: f64) -> u32 {
    if score < TIER_BASE {
        return 0;
    }
    (score / TIER_BASE).log(TIER_GROWTH).floor() as u32 + 1
}

/// Compact human number: 1234 -> "1.2k", 2_500_000 -> "2.50M".
fn fmt_num(v: f64) -> String {
    let v = v.max(0.0);
    if v < 1000.0 {
        format!("{:.0}", v)
    } else if v < 1_000_000.0 {
        format!("{:.1}k", v / 1000.0)
    } else if v < 1_000_000_000.0 {
        format!("{:.2}M", v / 1_000_000.0)
    } else {
        format!("{:.2}B", v / 1_000_000_000.0)
    }
}

/// The score is every credit ever earned: what you still hold plus what you spent.
fn score(world: &ReactorWorld, shop: &Shop) -> f64 {
    world.credits + shop.spent
}

/// Green below amber, amber up to red, red at/above `HEAT_RED`.
fn heat_color(heat: f64) -> Color {
    if heat >= HEAT_RED {
        Color::srgb(1.0, 0.28, 0.28)
    } else if heat >= HEAT_AMBER {
        Color::srgb(1.0, 0.78, 0.2)
    } else {
        Color::srgb(0.4, 1.0, 0.5)
    }
}

// --- Setup ------------------------------------------------------------------

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Name::new("UI Camera"), Camera2d));

    commands.insert_resource(SfxAssets {
        menu_select: asset_server.load("sounds/menu_select.wav"),
        tap: asset_server.load("sounds/pickup.wav"),
        buy: asset_server.load("sounds/golden.wav"),
        alarm: asset_server.load("sounds/alarm.wav"),
        tier_up: asset_server.load("sounds/level_up.wav"),
        game_over: asset_server.load("sounds/game_over.wav"),
    });

    // A compact telemetry HUD in the corner, exercising `ui/status`: the value
    // closures read `ReactorWorld` / `Shop` straight out of the World each frame.
    commands.spawn((status_bar(StatusBarRootConfig::default()),));

    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: |world: &World| {
            world
                .get_resource::<Progress>()
                .map(|p| Arc::new(p.tier + 1) as Arc<dyn StatusValue>)
        },
        color_fn: |_| Some(Color::srgb(0.7, 0.8, 1.0)),
        prefix: "TIER".to_string(),
        suffix: "".to_string(),
    }),));

    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: |world: &World| {
            world
                .get_resource::<Shop>()
                .map(|s| Arc::new(s.total_parts()) as Arc<dyn StatusValue>)
        },
        color_fn: |_| Some(Color::srgb(0.8, 0.85, 0.95)),
        prefix: "PARTS".to_string(),
        suffix: "".to_string(),
    }),));

    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: status_fps_value_fn(),
        color_fn: status_fps_color_fn(),
        prefix: "".to_string(),
        suffix: "fps".to_string(),
    }),));
}

// --- Shared UI helpers ------------------------------------------------------

fn centered_screen() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        row_gap: Val::Px(14.0),
        ..default()
    }
}

fn screen_text(text: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(text.into()),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        TextLayout {
            justify: Justify::Center,
            ..default()
        },
    )
}

fn best_line(best: f64) -> String {
    if best > 0.0 {
        format!("Best: {} credits", fmt_num(best))
    } else {
        "No run yet".to_string()
    }
}

// --- Menu -------------------------------------------------------------------

fn spawn_menu(mut commands: Commands, high: Res<HighScore>) {
    commands
        .spawn((
            Name::new("Menu"),
            DespawnOnExit(GameState::Menu),
            centered_screen(),
        ))
        .with_children(|parent| {
            parent.spawn((
                MenuTitle,
                screen_text("REACTOR", 84.0, Color::srgb(0.5, 0.9, 1.0)),
            ));
            parent.spawn(screen_text(
                "RULES-AS-MACHINE",
                26.0,
                Color::srgb(0.75, 0.8, 0.9),
            ));
            parent.spawn(screen_text(
                "Every rule is a modding-bus handler. Tap for energy, sell it for",
                20.0,
                Color::srgb(0.7, 0.75, 0.85),
            ));
            parent.spawn(screen_text(
                "credits, then buy machine parts that react to each reactor tick.",
                20.0,
                Color::srgb(0.7, 0.75, 0.85),
            ));
            parent.spawn(screen_text(
                "Fuel rods make energy AND heat -- keep HEAT under 100 or it melts down.",
                18.0,
                Color::srgb(0.6, 0.65, 0.75),
            ));
            parent.spawn(screen_text(
                best_line(high.0),
                22.0,
                Color::srgb(0.95, 0.85, 0.25),
            ));
            parent.spawn(screen_text(
                "Click or press any key to begin",
                24.0,
                Color::srgb(0.9, 0.9, 0.9),
            ));
        });
}

fn pulse_menu_title(time: Res<Time>, mut q: Query<&mut TextColor, With<MenuTitle>>) {
    let t = (time.elapsed_secs() * 2.4).sin() * 0.5 + 0.5;
    let brightness = 0.55 + 0.45 * t;
    for mut color in &mut q {
        color.0 = Color::srgb(0.3 * brightness, 0.7 * brightness + 0.1, brightness);
    }
}

/// Any click or key press begins a run. Reading input here also satisfies the
/// browser's audio-unlock gesture so the first sound plays on the web build.
fn menu_start(
    mut commands: Commands,
    sfx: Res<SfxAssets>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<GameState>>,
) {
    let pressed = mouse.just_pressed(MouseButton::Left) || keys.get_just_pressed().next().is_some();
    if pressed {
        commands.play_sfx_volume(sfx.menu_select.clone(), 0.7);
        next.set(GameState::Playing);
    }
}

// --- Run start --------------------------------------------------------------

/// Reset the world and install the built-in handlers. The shop parts are added
/// later, as they are bought; these two ship with every reactor.
fn start_run(
    mut commands: Commands,
    registry: Res<EventHandlerRegistry<ReactorWorld>>,
    mut world: ResMut<ReactorWorld>,
    mut shop: ResMut<Shop>,
    mut progress: ResMut<Progress>,
) {
    *world = ReactorWorld::default();
    shop.reset();
    progress.reset();

    for spec_json in builtin_specs() {
        let spec = serde_json::from_str::<HandlerSpec>(&spec_json)
            .expect("built-in handler spec should be valid JSON");
        let handler = registry
            .build_handler(&spec)
            .expect("built-in handler should use only registered names");
        let name = spec.name.clone().unwrap_or_else(|| spec.event.clone());
        commands.spawn((
            Name::new(name),
            InstalledHandler,
            handler,
            DespawnOnExit(GameState::Playing),
        ));
    }
}

// --- HUD --------------------------------------------------------------------

fn button_node() -> Node {
    Node {
        padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
        margin: UiRect::all(Val::Px(4.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(2.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),
        ..default()
    }
}

fn spawn_hud(mut commands: Commands) {
    // Root column: readouts, heat bar, manual controls, then the shop grid.
    commands
        .spawn((
            Name::new("HUD"),
            DespawnOnExit(GameState::Playing),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(14.0),
                ..default()
            },
        ))
        .with_children(|root| {
            // Big resource readouts.
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(48.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Readout::Energy,
                    screen_text("ENERGY 0", 34.0, Color::srgb(0.6, 0.95, 1.0)),
                ));
                row.spawn((
                    Readout::Credits,
                    screen_text("CREDITS 0", 34.0, Color::srgb(0.95, 0.85, 0.25)),
                ));
            });

            // Heat bar: a track with a coloured fill whose width tracks HEAT.
            root.spawn((
                Node {
                    width: Val::Px(520.0),
                    height: Val::Px(28.0),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.4, 0.45, 0.55)),
                BackgroundColor(Color::srgb(0.1, 0.11, 0.15)),
            ))
            .with_children(|track| {
                track.spawn((
                    HeatBarFill,
                    Node {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(heat_color(0.0)),
                ));
            });
            root.spawn((
                HeatLabel,
                screen_text("HEAT 0 / 100", 20.0, heat_color(0.0)),
            ));

            // Manual controls.
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(16.0),
                margin: UiRect::top(Val::Px(6.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    TapButton,
                    Button,
                    button_node(),
                    BackgroundColor(Color::srgb(0.16, 0.3, 0.4)),
                    BorderColor::all(Color::srgb(0.5, 0.8, 1.0)),
                ))
                .with_child(screen_text("TAP  (+energy)", 22.0, Color::WHITE));
                row.spawn((
                    SellButton,
                    Button,
                    button_node(),
                    BackgroundColor(Color::srgb(0.32, 0.28, 0.12)),
                    BorderColor::all(Color::srgb(1.0, 0.85, 0.3)),
                ))
                .with_child(screen_text(
                    "SELL  (energy->credits)",
                    22.0,
                    Color::WHITE,
                ));
            });

            // Shop grid: one card per part.
            root.spawn((
                Name::new("Shop"),
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    max_width: Val::Px(760.0),
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    margin: UiRect::top(Val::Px(12.0)),
                    ..default()
                },
            ))
            .with_children(|shop| {
                for (idx, part) in PARTS.iter().enumerate() {
                    shop.spawn((
                        ShopCard(idx),
                        Button,
                        Node {
                            width: Val::Px(240.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.14, 0.2)),
                        BorderColor::all(Color::srgb(0.3, 0.35, 0.45)),
                    ))
                    .with_children(|card| {
                        card.spawn(screen_text(
                            format!("{}  {}", idx + 1, part.name),
                            22.0,
                            Color::srgb(0.9, 0.95, 1.0),
                        ));
                        card.spawn(screen_text(part.desc, 15.0, Color::srgb(0.7, 0.75, 0.85)));
                        card.spawn((
                            ShopCardStatus(idx),
                            screen_text("cost 0   x0", 17.0, Color::srgb(0.95, 0.85, 0.25)),
                        ));
                    });
                }
            });
        });

    // Centre-screen alarm banner, hidden until HEAT is red.
    commands.spawn((
        AlarmBanner,
        DespawnOnExit(GameState::Playing),
        Visibility::Hidden,
        screen_text("!! HEAT CRITICAL !!", 40.0, Color::srgb(1.0, 0.3, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(28.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
    ));
}

// --- Simulation -------------------------------------------------------------

/// Fire the reactor `tick` on the fixed cadence, apply ambient (tier) heat, and
/// advance the survival clock / grid tier. The tick event drains through the
/// modding queue in `PostUpdate`, running every installed handler.
fn fire_ticks(
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<SfxAssets>,
    mut timer: ResMut<TickTimer>,
    mut world: ResMut<ReactorWorld>,
    shop: Res<Shop>,
    mut progress: ResMut<Progress>,
) {
    progress.elapsed += time.delta_secs();

    // Grid tier follows the total-earned score; crossing one chimes.
    let new_tier = tier_for_score(score(&world, &shop));
    if new_tier > progress.tier {
        progress.tier = new_tier;
        commands.play_sfx_volume(sfx.tier_up.clone(), 0.6);
    }

    if timer.0.tick(time.delta()).just_finished() {
        // Ambient heat from the grid: harder every tier, applied per tick.
        world.heat += AMBIENT_PER_TIER * progress.tier as f64 * TICK_INTERVAL as f64;
        commands.fire::<TickEvent>(());
    }
}

/// Handle the TAP / SELL buttons (and their keyboard shortcuts) by firing the
/// matching event onto the bus, where the built-in handlers act on it.
fn manual_controls(
    mut commands: Commands,
    sfx: Res<SfxAssets>,
    keys: Res<ButtonInput<KeyCode>>,
    tap_q: Query<&Interaction, (Changed<Interaction>, With<TapButton>)>,
    sell_q: Query<&Interaction, (Changed<Interaction>, With<SellButton>)>,
) {
    let tap_clicked = tap_q.iter().any(|i| *i == Interaction::Pressed);
    let sell_clicked = sell_q.iter().any(|i| *i == Interaction::Pressed);

    if tap_clicked || keys.just_pressed(KeyCode::Space) {
        commands.fire::<ClickEvent>(());
        commands.play_sfx_volume(sfx.tap.clone(), 0.4);
    }
    if sell_clicked || keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter)
    {
        commands.fire::<SellEvent>(());
    }
}

/// Try to install part `idx`: charge its scaled cost in credits, spawn the built
/// handler entity, and bump the owned count. No-op if unaffordable.
fn try_buy(
    idx: usize,
    commands: &mut Commands,
    registry: &EventHandlerRegistry<ReactorWorld>,
    world: &mut ReactorWorld,
    shop: &mut Shop,
    sfx: &SfxAssets,
) {
    let cost = part_cost(idx, shop.owned[idx]);
    if world.credits < cost {
        return;
    }
    let spec = serde_json::from_str::<HandlerSpec>(PARTS[idx].spec)
        .expect("part spec should be valid JSON");
    let handler = registry
        .build_handler(&spec)
        .expect("part handler should use only registered names");
    commands.spawn((
        Name::new(PARTS[idx].name),
        InstalledHandler,
        handler,
        DespawnOnExit(GameState::Playing),
    ));

    world.credits -= cost;
    shop.spent += cost;
    shop.owned[idx] += 1;
    commands.play_sfx_volume(sfx.buy.clone(), 0.6);
}

/// Buy parts from shop-card clicks or number keys.
fn buy_input(
    mut commands: Commands,
    sfx: Res<SfxAssets>,
    keys: Res<ButtonInput<KeyCode>>,
    registry: Res<EventHandlerRegistry<ReactorWorld>>,
    mut world: ResMut<ReactorWorld>,
    mut shop: ResMut<Shop>,
    card_q: Query<(&Interaction, &ShopCard), Changed<Interaction>>,
) {
    // Number-key shortcuts.
    for (idx, part) in PARTS.iter().enumerate() {
        if keys.just_pressed(part.key) {
            try_buy(idx, &mut commands, &registry, &mut world, &mut shop, &sfx);
        }
    }
    // Card clicks.
    for (interaction, card) in &card_q {
        if *interaction == Interaction::Pressed {
            try_buy(
                card.0,
                &mut commands,
                &registry,
                &mut world,
                &mut shop,
                &sfx,
            );
        }
    }
}

/// End the run when HEAT reaches the meltdown ceiling.
fn check_meltdown(world: Res<ReactorWorld>, mut next: ResMut<NextState<GameState>>) {
    if world.heat >= HEAT_MAX {
        next.set(GameState::GameOver);
    }
}

// --- HUD updates ------------------------------------------------------------

fn update_readouts(world: Res<ReactorWorld>, mut q: Query<(&Readout, &mut Text)>) {
    for (readout, mut text) in &mut q {
        match readout {
            Readout::Energy => *text = Text::new(format!("ENERGY {}", fmt_num(world.energy))),
            Readout::Credits => *text = Text::new(format!("CREDITS {}", fmt_num(world.credits))),
        }
    }
}

fn update_heat_bar(
    world: Res<ReactorWorld>,
    mut fill_q: Query<(&mut Node, &mut BackgroundColor), With<HeatBarFill>>,
    mut label_q: Query<(&mut Text, &mut TextColor), With<HeatLabel>>,
) {
    let pct = (world.heat / HEAT_MAX).clamp(0.0, 1.0) * 100.0;
    let color = heat_color(world.heat);
    for (mut node, mut bg) in &mut fill_q {
        node.width = Val::Percent(pct as f32);
        bg.0 = color;
    }
    for (mut text, mut tc) in &mut label_q {
        *text = Text::new(format!(
            "HEAT {} / {}",
            world.heat.round() as i64,
            HEAT_MAX as i64
        ));
        tc.0 = color;
    }
}

/// Refresh each shop card's price / count line and dim it when unaffordable.
fn update_shop_cards(
    world: Res<ReactorWorld>,
    shop: Res<Shop>,
    mut status_q: Query<(&ShopCardStatus, &mut Text)>,
    mut card_q: Query<(&ShopCard, &mut BackgroundColor, &mut BorderColor)>,
) {
    for (status, mut text) in &mut status_q {
        let idx = status.0;
        let cost = part_cost(idx, shop.owned[idx]);
        *text = Text::new(format!("cost {}   x{}", fmt_num(cost), shop.owned[idx]));
    }
    for (card, mut bg, mut border) in &mut card_q {
        let cost = part_cost(card.0, shop.owned[card.0]);
        let affordable = world.credits >= cost;
        bg.0 = if affordable {
            Color::srgb(0.16, 0.22, 0.3)
        } else {
            Color::srgb(0.1, 0.11, 0.15)
        };
        *border = BorderColor::all(if affordable {
            Color::srgb(0.5, 0.8, 1.0)
        } else {
            Color::srgb(0.28, 0.32, 0.4)
        });
    }
}

/// Show and pulse the alarm banner while HEAT is red, beeping on an interval.
fn update_alarm(
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<SfxAssets>,
    world: Res<ReactorWorld>,
    mut progress: ResMut<Progress>,
    mut q: Query<(&mut Visibility, &mut TextColor), With<AlarmBanner>>,
) {
    let red = world.heat >= HEAT_RED;
    for (mut vis, mut color) in &mut q {
        *vis = if red {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if red {
            let t = (time.elapsed_secs() * 9.0).sin() * 0.5 + 0.5;
            color.0 = Color::srgb(1.0, 0.2 + 0.2 * t, 0.2 + 0.2 * t);
        }
    }

    if !red {
        progress.alarm_timer = 0.0;
        return;
    }
    progress.alarm_timer -= time.delta_secs();
    if progress.alarm_timer <= 0.0 {
        progress.alarm_timer = ALARM_INTERVAL;
        commands.play_sfx_volume(sfx.alarm.clone(), 0.5);
    }
}

fn giveup_on_escape(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(GameState::GameOver);
    }
}

// --- Game over --------------------------------------------------------------

fn record_high_score(world: Res<ReactorWorld>, shop: Res<Shop>, mut high: ResMut<HighScore>) {
    let s = score(&world, &shop);
    if s > high.0 {
        high.0 = s;
    }
}

fn spawn_game_over(
    mut commands: Commands,
    world: Res<ReactorWorld>,
    shop: Res<Shop>,
    high: Res<HighScore>,
) {
    let final_score = score(&world, &shop);
    // `high.0` is still the pre-run best here (this runs before `record_high_score`
    // in the chain), so a strict `>` means only beating it -- not tying -- counts.
    let new_best = final_score > high.0;
    let melted = world.heat >= HEAT_MAX;

    commands
        .spawn((
            Name::new("Game Over"),
            DespawnOnExit(GameState::GameOver),
            centered_screen(),
        ))
        .with_children(|parent| {
            let (title, tint) = if melted {
                ("MELTDOWN", Color::srgb(1.0, 0.35, 0.3))
            } else {
                ("SHUT DOWN", Color::srgb(0.7, 0.85, 1.0))
            };
            parent.spawn(screen_text(title, 84.0, tint));
            parent.spawn(screen_text(
                format!("You earned {} credits", fmt_num(final_score)),
                30.0,
                Color::srgb(0.9, 0.9, 0.9),
            ));
            if new_best {
                parent.spawn(screen_text(
                    "New best!",
                    26.0,
                    Color::srgb(0.95, 0.85, 0.25),
                ));
            } else {
                parent.spawn(screen_text(
                    best_line(high.0),
                    24.0,
                    Color::srgb(0.8, 0.8, 0.85),
                ));
            }
            parent.spawn(screen_text(
                "Click or press any key for the menu",
                22.0,
                Color::srgb(0.7, 0.75, 0.85),
            ));
        });
}

fn play_game_over_sfx(mut commands: Commands, sfx: Res<SfxAssets>) {
    commands.play_sfx_volume(sfx.game_over.clone(), 0.8);
}

fn gameover_dismiss(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<GameState>>,
) {
    let pressed = mouse.just_pressed(MouseButton::Left) || keys.get_just_pressed().next().is_some();
    if pressed {
        next.set(GameState::Menu);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> EventHandlerRegistry<ReactorWorld> {
        // Built from the same `register_all` the game uses, so tests cannot pass
        // against a registry that differs from the shipped one.
        let mut registry = EventHandlerRegistry::<ReactorWorld>::new();
        register_all(&mut registry);
        registry
    }

    #[test]
    fn actions_mutate_world_and_clamp_at_zero() {
        let mut w = ReactorWorld::default();
        w.energy = 0.0;
        w.heat = 0.0;
        w.credits = 0.0;
        let info = GameEventInfo::default();

        AddEnergy { amount: 9.0 }.action(&mut w, &info);
        assert!((w.energy - 9.0).abs() < 1e-9);
        // A cost larger than the balance floors at zero, never negative.
        AddEnergy { amount: -20.0 }.action(&mut w, &info);
        assert_eq!(w.energy, 0.0);

        AddHeat { amount: 3.2 }.action(&mut w, &info);
        assert!((w.heat - 3.2).abs() < 1e-9);
        AddHeat { amount: -10.0 }.action(&mut w, &info);
        assert_eq!(w.heat, 0.0);

        AddCredits { amount: 5.0 }.action(&mut w, &info);
        assert!((w.credits - 5.0).abs() < 1e-9);
    }

    #[test]
    fn sell_all_converts_every_unit_of_energy() {
        let mut w = ReactorWorld::default();
        w.energy = 40.0;
        w.credits = 2.0;
        SellAll { rate: 0.6 }.action(&mut w, &GameEventInfo::default());
        assert_eq!(w.energy, 0.0);
        // 2 held + 40 * 0.6 sold.
        assert!((w.credits - (2.0 + 24.0)).abs() < 1e-9);
    }

    #[test]
    fn filters_gate_on_world_state() {
        let info = GameEventInfo::default();
        let mut w = ReactorWorld::default();
        w.energy = 3.0;
        w.heat = 10.0;

        assert!(MinEnergy { amount: 2.0 }.filter(&w, &info));
        assert!(!MinEnergy { amount: 4.0 }.filter(&w, &info));
        assert!(MinHeat { amount: 10.0 }.filter(&w, &info));
        assert!(!MinHeat { amount: 18.0 }.filter(&w, &info));
    }

    #[test]
    fn every_part_and_builtin_spec_builds_from_json() {
        // The JSON in every shop card and built-in must name only registered
        // events / filters / actions, or a card would be dead on arrival.
        let registry = registry();
        let part_specs = PARTS.iter().map(|p| p.spec.to_string());
        for spec_json in part_specs.chain(builtin_specs()) {
            let spec =
                serde_json::from_str::<HandlerSpec>(&spec_json).expect("spec should be valid JSON");
            registry
                .build_handler(&spec)
                .expect("spec should build against the registry");
        }
    }

    /// End-to-end: install the Fuel Rod handler from its JSON, fire a `tick`
    /// through the real modding bus, and assert the world changed. This drives
    /// the whole registry -> queue -> action path, not just the pieces.
    #[test]
    fn fuel_rod_handler_runs_on_a_tick_through_the_bus() {
        let mut app = App::new();
        app.add_plugins(GameEventsPlugin::<ReactorWorld>::default());
        // The plugin inserted an empty registry; populate it, then build.
        {
            let mut registry = app
                .world_mut()
                .resource_mut::<EventHandlerRegistry<ReactorWorld>>();
            register_all(&mut registry);
        }
        let fuel = PARTS
            .iter()
            .find(|p| p.name == "Fuel Rod")
            .expect("Fuel Rod part exists");
        let spec = serde_json::from_str::<HandlerSpec>(fuel.spec).unwrap();
        let handler = app
            .world()
            .resource::<EventHandlerRegistry<ReactorWorld>>()
            .build_handler(&spec)
            .expect("builds");
        app.world_mut().spawn((InstalledHandler, handler));

        // Start from a known state, fire one tick, and let PostUpdate drain it.
        {
            let mut w = app.world_mut().resource_mut::<ReactorWorld>();
            w.energy = 0.0;
            w.heat = 0.0;
        }
        app.world_mut()
            .trigger(GameEvent::new(TickEvent::name(), GameEventInfo::default()));
        app.update();

        let w = app.world().resource::<ReactorWorld>();
        assert!((w.energy - 9.0).abs() < 1e-6, "energy rose by the fuel rod");
        assert!((w.heat - 3.2).abs() < 1e-6, "heat rose by the fuel rod");
    }

    #[test]
    fn part_cost_grows_geometrically() {
        // Each owned copy multiplies the price by the growth factor.
        let base = PARTS[0].base_cost;
        assert!((part_cost(0, 0) - base).abs() < 1e-9);
        assert!(part_cost(0, 1) > part_cost(0, 0));
        assert!((part_cost(0, 2) - base * PARTS[0].growth.powi(2)).abs() < 1e-9);
    }

    #[test]
    fn tier_climbs_on_an_uncapped_geometric_ladder() {
        // Below the base score the grid is stable (tier 0); tier k is reached at
        // TIER_BASE * TIER_GROWTH^(k-1).
        assert_eq!(tier_for_score(0.0), 0);
        assert_eq!(tier_for_score(TIER_BASE - 0.1), 0);
        assert_eq!(tier_for_score(TIER_BASE), 1);
        assert_eq!(tier_for_score(TIER_BASE * TIER_GROWTH), 2);
        assert_eq!(tier_for_score(TIER_BASE * TIER_GROWTH.powi(2)), 3);
        // The ladder never plateaus: a much larger score is a strictly higher
        // tier, and the tier keeps rising with the score.
        assert!(tier_for_score(1.0e6) > tier_for_score(1.0e4));
    }

    #[test]
    fn score_is_held_plus_spent() {
        let mut w = ReactorWorld::default();
        w.credits = 30.0;
        let mut shop = Shop::default();
        shop.spent = 120.0;
        assert!((score(&w, &shop) - 150.0).abs() < 1e-9);
    }

    #[test]
    fn heat_color_crosses_thresholds() {
        let green = Color::srgb(0.4, 1.0, 0.5);
        let amber = Color::srgb(1.0, 0.78, 0.2);
        let red = Color::srgb(1.0, 0.28, 0.28);
        assert_eq!(heat_color(0.0), green);
        assert_eq!(heat_color(HEAT_AMBER - 0.1), green);
        assert_eq!(heat_color(HEAT_AMBER), amber);
        assert_eq!(heat_color(HEAT_RED - 0.1), amber);
        assert_eq!(heat_color(HEAT_RED), red);
        assert_eq!(heat_color(HEAT_MAX), red);
    }
}
