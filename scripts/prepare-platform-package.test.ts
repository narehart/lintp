import { afterEach, beforeEach, describe, expect, it } from "vitest";
import fs from "fs";
import os from "os";
import path from "path";

import {
  getKnownTargets,
  preparePlatformPackage,
} from "./prepare-platform-package";

/**
 * These tests run against a real temporary directory rather than a mocked
 * `fs`: the whole job of this script is to lay out files on disk in the shape
 * npm expects, so asserting on mock call sequences would test almost nothing.
 */
describe("prepare-platform-package.ts", () => {
  let rootDir: string;

  /** Stand up a fake repo root with a package.json and a built binary. */
  function seedRoot(target: string, binaryName = "lintp"): void {
    fs.writeFileSync(
      path.join(rootDir, "package.json"),
      JSON.stringify({
        version: "1.2.3",
        license: "MIT",
        repository: { type: "git", url: "git+https://example.com/lintp.git" },
      })
    );

    const releaseDir = path.join(rootDir, "target", target, "release");
    fs.mkdirSync(releaseDir, { recursive: true });
    fs.writeFileSync(path.join(releaseDir, binaryName), "binary contents");
  }

  function readManifest(packageDir: string): Record<string, unknown> {
    return JSON.parse(
      fs.readFileSync(path.join(packageDir, "package.json"), "utf8")
    );
  }

  beforeEach(() => {
    rootDir = fs.mkdtempSync(path.join(os.tmpdir(), "lintp-platform-"));
  });

  afterEach(() => {
    fs.rmSync(rootDir, { recursive: true, force: true });
  });

  describe("getKnownTargets", () => {
    it("covers every triple the release matrix builds", () => {
      // Keep in step with the build matrix in .github/workflows/release.yml
      expect(getKnownTargets().sort()).toEqual(
        [
          "aarch64-apple-darwin",
          "aarch64-unknown-linux-gnu",
          "aarch64-unknown-linux-musl",
          "x86_64-apple-darwin",
          "x86_64-pc-windows-msvc",
          "x86_64-unknown-linux-gnu",
          "x86_64-unknown-linux-musl",
        ].sort()
      );
    });
  });

  describe("preparePlatformPackage", () => {
    it("rejects an unknown target and names the ones it knows", () => {
      expect(() =>
        preparePlatformPackage("sparc-unknown-none", rootDir)
      ).toThrowError(/Unknown target: sparc-unknown-none/);
      expect(() =>
        preparePlatformPackage("sparc-unknown-none", rootDir)
      ).toThrowError(/aarch64-apple-darwin/);
    });

    it("fails loudly when the binary was never built", () => {
      fs.writeFileSync(
        path.join(rootDir, "package.json"),
        JSON.stringify({ version: "1.2.3", license: "MIT", repository: {} })
      );

      expect(() =>
        preparePlatformPackage("x86_64-apple-darwin", rootDir)
      ).toThrowError(/Built binary not found/);
    });

    it("copies the binary into the package's bin directory", () => {
      seedRoot("aarch64-apple-darwin");

      const packageDir = preparePlatformPackage(
        "aarch64-apple-darwin",
        rootDir
      );

      expect(packageDir).toBe(path.join(rootDir, "npm", "lintp-darwin-arm64"));
      expect(
        fs.readFileSync(path.join(packageDir, "bin", "lintp"), "utf8")
      ).toBe("binary contents");
    });

    it("takes the version, license and repository from the main package", () => {
      seedRoot("aarch64-apple-darwin");

      const manifest = readManifest(
        preparePlatformPackage("aarch64-apple-darwin", rootDir)
      );

      expect(manifest.name).toBe("lintp-darwin-arm64");
      expect(manifest.version).toBe("1.2.3");
      expect(manifest.license).toBe("MIT");
      expect(manifest.repository).toEqual({
        type: "git",
        url: "git+https://example.com/lintp.git",
      });
      expect(manifest.files).toEqual(["bin"]);
    });

    it("constrains the package to one os/cpu pair", () => {
      seedRoot("x86_64-unknown-linux-gnu");

      const manifest = readManifest(
        preparePlatformPackage("x86_64-unknown-linux-gnu", rootDir)
      );

      expect(manifest.os).toEqual(["linux"]);
      expect(manifest.cpu).toEqual(["x64"]);
    });

    it("marks libc on linux so npm can tell glibc and musl apart", () => {
      seedRoot("x86_64-unknown-linux-gnu");
      seedRoot("x86_64-unknown-linux-musl");

      const glibc = readManifest(
        preparePlatformPackage("x86_64-unknown-linux-gnu", rootDir)
      );
      const musl = readManifest(
        preparePlatformPackage("x86_64-unknown-linux-musl", rootDir)
      );

      expect(glibc.libc).toEqual(["glibc"]);
      expect(musl.libc).toEqual(["musl"]);
    });

    it("omits libc where it has no meaning", () => {
      seedRoot("aarch64-apple-darwin");

      const manifest = readManifest(
        preparePlatformPackage("aarch64-apple-darwin", rootDir)
      );

      expect(manifest).not.toHaveProperty("libc");
    });

    it("uses the .exe binary name on windows", () => {
      seedRoot("x86_64-pc-windows-msvc", "lintp.exe");

      const packageDir = preparePlatformPackage(
        "x86_64-pc-windows-msvc",
        rootDir
      );

      expect(fs.existsSync(path.join(packageDir, "bin", "lintp.exe"))).toBe(
        true
      );
    });

    it("makes the unix binary executable", () => {
      seedRoot("aarch64-apple-darwin");

      const packageDir = preparePlatformPackage(
        "aarch64-apple-darwin",
        rootDir
      );

      const { mode } = fs.statSync(path.join(packageDir, "bin", "lintp"));
      // 0o111 — the execute bit for user, group and other
      expect(mode & 0o111).toBe(0o111);
    });

    it("writes a manifest npm can parse, newline-terminated", () => {
      seedRoot("aarch64-apple-darwin");

      const packageDir = preparePlatformPackage(
        "aarch64-apple-darwin",
        rootDir
      );
      const raw = fs.readFileSync(
        path.join(packageDir, "package.json"),
        "utf8"
      );

      expect(raw.endsWith("\n")).toBe(true);
      expect(() => JSON.parse(raw)).not.toThrow();
    });

    it("overwrites a stale package left by a previous run", () => {
      seedRoot("aarch64-apple-darwin");
      const packageDir = path.join(rootDir, "npm", "lintp-darwin-arm64");
      fs.mkdirSync(path.join(packageDir, "bin"), { recursive: true });
      fs.writeFileSync(path.join(packageDir, "bin", "lintp"), "stale binary");

      preparePlatformPackage("aarch64-apple-darwin", rootDir);

      expect(
        fs.readFileSync(path.join(packageDir, "bin", "lintp"), "utf8")
      ).toBe("binary contents");
    });
  });
});
