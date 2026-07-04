# Review: input/pointer unified mouse+touch pointer resource

- TASK: 20260704-161508
- BRANCH: feat/input-pointer

## Round 1

- VERDICT: APPROVE

Clean Wave A harvest. New `input/` top-level module wired into `lib.rs` and the
crate prelude following the established pattern; `input/pointer` ships a
`UnifiedPointer` resource maintained by `UnifiedPointerPlugin` in `PreUpdate`,
plus the pure `active_pointer_pos` helper with the touch-wins-over-cursor unit
tests (moved out of 06). Module/item docs and the `debug!`/`trace!` logging all
match the crate conventions.

Spec check:
- Reads raw `Touches` only, forces no `bevy_enhanced_input` dependency, and
  exposes `active_pointer_pos` for the enhanced-input path -- exactly the
  spike's resolution of the open question. The enhanced-input bridge is left
  out of core (deferred to `helpers/`), so 06 keeps its own press handling.
- Proof-by-refactor: 10_asteroids fully adopts the plugin (its local `Pointer`
  + `update_pointer` deleted); 06_fruitninja dedups the shared
  `active_pointer_pos` while keeping its enhanced_input press bridge. Meets
  "asteroids (and ideally one more)".

Behavior-preservation verified against the deleted asteroids `update_pointer`:
`Single<&Window>` -> `Query<&Window, With<PrimaryWindow>>().iter().next()` (more
robust, equivalent for a single-window game), `or_else` -> `or` (same result,
no side effects), same `PreUpdate` schedule. 06's local `struct Pointer` does
not collide with the crate's `UnifiedPointer` (distinct names) nor with bevy's
prelude `Pointer`.

Naming: renaming the resource `Pointer` -> `UnifiedPointer` is the right call --
`Pointer` in the crate prelude collides with bevy's prelude `Pointer` (the
`bevy_picking` event, verified E0659), and the crate's charter is prelude-and-go.
The rename is documented at the type and module level.

Verified independently in the worktree: `cargo fmt --check`,
`cargo clippy --all-targets` (default and `--features debug`), `cargo test`
(57 unit + 26 doctests), `cargo test --examples` (06 now 16, having shed the 3
moved tests) and `scripts/check-ascii.sh` all pass; 10_asteroids boots to the
render loop.

- [x] R1.1 (NIT) examples/10_asteroids.rs:209 - the comment says "the crate's
  `PointerPlugin`" but the plugin is `UnifiedPointerPlugin`. Update the comment
  to the real name.
  - Response: Fixed. Comment now reads `UnifiedPointerPlugin`.
- [x] R1.2 (NIT) examples/06_fruitninja.rs:283 - 06 still carries a local
  `struct Pointer` whose shape duplicates `UnifiedPointer`; it stays because its
  `pressed`/`just_pressed` come from `bevy_enhanced_input` observers, which the
  task deferred to a `helpers/` bridge. Not fixable within this task's scope;
  worth a follow-up tatr task to add the enhanced-input bridge and retire the
  last copy. Informational, not blocking.
  - Response: Tracked as follow-up tatr 20260704-173937 (helpers/ enhanced-input
    bridge to UnifiedPointer), per the flow "new work becomes a new task" rule.
    Left as designed for this task's scope.
