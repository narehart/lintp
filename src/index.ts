#!/usr/bin/env node

import { ChildProcess, spawn } from "child_process";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from "fs";
import path from "path";
import os from "os";
import https from "https";
import { createHash } from "crypto";

const MAX_REDIRECTS = 5;
const REQUEST_TIMEOUT_MS = 30_000;

export function getPlatformTarget(): string {
  const platform = os.platform();
  const arch = os.arch();

  if (platform === "win32") {
    // Windows on ARM has no native build; run the x64 binary under emulation.
    return "x86_64-pc-windows-msvc";
  } else if (platform === "darwin") {
    if (arch === "arm64") {
      return "aarch64-apple-darwin";
    } else if (arch === "x64") {
      return "x86_64-apple-darwin";
    }
  } else if (platform === "linux") {
    if (arch === "arm64") {
      return "aarch64-unknown-linux-gnu";
    } else if (arch === "x64") {
      return "x86_64-unknown-linux-gnu";
    }
  }

  throw new Error(`Unsupported platform: ${platform} ${arch}`);
}

export function getBinaryName(): string {
  return os.platform() === "win32" ? "lintp.exe" : "lintp";
}

export function getAssetName(): string {
  const suffix = os.platform() === "win32" ? ".exe" : "";
  return `lintp-${getPlatformTarget()}${suffix}`;
}

const PLATFORM_PACKAGES = new Set([
  "darwin-arm64",
  "darwin-x64",
  "linux-arm64",
  "linux-x64",
  "win32-x64",
]);

export function getPlatformPackageName(): string | null {
  const key = `${os.platform()}-${os.arch()}`;
  if (PLATFORM_PACKAGES.has(key)) {
    return `lintp-${key}`;
  }
  // Windows on ARM runs x64 binaries via emulation
  if (os.platform() === "win32") {
    return "lintp-win32-x64";
  }
  return null;
}

/**
 * Preferred distribution path: the platform-specific package installed by
 * npm via optionalDependencies (the esbuild/Biome model). npm delivers and
 * integrity-checks the binary; no runtime download needed.
 */
export function resolveInstalledBinary(): string | null {
  const packageName = getPlatformPackageName();
  if (!packageName) {
    return null;
  }
  try {
    return require.resolve(`${packageName}/bin/${getBinaryName()}`);
  } catch {
    return null;
  }
}

// Works from both src/ (dev, tests) and dist/src/ (published build)
export function getPackageVersion(): string {
  let dir = __dirname;
  for (let i = 0; i < 3; i++) {
    const candidate = path.join(dir, "package.json");
    if (existsSync(candidate)) {
      const pkg = JSON.parse(readFileSync(candidate, "utf8")) as {
        version: string;
      };
      return pkg.version;
    }
    dir = path.dirname(dir);
  }
  throw new Error("Could not locate package.json to determine lintp version");
}

export function getBinaryPath(): string {
  const homeDir = os.homedir();
  return path.join(
    homeDir,
    ".lintp",
    "bin",
    getPackageVersion(),
    getBinaryName()
  );
}

function httpGet(url: string, redirectsLeft = MAX_REDIRECTS): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    const request = https
      .get(url, (response) => {
        const status = response.statusCode ?? 0;

        if (status >= 301 && status <= 308 && response.headers.location) {
          response.resume();
          if (redirectsLeft <= 0) {
            reject(new Error(`Too many redirects fetching ${url}`));
            return;
          }
          httpGet(response.headers.location, redirectsLeft - 1)
            .then(resolve)
            .catch(reject);
          return;
        }

        if (status !== 200) {
          response.resume();
          reject(new Error(`Failed to download ${url}: HTTP ${status}`));
          return;
        }

        const chunks: Buffer[] = [];
        response.on("data", (chunk: Buffer) => chunks.push(chunk));
        response.on("end", () => resolve(Buffer.concat(chunks)));
        response.on("error", reject);
      })
      .on("error", reject);

    request.setTimeout(REQUEST_TIMEOUT_MS, () => {
      request.destroy(
        new Error(
          `Request timed out after ${REQUEST_TIMEOUT_MS}ms fetching ${url}`
        )
      );
    });
  });
}

