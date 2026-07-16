// The wiki manifest: the single source of truth for the whole wiki. Every piece
// of chrome (the sidebar, search, tag chips, "see also", and the wiki index) is
// a view of this array, so adding or renaming a page is a one-line edit here
// (plus authoring its markdown under src/wiki/ and the one-line wikiDocPage()
// registration in webpack.config.js). Keep this list in sync with the
// WIKI_DOC_PAGES list in webpack.config.js.

export interface WikiPage {
  // URL segment under /wiki/, e.g. "mesh" -> /wiki/mesh/.
  slug: string;
  title: string;
  // Sidebar group; must be one of the categories listed in WIKI_SECTIONS.
  category: string;
  // Small controlled taxonomy - drives tag chips and search, and the auto
  // "shares a tag" half of See also.
  tags: string[];
  // One line, shown on the index cards and in search results.
  summary: string;
  // Explicit cross-links (slugs), shown first under See also.
  related: string[];
  // Section headings, so search matches on in-page topics too.
  headings: string[];
  // Not yet written - rendered as a muted, non-navigable "coming soon" entry.
  comingSoon?: boolean;
  // Slug of the parent page, for two-level pages. Children nest under their
  // parent in the sidebar and appear as a grid on the parent's overview page.
  parent?: string;
}

// The wiki nav is segmented into audience BANDS, each holding an ordered list of
// category groups. Every page's `category` must be one of the categories listed
// here. WIKI_CATEGORIES is the flattened order, derived from the bands.
export interface WikiSection {
  name: string;
  categories: string[];
}

export const WIKI_SECTIONS: WikiSection[] = [
  {
    name: 'Get started',
    categories: ['Introduction', 'Project'],
  },
  {
    name: 'Modules',
    categories: [
      'Rendering & meshes',
      'Motion & physics',
      'Gameplay',
      'Interface & feedback',
      'Systems & data',
    ],
  },
];

export const WIKI_CATEGORIES: string[] = WIKI_SECTIONS.flatMap((s) => s.categories);

