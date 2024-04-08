import { rollupBundlePlugin } from "@web/dev-server-rollup";

import resolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import json from "@rollup/plugin-json";
import alias from "@rollup/plugin-alias";
import inject from "@rollup/plugin-inject";
import replace from "@rollup/plugin-replace";
import wasm from "@rollup/plugin-wasm";
import { playwrightLauncher } from "@web/test-runner-playwright";

import stdLibBrowser from "node-stdlib-browser";

export default {
  rootDir: "./",
  files: "./tests/test.browser.js",
  coverage: true,
  browsers: [
    playwrightLauncher({ product: "chromium" }),
    playwrightLauncher({ product: "firefox" }),
  ],
  nodeResolve: true,
  testFramework: "mocha",
  plugins: [
    rollupBundlePlugin({
      rollupConfig: {
        input: ["tests/test.browser.js"],
        plugins: [
          replace({
            include: ["tests/*.js"],
            __environment__: '"development"',
          }),
          alias({
            entries: stdLibBrowser,
          }),
          resolve({
            browser: true,
          }),
          commonjs(),
          wasm({
            include: ["browsers/*.wasm"],
            maxFileSize: 25600000,
          }),
          json(),
          inject({
            process: stdLibBrowser.process,
            Buffer: [stdLibBrowser.buffer, "Buffer"],
          }),
        ],
      },
    }),
  ],
};
