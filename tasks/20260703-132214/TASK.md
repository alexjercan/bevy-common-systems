# Fruit ninja: combo scoring and combo text

- STATUS: OPEN
- PRIORITY: 80
- TAGS: feature,example

## Goal

Reward slicing several fruits in one continuous swipe: each fruit in the swipe
is worth one more point than the last (1, 2, 3, ...), and a multi-fruit swipe
shows a flashy "COMBO xN" banner. This turns the flat "+1" into escalating
"+N" popups plus combo feedback.

## Steps

- [ ] Add a `Combo { count: usize }` resource; `init_resource` it in `main`.
- [ ] In `slice_objects`, reset `Combo.count` to 0 in the not-pressed branch
      (a released button ends the swipe / combo), alongside the existing
      `CursorTrail` / `BladeTrail` reset.
- [ ] When a fruit is sliced, increment `Combo.count`, award `points =
      Combo.count` to `Score` (so the k-th fruit in a swipe gives k points), and
      show the "+points" popup (reuse `spawn_floating_text` from
      20260703-132210) with a size/color that scales with the combo for punch.
- [ ] When `Combo.count >= 2` on a slice, also spawn a "COMBO xN" banner via
      `spawn_floating_text` - larger font, vivid color (e.g. orange/gold),
      placed at the slice screen position or just above it - so a combo reads
      as special.
- [ ] Update the module `//!` doc and AGENTS.md `06_fruitninja` description to
      mention the blade trail, floating score, and combos.
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, and a real boot: slicing several
      fruit in one swipe escalates the "+N" and shows the combo banner
      (throwaway auto-multi-slice if needed).

## Notes

- Depends on: 20260703-132210 (reuses `spawn_floating_text`) and, for the reset
  site, 20260703-132207 (`BladeTrail` clear lives in the same not-pressed
  branch). Priority orders it last.
- Design (chosen default): a "combo" is one continuous LMB press. Multiple
  fruit sliced during that press - whether in the same frame (one segment
  crossing several) or across frames of the hold - all count toward the same
  combo; releasing the button resets it. This matches the fruit-ninja feel and
  is simple; note it in a comment. If the user wants a time-window combo
  instead, that is a follow-up.
- Points model: k-th fruit in a swipe = k points (1,2,3,...). This is the
  "+1 extra per combo" the request asked for. `Score` remains a running total.
- Keep the combo banner cheap: it is just another `FloatingText` with a bigger
  font and a bright color; no new system needed beyond `animate_floating_text`.
- No new dependencies.
