#!/usr/bin/env node
// `npx @phygital-key/a2a ...` starts the released program built for this computer.
// npm installs only the matching one of the optional packages listed in package.json
// (@phygital-key/a2a-darwin-arm64 and the rest); this file finds it and runs it.
"use strict";

const { spawn } = require("node:child_process");

const BIN = "phygital-key-a2a";
const pkg = `@phygital-key/a2a-${process.platform}-${process.arch}`;
const exe = process.platform === "win32" ? `${BIN}.exe` : BIN;

let path;
try {
  path = require.resolve(`${pkg}/bin/${exe}`);
} catch {
  console.error(
    `${BIN}: no build for ${process.platform}-${process.arch} (looked for the npm package ${pkg}).\n` +
      `If you installed with --no-optional or --omit=optional, install again without it.\n` +
      `Other ways to install: https://github.com/Phygital-Key/${BIN}#install`
  );
  process.exit(1);
}

const child = spawn(path, process.argv.slice(2), { stdio: "inherit" });
for (const sig of ["SIGINT", "SIGTERM", "SIGHUP"]) {
  process.on(sig, () => child.kill(sig));
}
child.on("error", (err) => {
  console.error(`${BIN}: ${err.message}`);
  process.exit(1);
});
child.on("exit", (code, signal) => {
  if (signal) {
    // End the same way the program did, so the caller sees the same signal.
    process.removeAllListeners(signal);
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 1);
  }
});
