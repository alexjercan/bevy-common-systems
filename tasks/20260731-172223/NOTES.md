# NOTES - 20260731-172223 (KISS pass: integrity/ + physics/)

Design/fix record. What changed, why, alternatives, difficulties.

## Scope and baseline

8 files in `src/integrity/` + `src/physics/`. Base carried **45 untagged
non-doc comment blocks** and **1 bare tatr HUID** (`pd_controller.rs:327`,
"nova task 20260709-125640").

## Comment triage

Keep = kept as-is (already tagged / already earning its keep). Compact =
survives as one tagged block, shorter. Drop = deleted, restatement or
narration. Line numbers are the BASE (master) positions.

### src/integrity/blast.rs (2 triaged, 1 untagged)

| Base | Call | Why |
| --- | --- | --- |
| 31 `linear falloff for now; other models could be added later` | drop | speculative roadmap, YAGNI. The code says linear; a maybe-later is not a hazard. |
| 60-63 `The blast owns its collision events...` | compact | load-bearing hazard (a future reader WILL try to delete `CollisionEventsEnabled`). 4 lines -> 2, opens `NOTE: do not drop.` |

### src/integrity/components.rs (0 blocks)

Clean. No change.

### src/integrity/mod.rs (0 blocks)

No comments. Fixed one rustdoc redundant explicit link target
(`[`IntegrityPlugin`](plugin::IntegrityPlugin)` -> `[`IntegrityPlugin`]`) and
added `mod damage;` for the split.

### src/integrity/plugin.rs (24 triaged, 22 untagged - the bulk)

| Base | Call | Why |
| --- | --- | --- |
| 87 `We exclude BlastDamageMarker... double-dipping` | compact | real invariant; retagged + reworded to the consequence, not the mechanism. |
| 157 `Maybe we want the distance between the colliders` | compact | a genuine limitation stated as a shrug. Rewritten as a `NOTE:` stating the actual behaviour (origin-to-origin, so a large body is judged by its centre). |
| 176-177 `Only act when this side of the event is the blast...` | drop | restates the `if` directly beneath it. |
| 362 `Dropping the hub to one neighbor makes it a leaf.` | drop | restates the assertion. |
| 383 `A disabled interior node is deactivated, not destroyed` | drop | the test name already says this. |
| 408 `The whole body dies when its root is disabled` | drop | ditto. |
| 422 `The headline sequence: damage -> health zero -> disabled -> destroyed.` | drop | test name is `damage_drives_a_leaf_from_full_health_to_destruction`. |
| 429 `Derive the leaf marker (no neighbors -> leaf).` | drop | restates `app.update()`. |
| 434 `Fatal damage.` | drop | restates `trigger(HealthApplyDamage(...))`. |
| 449-451 `A line A-B-C, all disabled...` | compact | the ONLY test comment kept: it explains the cascade shape the asserts cannot show (why B dies second). One tagged `NOTE:`. |
| 463 `Let the leaf derivation and chain reaction settle.` | drop | restates a loop of `update()`. |
| 511-514 `A real collision is left to the solver...` | compact | genuinely non-obvious test design (why the event is injected rather than simulated). Kept, tagged, tightened. |
| 520 `Head-on closing velocity of 40 (=20 - -20).` | compact | -> end-of-line label on the binding: `let rel = 40.0_f32; // head-on closing velocity, 20 - -20`. |
| 536-537 `Recompute the expected damage from the real mass...` | drop | the code recomputes visibly from the named constants. |
| 544 `Damage lands on collider1 ... only there` | drop | two asserts say exactly this. |
| 551-552 `Two bodies barely moving... resting stacks would grind` | compact | the rationale (resting debris) is not derivable; kept as one tagged block. |
| 584-587 `Unlike the impact case, a sensor overlap fires a real...` | compact | explains why this test drives the engine and the previous one does not. Kept, tightened. |
| 590 `Blast centred 4.0 away, radius 10.0 -> 0.6 -> 60` | compact | -> end-of-line label on the expected-damage binding. |
| 608-612 `Regression for the ordering bug...` | compact | a real regression record. Retagged `BUG:` and cut to the failing scenario. |
| 618 `Collider spawned without Health, so no events` | drop | restates the spawn. |
| 630-631 `Now give it Health to take damage...` | drop | restates the insert; the parenthetical duplicates the kept `BUG:` block above. |
| 647-649 `avian raises both orderings... damaged exactly once` | compact | the "60, not 120" is the point of the test; folded into an end-of-line label + the assert message. |
| 668-669 `A body beyond the sensor's reach never overlaps` | drop | test name says it. |
| 672 `Blast radius 5 centred 8 away; target ~7 out` | compact | -> end-of-line geometry label. |

### src/physics/doom_controller.rs (5 triaged, 5 untagged)

