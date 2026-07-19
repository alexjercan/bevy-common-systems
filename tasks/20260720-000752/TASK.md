# Harness completion protocol: collectors register/done, deadline names laggards; autopilot/screenshot converted (no unilateral AppExit)

- STATUS: CLOSED
- PRIORITY: 80
- TAGS: harness,testing


## Close-out (2026-07-20)

Shipped as v0.19.3 (master 3f6f7c8). Protocol in src/completion.rs at the
crate root (ungated - external feature-less collectors), re-exported at
debug::harness::completion; autopilot + screenshot converted;
self_completing() for script-owned runs. 5 protocol + 8 harness lib tests
green in both feature configs. Driven by nova-protocol task
20260720-000609 (spike tasks/20260719-235305 there); nova's e2e proved
the headline case (a frame capture outliving the autopilot timeline now
completes instead of losing its window).
