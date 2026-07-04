# Review: radial gravity - RadialGravity component or documented recipe

- TASK: 20260704-161522
- BRANCH: feat/radial-gravity-recipe

## Round 1

- VERDICT: APPROVE

The right sketch-then-commit call, and the task explicitly sanctioned it
("only 2 games and avian-coupled; a documented recipe is an acceptable
outcome"). Verified the evidence independently: `10_asteroids` only sets
`Gravity(Vec3::ZERO)` (flat top-down, no radial pull), so `08_dropzone` is the
*sole* radial-gravity user, and it fuses the pull with wind in one
`ConstantLinearAcceleration` write (`:1624`). A `RadialGravity` component that
owned that channel would fight the wind term, and the idiom is a single line of
vector math -- so a component + system + plugin would be ceremony around a
one-liner. Recipe is clearly the better outcome; the decision is sound.

Doc quality is high and every claim checks out against the code:
- disable-global `Gravity(Vec3::ZERO)` matches `08:553` / `10:173`;
- the direction math `-position.normalize_or(Vec3::Y) * strength` matches
  `08:1624` (`-radial_up * GRAVITY`, `radial_up = position.normalize_or(...)`);
- world-space `ConstantLinearAcceleration` for gravity/wind vs local-space
  `ConstantLocalLinearAcceleration` for thrust matches `08:1609-1610` and the
  spawn at `:1279-1280`;
- the worked-example pointer `apply_ship_forces` is the real function (`:1600`);
- the off-origin `(c - position)` variant is correct.

Nice touches: the direction math is a *compiling, asserting* doctest (so even
the recipe carries a real assertion, satisfying the task's "unit-test the
direction math" where it applies), the avian wiring snippet is correctly marked
`ignore` (it references game-specific `Wind`/`GRAVITY`), and the change gives the
previously doc-less `physics` module its module-level doc.

Verified in the worktree: `cargo fmt --check`, `cargo clippy --all-targets`,
`cargo test` (67 unit + 34 doctests, the 2 new physics doctests among them) and
`scripts/check-ascii.sh` all pass. Diff is a single file, +70 lines, docs only.

No findings.
