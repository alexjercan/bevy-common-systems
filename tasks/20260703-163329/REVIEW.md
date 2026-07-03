# Review: Verify web audio playback and handle the browser autoplay policy

- TASK: 20260703-163329
- BRANCH: feature/web-audio

## Round 1

- VERDICT: APPROVE

Diff reviewed: the task-2 commit adds an "Audio and the autoplay policy"
subsection to `docs/wasm-web-builds.md` and repoints the Assets-section
autoplay pointer at it. No code changed -- the deliberate outcome of the
task's decision point.

Judged against the Goal and Steps:

- **The "no code change" decision is correct and well-supported.** The Outcome
  cites grounded sources (MDN autoplay guide, Chrome web-audio-autoplay, cpal
  PR #774, bevy#15273) for the mechanism: context starts suspended, browsers
  auto-resume once the user has interacted and a source `start()`s. Verified
  the two load-bearing claims against the repo: (a) the first sound really does
  fire on the in-canvas click -- `examples/06_fruitninja.rs:855-856`,
  `menu_click` plays `menu_select` on `MouseButton::Left` just-pressed in the
  Menu state; (b) the iframe already carries `allow="autoplay; fullscreen;
  gamepad"` -- `web/src/index.html:38`. So there is genuinely nothing to add in
  Rust or in the web layer. Adding a speculative unlock would violate the
  crate's minimalism and, per the research, isn't cleanly possible (Bevy does
  not expose its `AudioContext`). Good call.
- **Verification is honest.** I re-ran the automated half: served `web/dist/`
  and curl'd the assets -- `index.html` 200, `.wasm` 200 `application/wasm`,
  all eight `assets/sounds/*.wav` 200 at `/games/06_fruitninja/assets/sounds/`.
  Matches the Outcome. The aural check is explicitly marked NOT done (headless,
  no audio device) and handed off with exact, runnable steps rather than
  claimed as success -- exactly what the Step asked for when the check can't run
  autonomously.
- **Docs are accurate and balanced** -- they state the auto-resume conditions,
  the in-iframe gesture requirement, the pre-gesture-sound caveat, and the
  #15273 timing quirk (not overstated as guaranteed). ASCII-clean; fmt/ascii
  pass; Rust CI unaffected.

Findings:

- [x] R1.1 (MINOR) tasks/20260703-163329 - the Goal asks for "concrete evidence
  about whether SFX are audible." What exists is strong *indirect* evidence
  (HTTP 200 fetches + grounded auto-resume research), not a human "I heard it"
  confirmation, which this headless environment cannot produce. This is within
  the Step's stated fallback ("if this cannot be run autonomously, state it
  needs the user and hand off steps"), so it does not block the branch -- but
  the goal is not fully closed until someone runs the handoff steps and
  confirms audio. Tracked as the one open user action, not a code change.
  - Response: Acknowledged; leaving as the single open user action (the aural check). Handoff steps are in the Outcome. No code implication. (implementer)

No BLOCKER/MAJOR findings. The branch does everything achievable without a
graphical session, the decision to add no code is justified and documented, and
the residual aural check is honestly deferred. Approved.

## Round 2

- VERDICT: APPROVE

User confirmed audio is audible in the browser build ("audio is fine in browser"). R1.1 resolved -- the aural check is done, the goal is fully closed. No further findings.