| Base | Call | Why |
| --- | --- | --- |
| 77 `Just under +/- 90deg, so the view cannot flip` | compact | guards a value (1.5708). Retagged `NOTE:`. |
| 260 `yaw -= 50 * 0.01 = -0.5` | compact | -> end-of-line label on the assert. |
| 286 `pitch -= look.y * 0.1; a big negative look.y...` | compact | the sign is the trap. Kept as a tagged block. |
| 299 `Two independent body+eye pairs...` | drop | restates the spawns. |
| 322 `Each eye takes ITS OWN controller's orientation` | drop | restates the two asserts. |

Also fixed a broken intra-doc link: ``[`examples/14_breach`]`` (no such item,
rendered as a dead link) -> plain code span `` `examples/14_breach.rs` ``.

### src/physics/mod.rs (0 blocks)

No comments, but the **module doc was stale**: it listed only
`pd_controller` while the module also ships `doom_controller` and
`rigid_body`. Added both entries.

### src/physics/pd_controller.rs (14 untagged in 10 rows)

| Base | Call | Why |
| --- | --- | --- |
| 115 `// PD gains` | drop | labels two fields named `kp`/`kd`. |
| 130 `Normalize axis (avoid NaNs if angle is zero)` | compact | -> end-of-line `// normalize_or_zero, not normalize: angle 0 gives no axis`. Found a **duplicated `axis = axis.normalize_or_zero();`** while doing it (see below). |
| 133 `// PD control (raw torque)` | drop | labels the one line under it. |
| 136-143 `Scale the raw PD acceleration by the world-space inertia tensor...` | compact | the single most load-bearing comment in scope - the `Q diag Q^-1` sandwich order is invisible in the code and wrong-way-round passes every identity-frame test. Kept, tagged `NOTE:`, 8 lines -> 5. |
| 149 `Optionally clamp final torque magnitude` | drop | restates `if let Some(max)`. |
| 320-327 nova HUID provenance banner | compact | the HUID pointed at another repo's tracker - dead reference here, and the DoD bans bare HUIDs. Kept the *scenario* (symmetric top, roll about its long axis, command frozen at release = the corkscrew), dropped the provenance. |
| 358-359 `Asset + mesh plugins: avian's collider cache reads AssetEvent<Mesh>` | compact | non-obvious dependency; keeps someone from trimming the plugin list. Tagged. |
| 377 `Avian initializes its diagnostics resources in Plugin::finish` | compact | guards a `finish()` call that looks removable. Tagged `NOTE: do not drop`. |
| 419-420 `Let avian link colliders and finalize mass properties` | compact | ordering hazard; tagged. |
| 445, 465, 527, 551, 570 `10 s / 30 s of sim at 60 Hz` | drop | five copies labelling five magic loop counts. Replaced by a `simulate_seconds(app, secs)` helper, which deletes the comment AND the magic number (see below). |

### src/physics/rigid_body.rs (3 triaged, 3 untagged)

| Base | Call | Why |
| --- | --- | --- |
| 70 `No rotation: every point moves at the body's linear velocity` | drop | test name says it. |
| 82 `omega x 0 = 0, so a muzzle on the COM...` | drop | ditto. |
| 94-95 `Spin about +Y ... omega x r = (0,2,0) x (3,0,0) = (0,0,-6)` | compact | the hand-computed oracle IS the value of the test. One tagged `NOTE:`. |

Per-file untagged blocks sum to the DoD's 45: 1 + 22 + 5 + 14 + 3.

**Result: 45 -> 0 untagged blocks, 1 -> 0 bare HUIDs.**

## Split decision: code-before-tests per file

Measured as lines before the first `#[cfg(test)]`.

| File | Base total / code | After total / code | Split? |
| --- | --- | --- | --- |
| `integrity/blast.rs` | 64 / 64 | 64 / 64 | keep |
| `integrity/components.rs` | 66 / 66 | 66 / 66 | keep |
| `integrity/mod.rs` | 26 / 13 | 28 / 15 | keep |
| `integrity/plugin.rs` | **679 / 317** | 304 / 166 | **SPLIT** |
| `integrity/damage.rs` (new) | - | 392 / 183 | - |
| `physics/doom_controller.rs` | 324 / 201 | 324 / 201 | keep |
| `physics/mod.rs` | 79 / 79 | 82 / 82 | keep |
| `physics/pd_controller.rs` | 581 / 157 | 564 / 150 | keep |
| `physics/rigid_body.rs` | 101 / 64 | 101 / 64 | keep |

### Why `integrity/plugin.rs` split

317 lines of code carrying **two concerns with different dependency sets**:

- collisions -> damage: needs avian (`CollisionStart`, `ColliderOf`,
  `ComputedMass`), owns three tuning constants, ~155 lines;
- disable -> destroy -> prune -> cascade: pure ECS graph work, no avian at
  all, ~103 lines.

