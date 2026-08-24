#!/usr/bin/env node

/**
 * Injects optionalDependencies for the platform binary packages into
 * package.json, pinned to the current version. Run in CI immediately
 * before publishing the main package so the dependency versions always
 * match the release being published.
 *
 * The optionalDependencies are intentionally NOT checked in: release-please
 * only bumps the version field, so committed entries would go stale.
 */

import fs from "fs";
import path from "path";

const PLATFORM_PACKAGES = [
  "lintp-darwin-arm64",
  "lintp-darwin-x64",
  "lintp-linux-arm64",
  "lintp-linux-arm64-musl",
  "lintp-linux-x64",
  "lintp-linux-x64-musl",
  "lintp-win32-x64",
];

/**
 * `rootDir` defaults to the repo root and exists so tests can drive this
 * against a temporary directory instead of rewriting the real package.json.
 */
export function syncOptionalDeps(
  rootDir: string = path.join(__dirname, "..")
): void {
  const packagePath = path.join(rootDir, "package.json");
  const packageJson = JSON.parse(fs.readFileSync(packagePath, "utf8")) as {
    version: string;
    optionalDependencies?: Record<string, string>;
  };

  packageJson.optionalDependencies = Object.fromEntries(
    PLATFORM_PACKAGES.map((name) => [name, packageJson.version])
  );

  fs.writeFileSync(packagePath, `${JSON.stringify(packageJson, null, 2)}\n`);
  console.log(
    `Pinned ${PLATFORM_PACKAGES.length} optionalDependencies to ${packageJson.version}`
  );
}

if (require.main === module) {
  syncOptionalDeps();
}
