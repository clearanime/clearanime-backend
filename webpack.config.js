const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

const dist = path.resolve(__dirname, "dist");

module.exports = {
    mode: "production",
    watch: true,
    entry: {
        index: "./src/index.ts",
    },
    output: {
        path: dist,
        filename: "bundle.js",
    },
    devServer: {
        port: 8000,
        static: {
            directory: "./dist",
        },
        headers: {
            "Access-Control-Allow-Origin": "*",
            "Access-Control-Allow-Methods": "*",
            "Access-Control-Allow-Headers": "*"
        }
    },
    experiments: {
        asyncWebAssembly: true,
    },
    performance: {
        hints: false
    },
    module: {
        rules: [
            {
                test: /\.([jt]s?)?$/,
                use: "swc-loader",
                exclude: /node_modules/,
            }
        ],
    },
    resolve: {
        extensions: [".js", ".json", ".ts", ".tsx"],
    },
    plugins: [
        new CopyPlugin({
            patterns: [
                { from: "static", to: "." },
            ],
        }),
        //new WasmPackPlugin({
        //    crateDirectory: "./third_party/ani-ss",
        //    outDir: path.resolve(__dirname, "./third_party/ani-ss/pkg")
        //}),
    ]
};