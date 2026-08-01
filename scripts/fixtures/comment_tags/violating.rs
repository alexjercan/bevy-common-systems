//! Fixture for `scripts/test-check-comment-tags.sh`: every test fn here is a
//! POSITIVE control for the `///`-guards-a-literal rule. Not compiled by cargo;
//! it lives under `scripts/` on purpose, outside every crate target.
//!
//! Identifiers are fictional so that a tree-wide grep for a real symbol never
//! lands here (lesson `a-tree-scanner-scans-itself`).

#[cfg(test)]
mod tests {
    /// A drift of 0.35 is the widest wobble the damper may leave behind, so
    /// the assertion holds it there.
    #[test]
    fn wobble_settles_under_the_drift_ceiling() {
        let drift = measure_wobble();
        assert!(drift < 0.35);
    }

    /// The two thresholds straddle the rounding boundary: 4.49 must floor and
    /// 4.51 must ceil.
    #[test]
    fn readout_rounds_at_the_half_step() {
        assert_eq!(readout(4.49), 4);
        assert_eq!(readout(4.51), 5);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod native_tests {
    /// A 12-slot ring is the smallest size that exercises the wrap, so the
    /// fixture builds one that size.
    #[test]
    fn ring_wraps_at_capacity() {
        let ring = Ring::with_capacity(12);
        assert!(ring.wraps());
    }
}

#[cfg(test)]
mod lifetime_tests {
    // NOTE: a lone lifetime tick is not an opening quote. When it was treated
    // as one, the rest of its line -- brace included -- was blanked, the brace
    // depth desynced, and every test fn below became invisible. Keep the ODD
    // tick count on the `struct` line below; that is what reproduces it.
    struct Borrowed<'a> {
        inner: &'a str,
    }

    impl<'a> Borrowed<'a> {
        fn width(&self) -> f32 {
            self.inner.len() as f32
        }
    }

    /// Below a lifetime header, and still reported: 7.25 is a value guard.
    #[test]
    fn a_test_below_a_lifetime_is_still_seen() {
        let borrowed = Borrowed { inner: "xy" };
        assert!(borrowed.width() < 7.25);
    }
}
