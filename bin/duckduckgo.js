#!/usr/bin/env node
const childProcess = require("node:child_process");

try {
  const { binaryPath } = require("../index.js");
  const result = childProcess.spawnSync(binaryPath(), process.argv.slice(2), { stdio: "inherit" });
  if (result.error) throw result.error;
  process.exit(result.status === null ? 1 : result.status);
} catch (error) {
  if (error && error.code === "MODULE_NOT_FOUND") {
    const { platformName } = require("../index.js");
    console.error(`duckduckgo-cli: no prebuilt binary for ${platformName()}.`);
    console.error("Try: npm i duckduckgo-cli --include=optional");
    console.error("or:  cargo binstall duckduckgo-cli");
    process.exit(1);
  }
  console.error(`duckduckgo-cli: ${error.message}`);
  process.exit(1);
}
