const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");
const webpack = require("webpack");

const PRODUCTION = "production";

const distPath = path.resolve(__dirname, "dist");
module.exports = (env, argv) => {
  return {
    devServer: {
      contentBase: distPath,
      compress: argv.mode === PRODUCTION,
      port: 8000,
    },
    entry: "./bootstrap.js",
    output: {
      path: distPath,
      publicPath: "/game-of-life-yew/",
      filename: "cellulelife.js",
      webassemblyModuleFilename: "cellulelife.wasm",
    },
    module: {
      rules: [
        {
          test: /\.s[ac]ss$/i,
          use: ["style-loader", "css-loader", "sass-loader"],
        },
      ],
    },
    plugins: [
      new CopyWebpackPlugin([{ from: "./static", to: distPath }]),
      new WasmPackPlugin({
        crateDirectory: ".",
        extraArgs: "--no-typescript",
      }),
      new webpack.EnvironmentPlugin([
        "API_URL_SUBMIT_RESULT",
        "API_URL_GET_HIGH_SCORES",
      ]),
    ],
    watch: argv.mode !== PRODUCTION,
  };
};
