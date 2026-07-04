//! Reveal-on-first-touch gating and pure hit-test primitives for on-screen
//! touch controls.
//!
//! Every touch-playable game in this crate hand-rolls the same two things: a
//! way to show an on-screen control pad only once a finger has actually touched
//! the screen (so a desktop/mouse session never sees it, with no
//! `#[cfg(wasm32)]` or `navigator.maxTouchPoints` sniffing), and a pure mapping
//! from a touch position to "which button/zone did it land in". This module
//! owns those PRIMITIVES -- deliberately not an opinionated pad widget: the
//! game still builds and lays out its own buttons, this just gates their
//! visibility and answers the hit-test.
//!
//! Two pieces:
//!
//! - [`TouchpadPlugin`] maintains a [`TouchSeen`] resource (flipped true on the
//!   first [`Touches::any_just_pressed`], never reset) and drives the
//!   [`Visibility`] of entities tagged [`RevealOnTouch`] (shown once touched)
//!   and [`HideOnTouch`] (hidden once touched, e.g. a keyboard legend the pad
//!   replaces).
//! - [`button_grid_at`] and [`stick_deflection`] are pure, window-fraction hit
//!   tests you call from your own input system. Because touch positions live in
//!   the window's logical-pixel space, they are unit-testable without a window.
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(TouchpadPlugin);
//!
//! // Spawn your pad root tagged so it is revealed on first touch:
//! fn spawn_pad(mut commands: Commands) {
//!     commands.spawn((Node::default(), RevealOnTouch));
//! }
//!
//! // Hit-test a touch against a 4-column bottom strip:
//! fn on_touch(touches: Res<Touches>, windows: Query<&Window>) {
//!     let Ok(window) = windows.single() else { return };
//!     let size = window.size();
//!     let zone = Rect::new(0.0, 0.84, 1.0, 1.0); // bottom 16% of the screen
//!     for touch in touches.iter_just_pressed() {
//!         if let Some(col) = button_grid_at(touch.position(), size, 4, 1, zone) {
//!             let _ = col; // ... vent gauge `col` ...
//!         }
//!     }
//! }
//! ```

use bevy::prelude::*;

pub mod prelude {
    pub use super::{
        button_grid_at, stick_deflection, HideOnTouch, RevealOnTouch, TouchSeen, TouchpadPlugin,
        TouchpadSystems,
    };
}

/// System sets for the touchpad plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TouchpadSystems {
    /// Flips [`TouchSeen`] on the first touch and applies the reveal/hide
    /// visibility to tagged roots. Runs in `PreUpdate`.
    Reveal,
}

/// True once any touch has been seen this session, and never reset.
///
/// [`TouchpadPlugin`] flips it the first frame [`Touches::any_just_pressed`] is
/// true. Read it to branch touch-only behaviour; the plugin already uses it to
/// gate [`RevealOnTouch`] / [`HideOnTouch`] roots.
#[derive(Resource, Default, Debug, Clone, Copy, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub struct TouchSeen(pub bool);

/// Marks a UI root to be hidden until the first touch, then revealed.
///
/// The plugin keeps it [`Visibility::Hidden`] until [`TouchSeen`] flips, then
/// sets it [`Visibility::Visible`]. Put it on an on-screen control pad so a
/// desktop session never sees it and a touch device reveals it on first contact.
#[derive(Component, Default, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct RevealOnTouch;

/// Marks a UI root to be hidden once the first touch is seen.
///
/// The inverse of [`RevealOnTouch`]: the plugin leaves it [`Visibility::Inherited`]
/// until [`TouchSeen`] flips, then sets it [`Visibility::Hidden`]. Put it on a
/// keyboard-hint legend that an on-screen pad replaces on touch devices.
#[derive(Component, Default, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct HideOnTouch;

