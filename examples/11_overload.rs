//! 11_overload -- a dashboard-survival game rendered entirely on the status bar.
//!
//! This is the headline demo of [`ui/status`](bevy_common_systems::ui::status):
//! the whole game surface is a `status_bar` full of `status_bar_item` gauges. You
//! run a failing reactor whose four gauges (HEAT / PRES / FLUX / CHRG) climb and
//! random-walk on their own; each item's `color_fn` shades the reading green ->
//! amber -> red at fixed thresholds. Press the number key under a gauge to vent
//! it back toward green -- but venting nudges a coupled neighbour up, so it is a
//! juggling act. While any gauge sits in the red the reactor takes damage through
//! [`HealthPlugin`]; when its `Health` hits zero (`HealthZeroMarker`) the run ends
//! at a meltdown screen. The climb rates ramp up with a difficulty level over
//! time, so the console eventually overwhelms you -- score is how long you lasted.
//!
//! It follows the `06_fruitninja` shape: `States` for menu / playing / game-over,
//! one-shot sounds via [`SfxPlugin`], and a wasm gallery build. No 3D scene is
//! needed, so it renders with a plain `Camera2d`.
//!
//! Controls: press 1 / 2 / 3 / 4 (or the numpad equivalents) to vent HEAT / PRES
//! / FLUX / CHRG. On a touchscreen an on-screen vent pad appears along the bottom
//! (revealed on the first touch) -- tap a gauge's button to vent it. Click, tap or
//! press any key to start and to dismiss the meltdown screen. Escape gives up.
//!
//! Run it: `cargo run --example 11_overload` (add `--features debug` for the
//! inspector).

use std::{any::Any, fmt, sync::Arc};

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use clap::Parser;
use rand::Rng;

#[derive(Parser)]
#[command(name = "11_overload")]
#[command(version = "1.0.0")]
#[command(
    about = "Keep a failing reactor's gauges out of the red on the status bar.",
    long_about = None
)]
struct Cli;

// --- Tuning -----------------------------------------------------------------
//
// These were reasoned first and then play-tested; see the task notes. A run
// should be frantic but survivable for a couple of minutes with good juggling.

/// How many gauges the console has. Four fits the top-right status bar and keeps
/// the juggling readable.
const GAUGE_COUNT: usize = 4;

/// Reading at/above which a gauge turns amber (warning).
const AMBER: f32 = 60.0;
/// Reading at/above which a gauge turns red (critical -- starts draining hull).
const RED: f32 = 85.0;
/// A gauge is clamped to this maximum.
const GAUGE_MAX: f32 = 100.0;

/// How much a single vent knocks a gauge down.
const VENT_AMOUNT: f32 = 34.0;
/// How much venting one gauge pushes its coupled neighbour up (the catch).
const COUPLING: f32 = 11.0;

/// Amplitude of the per-gauge random walk, in units/second. Makes the climb
/// uneven so you cannot vent on a fixed rhythm.
const DRIFT: f32 = 7.0;

/// Reactor hull hit points. One red gauge drains it in ~11 s if ignored.
const REACTOR_HEALTH: f32 = 100.0;
/// Hull damage per second for each gauge currently in the red.
const RED_DAMAGE_PER_SEC: f32 = 9.0;

/// Seconds between alarm beeps while any gauge is red.
const ALARM_INTERVAL: f32 = 0.55;

/// Seconds between difficulty levels.
const LEVEL_INTERVAL: f32 = 14.0;
/// Each level multiplies every gauge's base climb by `1 + LEVEL_CLIMB_STEP * level`.
const LEVEL_CLIMB_STEP: f32 = 0.16;

/// Height of the on-screen touch vent pad, as a fraction of the window height.
/// The pad is a bottom strip split into `GAUGE_COUNT` equal columns (one button
/// per gauge); `vent_button_at` and `spawn_vent_pad` both key off this fraction
/// so the visual buttons line up with the touch hit zones.
const VENT_ZONE_H_FRAC: f32 = 0.16;

/// One gauge's fixed properties.
struct GaugeSpec {
    /// Short label shown in the status bar (prefixed with its vent key number).
    label: &'static str,
    /// Primary vent key.
    key: KeyCode,
    /// Alternate vent key (numpad) for laptops without a comfy number row.
    key_alt: KeyCode,
    /// Index of the gauge this one pushes up when vented (a cycle).
    couples_to: usize,
    /// Base climb rate at level 0, in units/second.
    base_climb: f32,
}

