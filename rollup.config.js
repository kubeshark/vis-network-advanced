import commonjs from "@rollup/plugin-commonjs";
import resolve from "@rollup/plugin-node-resolve";
import babel from "@rollup/plugin-babel";
import typescript from "@rollup/plugin-typescript";
import terser from "@rollup/plugin-terser";
import { generateHeader } from "vis-dev-utils";
import assets from "postcss-assets";
import postcss from "rollup-plugin-postcss";

// TypeScript because Babel transpiles modules in isolation, therefore no type reexports.
// CommonJS because Babel is not 100 % ESM.

const babelConfig = {
  extensions: [".ts", ".js"],
  babelHelpers: "runtime",
  exclude: "**/node_modules/**",
};
const resolveConfig = {
  browser: true,
  mainFields: ["module", "main"],
  extensions: [...babelConfig.extensions, ".json"],
};
const banner = generateHeader();
const typescriptConfig = {
  tsconfig: "tsconfig.code.json",
};
const terserConfig = {
  output: {
    comments: (_node, { value }) => /@license/.test(value),
  },
};
const postCSSRawConfig = {
  extract: "dist/vis-network-advanced.css",
  inject: false,
  minimize: false,
  sourceMap: false,
  plugins: [
    assets({
      loadPaths: ["lib/assets/"],
    }),
  ],
};
const postCSSMinConfig = {
  extract: "dist/vis-network-advanced.min.css",
  inject: false,
  minimize: true,
  sourceMap: false,
  plugins: [
    assets({
      loadPaths: ["lib/assets/"],
    }),
  ],
};

export default [
  {
    input: "lib/index-legacy-bundle.ts",
    output: [
      {
        file: "dist/vis-network-advanced.esm.js",
        format: "esm",
        banner,
        sourcemap: true,
      },
      {
        file: "dist/vis-network-advanced.mjs",
        format: "esm",
        banner,
        sourcemap: true,
      },
      {
        file: "dist/vis-network-advanced.js",
        format: "umd",
        exports: "named",
        name: "vis",
        extend: true,
        banner,
        sourcemap: true,
      },
      {
        file: "dist/vis-network-advanced.cjs",
        format: "umd",
        exports: "named",
        name: "vis",
        extend: true,
        banner,
        sourcemap: true,
      },
    ],
    plugins: [
      resolve(resolveConfig),
      postcss(postCSSRawConfig),
      typescript(typescriptConfig),
      commonjs(),
      babel(babelConfig),
    ],
  },
  {
    input: "lib/index-legacy-bundle.ts",
    output: [
      {
        file: "dist/vis-network-advanced.esm.min.js",
        format: "esm",
        banner,
        sourcemap: true,
      },
      {
        file: "dist/vis-network-advanced.min.mjs",
        format: "esm",
        banner,
        sourcemap: true,
      },
      {
        file: "dist/vis-network-advanced.min.js",
        format: "umd",
        exports: "named",
        name: "vis",
        extend: true,
        banner,
        sourcemap: true,
      },
      {
        file: "dist/vis-network-advanced.min.cjs",
        format: "umd",
        exports: "named",
        name: "vis",
        extend: true,
        banner,
        sourcemap: true,
      },
    ],
    plugins: [
      resolve(resolveConfig),
      postcss(postCSSMinConfig),
      typescript(typescriptConfig),
      commonjs(),
      babel(babelConfig),
      terser(terserConfig),
    ],
  },
];
