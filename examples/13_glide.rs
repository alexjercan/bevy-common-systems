//! 13_glide -- a slide-merge (2048-style) number puzzle, rendered entirely in
//! Bevy UI.
//!
//! This is the headline demo of [`tween`](bevy_common_systems::tween),
//! [`ui/animate`](bevy_common_systems::ui::animate) and of
//! [`persist`](bevy_common_systems::persist) + [`HighScore`]: the whole board is
//! a UI tree, and the crate's `ui/animate` markers copy each tile's `Tween`
//! into a plain UI field -- a `Tween<Vec2>` slide into `Node.left/top`
//! ([`TweenNodeOffset`]), a `Tween<f32>` pop into the face size
//! ([`TweenNodeScale`]), and a `Tween<Vec4>` merge flash into `BackgroundColor`
//! ([`TweenNodeBackground`], built with [`node_flash`]). The score readout
//! *rolls* to its new value on another `Tween<f32>` (kept game-local: its
//! source and text format are game-specific). The best score is saved across
//! launches through `PersistPlugin::<HighScore<u32>>` -- a 2048 lives on its high
//! score, so the save primitive is load-bearing, not incidental.
//!
//! The UI structure is the reusable pattern the example teaches: a fixed-size,
//! centered board with a static cell underlay, and a separate absolutely-
//! positioned tile layer. Each tile is a positioning *wrapper* node (moved by
//! the slide tween) wrapping a *face* node (sized by the pop tween, coloured by
//! the flash tween) -- so position, scale and colour animate independently
//! without fighting over the same `Node` field, and the pop grows from the
//! centre because the wrapper centres the face. No `Transform` scale on UI is
//! used; everything animates plain `Node`/`BackgroundColor` fields, which is
//! version-robust.
//!
//! It follows the `06_fruitninja` shape: `States` for menu / playing /
//! game-over, one-shot sounds via [`SfxPlugin`], and a wasm gallery build. No 3D
//! scene is needed, so it renders with a plain `Camera2d`.
//!
//! Controls: swipe (or drag with the mouse) up / down / left / right to slide
//! the board, or use the arrow keys / WASD. Equal tiles that collide merge into
//! their sum; a new tile appears after each move. Fill the board with no legal
//! move left and the run ends. Click, tap or press any key to start and to
//! dismiss the game-over screen. Escape gives up.
//!
//! Run it: `cargo run --example 13_glide` (add `--features debug` for the
//! inspector and the headless harness).

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use clap::Parser;
use rand::Rng;

#[derive(Parser)]
#[command(name = "13_glide")]
#[command(version = "1.0.0")]
#[command(
    about = "Slide and merge numbered tiles to reach a high score.",
    long_about = None
)]
struct Cli;

// --- Tuning -----------------------------------------------------------------

/// Board side length in cells (a classic 4x4).
const BOARD_N: usize = 4;

/// Edge length of a single tile, in logical px.
const CELL: f32 = 74.0;
/// Gap between cells (and the board's outer padding), in logical px.
const GAP: f32 = 10.0;
/// Corner radius shared by the board, cells and tiles.
const RADIUS: f32 = 8.0;

/// Total board side, in px: N cells plus N+1 gaps. At N=4 this is 346px, which
/// fits a 390px-wide phone with margin (see the mobile-layout screenshot).
const BOARD_PX: f32 = BOARD_N as f32 * CELL + (BOARD_N as f32 + 1.0) * GAP;

/// Seconds a tile takes to slide to its destination cell.
const MOVE_DURATION: f32 = 0.11;
/// Seconds a spawn / merge pop takes to settle.
const POP_DURATION: f32 = 0.16;
/// Seconds a merge colour flash takes to fade back to the tile colour.
const FLASH_DURATION: f32 = 0.28;
/// Seconds the score readout takes to roll to a new value.
const SCORE_ROLL: f32 = 0.3;

/// Minimum pointer travel (px) for a drag to count as a swipe.
const SWIPE_MIN: f32 = 24.0;

/// The value at which the "you won" banner first shows (classic 2048 goal). The
/// game keeps going afterwards.
const WIN_VALUE: u32 = 2048;

// --- App --------------------------------------------------------------------