/// Map a point (window logical pixels) to the flat index of the grid cell it
/// lands in, or `None` when it is outside the active zone.
///
/// `zone` is a rectangle in window *fractions* (0..1 on each axis); it is scaled
/// by `window` (the window size in logical pixels) to get the live area, which
/// is split into a `cols` by `rows` grid. The returned index is row-major
/// (`row * cols + col`), so a single-row strip returns the column directly. A
/// point exactly on the far edge clamps into the last cell rather than spilling
/// out of range; a point outside the zone, or a degenerate window/grid, is
/// `None` (never a panic).
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_common_systems::prelude::*;
/// let window = Vec2::new(800.0, 600.0);
/// // A full-width strip across the bottom 16%, split into 4 columns.
/// let zone = Rect::new(0.0, 0.84, 1.0, 1.0);
/// // A touch in the third column of the strip.
/// let p = Vec2::new(800.0 * 0.625, 590.0);
/// assert_eq!(button_grid_at(p, window, 4, 1, zone), Some(2));
/// // A touch above the strip is a miss.
/// assert_eq!(button_grid_at(Vec2::new(400.0, 100.0), window, 4, 1, zone), None);
/// ```
pub fn button_grid_at(
    point: Vec2,
    window: Vec2,
    cols: usize,
    rows: usize,
    zone: Rect,
) -> Option<usize> {
    if window.x <= 0.0 || window.y <= 0.0 || cols == 0 || rows == 0 {
        return None;
    }
    let min = zone.min * window;
    let max = zone.max * window;
    let size = max - min;
    if size.x <= 0.0 || size.y <= 0.0 {
        return None;
    }
    if point.x < min.x || point.x >= max.x || point.y < min.y || point.y >= max.y {
        return None;
    }
    let col = (((point.x - min.x) / size.x) * cols as f32) as usize;
    let row = (((point.y - min.y) / size.y) * rows as f32) as usize;
    Some(row.min(rows - 1) * cols + col.min(cols - 1))
}

/// Map a stick deflection (finger offset from a floating origin, logical pixels)
/// to a normalized deflection vector in the unit disc, with a dead zone.
///
/// Returns `Vec2::ZERO` inside `dead`, and otherwise the direction of `offset`
/// scaled by a magnitude that ramps from 0 at the dead-zone edge to 1 at
/// `radius` (clamped past it), so the result never leaves the unit disc. The
/// caller applies its own per-axis sign and scale (screen +y grows downward, so
/// a game that pitches back on drag-down negates or keeps y as it needs).
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_common_systems::prelude::*;
/// let (radius, dead) = (100.0, 10.0);
/// // Inside the dead zone: no deflection.
/// assert_eq!(stick_deflection(Vec2::new(5.0, 0.0), radius, dead), Vec2::ZERO);
/// // At the radius: full deflection along the drag direction.
/// assert_eq!(stick_deflection(Vec2::new(radius, 0.0), radius, dead), Vec2::X);
/// // Past the radius: still clamped to the unit disc.
/// assert!(stick_deflection(Vec2::new(radius * 3.0, 0.0), radius, dead).length() <= 1.0 + 1e-4);
/// ```
pub fn stick_deflection(offset: Vec2, radius: f32, dead: f32) -> Vec2 {
    let len = offset.length();
    if len <= dead || radius <= dead {
        return Vec2::ZERO;
    }
    let dir = offset / len;
    let mag = ((len - dead) / (radius - dead)).clamp(0.0, 1.0);
    dir * mag
}

/// Maintains [`TouchSeen`] and applies the touch-reveal visibility to tagged
/// roots.
pub struct TouchpadPlugin;

impl Plugin for TouchpadPlugin {
    fn build(&self, app: &mut App) {
        debug!("TouchpadPlugin: build");

        app.init_resource::<TouchSeen>()
            .register_type::<TouchSeen>()
            .register_type::<RevealOnTouch>()
            .register_type::<HideOnTouch>()
            .add_systems(
                PreUpdate,
                (mark_touch_seen, apply_touch_reveal)
                    .chain()
                    .in_set(TouchpadSystems::Reveal),
            );
    }
}

/// Flip [`TouchSeen`] the first frame any touch begins.
fn mark_touch_seen(touches: Res<Touches>, mut seen: ResMut<TouchSeen>) {
    if !seen.0 && touches.any_just_pressed() {
        trace!("mark_touch_seen: first touch");
        seen.0 = true;
    }
}