/// The four gauges. Coupling forms a cycle (0 -> 1 -> 2 -> 3 -> 0), so there is
/// no gauge you can vent for free.
const GAUGES: [GaugeSpec; GAUGE_COUNT] = [
    GaugeSpec {
        label: "HEAT",
        key: KeyCode::Digit1,
        key_alt: KeyCode::Numpad1,
        couples_to: 1,
        base_climb: 2.4,
    },
    GaugeSpec {
        label: "PRES",
        key: KeyCode::Digit2,
        key_alt: KeyCode::Numpad2,
        couples_to: 2,
        base_climb: 2.9,
    },
    GaugeSpec {
        label: "FLUX",
        key: KeyCode::Digit3,
        key_alt: KeyCode::Numpad3,
        couples_to: 3,
        base_climb: 2.2,
    },
    GaugeSpec {
        label: "CHRG",
        key: KeyCode::Digit4,
        key_alt: KeyCode::Numpad4,
        couples_to: 0,
        base_climb: 2.7,
    },
];

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

    // avian is not used for gameplay here, but the debug inspector's physics
    // gizmos expect it, so keep it added for a clean `--features debug` boot.
    app.add_plugins(PhysicsPlugins::default());

    #[cfg(feature = "debug")]
    app.add_plugins(InspectorDebugPlugin);

    if !app.is_plugin_added::<bevy::diagnostic::FrameTimeDiagnosticsPlugin>() {
        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
    }

    app.add_plugins(StatusBarPlugin);
    app.add_plugins(HealthPlugin);
    app.add_plugins(SfxPlugin);

    app.insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.08)));
    app.init_resource::<ReactorState>();
    app.init_resource::<HighScore>();
    app.init_resource::<TouchSeen>();

    app.init_state::<GameState>();

    app.add_systems(Startup, setup);

    // Main menu.
    app.add_systems(OnEnter(GameState::Menu), spawn_menu);
    app.add_systems(
        Update,
        (menu_start, pulse_menu_title).run_if(in_state(GameState::Menu)),
    );

    // Playing.
    app.add_systems(
        OnEnter(GameState::Playing),
        (start_run, spawn_hud, spawn_vent_pad),
    );
    app.add_systems(
        Update,
        (
            advance_run,
            simulate_gauges,
            vent_input,
            touch_vent_input,
            update_touch_pad,
            apply_danger,
            mirror_health,
            update_alarm_banner,
            giveup_on_escape,
        )
            .run_if(in_state(GameState::Playing)),
    );

    // Meltdown / game over.
    app.add_systems(
        OnEnter(GameState::GameOver),
        (record_high_score, spawn_game_over, play_game_over_sfx).chain(),
    );
    app.add_systems(
        Update,
        gameover_dismiss.run_if(in_state(GameState::GameOver)),
    );

    app.add_observer(on_reactor_died);

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

// --- Resources --------------------------------------------------------------

/// The live reactor simulation. The status-bar `value_fn` closures read this
/// resource straight out of the `World`, so it is the single source of truth for
/// everything shown on the bar.
#[derive(Resource)]
struct ReactorState {
    /// Current reading of each gauge, 0..=GAUGE_MAX.
    gauges: [f32; GAUGE_COUNT],
    /// Current per-second climb of each gauge (base scaled by the level).
    climb: [f32; GAUGE_COUNT],
    /// Difficulty level, bumped every `LEVEL_INTERVAL` seconds.
    level: u32,
    /// Seconds survived this run (the score).
    elapsed: f32,
    /// Wall time of the next level-up.
    next_level_at: f32,
    /// Counts down to the next alarm beep while any gauge is red.
    alarm_timer: f32,
    /// Hull hit points mirrored from the reactor's `Health` for the HULL item.
    health: f32,
}

impl Default for ReactorState {
    fn default() -> Self {
        Self {
            gauges: [0.0; GAUGE_COUNT],
            climb: [0.0; GAUGE_COUNT],
            level: 0,
            elapsed: 0.0,
            next_level_at: LEVEL_INTERVAL,
            alarm_timer: 0.0,
            health: REACTOR_HEALTH,
        }
    }
}

impl ReactorState {
    /// Reset for a fresh run: gauges start scattered but comfortably in the green
    /// (below `AMBER`) and already climbing, at level 0.
    fn reset(&mut self, rng: &mut impl Rng) {
        for gauge in &mut self.gauges {
            *gauge = rng.random_range(18.0..40.0);
        }
        self.level = 0;
        self.elapsed = 0.0;
        self.next_level_at = LEVEL_INTERVAL;
        self.alarm_timer = 0.0;
        self.health = REACTOR_HEALTH;
        self.recompute_climb();
    }

    /// Refresh the climb rates from the base specs and the current level.
    fn recompute_climb(&mut self) {
        let scale = 1.0 + LEVEL_CLIMB_STEP * self.level as f32;
        for (climb, spec) in self.climb.iter_mut().zip(GAUGES.iter()) {
            *climb = spec.base_climb * scale;
        }
    }