fn main() {
    let _ = Cli::parse();
    let mut app = App::new();

    let primary_window = Window {
        title: "13_glide".into(),
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

    // Headless verification harness (dev tooling, `debug` feature). Inert unless
    // BCS_AUTOPILOT / BCS_SHOT is set; see `docs/dev-harness.md`.
    #[cfg(feature = "debug")]
    {
        app.add_plugins(
            AutopilotPlugin::new()
                .hold(GameState::Menu, 0.6)
                .hold(GameState::Playing, 4.0)
                .hold(GameState::GameOver, 0.8)
                .input(|world, elapsed| {
                    // Only drive moves while playing; tapping keys in the menu or
                    // game-over screen would trip their "any key" transitions.
                    if *world.resource::<State<GameState>>().get() != GameState::Playing {
                        return;
                    }
                    // One arrow key per third of a second. `player_move` reads
                    // `just_pressed`, so reset the input each frame to re-trigger
                    // a fresh press.
                    let beat = (elapsed * 3.0) as u32;
                    let key = [
                        KeyCode::ArrowLeft,
                        KeyCode::ArrowUp,
                        KeyCode::ArrowRight,
                        KeyCode::ArrowDown,
                    ][(beat % 4) as usize];
                    let mut keys = world.resource_mut::<ButtonInput<KeyCode>>();
                    keys.reset_all();
                    keys.press(key);
                }),
        );
        app.add_plugins(ScreenshotPlugin::new(GameState::Playing).settle_frames(30));
    }

    app.add_plugins(SfxPlugin);
    app.add_plugins(TweenPlugin);
    // Drives the tile slide / pop / flash from `Tween` values into Node /
    // BackgroundColor fields (the crate's UI-animate markers).
    app.add_plugins(UiAnimatePlugin);
    app.add_plugins(PopupPlugin);
    app.add_plugins(MenuPlugin);
    app.add_plugins(UnifiedPointerPlugin);
    // Persist the best score across launches (native JSON / wasm localStorage).
    app.add_plugins(PersistPlugin::<HighScore<u32>>::new("13_glide.high_score"));

    app.insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.09)));
    app.init_resource::<Board>();
    app.init_resource::<Score>();
    app.init_resource::<MoveAnim>();
    app.init_resource::<SwipeTracker>();

    app.init_state::<GameState>();

    app.add_systems(Startup, setup);

    // Main menu.
    app.add_systems(OnEnter(GameState::Menu), spawn_menu);
    app.add_systems(Update, menu_start.run_if(in_state(GameState::Menu)));

    // Playing.
    app.add_systems(
        OnEnter(GameState::Playing),
        (start_run, spawn_board).chain(),
    );
    // Gameplay logic runs BEFORE the tween advance: `tick_move_anim` despawns
    // merged-away tiles exactly when their slide tween would complete, so it must
    // run first, or the plugin's completion handling races the despawn.
    app.add_systems(
        Update,
        (
            player_move,
            tick_move_anim,
            set_state_on_key(KeyCode::Escape, GameState::GameOver),
        )
            .chain()
            .before(TweenSystems::Advance)
            .run_if(in_state(GameState::Playing)),
    );
    // The tile slide / pop / flash appliers now come from `UiAnimatePlugin`; the
    // rolling score readout stays game-local (its source and text format are
    // game-specific -- see the harvest note).
    app.add_systems(
        Update,
        update_score_text
            .after(TweenSystems::Advance)
            .run_if(in_state(GameState::Playing)),
    );

    // Game over.
    app.add_systems(
        OnEnter(GameState::GameOver),
        (record_high_score, spawn_game_over, play_game_over_sfx).chain(),
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

// --- Sounds -----------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Sfx {
    Select,
    Merge,
    Big,
    GameOver,
}

// --- Direction and pure move logic ------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// A tile moving from one line index to another within a single row/column.
/// `merged` is true when this tile lands on a partner and the two become one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct LineMove {
    from: usize,
    to: usize,
    merged: bool,
}

/// Slide a single line toward index 0, merging equal neighbours once. Returns
/// the resulting line, the score gained (sum of every merged tile's new value),
/// and the per-tile moves needed to animate it.
fn resolve_line(line: [u32; BOARD_N]) -> ([u32; BOARD_N], u32, Vec<LineMove>) {
    let mut out = [0u32; BOARD_N];
    let mut moves = Vec::new();
    let mut gained = 0;
    let mut w = 0; // next free write slot
    let mut last_merged = false; // did out[w-1] just form from a merge?

    for (i, &v) in line.iter().enumerate() {
        if v == 0 {
            continue;
        }
        if w > 0 && out[w - 1] == v && !last_merged {
            out[w - 1] = v * 2;
            gained += v * 2;
            moves.push(LineMove {
                from: i,
                to: w - 1,
                merged: true,
            });
            last_merged = true;
        } else {
            out[w] = v;
            moves.push(LineMove {
                from: i,
                to: w,
                merged: false,
            });
            w += 1;
            last_merged = false;
        }
    }

    (out, gained, moves)
}

/// A tile move in board coordinates, ready to animate. `new_value` is the value
/// the destination cell holds after the move (doubled for a merge).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct GridMove {
    from: (usize, usize),
    to: (usize, usize),
    merged: bool,
    new_value: u32,
}

type Grid = [[u32; BOARD_N]; BOARD_N];

/// Map a line index (0 = the wall the tiles slide toward) to a board cell for a
/// given direction and line number (row for Left/Right, column for Up/Down).
fn line_to_cell(dir: Direction, line: usize, idx: usize) -> (usize, usize) {
    let last = BOARD_N - 1;
    match dir {
        Direction::Left => (line, idx),
        Direction::Right => (line, last - idx),
        Direction::Up => (idx, line),
        Direction::Down => (last - idx, line),
    }
}

