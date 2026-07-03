# Enlarge game embed + center game page title

- STATUS: CLOSED
- PRIORITY: 100
- TAGS: web,polish

## Goal

On the game page overlay: make the game embed a bit larger and slightly wider
(~560x700px, aspect 4:5) and horizontally center the "Fruit Ninja" title (it
currently sits left, next to the back button).

## Steps

- [x] In `web/src/style.css`, change `.game-embed` `max-width` from 430px to
      560px and `aspect-ratio` from `7 / 10` to `4 / 5` (= 560:700). 4:5 (0.8)
      still fits the fruit spawn range horizontally.
- [x] Center the title: make `.game-page__topbar` a centered row with the back
      button positioned on the left (absolute) so `.game-page__title` centers
      in the page. Bump the title size a touch.
- [x] Verify: `npm run ci` (prettier + eslint) and `npm run build:web` clean; a
      static-serve smoke test still 200s. No wasm rebuild needed (the game fits
      its canvas to the parent, so a larger frame just renders larger).

## Notes

- Pure gallery CSS in `web/src/style.css`; no game or wasm changes.
- FOV check: at aspect 0.8 the horizontal half-view is ~7.3 world units, so
  fruit spawning at x up to +/-6 stay on screen.
- On the web-showcase worktree with the rest of the site work.

## Close-out

.game-embed enlarged to max-width 560px and aspect 4/5 (560:700), a bit wider
and bigger; .game-info widened to 560 to match. Title centered by making the
topbar a centered row with the back button absolutely positioned left, and
bumped to 1.5rem. Pure CSS; no wasm rebuild. Verified ci + build:web + serve.
