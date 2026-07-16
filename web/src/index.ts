import './style.css';
import { initSite } from './site';
import { WIKI_PAGES, WIKI_SECTIONS } from './wiki-pages';

// Webpack injects the configured public path here, so links resolve correctly
// both locally ("/") and under a GitHub Pages subpath.
const BASE = __webpack_public_path__;

function baseUrl(path: string): string {
  return `${BASE}${path}`.replace(/([^:]\/)\/+/g, '$1');
}

initSite();

// Render the "What's inside" module grid straight from the wiki manifest so the
// landing page never drifts from the docs. Only the "Modules" band is shown; the
// first sentence of each page's summary becomes the card blurb.
function renderModuleGrid(): void {
  const host = document.getElementById('module-grid');
  if (!host) return;

  const band = WIKI_SECTIONS.find((s) => s.name === 'Modules');
  if (!band) return;
  const cats = new Set(band.categories);

  const modules = WIKI_PAGES.filter((p) => cats.has(p.category) && !p.parent);
  for (const p of modules) {
    const card = document.createElement('a');
    card.className = 'module-card';
    card.href = baseUrl(`wiki/${p.slug}/`);

    const name = document.createElement('span');
    name.className = 'module-card__name';
    name.textContent = p.title;
    card.appendChild(name);

    const blurb = document.createElement('p');
    blurb.className = 'module-card__blurb';
    blurb.textContent = p.summary.split(/(?<=[.:])\s/)[0];
    card.appendChild(blurb);

    host.appendChild(card);
  }
}

renderModuleGrid();
