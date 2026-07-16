import './style.css';
import { initSite } from './site';
import { GAMES, gamePath, type Game } from './games';

initSite();

// Webpack injects the configured public path here, so links resolve correctly
// both locally ("/") and under a GitHub Pages subpath.
const BASE = __webpack_public_path__;

const SOURCE_ROOT = 'https://github.com/alexjercan/bevy-common-systems/blob/master/examples';

function baseUrl(path: string): string {
  return `${BASE}${path}`.replace(/([^:]\/)\/+/g, '$1');
}

function card(game: Game): HTMLLIElement {
  const li = document.createElement('li');
  li.className = 'game-card';
  li.style.setProperty('--accent-card', game.accent);

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

const overlay = (): HTMLDivElement => document.getElementById('game-overlay') as HTMLDivElement;
const frame = (): HTMLIFrameElement => document.getElementById('game-frame') as HTMLIFrameElement;

function openGame(game: Game): void {
  const title = document.getElementById('game-overlay-title') as HTMLSpanElement;
  const controls = document.getElementById('game-overlay-controls') as HTMLSpanElement;
  const links = document.getElementById('game-overlay-links') as HTMLParagraphElement;
  title.textContent = game.title;
  controls.textContent = game.controls;
  links.innerHTML = `<a href="${SOURCE_ROOT}/${game.id}.rs" target="_blank" rel="noopener">View source (examples/${game.id}.rs) &rarr;</a>`;
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
