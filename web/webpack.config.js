const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyPlugin = require('copy-webpack-plugin');

// "/" for local dev; set to the GitHub Pages subpath in CI, e.g.
// "/bevy-common-systems/". Must match the PUBLIC_PATH used by build-games.sh.
const publicPath = process.env.PUBLIC_PATH || '/';

module.exports = {
  entry: {
    index: './src/index.ts',
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
    new HtmlWebpackPlugin({
      template: 'src/index.html',
      chunks: ['index'],
    }),
    // Copy the trunk-built games (from the staging dir) into dist/games. Run
    // `npm run build:games` first so build/games exists.
    new CopyPlugin({
      patterns: [
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
  },
  experiments: {
    asyncWebAssembly: true,
  },
};
