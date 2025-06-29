#!/usr/bin/env node

import { ChildProcess, spawn } from "child_process";
import path from "path";
import os from "os";

export function getBinaryName(): string {
  const platform = os.platform();
  const arch = os.arch();

  let binaryName = "lintp";

  if (platform === "win32") {
    binaryName += ".exe";
  } else if (platform === "darwin") {
    binaryName += "-macos";
    if (arch === "arm64") {
      binaryName += "-arm64";
    } else {
      binaryName += "-x64";
    }
  } else if (platform === "linux") {
    binaryName += "-linux";
    if (arch === "arm64") {
      binaryName += "-arm64";
    } else {
      binaryName += "-x64";
    }
  }

  return binaryName;
}

export function spawnBinary(
  binaryPath: string,
  args: string[],
  onError: (err: NodeJS.ErrnoException) => void,
  onExit: (code: number | null, signal: NodeJS.Signals | null) => void
): ChildProcess {
  const child = spawn(binaryPath, args, {
    stdio: "inherit",
    env: process.env,
  });

  child.on("error", onError);
  child.on("exit", onExit);

  return child;
}

export function handleBinaryError(
  err: NodeJS.ErrnoException,
  binaryPath: string
): void {
  if (err.code === "ENOENT") {
    // eslint-disable-next-line no-console
    console.error(
      `Error: Binary not found for your platform (${os.platform()} ${os.arch()})`
    );
    // eslint-disable-next-line no-console
    console.error(`Expected binary at: ${binaryPath}`);
    // eslint-disable-next-line no-console
    console.error(
      "Please ensure you have the correct binary for your platform."
    );
    process.exit(1);
  }
  throw err;
}

export function handleBinaryExit(
  code: number | null,
  signal: NodeJS.Signals | null
): void {
  if (signal) {
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 0);
  }
}

export function main(): void {
  const binaryName = getBinaryName();
  const binaryPath = path.join(__dirname, "bin", binaryName);

  spawnBinary(
    binaryPath,
    process.argv.slice(2),
    (err) => handleBinaryError(err, binaryPath),
    handleBinaryExit
  );
}

if (require.main === module) {
  main();
}
