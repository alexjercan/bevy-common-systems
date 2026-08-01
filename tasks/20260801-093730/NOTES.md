# NOTES - check-comment-tags rule 2

Design record for the promotion of `state-what-the-checker-cannot-see`.

## The defect being checked

`check-comment-tags.sh` exempts rustdoc, because rustdoc is the public API
surface. Inside `#[cfg(test)]` that reasoning does not hold: a `///` on a test
fn is rendered nowhere, so the exemption buys no API documentation and simply
creates an untagged place to put prose. It is still the RIGHT home for what a
test proves -- it outlives any line in the body and `cargo test -- --list`
shows it -- which is exactly why it gets misused: the same comment block that
would need a `NOTE:` in the body needs nothing above the `#[test]`.

Three consecutive authors made the move: 20260731-172224, -172232, -172233. The
clearest instance is `ui/health_display.rs`, where the justification for three
magic inputs (`0.4`, `2.29`, `2.3`) moved into the `///` and left the body
bare.

## Why a checker and not a line of prose

The rejected alternative was one AGENTS.md sentence. The ledger entry already
records why that is not enough: the convention's own AUTHOR violated it on the
second outing, and the third author inherited it and violated it too. Prose
that has been read and broken three times is not the missing input.

## Choosing the rule: measured, not reasoned

The DoD's original definition was "a literal with no tagged `NOTE:` block and
no end-of-line comment on its line", inside any `///`-documented test fn. Run
on the base tree that is unshippable:

| Definition | Hits, src | Hits, +examples |
| --- | --- | --- |
| any untagged literal in a documented test fn | 183 | 253 |
| the `///` block contains any literal | 26 | -- |
| literal in BOTH the `///` and the body (shipped) | 8 | 8 |

The middle row is the interesting failure. It is close to the right idea but
fires on intent that merely mentions a number -- "Yawing 90 degrees to the
right from the -Z default", where 90 is the scenario's NAME, not a tuned value
in the body.

The shipped rule is the correlation. If the doc spells out a number AND the
body uses that same number, the doc is explaining that value, which is the
migration itself. It reproduces the historical `ui/health_display.rs`
occurrence and stays green on its fixed form, which now carries a body `NOTE:`.

Three narrowings, each paid for by the measurement above:

- a body holding ANY tagged block exempts the whole fn. Which literal a given
  block covers is a review call, not a grep, and the author demonstrably
  applied the split.
- a literal on a line with an end-of-line comment is exempt, matching rule 1.
- bare `0`, `1`, `2` never count. They collide with prose far more than they
  name a magic value. Measured by running the shipped script with that clause
  removed over the base tree: 14 hits instead of 8, and all 6 of the difference
  are false, e.g. a doc's "at least 1%" meeting `assert_eq!(display_percent(0.4,
  230.0), 1)`, or "the mirror of test 1" meeting a `1e-6` tolerance.

Accepted gap, recorded rather than engineered around: a doc that justifies a
value WITHOUT writing its digits ("the sliver value") is invisible. All three
recorded occurrences wrote the digits, and KISS says stop there.

## Implementation notes

- Regions are tracked by brace depth, not "the rest of the file". The negative
  fixture ends with a documented `pub fn` AFTER the test module precisely to
  hold that line: a public doc comment must not be read as test doc.
- The same blanking is why the shipped run reports 8 hits where the throwaway
  prototype reported 11: the prototype counted numbers inside assert-message
  strings as body occurrences.
- `#[cfg(all(test, not(target_arch = "wasm32")))]` matches, per
  `probe-a-new-checker-both-ways` -- an earlier grep in this repo missed exactly
  that form. The positive fixture's second module is that shape.
- Both rules always run; neither short-circuits the other, so one pass reports
  everything. Exit 1 if either fires, 2 on a usage or parse error.

## The brace-depth desync family (review round 1)

Tracking regions by brace depth means anything that miscounts a brace silently
voids the file. Round 1 found FOUR separate causes, and every one presented the
same way: exit 0, "no test fn's /// guards a literal". Not one produced a false
report, which is why reading the code found none of them and probing found all
of them.

| Cause | What it looked like |
| --- | --- |
| a lone lifetime tick read as an opening quote | `struct Borrowed<'a> {` -- odd tick count, so the brace was blanked with the rest of the line |
| a brace inside COMMENT prose | a `NOTE:` block naming a brace shifted the depth; the fixture written to cover the char-literal case tripped it by accident |
| a brace inside a raw string | `r#"[{ ... }]"#`, i.e. every JSON fixture in `modding/` |
| a raw string SPANNING lines | the open-literal state was per-line, so the continuation lines counted as code |