export function verifyChecksum(data: Buffer, checksumFile: string): boolean {
  // Checksum asset format: "<hex digest>  <filename>" (or just the digest)
  const expected = checksumFile.trim().split(/\s+/)[0].toLowerCase();
  const actual = createHash("sha256").update(data).digest("hex");
  return expected === actual;
}

export async function downloadBinary(url: string, dest: string): Promise<void> {
  const data = await httpGet(url);
  const checksum = await httpGet(`${url}.sha256`);

  if (!verifyChecksum(data, checksum.toString("utf8"))) {
    throw new Error(
      `Checksum verification failed for ${url}. ` +
        "The downloaded binary does not match its published SHA256 digest."
    );
  }

  const dir = path.dirname(dest);
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }

  writeFileSync(dest, data);
  if (os.platform() !== "win32") {
    chmodSync(dest, 0o755);
  }
}

export async function ensureBinary(): Promise<string> {
  // 1. Platform package installed through optionalDependencies
  const installed = resolveInstalledBinary();
  if (installed) {
    return installed;
  }

  // 2. Previously downloaded binary cached under ~/.lintp
  const binaryPath = getBinaryPath();

  if (existsSync(binaryPath)) {
    return binaryPath;
  }

  // 3. Fallback: download from the GitHub release (checksum-verified)

  const version = getPackageVersion();
  const downloadUrl = `https://github.com/narehart/lintp/releases/download/v${version}/${getAssetName()}`;

  console.log(`Downloading lintp binary for ${getPlatformTarget()}...`);

  try {
    await downloadBinary(downloadUrl, binaryPath);
    console.log(`Binary downloaded successfully to ${binaryPath}`);
    return binaryPath;
  } catch (error) {
    console.error(`Failed to download binary: ${error}`);
    console.error(`Please download manually from: ${downloadUrl}`);
    process.exit(1);
    // process.exit(1) never returns in production. This throw exists so
    // that if process.exit is stubbed out (e.g. in tests, or some other
    // unusual host environment), ensureBinary still can't fall through and
    // return `undefined` where a string binaryPath is expected.
    throw error;
  }
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
    console.error(`Error: Binary not found at: ${binaryPath}`);
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

/**
 * Installs SIGINT/SIGTERM handlers on this (wrapper) process so that Node's
 * default signal disposition — immediate termination — can't preempt
 * handleBinaryExit's exit-code forwarding.
 *
 * Without this, hitting Ctrl+C sends SIGINT to the whole process group
 * (wrapper and child alike). If the wrapper has no SIGINT listener, Node
 * kills it immediately per the default disposition, racing the child's own
 * shutdown and potentially exiting the wrapper before the child's "exit"
 * event (and its real exit code) is ever observed.
 *
 * Installing a handler suppresses that default disposition. The handler
 * simply forwards the signal to the child if it's still running; if the
 * child has already exited, it does nothing, leaving the child's "exit"
 * event (wired to handleBinaryExit) as the sole thing that decides how and
 * when the wrapper itself exits.
 */
export function installSignalForwarding(child: ChildProcess): void {
  const forward = (signal: NodeJS.Signals): void => {
    if (child.exitCode === null && child.signalCode === null) {
      child.kill(signal);
    }
  };

  process.on("SIGINT", forward);
  process.on("SIGTERM", forward);
}

export async function main(): Promise<void> {
  try {
    const binaryPath = await ensureBinary();

    const child = spawnBinary(
      binaryPath,
      process.argv.slice(2),
      (err) => handleBinaryError(err, binaryPath),
      handleBinaryExit
    );

    installSignalForwarding(child);
  } catch (error) {
    console.error("Failed to start lintp:", error);
    process.exit(1);
  }
}

if (require.main === module) {
  main().catch((error) => {
    console.error("Unexpected error:", error);
    process.exit(1);
  });
}
