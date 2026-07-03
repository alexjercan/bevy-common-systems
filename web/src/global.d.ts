// Webpack injects this global with the configured `output.publicPath`.
declare const __webpack_public_path__: string;

// Allow importing CSS as a side-effect module (handled by style-loader).
declare module '*.css';