The decisive evidence was that **the tests had already drawn the seam**: the
file had two test modules, `mod tests` (avian-free cascade) and
`mod physics_tests` (avian-driven damage), and they never shared a helper.
The split follows that line exactly. New private `src/integrity/damage.rs`
takes the damage half plus `mod physics_tests`; `plugin.rs` keeps the wiring
and the cascade.

Public API is unchanged: `damage` is a *private* module, the three moved
observers become `pub(super)`, and `IntegrityPlugin` still registers all
eight observers itself. Nothing left a prelude.

### Why `pd_controller.rs` did NOT split

564 lines but only **150 of code** - the bulk is one avian integration test
rig. One concern (PD attitude control), one config/input/output/state family.
Splitting would move the test rig away from the code it exercises for no
gain. Same reasoning for `doom_controller.rs` (201 code lines, one concern).

## Code changes beyond comments

Two, both surfaced by the comment pass rather than sought:

1. **Duplicated normalisation** in `pd_controller`'s torque path: `axis` was
   run through `normalize_or_zero()` twice. Removing the second is a no-op
   semantically (idempotent) but the duplicate reads as if the two calls do
   different things.
2. **`simulate_seconds(app, seconds)` test helper** in `pd_controller`'s test
   module, replacing five `for _ in 0..600 {}` / `0..1800 {}` loops. This is
   the fix for the five duplicated "N s of sim at 60 Hz" comments: the
   duration is now in the call, and the 60 Hz conversion lives in one place
   next to the `physics_app` that sets that timestep.

## Difficulties

- **`cargo fmt` defeated the end-of-line-comment escape hatch.** The
  convention exempts end-of-line comments from the tag rule, so
  `for _ in 0..600 { // 10 s of sim at 60 Hz` looked like a clean compaction.
  rustfmt relocated each comment into the loop body, turning five exempt
  labels into five untagged BLOCKS and re-failing `check-comment-tags.sh`.
  Lesson: an end-of-line comment is only safe where rustfmt will not reflow
  the line - after a simple statement, never after an opening brace. The real
  fix (extracting `simulate_seconds`) was better anyway.
- **A tatr HUID from another repo.** `pd_controller`'s test banner cited
  "nova task 20260709-125640", which resolves to nothing in this tracker. The
  DoD's no-bare-HUID rule caught it as a formatting issue; it was actually a
  dead cross-repo pointer.
- **Out-of-scope clippy blocker.** `src/completion.rs` carried a pre-existing
  `manual_contains` warning that would fail the "lints clean" proof. Verified
  it pre-dates this branch, then fixed the one-liner
  (`iter().any(|p| *p == name)` -> `contains(&name)`) rather than leave a
  proof unmeetable for an unrelated reason.
- **Rustdoc: linking a private module from public docs warns.** The first
  draft of `plugin.rs`'s module doc linked `[`damage`](super::damage)`, which
  rustdoc rejects (public docs -> private item). Reworded to name the module
  in prose without a link.

## Alternatives considered

- **Keep `plugin.rs` whole and just compact comments.** Rejected: the file
  was already the largest plugin body in the crate, and the KISS brief asked
  for the split-or-keep call to be evidence-driven. The two-test-module
  structure was decisive evidence.
- **Split `damage.rs` further** (impact vs blast). Rejected as YAGNI: they
  share the three damage constants and both funnel into `HealthApplyDamage`.
  One concern, one file.
- **Make `damage` public.** Rejected: it would enlarge the public API, which
  the task's constraint forbids and nothing needs.

## Verification

Run in the worktree:

| Proof | Result |
| --- | --- |
| `./scripts/check-comment-tags.sh src/integrity src/physics` | exit 0 |
| bare-HUID grep | 0 matches |
| `./scripts/check-ascii.sh` | exit 0 |
| `cargo fmt --check` | exit 0 |
| `cargo check --all-targets` | exit 0, only the expected `proc-macro-error2` future-incompat note |
| `cargo doc --no-deps --features debug` | in-scope warnings 0 (base 4) |
| `cargo clippy` (both feature configs) | **passed earlier in the session, then left to the user** - see below |
| `cargo test`, `--features debug`, `--examples` | **UNRUN** - see below |

**Clippy and tests are the user's to run.** Mid-task the user set a standing
rule ("STOP RUNNING CLIPPY ... I WILL RUN IT MYSELF AT THE END"), now
recorded in `AGENTS.md` under "Build, Verify, Run". Both clippy
configurations did pass (exit 0, zero warnings) earlier in the session, after
the `completion.rs` fix and before the final `simulate_seconds` and rustdoc
edits. `cargo test` is separately blocked: the user reports `rust-lld`
exhausting system RAM when linking test binaries, which is under
investigation as its own concern. `cargo check --all-targets` covers
compilation of every target including the test modules; only actual test
EXECUTION is outstanding.
