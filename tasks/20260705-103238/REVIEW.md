# Review: Harvest FP character controller / camera-wasd upgrades from 14_breach

- TASK: 20260705-103238
- BRANCH: fps-harvest

## Round 1

- VERDICT: REQUEST_CHANGES

Verification run: `cargo clippy --all-targets` clean (plain + `--features debug`),
102 lib tests + 54 doctests pass, `check-ascii` + `cargo fmt --check` clean, the
rewired `14_breach` autopilot runs Menu->Playing->GameOver "cycle complete, no panic"
with 6 kills. Rewire fidelity confirmed (eye euler, look accumulation, velocity
application byte-equivalent to the originals; `move_dir`/pitch tests moved, not lost;
gun query -> `With<DoomEye>` and autopilot -> `DoomControllerState` updated correctly).

- [ ] R1.1 (MAJOR) `examples/14_breach.rs` Update systems (~:214-216) - `feed_look`
  and `feed_move` write `DoomControllerInput` but carry no ordering edge relative to
  `DoomControllerSystems::Drive` (only `apply_move_velocity` has `.after(Drive)`). The
  executor may run `drive_controller` before the writers, so the module consumes last
  frame's look/move -- a one-frame lag on mouse-look AND movement. This is a feel
  regression from the pre-harvest code, where `player_look` read the mouse and wrote
  the camera in one system (zero lag), and the autopilot can't catch it (it force-writes
  `DoomControllerState.yaw`, bypassing the input path). Fix: pin the writers with a
  direct edge -- `feed_look.before(DoomControllerSystems::Drive)`,
  `feed_move.before(DoomControllerSystems::Drive)` (and keep `read_touch` before the
  feeds, since they read `TouchInput`). (Same "pin the writer, do not rely on insertion
  order" lesson as the camera/shake retro.)
  - Response: Fixed. In 14_breach the input feeds are now `(read_touch, feed_look, feed_move).chain().before(DoomControllerSystems::Drive)`, so the controller reads this frame's input; apply_move_velocity stays `.after(Drive)`.
- [ ] R1.2 (MAJOR) `src/physics/doom_controller.rs` `DoomControllerSystems::Drive` doc
  (~:136-139) + module doc (~:40) - the module documents only the downstream half
  ("your velocity-application system should run after this set") and never states that
  game code must write `DoomControllerInput` BEFORE `Drive`. Because the contract is
  undocumented, the example got it wrong (R1.1) and any future consumer will too. Fix:
  add the `.before(DoomControllerSystems::Drive)` contract to the `Drive` doc alongside
  the existing `.after` guidance, and to the module-level usage note.
  - Response: Fixed. The Drive-set doc and the module usage note now state the full contract: write DoomControllerInput `.before(DoomControllerSystems::Drive)` and apply the velocity output `.after` it.
- [ ] R1.3 (MINOR) `src/physics/doom_controller.rs` `orient_eye` (~:186-195) - the
  behavior-driving system (the `ChildOf` pairing, the `.get()` on a parent that may lack
  state, the YXZ euler write) has zero test coverage; the App test only exercises
  `drive_controller`. Per the repo standard ("test logic-that-drives-behaviour, not just
  the easy half"), add a test that spawns a body + a `DoomEye` child, sets `State`,
  updates, and asserts the child `Transform.rotation ~= Quat::from_euler(YXZ, yaw, pitch,
  0)` -- ideally a second body/eye pair to prove each eye pairs to ITS own controller.
  - Response: Fixed. Added `orient_eye_writes_each_eye_from_its_own_controller`: two body+eye pairs with different orientations, asserts each DoomEye child's rotation matches ITS OWN controller's yaw/pitch (proves the ChildOf pairing + the euler write). 7 module tests now.
- [ ] R1.4 (NIT) `src/physics/doom_controller.rs` `DoomControllerOutput` (~:110-115) -
  a single-field struct where the cited template `PDControllerOutput(pub Vec3)` is a
  `Deref` newtype. Defensible (`.velocity` self-documents and leaves room for a future
  field), so take-it-or-leave-it; noting the inconsistency.
  - Response: Accepted (NIT). Kept `DoomControllerOutput { velocity }` a named struct: it self-documents and leaves room for a future field (e.g. grounded flag); AGENTS.md only mandates Deref for newtypes, so nothing is violated.
- [ ] R1.5 (NIT) `tasks/20260705-103238/TASK.md` (~:67-71) - the step still says the
  plugin should `add_observer` to init the companions; the code uses `#[require(...)]`
  instead (an improvement -- avoids pd_controller's redundant require+observer). Update
  the task text so it matches the shipped code.
  - Response: Fixed. Updated the TASK.md step to say init is via `#[require(...)]` (not add_observer), matching the shipped code.

Confirmed correct (not findings): `ChildOf::parent()` is the right 0.19 API; `#[require]`
correctly replaces the observer; prelude/mod wiring correct; `input/cursor` 0.19 usage;
docs (`tasks/20260705-103238/NOTES.md`, AGENTS.md) match the code without
overclaiming.

## Round 2

- VERDICT: APPROVE

Re-verified the addressed diff:

- [x] R1.1 (MAJOR) - resolved. Input feeds chained `.before(DoomControllerSystems::Drive)`;
  no more one-frame lag. Autopilot still clean (6 kills).
- [x] R1.2 (MAJOR) - resolved. The write-before / apply-after contract is documented on
  the Drive set and in the module usage note.
- [x] R1.3 (MINOR) - resolved. `orient_eye` covered by a two-pair pairing test.
- [x] R1.4 (NIT) - accepted (named struct kept, defensible).
- [x] R1.5 (NIT) - resolved. TASK.md step updated to `#[require]`.

Full gate green: `cargo clippy --all-targets` clean (plain + `--features debug`), 103 lib
tests (7 doom_controller) + doctests pass, `check-ascii` + `cargo fmt --check` clean, the
rewired `14_breach` autopilot runs Menu->Playing->GameOver "cycle complete, no panic" with
6 kills.

All findings resolved. Approved.
