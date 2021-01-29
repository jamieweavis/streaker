'use strict';

const path = require('path');
const TsconfigPathsPlugin = require('tsconfig-paths-webpack-plugin');

module.exports = {
  mode: 'development',
  output: {
    path: path.resolve(__dirname, '../', '../', 'build'),
    filename: '[name].js',
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js', '.json'],
    plugins: [new TsconfigPathsPlugin()],
  },
  devtool: 'source-map',
  plugins: [],
};