    /// How many gauges are currently in the red.
    fn red_count(&self) -> usize {
        self.gauges.iter().filter(|&&v| v >= RED).count()
    }
}

/// Best survival time seen this process, shown on the menu and meltdown screens.
#[derive(Resource, Default)]
struct HighScore(f32);

/// True once any touch has been seen this session. Gates the reveal of the
/// on-screen vent pad so a PC/mouse session never shows it and a phone reveals it
/// the instant a thumb lands -- runtime touch detection, no `#[cfg(wasm)]` (which
/// would also fire on desktop browsers) or JS probe. Mirrors `08_dropzone`.
#[derive(Resource, Default)]
struct TouchSeen(bool);

/// Handles for the one-shot sound effects, loaded once in `setup`.
#[derive(Resource)]
struct SfxAssets {
    menu_select: Handle<AudioSource>,
    vent: Handle<AudioSource>,
    alarm: Handle<AudioSource>,
    level_up: Handle<AudioSource>,
    game_over: Handle<AudioSource>,
}

// --- Components --------------------------------------------------------------

/// The reactor entity that owns the run's `Health`. Damage is applied to it and
/// its `HealthZeroMarker` ends the run.
#[derive(Component)]
struct Reactor;

/// Marker for the menu title text (pulses).
#[derive(Component)]
struct MenuTitle;

/// Marker for the central alarm banner shown while a gauge is red.
#[derive(Component)]
struct AlarmBanner;

/// Root of the on-screen touch vent pad; its visibility is gated on [`TouchSeen`].
#[derive(Component)]
struct TouchPad;

/// Marker for the keyboard-legend line at the bottom of the HUD. Hidden once a
/// touch is seen, since the vent pad then covers the same bottom strip.
#[derive(Component)]
struct HudLegend;

// --- Status-bar readings -----------------------------------------------------

/// Format a 0..=100 reading as a right-aligned integer percent. Shared by both
/// reading newtypes so the formatting lives in one place.
fn fmt_percent(value: f32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:>3}", value.round() as i32)
}

/// A gauge reading. The status bar stores the boxed value and hands it back to
/// `color_fn`, so the type also carries the number the threshold logic downcasts
/// to; `GaugeReading` and `HullReading` are distinct so their opposite colour
/// ramps cannot be applied to the wrong item.
#[derive(Clone, Copy)]
struct GaugeReading(f32);

impl fmt::Display for GaugeReading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_percent(self.0, f)
    }
}

/// Green below amber, amber up to red, red at/above `RED`. Returned as a closure
/// (like the crate's `status_fps_color_fn`) so its boxed `dyn Any` parameter fits
/// the `color_fn` bound without tripping clippy's free-function `Box` lints.
fn gauge_color_fn() -> impl Fn(Box<&dyn Any>) -> Option<Color> + Send + Sync + 'static {
    move |value: Box<&dyn Any>| {
        let reading = (*value).downcast_ref::<GaugeReading>()?;
        let color = if reading.0 >= RED {
            Color::srgb(1.0, 0.28, 0.28)
        } else if reading.0 >= AMBER {
            Color::srgb(1.0, 0.78, 0.2)
        } else {
            Color::srgb(0.4, 1.0, 0.5)
        };
        Some(color)
    }
}

/// Hull percent; unlike a gauge, low is bad, so the colour ramp is inverted.
#[derive(Clone, Copy)]
struct HullReading(f32);

impl fmt::Display for HullReading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_percent(self.0, f)
    }
}

fn hull_color_fn() -> impl Fn(Box<&dyn Any>) -> Option<Color> + Send + Sync + 'static {
    move |value: Box<&dyn Any>| {
        let reading = (*value).downcast_ref::<HullReading>()?;
        let color = if reading.0 <= 25.0 {
            Color::srgb(1.0, 0.28, 0.28)
        } else if reading.0 <= 55.0 {
            Color::srgb(1.0, 0.78, 0.2)
        } else {
            Color::srgb(0.4, 1.0, 0.5)
        };
        Some(color)
    }
}

/// Build a `status_bar_item` for one gauge. Each captures its `idx` and reads
/// that slot out of `ReactorState`.
fn gauge_item(idx: usize) -> impl Bundle {
    status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: move |world: &World| {
            world
                .get_resource::<ReactorState>()
                .map(|r| Arc::new(GaugeReading(r.gauges[idx])) as Arc<dyn StatusValue>)
        },
        color_fn: gauge_color_fn(),
        prefix: format!("{}:{}", idx + 1, GAUGES[idx].label),
        suffix: "%".to_string(),
    })
}