The first was found against a real file: injecting a textbook violation into
`src/modding/registry.rs`'s test module and getting exit 0. That canary kept
failing after each of the first three fixes, which is what surfaced the next
one each time -- a plain reminder that fixing the cause you found is not the
same as fixing the class.

So the class now fails LOUD. A `.rs` file always balances to depth 0; if it
does not, or a string literal is still open at EOF, the parser lost the braces
and rule 2's verdict on that file is void. That is a TOOL error, exit 2, not a
finding -- so `! check-comment-tags.sh <path>` cannot read it as clean. The
probe suite covers it with a deliberately unbalanced file.

Every one of the four causes has a fixture line, and each was confirmed to turn
the suite red when its fix is reverted. The lifetime revert now fails as an
exit-2 desync rather than a silent pass, which is the guard doing its job.

## The `#[cfg(test)]` latch (review round 1)

`#[cfg(test)]` on a BRACE-LESS item -- `#[cfg(test)] use ...;`, the common Rust
form -- used to latch until the next brace anywhere below, marking production
code as a test region and reporting public rustdoc. That is the one thing the
rule promises never to do.

The latch now bridges only from the attribute to its item's brace, and drops on
a statement end or a closing brace. Same branch closes a latent false negative:
`#[cfg(test)] mod t { ... }` written on one line used to be skipped entirely.

## Fixing the base-tree hits (8 literals, 6 test fns, 2 files)

All six are the misuse, not false positives, so each keeps an intent-only
`///` and gains a body `NOTE:` naming why the value is that value.

| Site | The value, and what now guards it |
| --- | --- |
| `camera/shake.rs` `offset_scales_with_amount_and_max_offset` | 0.6 is the peak `shake_app` configures, so the pure-math and through-the-plugin tests state the same bound |
| `camera/shake.rs` `shake_offset_stays_within_the_configured_bound` | same 0.6, which must stay in step with `shake_app` or the bound stops describing the camera under test |
| `physics/pd_controller.rs` `fast_roll_despins_when_command_tracks_attitude` | 1.5 rad/s is past the rate one tick's torque can cancel, which is what makes it the "fast" roll |
| `physics/pd_controller.rs` `fast_roll_despins_with_frozen_command` | same 1.5, and the two tests differ only in whether the command tracks |
| `physics/pd_controller.rs` `fast_roll_despins_under_a_saturating_torque_budget` | 100 is what makes the case saturating: one tick's impulse exceeds twice the spin |
| `physics/pd_controller.rs` `moderate_spin_despins_with_frozen_command` | 0.7 is "moderate" because one tick's budget cancels it outright, below the saturating regime |

One correction during the pass: the first draft of the `shake.rs` note claimed
0.3 was exact in binary floating point. It is not -- 0.6 is not representable,
and the test asserts through a `1e-6` tolerance for that reason. The note now
states the actual reason for the value (agreement with `shake_app`).

## Probing the checker

`scripts/test-check-comment-tags.sh` covers match, no-match and tool error, per
`probe-a-new-checker-both-ways` and `probe-the-argument-surface-too`. The
tool-error probes matter most: the natural positive-control idiom is
`! checker <fixture>`, which reads a crash as a pass, so exit 2 has to stay
distinguishable from exit 1.

One probe is about the fixtures rather than the checker: the violating fixture
must NOT trip rule 1, or its exit of 1 stops being evidence about rule 2.

The prober was itself probed -- disabling rule 2's reporting branch turns the
suite red on 2 probes -- so it is not a test that can only pass.

The suite runs argument-free, so it goes in CI beside `check-ascii.sh`. A
fixture nobody executes decays silently (`a-gotcha-needs-a-witness`).

## Out of scope, discovered here

`examples/` has never been under rule 1 and carries 769 untagged comment
blocks. Bringing it in is a separate task; rule 2 alone reads 0 hits there, so
the gate is scoped to `src bevy_common_systems_macros/src` as it was before.

## Bounded limitation, recorded rather than coded around

A fn whose entire body sits on the `fn` line -- `fn seen() { assert!(x < 5.5); }`
-- is not scanned: the body-scanning branch starts on the line AFTER the
signature. It is unreachable in this repo, because `cargo fmt --check` is a CI
gate and rustfmt splits both that form and a one-line `#[cfg(test)] mod t {`
onto separate lines (verified by running rustfmt over both). Confirmed that the
formatted shape IS caught. Adding a same-line branch would buy nothing a
formatter already guarantees.
