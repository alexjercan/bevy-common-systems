# ui/popup + feedback: consume tween::Tween for fade/decay (realize foundation)

- STATUS: CLOSED
- PRIORITY: 20
- TAGS: feature,tween,ui,feedback

> Follow-up from tatr 20260704-134630 (tween), review MINOR R1.1.
> Spike: docs/spikes/20260704-134035-game-juice-and-scaffolding-kit.md.

## Goal

The `tween` module (tatr 20260704-134630) shipped `Tween<T>` and proved it on
06's slice pop, but the spike's justification for it being a *foundation* (not a
leaf) is that `ui/popup` and `feedback` should consume it too. Realize that:

- `ui/popup` (`src/ui/popup.rs`): route the fade -- the linear `base_alpha -> 0`
  over `lifetime` ramp (`popup_alpha`) -- onto a `Tween<f32>` (or `Tween<Vec4>`
  for the whole color). The rise is velocity-based (no fixed end), so it stays a
  plain `top -= rise_speed * dt`; only the fade is an A->B tween. Despawn on the
  lifetime (a `TweenOnComplete::Despawn` on a lifetime-length tween is a clean
  fit for the whole popup).
- `feedback` (`src/feedback/flash.rs`, `screen_flash.rs`): the flash/overlay
  decay (alpha or a clone easing back) is an A->B over a duration -- route it
  onto a `Tween`.

Keep behaviour identical (the current ramps are linear -> `EaseFunction::Linear`).
Prove by the refactor itself; the games that use popup/flash (06/07/08) are the
integration exercise. This closes the "foundation, not a leaf" claim.
