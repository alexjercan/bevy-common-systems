const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyPlugin = require('copy-webpack-plugin');
const HtmlPartialsPlugin = require('./webpack-partials');
const { wikiDocPage } = require('./markdown');

// "/" for local dev; set to the GitHub Pages subpath in CI, e.g.
// "/bevy-common-systems/". Must match the PUBLIC_PATH used by build-games.sh.
const publicPath = process.env.PUBLIC_PATH || '/';

// One HtmlWebpackPlugin per hand-authored page. `filename` with a trailing
// `index.html` gives clean directory URLs (/play/, /wiki/, ...). `basePath` is
// read by HtmlPartialsPlugin (for the shared header/footer links).
const page = (chunk, template, filename) =>
  new HtmlWebpackPlugin({
    template,
    filename,
    chunks: [chunk],
    basePath: publicPath,
  });

// Every wiki page is markdown under `src/wiki/`, rendered at build time (see
// markdown.js) and served at `/wiki/<slug>/`; all share the `wiki` chunk (the
// manifest-driven sidebar/search/see-also from wiki.ts + wiki-pages.ts). To add
// a page: drop the `.md` under `src/wiki/`, add an entry here, and add a manifest
// entry in src/wiki-pages.ts. Keep this list in sync with wiki-pages.ts.
const WIKI_DOC_PAGES = [
  // Get started
  { slug: 'introduction', title: 'Introduction' },
  { slug: 'quickstart', title: 'Quickstart' },
  { slug: 'conventions', title: 'Module conventions' },
  { slug: 'examples', title: 'Example games' },
  { slug: 'web-builds', title: 'Web builds' },
  // Modules
  { slug: 'mesh', title: 'mesh' },
  { slug: 'material', title: 'material' },
  { slug: 'camera', title: 'camera' },
  { slug: 'transform', title: 'transform' },
  { slug: 'physics', title: 'physics' },
  { slug: 'meth', title: 'meth' },
  { slug: 'tween', title: 'tween' },
  { slug: 'health', title: 'health' },
  { slug: 'integrity', title: 'integrity' },
  { slug: 'scoring', title: 'scoring' },
  { slug: 'time', title: 'time' },
  { slug: 'ui', title: 'ui' },
  { slug: 'feedback', title: 'feedback' },
  { slug: 'audio', title: 'audio' },
  { slug: 'input', title: 'input' },
  { slug: 'helpers', title: 'helpers' },
  { slug: 'modding', title: 'modding' },
  { slug: 'persist', title: 'persist' },
  { slug: 'debug', title: 'debug' },
];
const docPage = ({ slug, title }) =>
  wikiDocPage({ slug, mdPath: `src/wiki/${slug}.md`, title, publicPath });

module.exports = {
  entry: {
    index: './src/index.ts',
    games: './src/games-page.ts',
    wiki: './src/wiki.ts',
  },
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: '[name].js',
    clean: true,
    publicPath,
  },
  resolve: {
    extensions: ['.ts', '.js'],
  },
  module: {
    rules: [
      { test: /\.ts$/, use: 'ts-loader', exclude: /node_modules/ },
      { test: /\.css$/i, use: ['style-loader', 'css-loader'] },
    ],
  },
  plugins: [
    page('index', 'src/index.html', 'index.html'),
    page('games', 'src/games.html', 'play/index.html'),
    page('wiki', 'src/wiki.html', 'wiki/index.html'),
    ...WIKI_DOC_PAGES.map(docPage),
    new HtmlPartialsPlugin({ basePath: publicPath }),
    // Static assets: the favicon shared by every page.
    new CopyPlugin({
      patterns: [
        { from: 'src/favicon.svg', to: 'favicon.svg' },
        // The trunk-built games (from the staging dir) into dist/games. Run
        // `npm run build:games` first so build/games exists.
        {
          from: path.resolve(__dirname, 'build/games'),
          to: 'games',
          noErrorOnMissing: true,
        },
      ],
    }),
  ],
  mode: 'development',
  devServer: {
    static: path.join(__dirname, 'dist'),
    port: 8080,
    // Clean directory URLs during dev: rewrite /play and /wiki/<slug> to their
    // generated index.html. Doc pages are listed before the bare /wiki so the
    // more specific path matches first.
    historyApiFallback: {
      rewrites: [
        { from: /^\/play/, to: '/play/index.html' },
        ...WIKI_DOC_PAGES.map(({ slug }) => ({
          from: new RegExp('^/wiki/' + slug),
          to: '/wiki/' + slug + '/index.html',
        })),
        { from: /^\/wiki/, to: '/wiki/index.html' },
      ],
    },
  },
  experiments: {
    asyncWebAssembly: true,
  },
};
