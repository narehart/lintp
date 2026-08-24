import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import fs from "fs";
import os from "os";
import path from "path";

import { getKnownTargets } from "./prepare-platform-package";
import { syncOptionalDeps } from "./sync-optional-deps";

describe("sync-optional-deps.ts", () => {
  let rootDir: string;

  function writePackageJson(contents: Record<string, unknown>): void {
    fs.writeFileSync(
      path.join(rootDir, "package.json"),
      `${JSON.stringify(contents, null, 2)}\n`
    );
  }

  function readPackageJson(): Record<string, unknown> {
    return JSON.parse(
      fs.readFileSync(path.join(rootDir, "package.json"), "utf8")
    );
  }

  beforeEach(() => {
    rootDir = fs.mkdtempSync(path.join(os.tmpdir(), "lintp-optional-deps-"));
    vi.spyOn(console, "log").mockImplementation(() => {});
  });

  afterEach(() => {
    fs.rmSync(rootDir, { recursive: true, force: true });
    vi.restoreAllMocks();
  });

  it("pins every platform package to the current version", () => {
    writePackageJson({ name: "lintp-cli", version: "2.0.0" });

    syncOptionalDeps(rootDir);

    expect(readPackageJson().optionalDependencies).toEqual({
      "lintp-darwin-arm64": "2.0.0",
      "lintp-darwin-x64": "2.0.0",
      "lintp-linux-arm64": "2.0.0",
      "lintp-linux-arm64-musl": "2.0.0",
      "lintp-linux-x64": "2.0.0",
      "lintp-linux-x64-musl": "2.0.0",
      "lintp-win32-x64": "2.0.0",
    });
  });

  it("replaces stale entries from an earlier release", () => {
    writePackageJson({
      name: "lintp-cli",
      version: "2.0.0",
      optionalDependencies: {
        "lintp-darwin-arm64": "1.0.0",
        "lintp-retired-platform": "1.0.0",
      },
    });

    syncOptionalDeps(rootDir);

    const deps = readPackageJson().optionalDependencies as Record<
      string,
      string
    >;
    expect(deps["lintp-darwin-arm64"]).toBe("2.0.0");
    expect(deps).not.toHaveProperty("lintp-retired-platform");
  });

  it("leaves the rest of the manifest untouched", () => {
    writePackageJson({
      name: "lintp-cli",
      version: "2.0.0",
      bin: { lintp: "./bin/lintp" },
      devDependencies: { vitest: "^3.2.4" },
    });

    syncOptionalDeps(rootDir);

    const pkg = readPackageJson();
    expect(pkg.name).toBe("lintp-cli");
    expect(pkg.bin).toEqual({ lintp: "./bin/lintp" });
    expect(pkg.devDependencies).toEqual({ vitest: "^3.2.4" });
  });

  it("leaves package.json newline-terminated for git and prettier", () => {
    writePackageJson({ name: "lintp-cli", version: "2.0.0" });

    syncOptionalDeps(rootDir);

    const raw = fs.readFileSync(path.join(rootDir, "package.json"), "utf8");
    expect(raw.endsWith("\n")).toBe(true);
  });

  /**
   * The two lists are maintained separately — one keyed by Rust target
   * triple, one by npm package name — so a new platform added to the release
   * matrix can easily land in one and not the other. That would publish a
   * platform package no install ever resolves.
   */
  it("declares exactly the packages the build matrix produces", () => {
    writePackageJson({ name: "lintp-cli", version: "2.0.0" });

    syncOptionalDeps(rootDir);

    const declared = Object.keys(
      readPackageJson().optionalDependencies as Record<string, string>
    );
    expect(declared.length).toBe(getKnownTargets().length);
  });
});
