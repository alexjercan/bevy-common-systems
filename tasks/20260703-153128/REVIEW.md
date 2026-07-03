# Review: enlarge game embed + center game page title

- TASK: 20260703-153128
- BRANCH: web-showcase (sprout worktree)

## Round 1

- VERDICT: APPROVE

Self-review. Embed is 560px / 4:5 (560:700) - larger and a touch wider; at
aspect 0.8 the horizontal half-view (~7.3 world units) still covers the fruit
spawn range (x up to +/-6), so nothing clips. Title centering uses an absolute
back button + centered flex, so "Fruit Ninja" sits centered in the page with
the back control on the left (no overlap for a short title). Info panel width
matched to the embed. No game/wasm change. Checks clean: prettier + eslint,
build:web exit 0, static-serve smoke 200.
