// The showcased games. To add one: build it with `web/scripts/build-games.sh`
// (add its example there) and append an entry here. `path` is the game's wasm
// page relative to the site root; the gallery resolves it against the base URL.
export interface Game {
  /** Example name / stable id (matches the trunk output dir under games/). */
  id: string;
  /** Display title. */
  title: string;
  /** One-line description shown on the card. */
  blurb: string;
  /** How you play, shown on the game overlay. */
  controls: string;
  /** Accent color for the card. */
  accent: string;
}

export const GAMES: Game[] = [
  {
    id: '06_fruitninja',
    title: 'Fruit Ninja',
    blurb:
      'Swipe to slice arcing fruit into exploding fragments, chain combos, and dodge the bombs.',
    controls: 'Hold the left mouse button and swipe across fruit to slice. Avoid the dark bombs.',
    accent: '#f2d94e',
  },
  {
    id: '07_orbit',
    title: 'Orbit Runner',
    blurb:
      'Ride a marker around a planet, sweep up the glowing orbs, and dodge the red hazards as it gets crowded.',
    controls:
      'Steer with A/D or the arrow keys, or hold the mouse / a finger to one side. Collect orbs, avoid the red hazards.',
    accent: '#5cc8ff',
  },
];

/** Path (relative to the site base) to a game's wasm page. */
export function gamePath(game: Game): string {
  return `games/${game.id}/`;
}
