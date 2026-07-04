# tween: easing engine generalizing LerpSnap (Wave 2)

- STATUS: CLOSED
- PRIORITY: 30
- TAGS: spike,feature,meth

> Spike: docs/spikes/20260704-134035-game-juice-and-scaffolding-kit.md (read
> first). Wave 2 -- foundation that also backs popup and flash.

## Goal

Generalize `meth/lerp::LerpSnap` (exponential lerp-with-snap only) into a
narrow, duration-based tween: animate a `Transform` / material color / scale
from A to B over `t` seconds with a Bevy 0.19 `EaseFunction`, firing a
completion event or inserting a done-marker. Five games (06, 07, 08, 10, 11)
ease something by hand; the `ui/popup` rise and `feedback` flash decay should
both consume this, so it is a foundation, not a leaf.

Keep it deliberately narrow to avoid framework machinery: a `Tween<T>`
component plus a handful of built-in target adapters, NOT a keyframe timeline
DSL. Resolve the two spike open questions during planning: (a) component-tween
vs reflection-driven field-tween -- favor the former; (b) how much this
overlaps `bevy_tweening` and whether depending on it is preferable to a
hand-roll. Prove it by refactoring one example's ad-hoc easing onto it.