/// Apply a move to the whole grid. Returns the new grid, the tile moves to
/// animate, the score gained, and whether anything actually moved.
fn apply_move(grid: &Grid, dir: Direction) -> (Grid, Vec<GridMove>, u32, bool) {
    let mut new_grid = [[0u32; BOARD_N]; BOARD_N];
    let mut moves = Vec::new();
    let mut gained = 0;

    for line in 0..BOARD_N {
        let mut input = [0u32; BOARD_N];
        for (idx, slot) in input.iter_mut().enumerate() {
            let (r, c) = line_to_cell(dir, line, idx);
            *slot = grid[r][c];
        }

        let (out, line_gained, line_moves) = resolve_line(input);
        gained += line_gained;

        for (idx, &v) in out.iter().enumerate() {
            let (r, c) = line_to_cell(dir, line, idx);
            new_grid[r][c] = v;
        }

        for m in line_moves {
            moves.push(GridMove {
                from: line_to_cell(dir, line, m.from),
                to: line_to_cell(dir, line, m.to),
                merged: m.merged,
                new_value: out[m.to],
            });
        }
    }

    let changed = new_grid != *grid;
    (new_grid, moves, gained, changed)
}

/// Is the board full with no legal move left?
fn is_game_over(grid: &Grid) -> bool {
    for row in grid {
        for &v in row {
            if v == 0 {
                return false;
            }
        }
    }
    // Any equal orthogonal neighbour means a merge is still possible.
    for r in 0..BOARD_N {
        for c in 0..BOARD_N {
            let v = grid[r][c];
            if c + 1 < BOARD_N && grid[r][c + 1] == v {
                return false;
            }
            if r + 1 < BOARD_N && grid[r + 1][c] == v {
                return false;
            }
        }
    }
    true
}

// --- Resources --------------------------------------------------------------

/// The board's logical state plus the wrapper entity occupying each cell.
#[derive(Resource, Default)]
struct Board {
    grid: Grid,
    tiles: [[Option<Entity>; BOARD_N]; BOARD_N],
    /// The board container node; tiles are spawned as its children.
    container: Option<Entity>,
    /// True once the win banner has been shown, so it fires only once.
    won: bool,
}

impl Board {
    fn reset(&mut self) {
        self.grid = [[0; BOARD_N]; BOARD_N];
        self.tiles = [[None; BOARD_N]; BOARD_N];
        self.won = false;
    }

    fn empty_cells(&self) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();
        for (r, row) in self.grid.iter().enumerate() {
            for (c, &v) in row.iter().enumerate() {
                if v == 0 {
                    cells.push((r, c));
                }
            }
        }
        cells
    }
}

/// Current run score, the value currently shown by the rolling readout, and the
/// target the current roll is animating toward (so a new merge can retarget a
/// roll that is still in flight).
#[derive(Resource, Default)]
struct Score {
    value: u32,
    shown: f32,
    roll_target: f32,
}

/// While a move's tiles are sliding, input is locked and the result is held
/// here to apply when the timer elapses.
#[derive(Resource, Default)]
struct MoveAnim {
    active: bool,
    timer: f32,
    /// Merge targets to bump on resolve: (row, col, new_value).
    merges: Vec<(usize, usize, u32)>,
    /// Source wrappers that merged away and must despawn on resolve.
    despawn: Vec<Entity>,
    gained: u32,
}

/// Tracks a pointer drag so a release can be classified as a swipe.
#[derive(Resource, Default)]
struct SwipeTracker {
    origin: Option<Vec2>,
    last: Option<Vec2>,
    prev_pressed: bool,
}

// --- Components --------------------------------------------------------------

/// A tile's positioning wrapper. Carries the slide `Tween<Vec2>` and points at
/// its face / text children so a merge can update them.
#[derive(Component)]
struct Tile {
    value: u32,
    face: Entity,
    text: Entity,
}

/// The score number in the HUD (rolls via a `Tween<f32>`).
#[derive(Component)]
struct ScoreText;

/// The menu title (pulses via `TitlePulse`).
#[derive(Component)]
struct MenuTitle;

// --- Geometry helpers -------------------------------------------------------

/// Top-left px of cell (row, col) within the board container.
fn cell_to_px(row: usize, col: usize) -> Vec2 {
    Vec2::new(
        GAP + col as f32 * (CELL + GAP),
        GAP + row as f32 * (CELL + GAP),
    )
}

/// Tile background colour by value: cool low tiers warming to bright as the
/// value climbs, matching the crate's neon-on-dark look.
fn tile_color(value: u32) -> Color {
    match value {
        0 => Color::srgb(0.12, 0.14, 0.20),
        2 => Color::srgb(0.20, 0.30, 0.45),
        4 => Color::srgb(0.20, 0.42, 0.52),
        8 => Color::srgb(0.20, 0.52, 0.45),
        16 => Color::srgb(0.28, 0.58, 0.35),
        32 => Color::srgb(0.52, 0.60, 0.28),
        64 => Color::srgb(0.70, 0.55, 0.25),
        128 => Color::srgb(0.80, 0.48, 0.28),
        256 => Color::srgb(0.85, 0.40, 0.32),
        512 => Color::srgb(0.88, 0.34, 0.40),
        1024 => Color::srgb(0.80, 0.30, 0.55),
        _ => Color::srgb(0.65, 0.30, 0.75),
    }
}

