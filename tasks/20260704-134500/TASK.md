# camera/shake: CameraShake trauma module (Wave 1)

- STATUS: OPEN
- PRIORITY: 40
- TAGS: spike,feature,camera

> Spike: docs/spikes/20260704-134035-game-juice-and-scaffolding-kit.md (read
> first). Wave 1 -- promote recurring example juice into the library.

## Goal

Add a `camera/shake` module beside `camera/chase` and `camera/post` that owns
the camera-shake code four example games (06, 07, 08, 10) currently hand-roll,
including the accumulate-vs-absolute drift bug the asteroids retro recorded
(`docs/retros/20260703-170744-asteroids-example.md`).

Follow the crate convention: a `CameraShake` config component (trauma decay
rate, max offset/kick), a way to add trauma each frame (an `*Input` component
or a small `Commands` extension / event), and an `*Output` that writes the
offset as `BASE + trauma^2 * random` -- never an accumulating `+=`. Must
compose with `camera/chase`: shake is applied *after* the chase camera writes
the transform, so give it an ordering `*Systems` set. Prove it by refactoring
at least one example (06_fruitninja) onto the module and deleting its local
copy. Decide during planning whether v1 shakes rotation (kick) or translation
only (spike open question).
