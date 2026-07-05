# Refresh AGENTS.md: stale dependency versions and incomplete Module Map

- STATUS: CLOSED
- PRIORITY: 40
- TAGS: docs

## Goal

AGENTS.md has drifted from the code and gave a false lead while writing the
public README (task 20260705-134942):

- "Features and Dependencies" lists bevy 0.18, avian3d 0.6 and
  bevy-inspector-egui 0.36, but Cargo.toml is now bevy 0.19.0, avian3d 0.7 and
  bevy-inspector-egui 0.37.
- The "Module Map" omits top-level modules that now exist in src/lib.rs:
  feedback, input, material, persist, scoring, time, tween -- plus several
  harvested submodules (camera/shake, camera/project, ui/menu, ui/popup,
  ui/touchpad, physics/doom_controller is present but the map should be
  cross-checked end to end).

Bring AGENTS.md's dependency versions and Module Map back in sync with the
actual code so it is a trustworthy orientation doc again.

## Steps

- [ ] Diff the "Module Map" section against `find src -name '*.rs'` and add
      every missing module with a one-line description sourced from its `//!`
      doc.
- [ ] Update the "Features and Dependencies" versions to match Cargo.toml
      (bevy 0.19, avian3d 0.7, bevy_enhanced_input 0.26, inspector 0.37);
      double-check each against the manifest, do not assume.
- [ ] Re-read the Examples section for any other drift while in there.
- [ ] Run scripts/check-ascii.sh; keep plain ASCII.

## Notes

- Surfaced by the README expansion retro
  (docs/retros/20260705-134942-readme-expansion.md). The README already routes
  around the stale facts; this task fixes the source.
- Docs-only; do not touch code.