/// Dark text on the pale low tiers, near-white on the saturated high tiers.
fn text_color(value: u32) -> Color {
    if value <= 4 {
        Color::srgb(0.85, 0.90, 0.98)
    } else {
        Color::srgb(0.98, 0.98, 1.0)
    }
}

/// Smaller font as the number gets more digits.
fn tile_font(value: u32) -> f32 {
    match value {
        0..=64 => 34.0,
        128..=512 => 28.0,
        _ => 22.0,
    }
}

// --- Setup ------------------------------------------------------------------

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Name::new("UI Camera"), Camera2d));

    commands.insert_resource(SoundBank::load(
        &asset_server,
        [
            (Sfx::Select, "menu_select"),
            (Sfx::Merge, "pickup"),
            (Sfx::Big, "golden"),
            (Sfx::GameOver, "game_over"),
        ],
    ));
}

fn best_line(best: u32) -> String {
    if best > 0 {
        format!("Best: {best}")
    } else {
        "No run yet".to_string()
    }
}

// --- Menu -------------------------------------------------------------------

fn spawn_menu(mut commands: Commands, high: Res<HighScore<u32>>) {
    commands
        .spawn((
            Name::new("Menu"),
            DespawnOnExit(GameState::Menu),
            centered_screen(),
        ))
        .with_children(|parent| {
            parent.spawn((
                MenuTitle,
                screen_text("GLIDE", 84.0, Color::srgb(0.45, 0.75, 0.95)),
                TitlePulse::new(Color::srgb(0.45, 0.75, 0.95)),
            ));
            parent.spawn(screen_text(
                "SLIDE - MERGE - CLIMB",
                26.0,
                Color::srgb(0.75, 0.82, 0.92),
            ));
            parent.spawn(screen_text(
                "Swipe or use the arrow keys to slide every tile. Equal tiles merge.",
                20.0,
                Color::srgb(0.7, 0.76, 0.86),
            ));
            parent.spawn(screen_text(
                best_line(high.best()),
                24.0,
                Color::srgb(0.95, 0.85, 0.35),
            ));
            parent.spawn(screen_text(
                "Click, tap or press any key to begin",
                24.0,
                Color::srgb(0.9, 0.9, 0.9),
            ));
        });
}

/// Any click / key / tap begins a run (and satisfies the browser audio-unlock
/// gesture for the web build).
fn menu_start(
    mut commands: Commands,
    sfx: Res<SoundBank<Sfx>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut next: ResMut<NextState<GameState>>,
) {
    let pressed = mouse.just_pressed(MouseButton::Left)
        || keys.get_just_pressed().next().is_some()
        || touches.any_just_pressed();
    if pressed {
        commands.play_sfx_volume(sfx.get(Sfx::Select), 0.7);
        next.set(GameState::Playing);
    }
}

// --- Run start / board build ------------------------------------------------

fn start_run(mut board: ResMut<Board>, mut score: ResMut<Score>, mut anim: ResMut<MoveAnim>) {
    board.reset();
    *score = Score::default();
    *anim = MoveAnim::default();
}

fn spawn_board(mut commands: Commands, mut board: ResMut<Board>, high: Res<HighScore<u32>>) {
    // Root: fills the screen and centres the board plus the HUD above it.
    commands
        .spawn((
            Name::new("Play Root"),
            DespawnOnExit(GameState::Playing),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
        ))
        .with_children(|root| {
            // HUD row: score (rolling) and best.
            root.spawn(Node {
                width: Val::Px(BOARD_PX),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|hud| {
                hud.spawn((
                    ScoreText,
                    screen_text("0", 40.0, Color::srgb(0.95, 0.95, 1.0)),
                ));
                hud.spawn(screen_text(
                    best_line(high.best()),
                    24.0,
                    Color::srgb(0.8, 0.8, 0.9),
                ));
            });

            // The board container: a fixed square with the cell underlay, and
            // the tile layer added as children.
            let container = root
                .spawn((
                    Name::new("Board"),
                    Node {
                        width: Val::Px(BOARD_PX),
                        height: Val::Px(BOARD_PX),
                        position_type: PositionType::Relative,
                        border_radius: BorderRadius::all(Val::Px(RADIUS + 2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.09, 0.10, 0.15)),
                ))
                .with_children(|b| {
                    // Static cell underlay.
                    for r in 0..BOARD_N {
                        for c in 0..BOARD_N {
                            let p = cell_to_px(r, c);
                            b.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(p.x),
                                    top: Val::Px(p.y),
                                    width: Val::Px(CELL),
                                    height: Val::Px(CELL),
                                    border_radius: BorderRadius::all(Val::Px(RADIUS)),
                                    ..default()
                                },
                                BackgroundColor(tile_color(0)),
                            ));
                        }
                    }
                })
                .id();

            board.container = Some(container);
        });

    // Two starting tiles.
    let container = board.container.expect("board container just spawned");
    let mut rng = rand::rng();
    for _ in 0..2 {
        spawn_random_tile(&mut commands, &mut board, container, &mut rng);
    }
}

