# Retro: 14_breach navigable main menu + options (persisted sensitivity)

- TASK: 20260705-151821
- BRANCH: feat/breach-options-menu (squash-merged to master as 7afba27)
- REVIEW ROUNDS: 1 (APPROVE)
- FOLLOW-UP FILED: tasks/20260705-154058 (game-over screen has the same no-camera bug)

## What went well

- Insisting on actually SEEING the menu (not just booting it) surfaced a real latent
  bug: the Menu state had no camera at all -- the only camera is the Playing Camera3d,
  a child of the player that despawns with it -- so Bevy UI never rendered. It had been
  invisible on master and nobody caught it, because every prior verification was blind to
  it: the autopilot force-transitions Menu->Playing (never needs the menu drawn) and the
  headless framebuffer capture (BCS_SHOT / ScreenshotPlugin) comes back black here. The
  fix (spawn a Camera2d in the menu) was one line once the cause was clear.
- The thing that finally let me see it was a REAL windowed run + `xdotool search` +
  `import -window <wid>` (not scrot of the root, not the app framebuffer). That grab
  showed the actual window content (dark clear-color, then after the fix the full menu),
  and a second grab after an xdotool click on OPTIONS showed the options panel. This is
  the reliable "see the screen" path in this environment -- worth more than either
  headless capture, both of which were useless (black).
- Menu navigation is a game-driven state change the autopilot structurally cannot prove
  (it forces the transition), exactly the breach lose-condition lesson. Because
  Interaction is a plain component, I could drive the REAL menu_buttons/menu_keys systems
  in a headless App by spawning entities with Interaction::Pressed and asserting the
  state/resource flips -- PLAY->Playing, OPTIONS/BACK nav, +/- step+clamp, and PLAY inert
  in the options panel. That is real coverage of the wiring, not just the pure math.
- Persistence was proven end to end, not assumed: the on-disk file read 0.8 and a fresh
  launch's options readout showed x0.8. PersistPlugin auto-saving on change did the work;
  I only had to make the resource type fit its bounds.

## What went wrong

- Process bug: I ran `tatr new` + populated the TASK.md during the plan phase while
  standing in the MAIN checkout, then sprouted the worktree WITHOUT committing the task
  first. The sprout branches from master's committed HEAD, so the task file was not in
  the worktree -- later `sed`/REVIEW.md writes in the worktree failed ("No such file"),
  and the untracked copy in the main checkout would have blocked the squash-merge. Fixed
  by copying the task dir into the worktree, committing it on the branch, and `rm`-ing the
  stray untracked copy from the main checkout before merging. Lesson: in flow, the plan
  phase's `tatr new` must be committed (or the task created from inside the worktree)
  BEFORE relying on it in the sprouted branch. Simplest rule: sprout first, create/commit
  the task on the branch.
- A generic helper over queries can't be a closure: `let clicked = |q| ...` monomorphised
  to the first marker type and gave four E0308s. A generic free fn
  (`fn any_pressed<M: Component>(q: &Query<&Interaction, (Changed<Interaction>, With<M>)>)`)
  is the right tool. Closures are not generic.
- Headless menu tests need InputPlugin (for ButtonInput<KeyCode>) and a SoundBank (so
  sfx.get does not panic; the PlaySfx trigger is a harmless no-op with no SfxPlugin). The
  first cut omitted InputPlugin and panicked with "Resource does not exist". MinimalPlugins
  is genuinely minimal -- add the specific plugins a system's params need.

## What to improve next time

- For any UI-state screen (menu, options, game-over, pause), verify it RENDERS with a real
  windowed run + xdotool/import, never trust a boot log or a headless framebuffer capture
  (black here). And check that the state actually HAS a camera -- a UI-only state whose
  only camera lives in another state renders nothing.
- In flow, treat "task exists on the branch" as a precondition: create the tatr task from
  inside the sprouted worktree, or commit it before sprouting. Do not populate a task in
  the main checkout and then branch off an older HEAD.

## Action items

- Follow-up task 20260705-154058: give the game-over screen its own Camera2d (same fix),
  and verify "YOU DIED" + score actually render with a real window grab.
