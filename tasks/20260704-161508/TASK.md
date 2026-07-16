# input/pointer: unified mouse+touch+cursor pointer resource (Wave A)

- STATUS: CLOSED
- PRIORITY: 30
- TAGS: spike,feature,input

> Spike: tasks/20260704-161210/SPIKE.md (read first). Wave A -- three independent copies exist.

## Goal

Add an `input/pointer` module owning a unified `Pointer` resource that collapses
mouse + touch + cursor into one per-frame abstraction
(`{ screen_pos, pressed, just_pressed }`), an active touch winning over the
cursor, resolved in `PreUpdate`. Three games independently re-implement this:
`examples/10_asteroids.rs:293` (`struct Pointer` + `update_pointer`),
`examples/06_fruitninja.rs:283`, and `examples/08_dropzone.rs:307`
(`TouchControl`), plus the shared `active_pointer_pos(touch, cursor)` helper
(`06:1504`, unit-tested; inlined in `10:325`).

RESOLVE FIRST (spike open question): fruitninja routes touch through
`bevy_enhanced_input` (`TouchInputId(CustomInput)`); asteroids/dropzone read raw
`Touches`. The core module should offer the plain `Touches` path and NOT force
the enhanced-input dependency -- leave any enhanced-input bridge to `helpers/`.
Unit-test the pure resolve (touch-wins-over-cursor) and prove it by refactoring
asteroids (and ideally one more) onto the resource.
