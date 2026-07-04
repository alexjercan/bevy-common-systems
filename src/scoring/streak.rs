//! A decaying hit-streak (combo) counter.
//!
//! A [`Streak`] counts hits that land in quick succession and decays when the
//! player goes quiet: each hit bumps the count and refreshes a time window, and
//! when the window lapses the streak ends. It is the shared bookkeeping behind
//! the "combo"/"streak" mechanic games re-implement (fruit-ninja `Combo`,
//! orbit-runner `Streak`, whose code comment literally reads "Modelled on
//! `06_fruitninja`'s `Combo`").
//!
//! It owns ONLY the count-and-decay bookkeeping, deliberately not the scoring
//! rule: what a hit at streak `n` is *worth*, and any running points tally, stay
//! in the game (they differ per game -- one scales points linearly, another
//! triangularly). [`Streak::hit`] returns the new count so the game can apply
//! its own multiplier, and [`Streak::tick`] returns the final count the frame
//! the streak ends so the game can flash its own "COMBO xN" tally.
//!
//! There is no plugin and no `Score` resource here on purpose: ticking is a
//! one-line call from the game's own system (games tick on different schedules
//! and against different state), and a plain score counter is a bare
//! `usize`/`f32` the game already owns. This module is just the decay state
//! machine.
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! // A resource wrapping the streak, plus whatever the game scores.
//! #[derive(Resource)]
//! struct Combo {
//!     streak: Streak,
//!     points: usize,
//! }
//!
//! fn on_hit(mut combo: ResMut<Combo>) {
//!     let count = combo.streak.hit(); // 1, 2, 3, ...
//!     combo.points += count; // game-specific value rule
//! }
//!
//! fn tick(time: Res<Time>, mut combo: ResMut<Combo>) {
//!     if let Some(final_count) = combo.streak.tick(time.delta_secs()) {
//!         if final_count >= 2 {
//!             // ... flash a "COMBO x{final_count} +{points}" tally ...
//!         }
//!         combo.points = 0;
//!     }
//! }
//! ```

use bevy::prelude::*;

pub mod prelude {
    pub use super::Streak;
}

/// A decaying hit-streak: a count that grows on each [`hit`](Streak::hit) and
/// resets when its time window lapses without another hit.
///
/// Construct with [`Streak::new`] (giving the window length in seconds), call
/// [`hit`](Streak::hit) on each scoring event, and [`tick`](Streak::tick) each
/// frame to advance the decay. Derives `Resource` so it can be used as one
/// directly, and is embeddable in a game's own resource when the game also
/// tracks points.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct Streak {
    count: usize,
    timer: f32,
    window: f32,
}

impl Streak {
    /// Create an inactive streak whose hit window is `window` seconds.
    ///
    /// A non-positive `window` is clamped to zero, which makes every
    /// [`tick`](Streak::tick) after a [`hit`](Streak::hit) end the streak
    /// immediately.
    pub fn new(window: f32) -> Self {
        Self {
            count: 0,
            timer: 0.0,
            window: window.max(0.0),
        }
    }

    /// Register a hit: bump the count, refresh the window to full, and return the
    /// new count (so the caller can scale the hit's value by the streak length).
    pub fn hit(&mut self) -> usize {
        self.count += 1;
        self.timer = self.window;
        self.count
    }

    /// Extend the remaining window to at least `seconds` without bumping the
    /// count (a bonus that buys more time, e.g. a special pickup). Never shortens
    /// the window.
    pub fn extend_to(&mut self, seconds: f32) {
        if self.count > 0 {
            self.timer = self.timer.max(seconds);
        }
    }

    /// Advance the decay by `dt` seconds. Returns `Some(final_count)` on the
    /// frame the streak ends (so the caller can show a tally), and `None`
    /// otherwise (including while inactive).
    pub fn tick(&mut self, dt: f32) -> Option<usize> {
        if self.count == 0 {
            return None;
        }
        self.timer -= dt;
        if self.timer > 0.0 {
            return None;
        }
        let final_count = self.count;
        self.count = 0;
        self.timer = 0.0;
        Some(final_count)
    }

    /// Reset the streak to inactive immediately, without returning a tally (for a
    /// run restart or a hit that breaks the chain).
    pub fn reset(&mut self) {
        self.count = 0;
        self.timer = 0.0;
    }

    /// The current streak length (0 when inactive).
    pub fn count(&self) -> usize {
        self.count
    }

    /// Whether the streak is currently running (count > 0).
    pub fn is_active(&self) -> bool {
        self.count > 0
    }

    /// Seconds left on the current window before the streak decays.
    pub fn remaining(&self) -> f32 {
        self.timer
    }

    /// The remaining window as a fraction of a full window (`remaining / window`),
    /// handy for fading a combo HUD. `0.0` when the window length is zero. Can
    /// exceed `1.0` if [`extend_to`](Streak::extend_to) pushed the timer past a
    /// full window; clamp at the call site if you need `0..=1`.
    pub fn remaining_frac(&self) -> f32 {
        if self.window > 0.0 {
            self.timer / self.window
        } else {
            0.0
        }
    }

    /// The full window length in seconds this streak was constructed with.
    pub fn window(&self) -> f32 {
        self.window
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_streak_is_inactive() {
        let s = Streak::new(1.5);
        assert_eq!(s.count(), 0);
        assert!(!s.is_active());
        assert_eq!(s.window(), 1.5);
    }

    #[test]
    fn hit_bumps_count_and_refreshes_window() {
        let mut s = Streak::new(1.5);
        assert_eq!(s.hit(), 1);
        assert_eq!(s.hit(), 2);
        assert!(s.is_active());
        assert_eq!(s.remaining(), 1.5);
        assert!((s.remaining_frac() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn tick_within_window_keeps_the_streak() {
        let mut s = Streak::new(1.0);
        s.hit();
        assert_eq!(s.tick(0.4), None);
        assert!(s.is_active());
        assert!((s.remaining() - 0.6).abs() < 1e-6);
    }

    #[test]
    fn tick_past_window_ends_with_final_count() {
        let mut s = Streak::new(1.0);
        s.hit();
        s.hit();
        // Two ticks that together exceed the window end the streak on the second.
        assert_eq!(s.tick(0.6), None);
        assert_eq!(s.tick(0.6), Some(2));
        assert!(!s.is_active());
        assert_eq!(s.count(), 0);
        // A tick on an inactive streak is a no-op.
        assert_eq!(s.tick(1.0), None);
    }

    #[test]
    fn extend_to_lengthens_but_never_shortens() {
        let mut s = Streak::new(1.0);
        s.hit();
        s.tick(0.5); // 0.5 left
        s.extend_to(2.0);
        assert!((s.remaining() - 2.0).abs() < 1e-6);
        // A shorter extend does nothing.
        s.extend_to(0.1);
        assert!((s.remaining() - 2.0).abs() < 1e-6);
        // Extending an inactive streak does nothing.
        let mut idle = Streak::new(1.0);
        idle.extend_to(5.0);
        assert_eq!(idle.remaining(), 0.0);
    }

    #[test]
    fn reset_clears_without_a_tally() {
        let mut s = Streak::new(1.0);
        s.hit();
        s.hit();
        s.reset();
        assert!(!s.is_active());
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn zero_window_ends_on_the_next_tick() {
        let mut s = Streak::new(0.0);
        assert_eq!(s.hit(), 1);
        assert_eq!(s.remaining_frac(), 0.0);
        assert_eq!(s.tick(0.0), Some(1));
    }
}