// --- Tile spawning ----------------------------------------------------------

/// Spawn a tile at (row, col) with the given value, popping in from `pop_from`
/// (as a fraction of full size). Records it in the board's tile map.
fn spawn_tile(
    commands: &mut Commands,
    board: &mut Board,
    container: Entity,
    row: usize,
    col: usize,
    value: u32,
    pop_from: f32,
) {
    let p = cell_to_px(row, col);

    let text = commands
        .spawn((
            Text::new(value.to_string()),
            TextFont {
                font_size: FontSize::Px(tile_font(value)),
                ..default()
            },
            TextColor(text_color(value)),
            TextLayout {
                justify: Justify::Center,
                ..default()
            },
        ))
        .id();

    let face = commands
        .spawn((
            Node {
                width: Val::Percent(pop_from * 100.0),
                height: Val::Percent(pop_from * 100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(RADIUS)),
                ..default()
            },
            BackgroundColor(tile_color(value)),
            // The crate's UI-animate markers: the pop `Tween<f32>` drives the
            // face size, and a merge flash `Tween<Vec4>` (inserted on merge) drives
            // its background. `UiAnimatePlugin` applies both after the tween ticks.
            TweenNodeScale,
            TweenNodeBackground,
            Tween::new(pop_from, 1.0, POP_DURATION, EaseFunction::BackOut)
                .with_on_complete(TweenOnComplete::Keep),
        ))
        .add_child(text)
        .id();

    let wrapper = commands
        .spawn((
            Tile { value, face, text },
            // A slide `Tween<Vec2>` (inserted on a move) drives left/top via the
            // crate's `TweenNodeOffset` marker.
            TweenNodeOffset,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(p.x),
                top: Val::Px(p.y),
                width: Val::Px(CELL),
                height: Val::Px(CELL),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .add_child(face)
        .id();

    commands.entity(container).add_child(wrapper);
    board.tiles[row][col] = Some(wrapper);
}

/// Spawn a `2` (90%) or `4` (10%) in a random empty cell. No-op if full.
fn spawn_random_tile(
    commands: &mut Commands,
    board: &mut Board,
    container: Entity,
    rng: &mut impl Rng,
) {
    let empties = board.empty_cells();
    let Some(&(r, c)) = empties.get(rng.random_range(0..empties.len().max(1))) else {
        return;
    };
    if board.grid[r][c] != 0 {
        return;
    }
    let value = if rng.random_bool(0.1) { 4 } else { 2 };
    board.grid[r][c] = value;
    spawn_tile(commands, board, container, r, c, value, 0.1);
}

// --- Input ------------------------------------------------------------------

/// Read a swipe / arrow key into a direction and, if it changes the board, kick
/// off the slide animation. Locked while a move is animating.
fn player_move(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    pointer: Res<UnifiedPointer>,
    mut tracker: ResMut<SwipeTracker>,
    mut board: ResMut<Board>,
    mut anim: ResMut<MoveAnim>,
) {
    // Track the drag regardless, so the release edge is always clean.
    if pointer.just_pressed {
        tracker.origin = pointer.screen_pos;
        tracker.last = pointer.screen_pos;
    }
    if pointer.pressed {
        if let Some(p) = pointer.screen_pos {
            tracker.last = Some(p);
        }
    }
    let released = tracker.prev_pressed && !pointer.pressed;
    tracker.prev_pressed = pointer.pressed;

    // Classify a released drag into a swipe direction, then clear the drag state
    // unconditionally (so a keyboard press on the same frame never leaves a stale
    // origin behind). Screen y grows downward, so a downward drag is Down.
    let mut swipe_dir = None;
    if released {
        if let (Some(o), Some(e)) = (tracker.origin, tracker.last) {
            let d = e - o;
            if d.length() >= SWIPE_MIN {
                swipe_dir = Some(if d.x.abs() > d.y.abs() {
                    if d.x > 0.0 {
                        Direction::Right
                    } else {
                        Direction::Left
                    }
                } else if d.y > 0.0 {
                    Direction::Down
                } else {
                    Direction::Up
                });
            }
        }
        tracker.origin = None;
        tracker.last = None;
    }

    if anim.active {
        return;
    }

    // Keyboard takes precedence over a same-frame swipe.
    let key_dir = if keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        Some(Direction::Up)
    } else if keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        Some(Direction::Down)
    } else if keys.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        Some(Direction::Left)
    } else if keys.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        Some(Direction::Right)
    } else {
        None
    };

    let Some(dir) = key_dir.or(swipe_dir) else {
        return;
    };

    let (new_grid, moves, gained, changed) = apply_move(&board.grid, dir);
    if !changed {
        return;
    }

    start_move(
        &mut commands,
        &mut board,
        &mut anim,
        new_grid,
        moves,
        gained,
    );
}