// --- Setup ------------------------------------------------------------------

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut reactor: ResMut<ReactorState>,
) {
    commands.spawn((Name::new("UI Camera"), Camera2d));

    commands.insert_resource(SfxAssets {
        menu_select: asset_server.load("sounds/menu_select.wav"),
        vent: asset_server.load("sounds/vent.wav"),
        alarm: asset_server.load("sounds/alarm.wav"),
        level_up: asset_server.load("sounds/level_up.wav"),
        game_over: asset_server.load("sounds/game_over.wav"),
    });

    // Seed the reactor once so the always-on bar shows a plausible idle console
    // behind the menu instead of all-zero gauges; `start_run` reseeds each run.
    reactor.reset(&mut rand::rng());

    // The status bar is the game board, and it is deliberately spawned once in
    // Startup and never despawned: the console stays visible in every state
    // (the whole game is the status bar). Spawn the root first so the plugin's
    // Add-observer finds it when each item below reparents itself.
    commands.spawn((status_bar(StatusBarRootConfig::default()),));

    // The four gauges, in order.
    for idx in 0..GAUGE_COUNT {
        commands.spawn((gauge_item(idx),));
    }

    // Hull integrity.
    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: |world: &World| {
            world
                .get_resource::<ReactorState>()
                .map(|r| Arc::new(HullReading(r.health)) as Arc<dyn StatusValue>)
        },
        color_fn: hull_color_fn(),
        prefix: "HULL".to_string(),
        suffix: "%".to_string(),
    }),));

    // Difficulty level.
    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: |world: &World| {
            world
                .get_resource::<ReactorState>()
                .map(|r| Arc::new(r.level + 1) as Arc<dyn StatusValue>)
        },
        color_fn: |_| Some(Color::srgb(0.7, 0.8, 1.0)),
        prefix: "LVL".to_string(),
        suffix: "".to_string(),
    }),));

    // Survival time (the score).
    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: |world: &World| {
            world
                .get_resource::<ReactorState>()
                .map(|r| Arc::new(r.elapsed.floor() as u32) as Arc<dyn StatusValue>)
        },
        color_fn: |_| Some(Color::srgb(0.95, 0.85, 0.25)),
        prefix: "TIME".to_string(),
        suffix: "s".to_string(),
    }),));

    // Frame rate, the crate's built-in item.
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

fn best_line(best: f32) -> String {
    if best > 0.0 {
        format!("Best: {}s", best.floor() as u32)
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
                screen_text("OVERLOAD", 84.0, Color::srgb(1.0, 0.5, 0.25)),
            ));
            parent.spawn(screen_text(
                "REACTOR CONTROL",
                26.0,
                Color::srgb(0.75, 0.8, 0.9),
            ));
            parent.spawn(screen_text(
                "The gauges climb on their own. Vent them before they hit the red.",
                20.0,
                Color::srgb(0.7, 0.75, 0.85),
            ));
            parent.spawn(screen_text(
                "Press 1 2 3 4 to vent HEAT PRES FLUX CHRG -- but each vent pushes another gauge up.",
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
        color.0 = Color::srgb(brightness, 0.5 * brightness + 0.1, 0.25 * brightness);
    }
}

/// Any click or key press begins a run. Reading a key here also satisfies the
/// browser's audio-unlock gesture so the first sound plays on the web build.
fn menu_start(
    mut commands: Commands,
    sfx: Res<SfxAssets>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut next: ResMut<NextState<GameState>>,
) {
    // A tap also starts, so the wasm build is enterable on a phone (winit-on-web
    // delivers taps as touches, not synthesized mouse clicks).
    let pressed = mouse.just_pressed(MouseButton::Left)
        || keys.get_just_pressed().next().is_some()
        || touches.any_just_pressed();
    if pressed {
        commands.play_sfx_volume(sfx.menu_select.clone(), 0.7);
        next.set(GameState::Playing);
    }
}

// --- Run start --------------------------------------------------------------

fn start_run(mut commands: Commands, mut reactor: ResMut<ReactorState>) {
    let mut rng = rand::rng();
    reactor.reset(&mut rng);

    commands.spawn((
        Name::new("Reactor"),
        Reactor,
        Health::new(REACTOR_HEALTH),
        DespawnOnExit(GameState::Playing),
    ));
}

fn spawn_hud(mut commands: Commands) {
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
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // The alarm banner sits centre-screen, hidden until a gauge is red.
            parent.spawn((
                AlarmBanner,
                Visibility::Hidden,
                screen_text("!! CRITICAL !!", 64.0, Color::srgb(1.0, 0.3, 0.3)),
            ));
        });

    // A quiet legend pinned to the bottom so the key mapping is always visible.
    // Hidden once a touch is seen (the vent pad covers the same strip).
    commands.spawn((
        Name::new("HUD legend"),
        HudLegend,
        DespawnOnExit(GameState::Playing),
        screen_text(
            "1 HEAT   2 PRES   3 FLUX   4 CHRG      keep them out of the red",
            20.0,
            Color::srgb(0.6, 0.65, 0.75),
        ),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(24.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
    ));
}