export const WIKI_PAGES: WikiPage[] = [
  // === Get started ========================================================
  {
    slug: 'introduction',
    title: 'Introduction',
    category: 'Introduction',
    tags: ['start'],
    summary:
      'What bevy_common_systems is, why it exists, and how to add it to a project - the copy-pastable crate of Bevy gameplay systems that lets you build games faster.',
    related: ['quickstart', 'conventions', 'examples'],
    headings: [
      'Why it exists',
      'Add it to your project',
      'Features',
      'The prelude',
      'How to read these docs',
    ],
  },
  {
    slug: 'quickstart',
    title: 'Quickstart',
    category: 'Introduction',
    tags: ['start'],
    summary:
      'From an empty App to a working feature in a few lines: add a plugin, spawn its config component, drive it with input, and read its output.',
    related: ['introduction', 'conventions', 'health'],
    headings: ['A minimal app', 'Add a system', 'Trigger an event', 'Next steps'],
  },
  {
    slug: 'conventions',
    title: 'Module conventions',
    category: 'Introduction',
    tags: ['start', 'pattern'],
    summary:
      'The shape every module shares - a *Plugin you add, a config component, an *Input you write each frame, and an *Output (or direct Transform) you read - so once you learn one, you know them all.',
    related: ['introduction', 'quickstart', 'transform'],
    headings: ['The plugin', 'Config, Input, Output', 'The prelude', 'Feature flags'],
  },
  {
    slug: 'examples',
    title: 'Example games',
    category: 'Project',
    tags: ['start', 'examples'],
    summary:
      'Fourteen small, complete games that double as integration tests and quickstart docs. Each headlines one or more modules; every one is playable in the browser.',
    related: ['introduction', 'web-builds', 'mesh'],
    headings: ['Foundations', 'The games', 'Which modules each one demos', 'Running them'],
  },
  {
    slug: 'web-builds',
    title: 'Web builds',
    category: 'Project',
    tags: ['start', 'examples', 'web'],
    summary:
      'How the example games are compiled to WebAssembly with Trunk and served by the showcase site, and how to build and run the whole thing locally.',
    related: ['examples', 'introduction'],
    headings: ['The showcase site', 'Building the games', 'Serving the site', 'Deploying'],
  },

  // === Modules: Rendering & meshes ========================================
  {
    slug: 'mesh',
    title: 'mesh',
    category: 'Rendering & meshes',
    tags: ['rendering', 'mesh', 'plugin'],
    summary:
      'TriangleMeshBuilder (procedural triangle meshes: octahedron spheres, subdivision, noise displacement, plane slicing) and ExplodeMeshPlugin, which slices a mesh into flying fragments.',
    related: ['material', 'physics', 'meth'],
    headings: [
      'TriangleMeshBuilder',
      'Spheres and planets',
      'Slicing a plane',
      'ExplodeMeshPlugin',
    ],
  },
  {
    slug: 'material',
    title: 'material',
    category: 'Rendering & meshes',
    tags: ['rendering', 'plugin'],
    summary:
      'glowing_material: the emissive StandardMaterial preset that actually blooms, tuned to work with the camera post pipeline.',
    related: ['mesh', 'camera', 'feedback'],
    headings: ['glowing_material', 'Making it bloom'],
  },
  {
    slug: 'camera',
    title: 'camera',
    category: 'Rendering & meshes',
    tags: ['rendering', 'camera', 'plugin'],
    summary:
      'A family of camera helpers: chase (third-person follow), wasd (first-person free-fly), post (tonemapping + bloom), skybox (stacked images to a cubemap), shake (trauma screen shake), and project (screen <-> world helpers).',
    related: ['material', 'physics', 'transform'],
    headings: ['chase', 'wasd', 'post', 'skybox', 'shake', 'project'],
  },

  // === Modules: Motion & physics ==========================================
  {
    slug: 'transform',
    title: 'transform',
    category: 'Motion & physics',
    tags: ['motion', 'transform', 'plugin'],
    summary:
      'Motion-driver components, each computing an Output you apply: sphere orbits (explicit, directional, random), point rotation, and smooth look rotation.',
    related: ['meth', 'physics', 'camera'],
    headings: [
      'Sphere orbit',
      'Directional and random orbit',
      'Point rotation',
      'Smooth look rotation',
    ],
  },
  {
    slug: 'physics',
    title: 'physics',
    category: 'Motion & physics',
    tags: ['physics', 'motion', 'plugin'],
    summary:
      'avian3d helpers: pd_controller (PD attitude torque toward a target rotation), doom_controller (a Doom-style first-person character controller), and rigid-body spawn helpers.',
    related: ['transform', 'meth', 'camera'],
    headings: ['PD attitude controller', 'Doom character controller', 'Rigid bodies'],
  },
  {
    slug: 'meth',
    title: 'meth',
    category: 'Motion & physics',
    tags: ['math', 'motion'],
    summary:
      'Vector math (the name is an intentional pun): the LerpSnap smoothing trait plus spherical coordinates and slerp.',
    related: ['transform', 'tween', 'physics'],
    headings: ['LerpSnap', 'Spherical coordinates', 'slerp'],
  },
  {
    slug: 'tween',
    title: 'tween',
    category: 'Motion & physics',
    tags: ['motion', 'math'],
    summary:
      'A narrow, duration-based value tween over a Bevy EaseFunction - the smallest possible interpolation you can drive a value or a UI field with.',
    related: ['meth', 'ui', 'transform'],
    headings: ['Tween', 'Ease functions', 'Driving a value'],
  },

  // === Modules: Gameplay ==================================================
  {
    slug: 'health',
    title: 'health',
    category: 'Gameplay',
    tags: ['gameplay', 'health', 'plugin'],
    summary:
      'HealthPlugin: a Health component, the HealthApplyDamage entity event (which propagates up the hierarchy), and a HealthZeroMarker inserted at zero.',
    related: ['integrity', 'feedback', 'scoring'],
    headings: ['Health', 'HealthApplyDamage', 'HealthZeroMarker', 'Reacting to death'],
  },
  {
    slug: 'integrity',
    title: 'integrity',
    category: 'Gameplay',
    tags: ['gameplay', 'health', 'plugin'],
    summary:
      'The structural-integrity pipeline: impact and blast (area-of-effect) damage, per-node health, and the components that turn a hit into a disabled or destroyed part.',
    related: ['health', 'mesh', 'feedback'],
    headings: ['Components', 'Applying damage', 'Blast damage', 'Destruction'],
  },
  {
    slug: 'scoring',
    title: 'scoring',
    category: 'Gameplay',
    tags: ['gameplay', 'score', 'plugin'],
    summary:
      'A Streak counter that grows on each hit and decays when the player goes quiet, and a generic HighScore<T> best-score resource with a "new best" edge.',
    related: ['health', 'persist', 'ui'],
    headings: ['Streak', 'HighScore', 'The new-best edge'],
  },
  {
    slug: 'time',
    title: 'time',
    category: 'Gameplay',
    tags: ['gameplay', 'time'],
    summary:
      'cooldown: a small countdown for fire gates and i-frames - tick it, ask if it is ready, and reset it when it fires.',
    related: ['scoring', 'health', 'physics'],
    headings: ['Cooldown', 'Fire gates', 'Invulnerability frames'],
  },

  // === Modules: Interface & feedback ======================================
  {
    slug: 'ui',
    title: 'ui',
    category: 'Interface & feedback',
    tags: ['ui', 'plugin'],
    summary:
      'status (a screen-corner metrics HUD), animate (copy a Tween into a UI Node field), menu (screen/button builders), popup (floating "+N" text), and touchpad (reveal-on-first-touch controls).',
    related: ['tween', 'feedback', 'input'],
    headings: ['status', 'animate', 'menu', 'popup', 'touchpad'],
  },
  {
    slug: 'feedback',
    title: 'feedback',
    category: 'Interface & feedback',
    tags: ['feedback', 'juice', 'plugin'],
    summary:
      'Short-lived "juice": flash (a material hit-flash) and screen_flash (a full-screen damage vignette) - the cheap effects that make a hit feel like a hit.',
    related: ['health', 'material', 'audio'],
    headings: ['flash', 'screen_flash', 'Wiring it to damage'],
  },
  {
    slug: 'audio',
    title: 'audio',
    category: 'Interface & feedback',
    tags: ['audio', 'plugin'],
    summary:
      'SfxPlugin: fire-and-forget one-shot sound effects. Trigger PlaySfx (or commands.play_sfx(handle)); a global SfxMasterVolume scales everything.',
    related: ['feedback', 'ui', 'input'],
    headings: ['SfxPlugin', 'Playing a sound', 'Master volume'],
  },
  {
    slug: 'input',
    title: 'input',
    category: 'Interface & feedback',
    tags: ['input', 'plugin'],
    summary:
      'A unified mouse + touch + cursor pointer resource, cursor-grab helpers, and small input-state utilities that read the same across desktop and mobile.',
    related: ['helpers', 'ui', 'camera'],
    headings: ['The pointer resource', 'Cursor grab', 'Input state helpers'],
  },
  {
    slug: 'helpers',
    title: 'helpers',
    category: 'Interface & feedback',
    tags: ['helpers', 'plugin'],
    summary:
      'The small stuff every game needs: DespawnEntity (despawn now), TempEntity (auto-despawn after N seconds), a pointer helper, and the WASD input controller binding.',
    related: ['input', 'time', 'physics'],
    headings: ['DespawnEntity', 'TempEntity', 'The WASD controller'],
  },

  // === Modules: Systems & data ============================================
  {
    slug: 'modding',
    title: 'modding',
    category: 'Systems & data',
    tags: ['modding', 'data', 'plugin'],
    summary:
      'A generic, serde-friendly event bus for modding and scripting: EventWorld, EventKind, EventHandler entities, and a JSON-authored EventHandlerRegistry. Event payloads travel as serde_json::Value.',
    related: ['persist', 'conventions', 'examples'],
    headings: ['The event bus', 'EventKind and the derive', 'Handlers', 'The JSON registry'],
  },
  {
    slug: 'persist',
    title: 'persist',
    category: 'Systems & data',
    tags: ['data', 'persist', 'plugin'],
    summary:
      'Cross-platform save/load of a Bevy Resource - a native data-dir file on desktop, localStorage on wasm - behind one small API.',
    related: ['scoring', 'modding', 'web-builds'],
    headings: ['Persisting a resource', 'Native vs wasm', 'Save and load'],
  },
  {
    slug: 'debug',
    title: 'debug',
    category: 'Systems & data',
    tags: ['debug', 'plugin'],
    summary:
      'Behind the debug feature: wireframe and inspector toggles plus harness - env-gated headless verification (AutopilotPlugin drives a game state machine, ScreenshotPlugin captures a frame).',
    related: ['conventions', 'introduction', 'physics'],
    headings: ['The debug feature', 'wireframe', 'inspector', 'The test harness'],
  },
];
