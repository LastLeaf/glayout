var path = require('path')
var UglifyJSPlugin = require('uglifyjs-webpack-plugin')

/* global __dirname: false */

module.exports = [{
  entry: path.resolve(__dirname, 'src/index.js'),
  output: {
    filename: 'glayout-lib.js',
    library: '__glayoutLib__',
    path: path.resolve(__dirname, 'bin')
  },
  devtool: 'source-map',
  target: 'web',
  module: {
    rules: [{
      test: /\.glsl$/,
      loader: 'raw-loader'
    }, {
      test: /\.js$/,
      exclude: /(node_modules|bower_components)/,
      use: {
        loader: 'babel-loader',
        options: {
          presets: ['env'],
          cacheDirectory: path.resolve(__dirname, 'bin/cache')
        }
      }
    }]
  },
  optimization:{
    minimize: false
  },
  plugins: [],
  cache: {}
}, {
  entry: path.resolve(__dirname, 'src/index.js'),
  output: {
    filename: 'glayout-lib.min.js',
    library: '__glayoutLib__',
    path: path.resolve(__dirname, 'bin')
  },
  devtool: 'none',
  target: 'web',
  module: {
    rules: [{
      test: /\.glsl$/,
      loader: 'raw-loader'
    }, {
      test: /\.js$/,
      exclude: /(node_modules|bower_components)/,
      use: {
        loader: 'babel-loader',
        options: {
          presets: ['env']
        }
      }
    }]
  },
  optimization:{
    minimize: true
  },
}]
