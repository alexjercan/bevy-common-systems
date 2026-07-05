# Expand README.md from a three-line stub into a proper crate README

- STATUS: CLOSED
- PRIORITY: 60
- TAGS: docs

## Goal

README.md is currently a three-line stub ("A collection of bevy examples that
can be used for creating a game." + an empty Quickstart). It is the first
thing a visitor sees on GitHub / crates.io, and it is both thin and slightly
stale (the crate is a library of copy-pastable Bevy systems, not "a collection
of examples"). Expand it into a real crate README so a newcomer can understand
what `bevy_common_systems` is, what it offers, how to add it, and where to look
next -- without reading AGENTS.md (the agent-facing orientation doc).

"Done" means README.md covers, accurately and concisely:

- What the crate is and its philosophy (copy-pastable, game-agnostic,
  plugin-per-concern building blocks to build games faster).
- How to add it as a dependency + the feature flags (`debug`/`dev`).
- A module map (what each module gives you), condensed from AGENTS.md.
- A quickstart showing the prelude import + a minimal usage snippet.
- The examples gallery (the numbered NN_name examples double as docs) with a
  one-line hook each and how to run one.
- Pointers onward: AGENTS.md for contributors, the web showcase, docs/.

Every claim must be cross-checked against the actual code (module names,
feature names, dependency versions, example list, run commands) -- the README
is public-facing, so nothing invented. Keep it plain ASCII per the repo style.

## Steps

- [x] Read the current README.md, the crate root doc (src/lib.rs), Cargo.toml
      (name, version, description, features, edition, license) and AGENTS.md
      so the README is sourced from ground truth, not memory.
- [x] Confirm the actual example list from `examples/` (ls the dir) so the
      gallery matches what ships, not what a doc claims.
- [x] Verify the prelude import path and a minimal usage snippet actually
      reflect the API (grep src/lib.rs / src/prelude for the public prelude).
- [x] Write the expanded README.md: title + one-paragraph pitch, "Add it"
      (Cargo dependency + features), "What's inside" (module map), Quickstart
      (prelude + snippet), Examples gallery (one line each + run command),
      and a short "For contributors / more" pointer section (AGENTS.md, web/,
      docs/). Plain ASCII, no em dashes / smart quotes.
- [x] Sanity-check: any command shown in the README (e.g. `cargo run --example
      01_sphere`) is one AGENTS.md already lists as verified; the crate
      name/version/features match Cargo.toml; the module list matches src/.
- [x] Per global guidelines, add a short docs/ decision note only if a
      non-obvious call was made (e.g. how much to duplicate vs. link
      AGENTS.md); otherwise skip -- a README expansion is self-explanatory.
      Skipped: the README-vs-AGENTS.md split (README is the friendly front
      door, links to AGENTS.md for depth) is self-explanatory and stated in
      the README's "More" section; no separate decision note warranted.

## Close-out

What changed and why:
- Replaced the three-line README.md stub with a full crate README: pitch,
  "Add it" (git dependency + feature flags), Quickstart (prelude import + a
  real HealthPlugin snippet adapted from src/health/mod.rs's tested doc),
  "What's inside" module map (one line per top-level module), an Examples
  gallery (all 14 with a one-line hook + run command), a "More" pointer
  section (AGENTS.md, web/, docs/) and the MIT license line.
- Sourced every claim from ground truth, not memory or the (partly stale)
  AGENTS.md: src/lib.rs for the real module list, Cargo.toml for versions
  (Bevy 0.19 / avian3d 0.7 -- AGENTS.md's dependency section still says
  0.18/0.6 and is incomplete on the harvested modules; left as-is, out of
  scope), examples/ listing + each example's own //! / clap `about` for the
  gallery hooks, and `git remote` for the dependency URL.

Verification:
- README.md confirmed pure ASCII (`grep -nP '[^\x00-\x7F]'` clean;
  scripts/check-ascii.sh also green, though it does not scan README).
- No source, examples, or Cargo.toml changed, so the compile/test suite has
  nothing new to exercise -- deliberately not run for a docs-only diff.
- Prelude path, HealthPlugin / Health::new / HealthApplyDamage, feature
  names, and the 14-example list all cross-checked against the code.

Self-reflection:
- AGENTS.md was NOT a safe single source: its module map omits the harvested
  modules (feedback, input, material, persist, scoring, time, tween, and
  several camera/ui submodules) and its dependency versions lag Cargo.toml.
  Reading src/lib.rs + the module //! docs directly was necessary to avoid
  propagating stale facts into the public README.

## Notes

- Source of truth: src/lib.rs crate doc, Cargo.toml, AGENTS.md "Module Map"
  and "Examples" sections, examples/ directory listing.
- README is user-facing (GitHub/crates.io); AGENTS.md is agent/contributor-
  facing. Avoid duplicating AGENTS.md wholesale -- README should be the
  friendly front door and link to AGENTS.md for the deep orientation.
- Style: plain ASCII only (repo rule + scripts/check-ascii.sh enforces it, so
  run it on the result). No em dashes, smart quotes, ellipsis chars, arrows.
- Do NOT change AGENTS.md or any code; docs-only task.
