//! A generic best-score resource with a "new best" edge.
//!
//! Every game keeps the best score across runs and flashes "New best!" when a
//! run beats it -- and hand-rolls the same two resources plus the same update
//! (`new_best = score > high; high = high.max(score)`). [`HighScore<T>`] is that
//! pair in one resource: it holds the best value and whether the last
//! [`record`](HighScore::record) set a new best.
//!
//! It is generic because the score type varies per game (`usize`, `f64`, `f32`);
//! any `PartialOrd + Copy` value works. It derives `Serialize`/`Deserialize`, so
//! it composes with [`PersistPlugin`](crate::persist::PersistPlugin) to survive a
//! restart -- the `new_best` flag is not serialized (it is per-run state).
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! # fn wire_up() {
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .insert_resource(HighScore::<u32>::default())
//!     // ... and, to persist it across launches:
//!     .add_plugins(PersistPlugin::<HighScore<u32>>::new("my_game.high_score"));
//! # }
//!
//! // On game over, record the run's score and react to a new best.
//! fn on_game_over(score: u32, mut high: ResMut<HighScore<u32>>) {
//!     if high.record(score) {
//!         // ... show "New best!" ...
//!     }
//!     let _ = high.best(); // ... show "Best: {best}" ...
//! }
//! ```

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use super::HighScore;
}

/// The best score seen so far, plus whether the last [`record`](Self::record)
/// beat it.
///
/// Insert it as a resource (`HighScore<YourScoreType>`); `record` a run's score
/// on game over, then read [`best`](Self::best) and [`is_new_best`](Self::is_new_best).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct HighScore<T> {
    best: T,
    /// Whether the most recent `record` set a new best. Per-run state, so it is
    /// not persisted.
    #[serde(skip)]
    #[reflect(ignore)]
    new_best: bool,
}

impl<T: Default> Default for HighScore<T> {
    fn default() -> Self {
        Self {
            best: T::default(),
            new_best: false,
        }
    }
}

impl<T: PartialOrd + Copy> HighScore<T> {
    /// A high score starting from `initial` (e.g. a non-zero floor), with no new
    /// best recorded yet.
    pub fn new(initial: T) -> Self {
        Self {
            best: initial,
            new_best: false,
        }
    }

    /// The best score so far.
    pub fn best(&self) -> T {
        self.best
    }

    /// Whether the most recent [`record`](Self::record) set a new best -- read it
    /// on the game-over screen to flash "New best!".
    pub fn is_new_best(&self) -> bool {
        self.new_best
    }

    /// Record a run's `score`: if it beats the current best, update the best and
    /// mark a new best. Returns whether it was a new best. A tie is not a new
    /// best (strictly greater wins).
    pub fn record(&mut self, score: T) -> bool {
        let is_new = score > self.best;
        self.new_best = is_new;
        if is_new {
            self.best = score;
        }
        is_new
    }

    /// Clear the new-best flag (e.g. when starting a fresh run) without touching
    /// the best value.
    pub fn clear_new_best(&mut self) {
        self.new_best = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_starts_at_default_with_no_new_best() {
        let hs = HighScore::<u32>::default();
        assert_eq!(hs.best(), 0);
        assert!(!hs.is_new_best());
    }

    #[test]
    fn record_updates_and_reports_a_new_best() {
        let mut hs = HighScore::<u32>::default();
        assert!(hs.record(10), "first score beats the default 0");
        assert_eq!(hs.best(), 10);
        assert!(hs.is_new_best());

        // A lower score is not a new best and does not lower the best.
        assert!(!hs.record(4));
        assert_eq!(hs.best(), 10);
        assert!(!hs.is_new_best());

        // A higher score is a new best.
        assert!(hs.record(15));
        assert_eq!(hs.best(), 15);
    }

    #[test]
    fn a_tie_is_not_a_new_best() {
        let mut hs = HighScore::new(10u32);
        assert!(!hs.record(10), "equalling the best is not a new best");
        assert_eq!(hs.best(), 10);
    }

    #[test]
    fn works_for_float_scores() {
        let mut hs = HighScore::<f64>::default();
        assert!(hs.record(3.5));
        assert!(!hs.record(2.0));
        assert!(hs.record(7.25));
        assert_eq!(hs.best(), 7.25);
    }

    #[test]
    fn clear_new_best_keeps_the_best() {
        let mut hs = HighScore::<u32>::default();
        hs.record(10);
        hs.clear_new_best();
        assert_eq!(hs.best(), 10);
        assert!(!hs.is_new_best());
    }

    #[test]
    fn new_best_flag_is_not_serialized() {
        let mut hs = HighScore::<u32>::default();
        hs.record(42);
        assert!(hs.is_new_best());
        let json = serde_json::to_string(&hs).unwrap();
        // Only the best is stored; the per-run flag is skipped.
        assert_eq!(json, "{\"best\":42}");
        let loaded: HighScore<u32> = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.best(), 42);
        assert!(!loaded.is_new_best(), "the flag defaults to false on load");
    }
}
