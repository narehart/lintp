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

  if (toml.package.name && packageJson.name !== toml.package.name) {
    packageJson.name = toml.package.name;
    changes.push(`name: ${packageJson.name}`);
    updated = true;
  }

  if (toml.package.version && packageJson.version !== toml.package.version) {
    packageJson.version = toml.package.version;
    changes.push(`version: ${packageJson.version}`);
    updated = true;
  }

  if (
    toml.package.description &&
    packageJson.description !== toml.package.description
  ) {
    packageJson.description = toml.package.description;
    changes.push(`description: ${packageJson.description}`);
    updated = true;
  }

  // Write back if updated
  if (updated) {
    fs.writeFileSync(packagePath, `${JSON.stringify(packageJson, null, 2)  }\n`);
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
