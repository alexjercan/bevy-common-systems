# PROPOSAL (needs user): 07_orbit particle bursts for pickups/hits - requires a particle crate

- STATUS: OPEN
- PRIORITY: 30
- TAGS: suggestion,example


PROPOSAL for the user -- NOT to be implemented autonomously; it needs a
dependency decision.

Orbit Runner currently signals a pickup/hit only with sound, a text popup and
(for hits) a camera shake and flash. Real particle bursts -- a spray of sparks
when an orb is collected, a puff when a hazard clips the marker, maybe a faint
trail behind the marker -- would lift it a lot, and would also give the crate a
reusable particle helper that several examples could share.

Why this needs you: it means taking on a new dependency. Options:

- `bevy_hanabi` -- GPU particles, powerful, but a heavier dep and its own
  version-tracking burden against Bevy 0.19; wasm support needs checking.
- `bevy_firework` / `bevy_enoki` -- lighter CPU/2d-ish particle crates.
- Roll a tiny CPU particle system in-crate (spawn short-lived emissive quads/
  meshes with velocity + fade), no new dep -- fits the "copy-pastable" ethos
  but is more code to own.

Decision needed: add a crate (which one) or build a small in-crate particle
helper. Once you pick, this becomes a concrete implementation task (and, if a
crate, a wasm-build verification step).
