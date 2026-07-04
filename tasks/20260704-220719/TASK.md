# build examples/13_turret -- first-person turret defense, demo of point_rotation + smooth_look_rotation

- STATUS: OPEN
- PRIORITY: 1
- TAGS: spike,examples,game

> Spike: docs/spikes/20260704-220530-new-prototype-game-ideas.md (read first).
> Recommended game #2 (Option B). Build after 12_warden (tasks/20260704-220736).

## Goal

A small (~2000 line) first-person turret-defense prototype -- the only concept
that homes **both** remaining transform gaps: `transform/point_rotation` (player
aim) and `transform/smooth_look_rotation` (enemy tracking). Neither is exercised
by any example today; the instant-aim-vs-smooth-track contrast teaches the two
APIs against each other on one screen.

Concept: you are a stationary turret at the origin. Mouse movement feeds
`PointRotationInput` to swing the barrel (snappy, free 360 aim -- exactly what
`point_rotation` is for); click fires a ray/tracer along the barrel forward.
Enemies advance from all bearings; rotate to prioritize and shoot them before
they reach the base (base `Health`). Distant enemy emplacements use
`SmoothLookRotation` to slowly, visibly track the player and fire telegraphed
shots -- rate-limited, clamped tracking as the deliberate contrast to the
player's instant aim. Kills insert `ExplodeMesh`; `ui/status` shows base
integrity / wave / score.

Reuse the established example shape: menu/playing/game-over `States`, `SfxPlugin`
one-shots, wasm/trunk build, and the juice kit (`ui/popup`, `camera/shake`,
`feedback`, `tween`, `scoring/streak`).

Main risk (from the spike): mouse-look wants pointer lock, fiddly on wasm and
awkward for touch. Prototype the aim control FIRST; provide a touch fallback
(drag-to-aim, or an on-screen look pad via `ui/touchpad`, proven in 08/11) and
fall back to drag-to-aim if pointer lock feels bad.

This task is stepless on purpose (spike output); run /plan to break it into
steps before /work.
