# build examples/12_bastion -- defend-the-core tower defense (camera/project + rotation modules)

- STATUS: OPEN
- PRIORITY: 2
- TAGS: spike,examples,game

> Spike: docs/spikes/20260704-220530-new-prototype-game-ideas.md (read first,
> the revised Recommendation). This merges the original 12_warden + 13_turret
> ideas into one game per the user's steer.

## Goal

A small (~2000 line, target ceiling) unconventional tower-defense prototype that
closes ALL THREE never-demoed modules in one game: `camera/project`,
`transform/smooth_look_rotation`, and `transform/point_rotation`.

Concept: a **Core** (a `Health` pool -- the thing you defend) sits at the center
of a circular play plane; **enemies spawn all around the border and converge
inward** from every bearing. Kills earn credits; spend them to **place towers**
on the plane around the Core and to **upgrade** them. An enemy reaching the Core
damages it (`HealthApplyDamage`); Core health zero ends the run. Waves ramp
count/speed/HP.

Module homes (the point of the example):

- **`camera/project` (headline).** `pointer_on_plane` maps mouse/tap to the
  world placement point (ghost tower + range ring there); `world_to_screen`
  anchors floating UI -- enemy health pips, "+N" credit popups on kill, the
  click-to-upgrade panel over a selected tower. First validated user of the
  harvested module (no example imports it today).
- **`transform/smooth_look_rotation` (tower turrets).** Towers auto-target the
  nearest enemy in range; the turret rotates toward it with `SmoothLookRotation`
  -- rate-limited, so a fast enemy can out-slew a cheap turret until upgraded.
  Turn-rate is a real tunable stat.
- **`transform/point_rotation` (orbit camera).** Camera rides a rig aimed at the
  Core; mouse-drag / A/D feed `PointRotationInput` to accumulate yaw/pitch and
  orbit the view around the battlefield.

Keep the first cut simple (the brief): 2-3 tower archetypes, a one-axis upgrade
per tower, 1-2 enemy types, hand-authored waves. Reuse the established shape and
juice kit: menu/playing/game-over `States`, `SfxPlugin` one-shots, `ui/status`
HUD (credits/wave/Core integrity), `mesh/explode` on kills, `ui/popup`,
`camera/shake`, `feedback`, `tween`, `scoring/streak`, `time/cooldown` (per-tower
fire cadence), `input/pointer`, wasm/trunk build. Touch-native: tap-to-place,
drag-to-orbit.

Build the tower/enemy stats as a **game-local serde catalog** (TowerSpec /
EnemySpec spawned by name) so the modding follow-up (tasks/20260704-220719) can
build on it -- but do NOT block this task on any crate-level abstraction.

Open at implementation (from the spike): camera angle (angled perspective vs
top-down; pick with `ScreenshotPlugin`); clamp the orbit pitch in-game
(`point_rotation` has no min/max -- consider harvesting one back); keep placement
usable at grazing angles (constrain pitch or snap to a ring/grid);
nearest-in-range auto-target is the simple default. Watch the line budget -- this
game is more mechanic than any single earlier example, so lean on the juice kit
and keep variety minimal.

This task is stepless on purpose (spike output); run /plan to break it into
steps before /work.
