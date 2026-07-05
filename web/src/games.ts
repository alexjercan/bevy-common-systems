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
  {
    id: '08_dropzone',
    title: 'Drop Zone',
    blurb:
      'Fly a lander down onto a noise planet: thrust against gravity while a PD controller holds your attitude, and touch down soft and upright to score.',
    controls:
      'Space/Up to thrust, W/S to pitch, A/D to roll. Land slow and level, or crash into fragments.',
    accent: '#5ad1ff',
  },
  {
    id: '09_reactor',
    title: 'Reactor',
    blurb:
      'A rules-as-machine incremental where the modding event bus is the game: tap for energy, sell it for credits, then buy handler "parts" that react to each reactor tick. Fuel rods make energy AND heat -- compose an escalating loop without melting down.',
    controls:
      'Click TAP (or Space) for energy, SELL (or Enter) for credits, and a shop card (or 1-6) to install a part. Keep HEAT under 100.',
    accent: '#5ae0ff',
  },
  {
    id: '10_asteroids',
    title: 'Asteroids',
    blurb:
      'Shoot drifting rocks into real physics-body shards that keep bouncing around as new hazards, and clear each wave without getting hit.',
    controls:
      'A/D rotate, W thrust, Space fires -- or hold the mouse / a finger to fly toward it and auto-fire. Clear every rock; avoid bumping them.',
    accent: '#7fd0ff',
  },
  {
    id: '11_overload',
    title: 'Overload',
    blurb:
      'Run a failing reactor whose gauges climb on their own. Vent them back to green before the console goes critical -- but every vent pushes another gauge up.',
    controls:
      'Press 1 / 2 / 3 / 4 to vent HEAT / PRES / FLUX / CHRG. Keep every gauge out of the red or the hull melts down.',
    accent: '#ff7a3c',
  },
  {
    id: '12_bastion',
    title: 'Bastion',
    blurb:
      'Defend the Core at the center of the arena: enemies close in from every side, so build towers around it and upgrade them to hold the line as the waves grow.',
    controls:
      'Drag (or A/D) to orbit the camera. Press 1 or 2 to pick a tower, then tap the ground (or Space) to build it. Tap a tower and press U to upgrade. Escape gives up.',
    accent: '#5fd0ff',
  },
  {
    id: '13_glide',
    title: 'Glide',
    blurb:
      'A slide-merge number puzzle rendered entirely in UI: swipe to slide every tile, collide equal ones to merge them into their sum, and chase a high score that saves across sessions.',
    controls:
      'Swipe (or use the arrow keys / WASD) to slide the board up, down, left or right. Equal tiles merge. Fill the board with no move left and the run ends.',
    accent: '#5fbfff',
  },
];

/** Path (relative to the site base) to a game's wasm page. */
export function gamePath(game: Game): string {
  return `games/${game.id}/`;
}
