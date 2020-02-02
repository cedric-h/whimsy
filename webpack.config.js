const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    entry: './index.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'index.js',
    },
	module: {
		rules: [
			{
				test: /\.css$/,
				use: ["style-loader", "css-loader"]
			},
			{
				test: /\.elm$/,
				exclude: [/elm-stuff/, /node_modules/],
				use: {
					loader: 'elm-webpack-loader',
					options: {
						optimize: true,
						runtimeOptions: ['-A128M', '-H128M', '-n8m'],
						files: [
							path.resolve(__dirname, "src/Main.elm"),
						]
					}
				}
			}
		]
	},
	plugins: [
		new HtmlWebpackPlugin(),
		new WasmPackPlugin({
			extraArgs: "--no-typescript",
			forceMode: "release",
			crateDirectory: path.resolve(__dirname, ".")
		}),
		// Have this example work in Edge which doesn't ship `TextEncoder` or
		// `TextDecoder` at this time.
		new webpack.ProvidePlugin({
			TextDecoder: ['text-encoding', 'TextDecoder'],
			TextEncoder: ['text-encoding', 'TextEncoder']
		})
	],
	mode: 'development'
};