/// The entity-level plan derived from a move's `GridMove` list: which tiles just
/// slide and survive, which merge away (despawn), and which cells become a merge
/// result (and to what value). Kept pure and separate from the ECS so it can be
/// unit-tested -- the bit that used to be wrong (see the review) lives here.
#[derive(Default, Debug, PartialEq, Eq)]
struct MovePlan {
    /// (from, to) for each tile that slides and stays alive.
    survivors: Vec<((usize, usize), (usize, usize))>,
    /// Source cell of each tile that merges into another and is despawned.
    despawns: Vec<(usize, usize)>,
    /// (row, col, new_value) for each cell that becomes a merge result.
    merges: Vec<(usize, usize, u32)>,
}

/// Classify the per-tile moves. `resolve_line` always emits the base
/// (`merged:false`) move to a cell before the incoming (`merged:true`) move, so
/// the base is the survivor and the incoming despawns; the merged move carries
/// the doubled `new_value` to bump the survivor to.
fn classify_moves(moves: &[GridMove]) -> MovePlan {
    let mut plan = MovePlan::default();
    for m in moves {
        if m.merged {
            plan.despawns.push(m.from);
            plan.merges.push((m.to.0, m.to.1, m.new_value));
        } else {
            plan.survivors.push((m.from, m.to));
        }
    }
    plan
}

/// Attach slide tweens to every moving tile and stage the merge results.
fn start_move(
    commands: &mut Commands,
    board: &mut Board,
    anim: &mut MoveAnim,
    new_grid: Grid,
    moves: Vec<GridMove>,
    gained: u32,
) {
    let plan = classify_moves(&moves);

    // Every moving tile (survivor or merged-away) slides to its destination cell.
    for m in &moves {
        let entity = board.tiles[m.from.0][m.from.1].expect("a move's source cell holds a tile");
        let start = cell_to_px(m.from.0, m.from.1);
        let end = cell_to_px(m.to.0, m.to.1);
        commands.entity(entity).insert(
            Tween::new(start, end, MOVE_DURATION, EaseFunction::QuadraticOut)
                .with_on_complete(TweenOnComplete::Keep),
        );
    }

    // Rebuild the tile map from the survivors; collect the tiles to despawn.
    let mut new_tiles: [[Option<Entity>; BOARD_N]; BOARD_N] = [[None; BOARD_N]; BOARD_N];
    for (from, to) in &plan.survivors {
        new_tiles[to.0][to.1] = board.tiles[from.0][from.1];
    }
    let despawn: Vec<Entity> = plan
        .despawns
        .iter()
        .map(|c| board.tiles[c.0][c.1].expect("a merged-away tile exists"))
        .collect();
    let merges = plan.merges;

    board.grid = new_grid;
    board.tiles = new_tiles;

    anim.active = true;
    anim.timer = MOVE_DURATION;
    anim.merges = merges;
    anim.despawn = despawn;
    anim.gained = gained;
}

// --- Move resolution --------------------------------------------------------