/// Reveal [`RevealOnTouch`] roots and hide [`HideOnTouch`] roots once a touch has
/// been seen. Writes are guarded on a change so this stays free of per-frame
/// change-detection churn, and it runs every frame so a root spawned after the
/// first touch is still gated correctly.
fn apply_touch_reveal(
    seen: Res<TouchSeen>,
    mut q_reveal: Query<&mut Visibility, (With<RevealOnTouch>, Without<HideOnTouch>)>,
    mut q_hide: Query<&mut Visibility, (With<HideOnTouch>, Without<RevealOnTouch>)>,
) {
    let reveal = if seen.0 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut vis in &mut q_reveal {
        if *vis != reveal {
            *vis = reveal;
        }
    }

    let hide = if seen.0 {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut vis in &mut q_hide {
        if *vis != hide {
            *vis = hide;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_grid_maps_columns_and_rejects_misses() {
        let win = Vec2::new(800.0, 600.0);
        let h_frac = 0.16;
        let zone = Rect::new(0.0, 1.0 - h_frac, 1.0, 1.0);
        let strip_top = win.y * (1.0 - h_frac);
        let in_strip_y = (strip_top + win.y) * 0.5;
        let cols = 4;

        // Each quarter-width column maps to its index, in order.
        for i in 0..cols {
            let x = (i as f32 + 0.5) / cols as f32 * win.x;
            assert_eq!(
                button_grid_at(Vec2::new(x, in_strip_y), win, cols, 1, zone),
                Some(i)
            );
        }

        // A point above the strip is a miss even if horizontally over a column.
        assert_eq!(
            button_grid_at(Vec2::new(win.x * 0.5, strip_top - 1.0), win, cols, 1, zone),
            None
        );

        // The far edges clamp into the first / last column, never out of range.
        assert_eq!(
            button_grid_at(Vec2::new(0.0, in_strip_y), win, cols, 1, zone),
            Some(0)
        );
        assert_eq!(
            button_grid_at(Vec2::new(win.x - 0.1, in_strip_y), win, cols, 1, zone),
            Some(cols - 1)
        );

        // Off-window x, and a degenerate window / grid, are misses, not panics.
        assert_eq!(
            button_grid_at(Vec2::new(-1.0, in_strip_y), win, cols, 1, zone),
            None
        );
        assert_eq!(
            button_grid_at(Vec2::new(win.x + 1.0, in_strip_y), win, cols, 1, zone),
            None
        );
        assert_eq!(
            button_grid_at(Vec2::new(10.0, 10.0), Vec2::ZERO, cols, 1, zone),
            None
        );
        assert_eq!(button_grid_at(Vec2::new(10.0, 10.0), win, 0, 1, zone), None);
    }

    #[test]
    fn button_grid_maps_rows_row_major() {
        let win = Vec2::new(100.0, 100.0);
        let zone = Rect::new(0.0, 0.0, 1.0, 1.0);
        // 2x2 grid over the whole window: quadrants map row-major 0,1 / 2,3.
        assert_eq!(
            button_grid_at(Vec2::new(25.0, 25.0), win, 2, 2, zone),
            Some(0)
        );
        assert_eq!(
            button_grid_at(Vec2::new(75.0, 25.0), win, 2, 2, zone),
            Some(1)
        );
        assert_eq!(
            button_grid_at(Vec2::new(25.0, 75.0), win, 2, 2, zone),
            Some(2)
        );
        assert_eq!(
            button_grid_at(Vec2::new(75.0, 75.0), win, 2, 2, zone),
            Some(3)
        );
    }

    #[test]
    fn stick_deflection_maps_and_clamps() {
        let (r, d) = (100.0, 10.0);
        // Inside the dead zone (and the degenerate origin) produce no deflection.
        assert_eq!(stick_deflection(Vec2::new(d * 0.5, 0.0), r, d), Vec2::ZERO);
        assert_eq!(stick_deflection(Vec2::ZERO, r, d), Vec2::ZERO);

        // At the radius the deflection is full along the drag direction.
        let right = stick_deflection(Vec2::new(r, 0.0), r, d);
        assert!((right - Vec2::X).length() < 1e-4);

        // Past the radius it clamps to the unit disc.
        let past = stick_deflection(Vec2::new(r * 3.0, 0.0), r, d);
        assert!((past - Vec2::X).length() < 1e-4);

        // Midway between dead and radius is a partial deflection (0..1).
        let mid = stick_deflection(Vec2::new(d + (r - d) * 0.5, 0.0), r, d);
        assert!((mid.x - 0.5).abs() < 1e-4);

        // Never leaves the unit disc for any offset.
        for &(x, y) in &[(r, r), (-r, r), (r * 5.0, -r * 5.0), (d + 1.0, d + 1.0)] {
            assert!(stick_deflection(Vec2::new(x, y), r, d).length() <= 1.0 + 1e-4);
        }
    }
}
