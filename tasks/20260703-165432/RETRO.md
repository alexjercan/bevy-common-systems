# Retro: 08_dropzone - land a ship on the noise planet with PD controller

- TASK: 20260703-165432
- BRANCH: feature/08-dropzone (merged to master as f663dff)
- REVIEW ROUNDS: 1 (APPROVE; two MINORs addressed same round)

See `tasks/20260703-165432/TASK.md` and `tasks/20260703-165432/NOTES.md`
for what was built and why; this is about how the working went.

## What went well

- Front-loaded the hard API verification. Before writing, I read the actual
  avian 0.7 and bevy 0.19 source for the non-obvious pieces: the reworked
  persistent force components (`ConstantTorque` / `ConstantLinearAcceleration` /
  `ConstantLocalLinearAcceleration`), `CollisionStart` + `CollisionEventsEnabled`,
  `Collider::trimesh` signature, and runtime `Image::new` for the skybox. All of
  those compiled first try. Reading the engine source rather than guessing paid
  off exactly where the risk was highest.
- Delegated the broad API/idiom digest and an independent skeptical review to
  subagents. The reviewer traced the crash -> fragments -> despawn ordering I was
  unsure about and confirmed it was race-free, and caught the two menu-camera
  MINORs. That is cheaper and more trustworthy than self-reviewing my own code.
- Isolated in a sprout worktree, so a full-from-scratch dependency build ran in
  the background without blocking the shared checkout or other jobs.

## What went wrong

- Nine compile errors on the first build, ALL from bevy 0.19 UI/light API drift
  I wrote from memory: `AmbientLight` is now a per-camera component (not a
  resource), `TextFont.font_size` is `FontSize::Px(..)` (not `f32`), and
  `TextLayout::new_with_justify(..)` is gone (struct literal now). Root cause: I
  verified the exotic APIs meticulously but treated text/light as "boring" and
  assumed older-Bevy spellings. The API-digest subagent had even quoted
  `font_size: FontSize::Px(size)` straight from 06_fruitninja, and I still wrote
  bare floats. The reference was in hand and not applied.
- This is a REPEAT. `tasks/20260703-150200/RETRO.md` already
  logged the same FontSize surprise and the fix ("a single `grep 'font_size:'`
  up front would have found all sites"). Same lesson, second occurrence -> it
  belongs in AGENTS.md, not just another retro.
- Two MINOR camera findings (menu renders from inside the planet; first run
  swoops out from the planet centre). Root cause: I gated `drive_chase_camera`
  to the Playing state without thinking about what a chase camera driven by an
  `*Input` component does in states where nothing writes that input - it sits at
  the default (origin). A plugin that consumes an Input every frame needs its
  behavior decided for ALL states, including menus with no target.

## What to improve next time

- When mirroring an existing example, copy the shared UI/engine idioms verbatim
  from that example (grep it for `font_size:`, `AmbientLight`, `TextLayout`,
  camera setup) instead of writing them from memory. Cheapest correctness win on
  an engine that moves this fast.
- For any `*Plugin` that reads an `*Input` component every frame, decide and wire
  its behavior in every state up front (menu/result/playing), not just the
  active one.

## Action items

- [x] Proposed AGENTS.md gotcha: when adding a new example's UI, copy bevy 0.19
  text/light idioms from an existing example rather than memory (FontSize::Px,
  AmbientLight per-camera, TextLayout struct literal).
- [ ] tatr 20260703-213510: play-test and tune 08_dropzone flight feel on a
  machine with a display (constants are reasoned, not flown).

## Addendum: shipped a startup hang (fixed on fix/dropzone-hang, merged 19a1fba)

The example hung on launch: `PLANET_RESOLUTION = 24` fed
`new_octahedron(depth)`, which subdivides recursively (`8 * 4^depth` triangles),
so `setup()` tried to build ~2e15 triangles and never returned - blank,
unresponsive window. Fixed by using depth 6 (32768 triangles).

Root cause and the real lesson: I never ran the example. I assumed a headless
background session could not launch a graphical Bevy app, but `DISPLAY=:0` was
set the whole time - a single `cargo run` would have caught the hang in seconds.
Reasoning about constants (the "tuned by reasoning, not flown" caveat) is no
substitute for running the thing once. Next time: before declaring an
interactive example done, actually run it (check `$DISPLAY`; run headless under
a timeout if needed) rather than assuming it cannot be run.

- [x] Fixed the hang (depth 24 -> 6) and clarified the constant is a recursive
  depth.
