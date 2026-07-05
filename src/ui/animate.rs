//! Tween-driven UI animation glue: opt-in markers that write a `Tween<T>` into a
//! `Node` layout field or a `BackgroundColor` each frame.
//!
//! This is the UI-node counterpart to the material-only
//! [`feedback/flash`](crate::feedback::flash): where `feedback` animates a
//! `StandardMaterial`, this animates plain `bevy_ui` fields, which is what an
//! all-UI game (a board, an inventory, a card layout) needs. It builds on
//! [`tween`](crate::tween) -- the game spawns a `Tween<Vec2>` / `Tween<f32>` /
//! `Tween<Vec4>` and tags the entity with the matching marker, and
//! [`UiAnimatePlugin`] copies the tweened value into the UI field after
//! [`TweenSystems::Advance`] each frame. Never drive UI layout from `Transform`
//! scale (bevy_ui owns the transform); tween the `Node`/`BackgroundColor` fields
//! directly, which is exactly what these markers do.
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! # fn spawn_tile(mut commands: Commands) {
//! // A tile that slides to a target pixel position and pops in as it arrives.
//! commands.spawn((
//!     Node::default(),
//!     TweenNodeOffset,
//!     Tween::new(Vec2::ZERO, Vec2::new(120.0, 40.0), 0.12, EaseFunction::QuadraticOut),
//! ));
//! # }
//! ```

use bevy::prelude::*;

use crate::tween::prelude::*;

pub mod prelude {
    pub use super::{
        color_to_vec4, node_flash, vec4_to_color, TweenNodeBackground, TweenNodeOffset,
        TweenNodeScale, UiAnimatePlugin, UiAnimateSystems,
    };
}

/// Decompose a `Color` into a linear-RGBA `Vec4`, the carrier a `Tween<Vec4>`
/// animates (Bevy's `Color` enum does not lerp component-wise on its own).
pub fn color_to_vec4(color: Color) -> Vec4 {
    let c = color.to_linear();
    Vec4::new(c.red, c.green, c.blue, c.alpha)
}

/// Rebuild a `Color` from a linear-RGBA `Vec4` produced by a `Tween<Vec4>`.
pub fn vec4_to_color(v: Vec4) -> Color {
    Color::linear_rgba(v.x, v.y, v.z, v.w)
}

/// A flash `Tween<Vec4>` from bright white to `to`, easing out over `duration` --
/// the "just changed" pop for a UI node's background. Pair it with
/// [`TweenNodeBackground`].
pub fn node_flash(to: Color, duration: f32) -> Tween<Vec4> {
    Tween::new(
        Vec4::ONE,
        color_to_vec4(to),
        duration,
        EaseFunction::QuadraticOut,
    )
    .with_on_complete(TweenOnComplete::Keep)
}

/// Marker: copy this entity's `Tween<Vec2>` into its `Node`'s `left`/`top` as
/// pixels each frame, so it slides across its positioned parent.
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct TweenNodeOffset;

/// Marker: copy this entity's `Tween<f32>` into its `Node`'s `width`/`height` as
/// a percent (a value of `1.0` -> `100%`), so it grows/shrinks from the centre of
/// its parent -- the "pop" on spawn or merge. Negative values clamp to zero.
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct TweenNodeScale;

/// Marker: copy this entity's `Tween<Vec4>` (linear RGBA) into its
/// `BackgroundColor` each frame -- a colour flash or fade. See [`node_flash`].
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct TweenNodeBackground;

/// System sets for [`UiAnimatePlugin`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UiAnimateSystems {
    /// Writes tweened values into `Node` / `BackgroundColor`. Runs in `Update`
    /// after [`TweenSystems::Advance`], so it reads this frame's advanced values.
    Apply,
}

/// Applies [`TweenNodeOffset`] / [`TweenNodeScale`] / [`TweenNodeBackground`]
/// each frame. Add it alongside [`TweenPlugin`].
pub struct UiAnimatePlugin;

impl Plugin for UiAnimatePlugin {
    fn build(&self, app: &mut App) {
        debug!("UiAnimatePlugin: build");

        app.register_type::<TweenNodeOffset>()
            .register_type::<TweenNodeScale>()
            .register_type::<TweenNodeBackground>()
            .add_systems(
                Update,
                (apply_offset, apply_scale, apply_background)
                    .in_set(UiAnimateSystems::Apply)
                    .after(TweenSystems::Advance),
            );
    }
}

/// Write each `TweenNodeOffset` entity's tweened `Vec2` into `Node.left/top` (px).
fn apply_offset(mut q: Query<(&mut Node, &Tween<Vec2>), With<TweenNodeOffset>>) {
    for (mut node, tween) in &mut q {
        let p = tween.value();
        node.left = Val::Px(p.x);
        node.top = Val::Px(p.y);
    }
}

/// Write each `TweenNodeScale` entity's tweened `f32` into `Node.width/height`
/// as a percent (value `1.0` -> `100%`, clamped at zero).
fn apply_scale(mut q: Query<(&mut Node, &Tween<f32>), With<TweenNodeScale>>) {
    for (mut node, tween) in &mut q {
        let pct = (tween.value() * 100.0).max(0.0);
        node.width = Val::Percent(pct);
        node.height = Val::Percent(pct);
    }
}

/// Write each `TweenNodeBackground` entity's tweened `Vec4` into `BackgroundColor`.
fn apply_background(mut q: Query<(&mut BackgroundColor, &Tween<Vec4>), With<TweenNodeBackground>>) {
    for (mut bg, tween) in &mut q {
        bg.0 = vec4_to_color(tween.value());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_vec4_round_trips() {
        let color = Color::srgb(0.2, 0.6, 0.9);
        let back = vec4_to_color(color_to_vec4(color));
        // Round-trip through linear space is exact for the linear components.
        let a = color.to_linear();
        let b = back.to_linear();
        assert!((a.red - b.red).abs() < 1e-6);
        assert!((a.green - b.green).abs() < 1e-6);
        assert!((a.blue - b.blue).abs() < 1e-6);
        assert!((a.alpha - b.alpha).abs() < 1e-6);
    }

    #[test]
    fn node_flash_starts_bright_white() {
        // The flash convention is "start bright, ease to the node's colour", so at
        // t=0 the tweened value must be white regardless of the target. (The end
        // value -- `color_to_vec4(target)` -- is exercised by the 13_glide merge
        // flash, since advancing a `Tween` is driven by `TweenPlugin`.)
        let tween = node_flash(Color::srgb(0.1, 0.3, 0.5), 0.2);
        assert_eq!(tween.value(), Vec4::ONE);
    }
}
