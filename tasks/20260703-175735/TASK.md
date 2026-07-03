# CI: run example unit tests (cargo test --examples)

- STATUS: OPEN
- PRIORITY: 60
- TAGS: ci,tests

## Goal

Make the CI test step actually run the `#[cfg(test)]` tests that live inside
`examples/`. Today `cargo test` builds the examples but does not run their unit
tests, so `examples/06_fruitninja.rs` alone has 19 in-file tests (geometry,
combo math, difficulty ramp, and the new `active_pointer_pos` cases) that never
execute in CI and can silently rot.

## Steps

- [ ] Add `cargo test --examples` (and `--examples --features debug` if it adds
      example coverage) to `.github/workflows/ci.yml`, or fold `--examples` into
      the existing `cargo test` invocations. Confirm the example tests run and
      pass (`cargo test --example 06_fruitninja` = 19 tests today).
- [ ] Update AGENTS.md "Build, Verify, Run" if the documented test command
      changes.

## Notes

- Surfaced by the retro for task 20260703-173128 (touchscreen support): the 3
  new `active_pointer_pos` tests pass under `cargo test --example
  06_fruitninja` but not under a plain `cargo test`. Pre-existing gap, not
  caused by that task.