fn tick_move_anim(
    mut commands: Commands,
    time: Res<Time>,
    mut board: ResMut<Board>,
    mut anim: ResMut<MoveAnim>,
    mut score: ResMut<Score>,
    mut tiles: Query<&mut Tile>,
    sfx: Res<SoundBank<Sfx>>,
    windows: Query<&Window>,
    mut next: ResMut<NextState<GameState>>,
) {
    if !anim.active {
        return;
    }
    anim.timer -= time.delta_secs();
    if anim.timer > 0.0 {
        return;
    }

    // Despawn the tiles that merged away.
    for e in anim.despawn.drain(..) {
        commands.entity(e).despawn();
    }

    // Bump each surviving merge target: new value, pop, colour flash, sound.
    let merges = std::mem::take(&mut anim.merges);
    let mut biggest = 0u32;
    for (r, c, new_value) in merges {
        biggest = biggest.max(new_value);
        if let Some(entity) = board.tiles[r][c] {
            if let Ok(mut tile) = tiles.get_mut(entity) {
                tile.value = new_value;
                // Pop the face (overshoot 1.25 -> 1.0) and flash it white, and
                // refresh its text.
                commands.entity(tile.face).insert((
                    Tween::new(1.25, 1.0, POP_DURATION, EaseFunction::QuadraticOut)
                        .with_on_complete(TweenOnComplete::Keep),
                    node_flash(tile_color(new_value), FLASH_DURATION),
                ));
                commands.entity(tile.text).insert(text_bundle(new_value));
            }
        }
    }

    if biggest > 0 {
        let key = if biggest >= 128 { Sfx::Big } else { Sfx::Merge };
        commands.play_sfx_volume(sfx.get(key), 0.55);
    }

    // Score, plus a floating "+N" popup near the top of the board (ui/popup
    // wants a screen-space point, so anchor it to the window centre).
    let gained = anim.gained;
    if gained > 0 {
        score.value += gained;
        // The rolling readout picks up the new `score.value` in update_score_text.
        if let Ok(window) = windows.single() {
            let pos = Vec2::new(window.width() * 0.5, window.height() * 0.32);
            let color = if biggest >= 128 {
                Color::srgb(0.95, 0.8, 0.35)
            } else {
                Color::srgb(0.9, 0.95, 1.0)
            };
            // No DespawnOnExit: the popup self-despawns via its own tween, so it
            // never races a state-exit despawn with the plugin's own cleanup.
            commands.spawn(popup(pos, format!("+{gained}"), 30.0, color));
        }
    }

    // A fresh tile after every successful move.
    if let Some(container) = board.container {
        let mut rng = rand::rng();
        spawn_random_tile(&mut commands, &mut board, container, &mut rng);
    }

    // Win banner once (the run continues afterwards), game over when stuck.
    if !board.won && biggest >= WIN_VALUE {
        board.won = true;
        if let Ok(window) = windows.single() {
            let pos = Vec2::new(window.width() * 0.5, window.height() * 0.5);
            commands.spawn(popup(
                pos,
                format!("{WIN_VALUE}!"),
                72.0,
                Color::srgb(0.98, 0.85, 0.35),
            ));
        }
    }
    if is_game_over(&board.grid) {
        next.set(GameState::GameOver);
    }

    anim.active = false;
    anim.gained = 0;
}

fn text_bundle(value: u32) -> impl Bundle {
    (
        Text::new(value.to_string()),
        TextFont {
            font_size: FontSize::Px(tile_font(value)),
            ..default()
        },
        TextColor(text_color(value)),
        TextLayout {
            justify: Justify::Center,
            ..default()
        },
    )
}

// --- Score readout ----------------------------------------------------------

/// Roll the HUD score toward `score.value`. When the target changes, (re)attach
/// a `Tween<f32>` from the currently shown value; each frame we read it back.
fn update_score_text(
    mut commands: Commands,
    mut score: ResMut<Score>,
    q: Query<(Entity, Option<&Tween<f32>>), With<ScoreText>>,
    mut texts: Query<&mut Text, With<ScoreText>>,
) {
    let Ok((entity, tween)) = q.single() else {
        return;
    };

    // Advance the shown value from the live tween, if any.
    if let Some(t) = tween {
        score.shown = t.value();
    }

    // Retarget the roll whenever the score changes -- even mid-roll -- so a fast
    // merge streak tracks live instead of stalling behind an in-flight tween.
    // Gated on the target actually changing, so it does not re-insert per frame.
    let target = score.value as f32;
    if (target - score.roll_target).abs() > 0.5 {
        score.roll_target = target;
        commands.entity(entity).insert(
            Tween::new(score.shown, target, SCORE_ROLL, EaseFunction::QuadraticOut)
                .with_on_complete(TweenOnComplete::Keep),
        );
    }

    if let Ok(mut text) = texts.single_mut() {
        let shown = score.shown.round().max(0.0) as u32;
        text.0 = shown.to_string();
    }
}

// --- Game over --------------------------------------------------------------

fn record_high_score(score: Res<Score>, mut high: ResMut<HighScore<u32>>) {
    high.record(score.value);
}

fn spawn_game_over(mut commands: Commands, score: Res<Score>, high: Res<HighScore<u32>>) {
    let new_best = high.is_new_best();
    commands
        .spawn((
            Name::new("Game Over"),
            DespawnOnExit(GameState::GameOver),
            centered_screen(),
        ))
        .with_children(|parent| {
            parent.spawn(screen_text(
                "NO MOVES LEFT",
                72.0,
                Color::srgb(0.9, 0.45, 0.5),
            ));
            parent.spawn(screen_text(
                format!("Score: {}", score.value),
                34.0,
                Color::srgb(0.95, 0.95, 1.0),
            ));
            if new_best {
                parent.spawn(screen_text(
                    "New best!",
                    28.0,
                    Color::srgb(0.95, 0.85, 0.35),
                ));
            } else {
                parent.spawn(screen_text(
                    best_line(high.best()),
                    26.0,
                    Color::srgb(0.8, 0.8, 0.9),
                ));
            }
            parent.spawn(screen_text(
                "Click or press any key for the menu",
                22.0,
                Color::srgb(0.7, 0.76, 0.86),
            ));
        });
}

fn play_game_over_sfx(mut commands: Commands, sfx: Res<SoundBank<Sfx>>) {
    commands.play_sfx_volume(sfx.get(Sfx::GameOver), 0.8);
}

