import './style.css';
import { GAMES, gamePath, type Game } from './games';

// Webpack injects the configured public path here, so links resolve correctly
// both locally ("/") and under a GitHub Pages subpath.
const BASE = __webpack_public_path__;

function baseUrl(path: string): string {
  return `${BASE}${path}`.replace(/\/{2,}/g, '/');
}

function card(game: Game): HTMLLIElement {
  const li = document.createElement('li');
  li.className = 'game-card';
  li.style.setProperty('--accent', game.accent);

  const button = document.createElement('button');
  button.className = 'game-card__button';
  button.type = 'button';
  button.innerHTML = `
    <span class="game-card__thumb" aria-hidden="true"></span>
    <span class="game-card__title">${game.title}</span>
    <span class="game-card__blurb">${game.blurb}</span>
    <span class="game-card__play">Play &rarr;</span>
  `;
  button.addEventListener('click', () => openGame(game));

  li.appendChild(button);
  return li;
}

const overlay = () => document.getElementById('game-overlay') as HTMLDivElement;
const frame = () => document.getElementById('game-frame') as HTMLIFrameElement;

function openGame(game: Game): void {
  const title = document.getElementById('game-overlay-title') as HTMLSpanElement;
  const controls = document.getElementById('game-overlay-controls') as HTMLSpanElement;
  title.textContent = game.title;
  controls.textContent = game.controls;
  frame().src = baseUrl(gamePath(game));
  overlay().hidden = false;
  document.body.classList.add('is-playing');
}

function closeGame(): void {
  overlay().hidden = true;
  overlay().scrollTop = 0;
  // Drop the iframe src so the wasm app stops and audio/input release.
  frame().src = 'about:blank';
  document.body.classList.remove('is-playing');
}

function main(): void {
  const grid = document.getElementById('game-grid') as HTMLUListElement;
  for (const game of GAMES) {
    grid.appendChild(card(game));
  }

  (document.getElementById('game-overlay-back') as HTMLButtonElement).addEventListener(
    'click',
    closeGame,
  );
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && !overlay().hidden) {
      closeGame();
    }
  });
}

main();
