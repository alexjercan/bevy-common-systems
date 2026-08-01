//! Fixture for `scripts/test-check-comment-tags.sh`: the NEGATIVE control for
//! the `///`-guards-a-literal rule. Every fn here is a near miss the rule must
//! stay silent on, so a matcher that over-fires shows up as a failing probe
//! rather than as tree-wide noise.
//!
//! Identifiers are fictional (lesson `a-tree-scanner-scans-itself`).

#[cfg(test)]
mod tests {
    /// The damper must settle the wobble rather than sustain it, and 0.35 is
    /// where "settled" stops meaning anything.
    #[test]
    fn a_tagged_body_block_suppresses_the_rule() {
        // NOTE: 0.35 is the widest wobble the damper may leave behind; past it
        // the settling is indistinguishable from a sustained oscillation.
        let drift = measure_wobble();
        assert!(drift < 0.35);
    }

    /// The two thresholds straddle the rounding boundary: 4.49 floors and
    /// 4.51 ceils.
    #[test]
    fn an_end_of_line_comment_suppresses_the_rule() {
        assert_eq!(readout(4.49), 4); // largest input that still floors
        assert_eq!(readout(4.51), 5); // smallest input that ceils
    }

    /// A 90 degree yaw about the up axis maps forward onto right.
    #[test]
    fn a_literal_absent_from_the_body_is_not_a_hit() {
        let turned = yaw(FRAC_PI_2);
        assert_eq!(turned.forward(), Axis::Right);
    }

    /// Both ends of a 2-slot ring are addressable.
    #[test]
    fn a_bare_zero_one_or_two_is_never_a_hit() {
        let ring = Ring::with_capacity(2);
        assert!(ring.get(0).is_some());
        assert!(ring.get(1).is_some());
    }

    #[test]
    fn an_undocumented_test_is_never_reported() {
        let ring = Ring::with_capacity(12);
        assert_eq!(ring.capacity(), 12);
    }
}

#[cfg(test)]
mod raw_string_tests {
    /// A multi-line raw string is nothing but braces, and none of them may
    /// reach the brace depth, or 6.5 below goes unread.
    #[test]
    fn a_multi_line_raw_string_does_not_shift_the_depth() {
        // NOTE: 6.5 is the parsed width; the JSON around it is here so the
        // fixture holds the shape that hid a whole test module -- a raw string
        // opening on one line and closing on another.
        let spec = r#"[
            { "kind": "gauge", "width": 6.5 }
        ]"#;
        assert_eq!(parse_width(spec), 6.5);
    }
}

#[cfg(test)]
mod char_literal_tests {
    /// A brace inside a char literal must not shift the brace depth, or this
    /// module ends early and 8.5 below goes unread.
    #[test]
    fn a_brace_in_a_char_literal_is_still_blanked() {
        // NOTE: 8.5 is the width the layout reserves for one glyph; the
        // literal `{` is here to prove the blanking still runs for real char
        // literals now that a lone lifetime tick is exempt from it.
        let open = '{';
        assert_eq!(width(open), 8.5);
    }
}

// NOTE: this `use` is brace-less, so `#[cfg(test)]` above it must NOT latch
// onto the next brace it finds -- which is `measure_wobble` below, production
// code whose rustdoc this rule must never read.
#[cfg(test)]
use std::fmt;

/// A 0.35 drift ceiling documented OUTSIDE `#[cfg(test)]` is the public API
/// surface, which this rule never reads.
pub fn measure_wobble() -> f32 {
    0.35
}
