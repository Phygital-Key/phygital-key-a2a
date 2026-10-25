#!/usr/bin/env node
// `npx @phygital-key/a2a ...` starts the released program built for this computer.
// The package holds the program for every release target, in dist/<os>-<cpu>/;
// this file picks the one for this computer and runs it.
"use strict";

const { spawn } = require("node:child_process");
const { existsSync } = require("node:fs");
const { join } = require("node:path");

const BIN = "phygital-key-a2a";
const target = `${process.platform}-${process.arch}`;
const exe = process.platform === "win32" ? `${BIN}.exe` : BIN;
const path = join(__dirname, "..", "dist", target, exe);

if (!existsSync(path)) {
  console.error(
    `${BIN}: no build for ${target} in this package.\n` +
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
