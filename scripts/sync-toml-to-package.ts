#!/usr/bin/env node

import fs from "fs";
import path from "path";
import * as TOML from "smol-toml";

interface TomlPackage {
  package?: {
    name?: string;
    version?: string;
    description?: string;
  };
}

interface PackageJson {
  name: string;
  version: string;
  description: string;
  [key: string]: string | number | boolean | object | null | undefined;
}

function syncTomlToPackageJson(): {
  updated: boolean;
  packageJson: PackageJson;
} {
  const tomlPath = path.join(__dirname, "..", "Cargo.toml");
  const packagePath = path.join(__dirname, "..", "package.json");

  // Read files
  const tomlContent = fs.readFileSync(tomlPath, "utf8");
  const packageJson = JSON.parse(fs.readFileSync(packagePath, "utf8"));

  // Parse TOML
  const toml = TOML.parse(tomlContent) as TomlPackage;

  if (!toml.package) {
    console.error("No [package] section found in Cargo.toml");
    process.exit(1);
  }

  // Sync values
  let updated = false;
  const changes = [];

  // The name is intentionally NOT synced: the crate is `lintp` but the npm
  // package is `lintp-cli` (npm rejected the bare name as too similar to
  // existing packages). Only the installed binary is named lintp.

  if (toml.package.version && packageJson.version !== toml.package.version) {
    const oldValue = packageJson.version;
    packageJson.version = toml.package.version;
    changes.push(`version: ${oldValue} → ${packageJson.version}`);
    updated = true;
  }

  if (
    toml.package.description &&
    packageJson.description !== toml.package.description
  ) {
    const oldValue = packageJson.description;
    packageJson.description = toml.package.description;
    changes.push(`description: ${oldValue} → ${packageJson.description}`);
    updated = true;
  }

  // Write back if updated
  if (updated) {
    fs.writeFileSync(packagePath, `${JSON.stringify(packageJson, null, 2)}\n`);
    console.log("✅ package.json updated:");
    changes.forEach((change) => console.log(`  ${change}`));
  } else {
    console.log("✅ package.json is already in sync with Cargo.toml");
  }

  return { updated, packageJson };
}

// Run if called directly
if (require.main === module) {
  syncTomlToPackageJson();
}

export { syncTomlToPackageJson };