fn gameover_dismiss(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut next: ResMut<NextState<GameState>>,
) {
    let pressed = mouse.just_pressed(MouseButton::Left)
        || keys.get_just_pressed().next().is_some()
        || touches.any_just_pressed();
    if pressed {
        next.set(GameState::Menu);
    }
}

// --- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_line_does_nothing() {
        let (out, gained, moves) = resolve_line([0, 0, 0, 0]);
        assert_eq!(out, [0, 0, 0, 0]);
        assert_eq!(gained, 0);
        assert!(moves.is_empty());
    }

    #[test]
    fn single_tile_slides_to_wall() {
        let (out, gained, moves) = resolve_line([0, 0, 2, 0]);
        assert_eq!(out, [2, 0, 0, 0]);
        assert_eq!(gained, 0);
        assert_eq!(
            moves,
            vec![LineMove {
                from: 2,
                to: 0,
                merged: false
            }]
        );
    }

    #[test]
    fn pair_merges_and_scores() {
        let (out, gained, _) = resolve_line([2, 2, 0, 0]);
        assert_eq!(out, [4, 0, 0, 0]);
        assert_eq!(gained, 4);
    }

    #[test]
    fn four_equal_makes_two_pairs() {
        let (out, gained, _) = resolve_line([2, 2, 2, 2]);
        assert_eq!(out, [4, 4, 0, 0]);
        assert_eq!(gained, 8);
    }

    #[test]
    fn no_triple_merge() {
        // The freshly merged 8 must not merge again with the trailing 8.
        let (out, gained, _) = resolve_line([4, 4, 8, 0]);
        assert_eq!(out, [8, 8, 0, 0]);
        assert_eq!(gained, 8);
    }

    #[test]
    fn apply_move_left_and_right_are_mirror() {
        let grid = [[2, 2, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];
        let (left, _, gained_l, changed_l) = apply_move(&grid, Direction::Left);
        assert_eq!(left[0], [4, 0, 0, 0]);
        assert_eq!(gained_l, 4);
        assert!(changed_l);

        let (right, _, gained_r, changed_r) = apply_move(&grid, Direction::Right);
        assert_eq!(right[0], [0, 0, 0, 4]);
        assert_eq!(gained_r, 4);
        assert!(changed_r);
    }

    #[test]
    fn apply_move_reports_unchanged() {
        let grid = [[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]];
        let (_, _, _, changed) = apply_move(&grid, Direction::Left);
        assert!(!changed);
        // A fully checkerboarded full board with no equal neighbours is stuck.
        assert!(is_game_over(&grid));
    }

    #[test]
    fn game_over_only_when_full_and_locked() {
        let has_gap = [[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 0], [4, 2, 4, 2]];
        assert!(!is_game_over(&has_gap));

        let mergeable = [[2, 2, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]];
        assert!(!is_game_over(&mergeable));
    }

    // The move -> entity classification is the bit that was wrong in review; assert
    // it directly rather than only the pure grid output.

    #[test]
    fn merge_plan_bumps_survivor_and_despawns_incoming() {
        // Row 0 = [2, 2, 0, 0] sliding Left: the two tiles collide into a 4.
        let grid = [[2, 2, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];
        let (_, moves, _, _) = apply_move(&grid, Direction::Left);
        let plan = classify_moves(&moves);

        // Exactly one merge target at (0,0) with the doubled value, one tile
        // despawned, and the surviving base tile slides to (0,0).
        assert_eq!(plan.merges, vec![(0, 0, 4)]);
        assert_eq!(plan.despawns.len(), 1);
        assert_eq!(plan.survivors.len(), 1);
        assert_eq!(plan.survivors[0].1, (0, 0));
    }

    #[test]
    fn merge_plan_mixes_a_merge_and_a_plain_slide() {
        // Row 0 = [0, 2, 2, 2] Left -> [4, 2, 0, 0]: one merge at (0,0) and the
        // trailing lone 2 just slides to (0,1).
        let grid = [[0, 2, 2, 2], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];
        let (new_grid, moves, gained, _) = apply_move(&grid, Direction::Left);
        assert_eq!(new_grid[0], [4, 2, 0, 0]);
        assert_eq!(gained, 4);

        let plan = classify_moves(&moves);
        assert_eq!(plan.merges, vec![(0, 0, 4)]);
        assert_eq!(plan.despawns.len(), 1);
        // Two survivors: the merge base -> (0,0) and the lone tile -> (0,1).
        assert_eq!(plan.survivors.len(), 2);
        assert!(plan.survivors.iter().any(|(_, to)| *to == (0, 1)));
    }

    #[test]
    fn plain_slide_has_no_merges_or_despawns() {
        // A single tile sliding has no merge bookkeeping at all.
        let grid = [[0, 0, 2, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];
        let (_, moves, _, _) = apply_move(&grid, Direction::Left);
        let plan = classify_moves(&moves);
        assert!(plan.merges.is_empty());
        assert!(plan.despawns.is_empty());
        assert_eq!(plan.survivors, vec![((0, 2), (0, 0))]);
    }
}
