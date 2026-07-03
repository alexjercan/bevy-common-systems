# Retro: Harden mesh slicer against crash-inducing edge cases

- TASK: 20260703-110915
- BRANCH: fix/slicer-hardening (merged to master, deleted)
- REVIEW ROUNDS: 1 (APPROVE, two NITs, no changes)

See TASK.md close-out for the technical detail; this is process only.

## What went well

- Reading the whole slicer before planning produced a plan that named all
  five crash sites with file:line, so the work was mechanical and the review
  had a concrete checklist to verify against. The "enumerate exit/degeneracy
  conditions up front" lesson from the CI retro paid off directly.
- The headless end-to-end finiteness test is the crux deliverable: the goal
  was "won't crash", and a test that runs the real explode pipeline and
  asserts no NaN proves it in CI, independent of the graphical example that
  cannot run in this session. Choosing the verifiable proof over the
  demo-that-cannot-be-run-here was the right instinct.
- As reviewer I stress-tested my own work harder than the committed test
  (cone with coplanar cap, off-origin centroid, high fragment count) before
  approving, rather than trusting the green suite. That is the adversarial
  reviewer lens working on my own code.

## What went wrong

- The first finiteness test asserted `fragments.len() >= 8` for
  fragment_count 8, which is simply wrong about the algorithm: re-slicing a
  hemisphere with a random plane through the origin often leaves one side
  empty, so growth is <= doubling and stochastic. Root cause: I wrote the
  assertion from the fragment_count parameter's intent, not from what the
  loop actually guarantees. I reasoned it out before it flaked, but it was a
  claim about behavior I had not traced.

## What to improve next time

- When asserting a quantity a loop produces, derive the bound from the
  loop's actual transition (here: "each round at most doubles, and a missed
  plane carries"), not from the caller's requested target. Assert the
  property you can prove (finiteness, non-empty), and only bound counts you
  have actually traced.

## Action items

- [x] Slicer is total: finite geometry or None, no panics; 6 new tests.
- [ ] Optional follow-up (noted in close-out, not filed): slice at the mesh
  centroid instead of the origin so off-origin meshes always split. This is
  a quality/robustness improvement, not a crash fix - file it only if the
  example (20260703-110851) shows off-origin meshes failing to explode in a
  way that matters.
