//! Screen-layout building blocks for menu / game-over overlays.
//!
//! Every game builds the same full-screen, centered-column overlay for its menu
//! and game-over screens, and fills it with the same centered text lines -- the
//! two builders here ([`centered_screen`] and [`screen_text`]) are copied
//! verbatim across the crate's examples. Plus a [`TitlePulse`] component for the
//! sine "breathe" the menu title does in most of them.
//!
//! These are opinion-light pieces, not a menu *framework*: they mirror
//! [`ui/status::status_bar_item`](crate::ui::status) (the module owns the widget
//! shape, the game owns the content and the state machine). Compose a screen
//! from a [`centered_screen`] root and [`screen_text`] children:
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! fn spawn_menu(mut commands: Commands) {
//!     commands
//!         .spawn(centered_screen())
//!         .with_children(|screen| {
//!             screen.spawn((
//!                 screen_text("MY GAME", 72.0, Color::WHITE),
//!                 TitlePulse::new(Color::srgb(0.95, 0.85, 0.25)),
//!             ));
//!             screen.spawn(screen_text("Tap to play", 32.0, Color::WHITE));
//!         });
//! }
//! ```

use bevy::prelude::*;

pub mod prelude {
    pub use super::{centered_screen, screen_text, MenuPlugin, MenuSystems, TitlePulse};
}

/// A full-screen, absolutely-positioned column centered on both axes, with a
/// gap between rows -- the root every menu / game-over overlay uses. Spawn it
/// and add [`screen_text`] children.
pub fn centered_screen() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        row_gap: Val::Px(16.0),
        ..default()
    }
}

/// A centered `Text` line at `size` px in `color` -- one row of a
/// [`centered_screen`]. The text is center-justified so multi-line strings stay
/// centered too.
pub fn screen_text(text: impl Into<String>, size: f32, color: Color) -> impl Bundle {
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

/// System sets for [`MenuPlugin`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MenuSystems {
    /// Animates [`TitlePulse`] title text. Runs in `Update`.
    Pulse,
}

/// Breathes a text entity's `TextColor` alpha in a sine wave, for the menu title
/// "look at me" pulse most games do.
///
/// Put it on a `Text` node (alongside [`screen_text`]); [`MenuPlugin`] keeps the
/// RGB from [`color`](Self::color) and oscillates the alpha between
/// [`min_alpha`](Self::min_alpha) and [`max_alpha`](Self::max_alpha) at
/// [`speed`](Self::speed) radians per second.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct TitlePulse {
    /// Base RGB of the title (its alpha is animated).
    pub color: Color,
    /// Sine speed in radians per second.
    pub speed: f32,
    /// Alpha at the trough of the pulse.
    pub min_alpha: f32,
    /// Alpha at the crest of the pulse.
    pub max_alpha: f32,
}

impl TitlePulse {
    /// A pulse on `color` with the common feel (breathes alpha between 0.3 and
    /// 1.0 at 2.5 rad/s), matching the hand-rolled `pulse_menu_title`.
    pub fn new(color: Color) -> Self {
        Self {
            color,
            speed: 2.5,
            min_alpha: 0.3,
            max_alpha: 1.0,
        }
    }

    /// Set the sine speed (radians per second, builder style).
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Set the alpha range (builder style).
    pub fn with_alpha_range(mut self, min_alpha: f32, max_alpha: f32) -> Self {
        self.min_alpha = min_alpha;
        self.max_alpha = max_alpha;
        self
    }
}

/// Animates [`TitlePulse`] title text.
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        debug!("MenuPlugin: build");

        app.register_type::<TitlePulse>()
            .add_systems(Update, pulse_titles.in_set(MenuSystems::Pulse));
    }
}

/// Oscillate each [`TitlePulse`]'s `TextColor` alpha in a sine wave, keeping the
/// RGB. `TitlePulse::new`'s defaults reproduce the hand-rolled
/// `0.65 + 0.35 * sin(t * 2.5)` alpha ramp.
fn pulse_titles(time: Res<Time>, mut q_title: Query<(&TitlePulse, &mut TextColor)>) {
    let t = time.elapsed_secs();
    for (pulse, mut text_color) in q_title.iter_mut() {
        let phase = 0.5 + 0.5 * (t * pulse.speed).sin(); // 0..=1
        let alpha = pulse.min_alpha + (pulse.max_alpha - pulse.min_alpha) * phase;
        text_color.0 = pulse.color.with_alpha(alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_pulse_new_has_the_common_feel() {
        let p = TitlePulse::new(Color::WHITE);
        assert_eq!(p.speed, 2.5);
        assert_eq!(p.min_alpha, 0.3);
        assert_eq!(p.max_alpha, 1.0);
    }

    #[test]
    fn builders_override_speed_and_range() {
        let p = TitlePulse::new(Color::WHITE)
            .with_speed(2.4)
            .with_alpha_range(0.55, 1.0);
        assert_eq!(p.speed, 2.4);
        assert_eq!(p.min_alpha, 0.55);
        assert_eq!(p.max_alpha, 1.0);
    }
}
