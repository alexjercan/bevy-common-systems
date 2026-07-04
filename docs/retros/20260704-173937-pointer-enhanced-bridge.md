# Retro: enhanced-input bridge to UnifiedPointer (retire 06's local Pointer)

- TASK: 20260704-173937
- BRANCH: feat/pointer-enhanced-bridge (squash-merged to master as 06be48d)
- REVIEW ROUNDS: 1 (APPROVE, one NIT accepted as doc-only)

A follow-up from the input/pointer harvest (its review NIT R1.2): lift 06's
enhanced-input pointer machinery into a `helpers/` bridge that drives the shared
`UnifiedPointer`, retiring the last local `struct Pointer` copy. The code was
straightforward; the verification was the whole story.

## What went well

- Answered the planning fork by looking at the resource ownership: both the raw
  `UnifiedPointerPlugin` and the bridge write `UnifiedPointer` every frame, so
  they must be mutually exclusive and the bridge must own the *whole* resource
  (press + position), not just feed the press edge. Documenting "use it instead
  of, not alongside" fell straight out of that.
- Faithful lift: kept the exact schedule (`Startup` register, `PreUpdate` stage
  after `InputSystems` before `EnhancedInputSystems::Prepare`, `Last` clear, two
  observers) so it is a relocation, not a re-derivation. Byte-for-byte diffable
  against 06's old code.
- Verified an input refactor the *right* way. After GUI click-injection failed
  (below), I used the crate's own `AutopilotPlugin` -- its per-frame input
  closure presses `ButtonInput<MouseButton>` in `PreUpdate`, the bridge's
  enhanced-input action processes it for real, and a temp probe confirmed
  `pressed=true`, a correct one-frame `just_pressed` edge, a resolved
  `screen_pos`, and a clean `Menu -> Playing -> GameOver, no panic`. That is the
  headline: the just-shipped harness is the correct tool for verifying an
  input-driven refactor end-to-end, far more reliable than synthetic X events.

## What went wrong

- Burned a lot of time fighting two dead-end verification paths before reaching
  the autopilot:
  1. **xdotool click injection** never registered with the winit/wgpu window
     (synthetic X events don't reliably reach it here). Worse, my test harness
     kept killing *itself*: `pkill -f "examples/06_fruitninja"` and
     `pgrep -af "target/debug/examples/06_fruitninja"` matched the running
     shell's own command line (which contained those strings), so the shell got
     SIGTERM'd mid-script (exit 144) and logs went missing.
  2. **A headless `MinimalPlugins` unit test** for the bridge panicked in
     enhanced-input's action spawn -- the framework needs a fuller app than a
     minimal test provides. Removed it; this is exactly why the crate exercises
     ECS via examples, and 06 + autopilot is that exercise (same as
     `helpers/wasd`).
  Root cause: reached for the flashy end-to-end GUI proof first instead of the
  tool built for this. The autopilot should have been the first move, not the
  third.

## What to improve next time

- To verify an input-driven change in a stateful example, reach for
  `AutopilotPlugin` first: `.input(|world, _| world.resource_mut::<ButtonInput<_>>()
  .press(..))` plus a temp probe on the resource under test. Do not start with
  xdotool synthetic events -- they are unreliable against winit here.
- Never write a `pkill`/`pgrep` pattern that also matches your own shell command
  line. Match the running binary by a string that is NOT in your command (or use
  the PID you captured from `$!`), or you SIGTERM yourself.

## Action items

- [x] `helpers/pointer` bridge shipped; 06's last local `Pointer` retired. The
  input/pointer harvest's deferred follow-up is closed.
- [x] Saved a session memory on the pkill/pgrep self-match footgun.
- [ ] Optional (from the dev-harness retro too): migrate 07/09/10 onto the
  harness for uniformity; not filed.
