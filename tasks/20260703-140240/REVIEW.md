# Review: slice pop flash

- TASK: 20260703-140240
- BRANCH: feature/ninja-flash

## Round 1

- VERDICT: APPROVE

Self-review. Scale-only pop (no material swap) keeps fragment colors correct
and avoids a flash-material handle; base scale is restored before ExplodeMesh
so fragments match size. Bombs deliberately skip the pop (instant explode +
instant loss), leaving the death beat intact. Checks clean; verified on real
GPU that fruit pop then burst into fragments (max_pop=1, max_frag=15), no panic.