/// A subtle per-gauge tint for its vent button, so the four read as distinct
/// targets at a glance.
fn vent_button_tint(idx: usize) -> Color {
    match idx {
        0 => Color::srgb(1.0, 0.5, 0.35),  // HEAT -- orange
        1 => Color::srgb(0.45, 0.75, 1.0), // PRES -- blue
        2 => Color::srgb(0.6, 1.0, 0.55),  // FLUX -- green
        _ => Color::srgb(0.85, 0.6, 1.0),  // CHRG -- purple
    }
}

/// Spawn the on-screen touch vent pad: a bottom strip of four labelled buttons,
/// one per gauge, laid out over the same window fractions `vent_button_at`
/// hit-tests. Spawned hidden and revealed by `update_touch_pad` once a touch is
/// seen, so a PC session never shows it. The buttons are purely visual; the touch
/// hit-test reads the raw `Touches` against the window, so nothing here needs to
/// be queried back.
fn spawn_vent_pad(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Vent Pad"),
            TouchPad,
            DespawnOnExit(GameState::Playing),
            // Hidden until the first touch reveals it (see `TouchSeen`).
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(VENT_ZONE_H_FRAC * 100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            for (idx, spec) in GAUGES.iter().enumerate() {
                let tint = vent_button_tint(idx);
                parent
                    .spawn((
                        Name::new(format!("Vent Button {}", idx + 1)),
                        Node {
                            flex_grow: 1.0,
                            flex_basis: Val::Px(0.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            row_gap: Val::Px(2.0),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(14.0)),
                            ..default()
                        },
                        BackgroundColor(tint.with_alpha(0.22)),
                        BorderColor::all(tint.with_alpha(0.6)),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new(format!("{}", idx + 1)),
                            TextFont {
                                font_size: FontSize::Px(30.0),
                                ..default()
                            },
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.92)),
                        ));
                        button.spawn((
                            Text::new(spec.label),
                            TextFont {
                                font_size: FontSize::Px(18.0),
                                ..default()
                            },
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                        ));
                    });
            }
        });
}

