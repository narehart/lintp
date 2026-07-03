#!/usr/bin/env node

/**
 * Builds a platform-specific npm package (e.g. lintp-darwin-arm64) around a
 * compiled binary, ready for `npm publish`. Run from CI once per build
 * matrix target:
 *
 *   tsx scripts/prepare-platform-package.ts --target aarch64-apple-darwin
 *
 * The package is written to npm/<package-name>/ and contains only the
 * binary plus a minimal package.json with os/cpu constraints, so npm
 * installs exactly one of these through the main package's
 * optionalDependencies.
 */

import fs from "fs";
import path from "path";

interface PlatformInfo {
  packageName: string;
  os: string;
  cpu: string;
  binaryName: string;
}

const TARGETS: Record<string, PlatformInfo> = {
  "x86_64-apple-darwin": {
    packageName: "lintp-darwin-x64",
    os: "darwin",
    cpu: "x64",
    binaryName: "lintp",
  },
  "aarch64-apple-darwin": {
    packageName: "lintp-darwin-arm64",
    os: "darwin",
    cpu: "arm64",
    binaryName: "lintp",
  },
  "x86_64-unknown-linux-gnu": {
    packageName: "lintp-linux-x64",
    os: "linux",
    cpu: "x64",
    binaryName: "lintp",
  },
  "aarch64-unknown-linux-gnu": {
    packageName: "lintp-linux-arm64",
    os: "linux",
    cpu: "arm64",
    binaryName: "lintp",
  },
  "x86_64-pc-windows-msvc": {
    packageName: "lintp-win32-x64",
    os: "win32",
    cpu: "x64",
    binaryName: "lintp.exe",
  },
};

export function preparePlatformPackage(target: string): string {
  const info = TARGETS[target];
  if (!info) {
    throw new Error(
      `Unknown target: ${target}. Known targets: ${Object.keys(TARGETS).join(
        ", "
      )}`
    );
  }

  const rootDir = path.join(__dirname, "..");
  const mainPackage = JSON.parse(
    fs.readFileSync(path.join(rootDir, "package.json"), "utf8")
  ) as { version: string; license: string; repository: object };

  const builtBinary = path.join(
    rootDir,
    "target",
    target,
    "release",
    info.binaryName
  );
  if (!fs.existsSync(builtBinary)) {
    throw new Error(`Built binary not found: ${builtBinary}`);
  }

  const packageDir = path.join(rootDir, "npm", info.packageName);
  const binDir = path.join(packageDir, "bin");
  fs.mkdirSync(binDir, { recursive: true });

  fs.copyFileSync(builtBinary, path.join(binDir, info.binaryName));
  if (info.os !== "win32") {
    fs.chmodSync(path.join(binDir, info.binaryName), 0o755);
  }

  const manifest = {
    name: info.packageName,
    version: mainPackage.version,
    description: `lintp binary for ${info.os}-${info.cpu}`,
    license: mainPackage.license,
    repository: mainPackage.repository,
    os: [info.os],
    cpu: [info.cpu],
    files: ["bin"],
  };

  fs.writeFileSync(
    path.join(packageDir, "package.json"),
    `${JSON.stringify(manifest, null, 2)}\n`
  );

  return packageDir;
}

export function getKnownTargets(): string[] {
  return Object.keys(TARGETS);
}

if (require.main === module) {
  const targetIndex = process.argv.indexOf("--target");
  const target = targetIndex >= 0 ? process.argv[targetIndex + 1] : undefined;

  if (!target) {
    console.error(
      "Usage: tsx scripts/prepare-platform-package.ts --target <rust-target-triple>"
    );
    process.exit(1);
  }

  const packageDir = preparePlatformPackage(target);
  console.log(`Platform package ready: ${packageDir}`);
}