/// Mark the session as touch-driven on the first touch, then reveal the vent pad
/// (and hide the keyboard legend it replaces). Runs only in Playing, where the pad
/// exists; the menu and meltdown screens read `Touches` directly to navigate.
fn update_touch_pad(
    touches: Res<Touches>,
    mut seen: ResMut<TouchSeen>,
    mut q_pad: Query<&mut Visibility, (With<TouchPad>, Without<HudLegend>)>,
    mut q_legend: Query<&mut Visibility, (With<HudLegend>, Without<TouchPad>)>,
) {
    if touches.any_just_pressed() {
        seen.0 = true;
    }
    if let Ok(mut vis) = q_pad.single_mut() {
        *vis = if seen.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = q_legend.single_mut() {
        *vis = if seen.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

// --- Simulation -------------------------------------------------------------

/// Advance the survival clock and ramp difficulty on schedule.
fn advance_run(
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<SfxAssets>,
    mut reactor: ResMut<ReactorState>,
) {
    reactor.elapsed += time.delta_secs();

    if reactor.elapsed >= reactor.next_level_at {
        reactor.level += 1;
        reactor.next_level_at += LEVEL_INTERVAL;
        reactor.recompute_climb();
        commands.play_sfx_volume(sfx.level_up.clone(), 0.6);
    }
}

/// Each gauge climbs at its level-scaled rate plus a random walk, clamped 0..MAX.
fn simulate_gauges(time: Res<Time>, mut reactor: ResMut<ReactorState>) {
    let dt = time.delta_secs();
    let mut rng = rand::rng();
    let r = &mut *reactor;
    for (gauge, &climb) in r.gauges.iter_mut().zip(r.climb.iter()) {
        let drift = rng.random_range(-DRIFT..DRIFT) * dt;
        *gauge = (*gauge + climb * dt + drift).clamp(0.0, GAUGE_MAX);
    }
}

/// Vent gauge `i`: knock it down by `VENT_AMOUNT`, push its coupled neighbour up
/// by `COUPLING`, and clamp both to 0..=GAUGE_MAX. Pure so the trade-off can be
/// unit-tested without an ECS world.
fn apply_vent(gauges: &mut [f32; GAUGE_COUNT], i: usize) {
    let partner = GAUGES[i].couples_to;
    gauges[i] = (gauges[i] - VENT_AMOUNT).max(0.0);
    gauges[partner] = (gauges[partner] + COUPLING).min(GAUGE_MAX);
}

/// Map a touch point (window logical pixels) to the vent-button column it lands
/// in, or `None` when it is above the bottom vent strip. The strip spans the full
/// width in `GAUGE_COUNT` equal columns and the bottom `VENT_ZONE_H_FRAC` of the
/// height; `spawn_vent_pad` renders the buttons over the same fractions, so this
/// is the single source of truth for the touch hit-test. Pure so the mapping can
/// be unit-tested without a window (touch positions share the window's logical
/// pixel space, exactly as `08_dropzone`'s zone split relies on).
fn vent_button_at(point: Vec2, window: Vec2) -> Option<usize> {
    if window.x <= 0.0 || window.y <= 0.0 {
        return None;
    }
    // Only the bottom strip is live.
    if point.y < window.y * (1.0 - VENT_ZONE_H_FRAC) {
        return None;
    }
    if point.x < 0.0 || point.x >= window.x {
        return None;
    }
    let col = (point.x / window.x * GAUGE_COUNT as f32) as usize;
    Some(col.min(GAUGE_COUNT - 1))
}

/// Play the vent SFX for a just-vented gauge, pitched up a touch when the gauge is
/// still in trouble. Shared by the keyboard (`vent_input`) and touch
/// (`touch_vent_input`) paths so the two input sources sound identical.
fn trigger_vent_sfx(commands: &mut Commands, sfx: &SfxAssets, gauge_value: f32) {
    let speed = if gauge_value >= AMBER { 1.15 } else { 1.0 };
    commands.trigger(
        PlaySfx::new(sfx.vent.clone())
            .with_volume(0.6)
            .with_speed(speed),
    );
}

/// Vent a gauge on its key, knocking it down but pushing its coupled neighbour up.
fn vent_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    sfx: Res<SfxAssets>,
    mut reactor: ResMut<ReactorState>,
) {
    for (i, spec) in GAUGES.iter().enumerate() {
        if keys.just_pressed(spec.key) || keys.just_pressed(spec.key_alt) {
            apply_vent(&mut reactor.gauges, i);
            trigger_vent_sfx(&mut commands, &sfx, reactor.gauges[i]);
        }
    }
}

/// Vent a gauge when a touch taps its column in the on-screen vent pad. Additive
/// to `vent_input` (keyboard): it feeds the SAME `apply_vent` and the same SFX, so
/// there is one gauge model with two input sources and the keyboard path is
/// unchanged. Reads just-pressed touches (frame-derived), so a finger still held
/// from the menu/meltdown tap that started the run never leaks a vent.
fn touch_vent_input(
    touches: Res<Touches>,
    windows: Query<&Window>,
    mut commands: Commands,
    sfx: Res<SfxAssets>,
    mut reactor: ResMut<ReactorState>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let win = Vec2::new(window.width(), window.height());
    for touch in touches.iter_just_pressed() {
        let Some(i) = vent_button_at(touch.position(), win) else {
            continue;
        };
        apply_vent(&mut reactor.gauges, i);
        trigger_vent_sfx(&mut commands, &sfx, reactor.gauges[i]);
    }
}

/// While any gauge is red, beep the alarm and drain the reactor's hull.
fn apply_danger(
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<SfxAssets>,
    mut reactor: ResMut<ReactorState>,
    q_reactor: Query<Entity, With<Reactor>>,
) {
    let reds = reactor.red_count();
    if reds == 0 {
        reactor.alarm_timer = 0.0;
        return;
    }

    // Damage scales with how many gauges are red, applied through HealthPlugin.
    if let Ok(entity) = q_reactor.single() {
        commands.trigger(HealthApplyDamage {
            entity,
            source: None,
            amount: RED_DAMAGE_PER_SEC * reds as f32 * time.delta_secs(),
        });
    }

    // Beep on a fixed interval, more urgently the more gauges are red.
    reactor.alarm_timer -= time.delta_secs();
    if reactor.alarm_timer <= 0.0 {
        reactor.alarm_timer = ALARM_INTERVAL;
        let speed = 1.0 + 0.12 * (reds - 1) as f32;
        commands.trigger(
            PlaySfx::new(sfx.alarm.clone())
                .with_volume(0.5)
                .with_speed(speed),
        );
    }
}

/// Copy the reactor's `Health` into `ReactorState` so the HULL status item can
/// read it (the status bar only sees the resource, not the component).
fn mirror_health(q: Query<&Health, With<Reactor>>, mut reactor: ResMut<ReactorState>) {
    if let Ok(health) = q.single() {
        reactor.health = health.current;
    }
}

/// Show and pulse the central alarm banner while any gauge is red.
fn update_alarm_banner(
    time: Res<Time>,
    reactor: Res<ReactorState>,
    mut q: Query<(&mut Visibility, &mut TextColor), With<AlarmBanner>>,
) {
    let red = reactor.red_count() > 0;
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
}

fn giveup_on_escape(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(GameState::GameOver);
    }
}

/// End the run when the reactor's health reaches zero.
fn on_reactor_died(
    add: On<Add, HealthZeroMarker>,
    q_reactor: Query<(), With<Reactor>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if q_reactor.contains(add.entity) && *state.get() == GameState::Playing {
        next.set(GameState::GameOver);
    }
}

// --- Game over --------------------------------------------------------------

fn record_high_score(reactor: Res<ReactorState>, mut high: ResMut<HighScore>) {
    if reactor.elapsed > high.0 {
        high.0 = reactor.elapsed;
    }
}

fn spawn_game_over(mut commands: Commands, reactor: Res<ReactorState>, high: Res<HighScore>) {
    let survived = reactor.elapsed.floor() as u32;
    let new_best = reactor.elapsed >= high.0 && reactor.elapsed > 0.0;

    commands
        .spawn((
            Name::new("Game Over"),
            DespawnOnExit(GameState::GameOver),
            centered_screen(),
        ))
        .with_children(|parent| {
            parent.spawn(screen_text("MELTDOWN", 84.0, Color::srgb(1.0, 0.35, 0.3)));
            parent.spawn(screen_text(
                format!("You held the reactor for {survived}s"),
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
    touches: Res<Touches>,
    mut next: ResMut<NextState<GameState>>,
) {
    // A tap also returns to the menu, so a phone can leave the meltdown screen.
    let pressed = mouse.just_pressed(MouseButton::Left)
        || keys.get_just_pressed().next().is_some()
        || touches.any_just_pressed();
    if pressed {
        next.set(GameState::Menu);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GREEN: Color = Color::srgb(0.4, 1.0, 0.5);
    const AMBER_C: Color = Color::srgb(1.0, 0.78, 0.2);
    const RED_C: Color = Color::srgb(1.0, 0.28, 0.28);

    fn gauge_color_at(value: f32) -> Color {
        let reading = GaugeReading(value);
        let any: &dyn Any = &reading;
        gauge_color_fn()(Box::new(any)).expect("gauge_color_fn returns a color")
    }

    fn hull_color_at(value: f32) -> Color {
        let reading = HullReading(value);
        let any: &dyn Any = &reading;
        hull_color_fn()(Box::new(any)).expect("hull_color_fn returns a color")
    }

    #[test]
    fn gauge_color_crosses_thresholds() {
        // A rising gauge goes green -> amber at AMBER, amber -> red at RED.
        assert_eq!(gauge_color_at(0.0), GREEN);
        assert_eq!(gauge_color_at(AMBER - 0.1), GREEN);
        assert_eq!(gauge_color_at(AMBER), AMBER_C);
        assert_eq!(gauge_color_at(RED - 0.1), AMBER_C);
        assert_eq!(gauge_color_at(RED), RED_C);
        assert_eq!(gauge_color_at(GAUGE_MAX), RED_C);
    }

    #[test]
    fn hull_color_is_inverted() {
        // Hull is healthy high and critical low -- the opposite ramp.
        assert_eq!(hull_color_at(100.0), GREEN);
        assert_eq!(hull_color_at(56.0), GREEN);
        assert_eq!(hull_color_at(55.0), AMBER_C);
        assert_eq!(hull_color_at(26.0), AMBER_C);
        assert_eq!(hull_color_at(25.0), RED_C);
        assert_eq!(hull_color_at(0.0), RED_C);
    }

    #[test]
    fn wrong_reading_type_yields_no_color() {
        // A downcast miss returns None so the item keeps its previous colour.
        let wrong: i64 = 5;
        let any: &dyn Any = &wrong;
        assert!(gauge_color_fn()(Box::new(any)).is_none());
    }

    #[test]
    fn readings_display_rounded_percent() {
        assert_eq!(GaugeReading(0.4).to_string().trim(), "0");
        assert_eq!(GaugeReading(72.6).to_string().trim(), "73");
        assert_eq!(GaugeReading(100.0).to_string().trim(), "100");
        // Both newtypes share `fmt_percent`, so the hull formats the same way.
        assert_eq!(HullReading(72.6).to_string().trim(), "73");
        assert_eq!(HullReading(0.0).to_string().trim(), "0");
    }

    #[test]
    fn vent_lowers_gauge_and_raises_its_partner() {
        // A vent trades the vented gauge down for its coupled neighbour up.
        let mut gauges = [50.0; GAUGE_COUNT];
        let partner = GAUGES[0].couples_to;
        apply_vent(&mut gauges, 0);
        assert!((gauges[0] - (50.0 - VENT_AMOUNT)).abs() < 1e-6);
        assert!((gauges[partner] - (50.0 + COUPLING)).abs() < 1e-6);
        // Untouched gauges (neither vented nor the partner) are unchanged.
        for (i, &v) in gauges.iter().enumerate() {
            if i != 0 && i != partner {
                assert!((v - 50.0).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn vent_and_coupling_clamp_at_bounds() {
        // Venting a nearly-empty gauge floors at 0, never negative.
        let mut gauges = [0.0; GAUGE_COUNT];
        gauges[0] = 5.0;
        apply_vent(&mut gauges, 0);
        assert_eq!(gauges[0], 0.0);

        // Coupling into an already-maxed partner caps at GAUGE_MAX.
        let mut gauges = [0.0; GAUGE_COUNT];
        let partner = GAUGES[1].couples_to;
        gauges[1] = 90.0;
        gauges[partner] = GAUGE_MAX;
        apply_vent(&mut gauges, 1);
        assert_eq!(gauges[partner], GAUGE_MAX);
    }

    #[test]
    fn recompute_climb_scales_with_level() {
        let mut state = ReactorState::default();
        state.level = 0;
        state.recompute_climb();
        for (climb, spec) in state.climb.iter().zip(GAUGES.iter()) {
            assert!((climb - spec.base_climb).abs() < 1e-6);
        }

        state.level = 3;
        state.recompute_climb();
        let scale = 1.0 + LEVEL_CLIMB_STEP * 3.0;
        for (climb, spec) in state.climb.iter().zip(GAUGES.iter()) {
            assert!((climb - spec.base_climb * scale).abs() < 1e-6);
            // Harder levels always climb faster.
            assert!(*climb > spec.base_climb);
        }
    }

    #[test]
    fn red_count_matches_gauges_over_threshold() {
        let mut state = ReactorState::default();
        state.gauges = [10.0, RED, RED + 5.0, 84.9];
        assert_eq!(state.red_count(), 2);

        state.gauges = [0.0; GAUGE_COUNT];
        assert_eq!(state.red_count(), 0);

        state.gauges = [GAUGE_MAX; GAUGE_COUNT];
        assert_eq!(state.red_count(), GAUGE_COUNT);
    }

    #[test]
    fn vent_button_hit_test_maps_columns_and_rejects_misses() {
        let win = Vec2::new(800.0, 600.0);
        // The live strip is the bottom VENT_ZONE_H_FRAC of the height.
        let strip_top = win.y * (1.0 - VENT_ZONE_H_FRAC);
        let in_strip_y = (strip_top + win.y) * 0.5;

        // Each quarter-width column maps to its gauge index, in order.
        for i in 0..GAUGE_COUNT {
            let x = (i as f32 + 0.5) / GAUGE_COUNT as f32 * win.x;
            assert_eq!(vent_button_at(Vec2::new(x, in_strip_y), win), Some(i));
        }

        // A touch above the strip is a miss even if horizontally over a column.
        assert_eq!(
            vent_button_at(Vec2::new(win.x * 0.5, strip_top - 1.0), win),
            None
        );

        // The far edges clamp into the first / last column, never out of range.
        assert_eq!(vent_button_at(Vec2::new(0.0, in_strip_y), win), Some(0));
        assert_eq!(
            vent_button_at(Vec2::new(win.x - 0.1, in_strip_y), win),
            Some(GAUGE_COUNT - 1)
        );
        // Off-window x (and a degenerate window) are misses, not panics.
        assert_eq!(vent_button_at(Vec2::new(-1.0, in_strip_y), win), None);
        assert_eq!(
            vent_button_at(Vec2::new(win.x + 1.0, in_strip_y), win),
            None
        );
        assert_eq!(vent_button_at(Vec2::new(10.0, 10.0), Vec2::ZERO), None);
    }

    #[test]
    fn coupling_forms_a_cycle_touching_every_gauge() {
        // Every gauge must be some other gauge's coupled partner, or a vent could
        // dodge the trade-off. The cycle 0->1->2->3->0 guarantees this.
        let mut targeted = [false; GAUGE_COUNT];
        for spec in &GAUGES {
            targeted[spec.couples_to] = true;
        }
        assert!(targeted.iter().all(|&t| t));
        // And no gauge couples to itself (that would be a free vent).
        for (i, spec) in GAUGES.iter().enumerate() {
            assert_ne!(i, spec.couples_to);
        }
    }
}
