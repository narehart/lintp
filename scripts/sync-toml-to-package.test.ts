import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { readFileSync, writeFileSync } from "fs";
import * as TOML from "smol-toml";

vi.mock("fs");
vi.mock("smol-toml");

describe("sync-toml-to-package.ts", () => {
  const mockReadFileSync = vi.mocked(readFileSync);
  const mockWriteFileSync = vi.mocked(writeFileSync);
  const mockParse = vi.mocked(TOML.parse);
  let consoleLog: ReturnType<typeof vi.spyOn>;
  let consoleError: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    vi.resetAllMocks();
    vi.resetModules();
    consoleLog = vi.spyOn(console, "log").mockImplementation(() => {});
    consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    vi.spyOn(process, "exit").mockImplementation(() => {
      throw new Error("process.exit called");
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("syncTomlToPackageJson", () => {
    const mockPackageJson = {
      name: "old-name",
      version: "0.1.0",
      description: "Old description",
    };

    const mockCargoToml = {
      package: {
        name: "new-name",
        version: "0.2.0",
        description: "New description",
      },
    };

    it("should update package.json when values differ", async () => {
      mockReadFileSync
        .mockReturnValueOnce("cargo toml content")
        .mockReturnValueOnce(JSON.stringify(mockPackageJson, null, 2));

      mockParse.mockReturnValue(mockCargoToml);

      const { syncTomlToPackageJson } = await import("./sync-toml-to-package");
      const result = syncTomlToPackageJson();

      expect(mockReadFileSync).toHaveBeenCalledWith(
        expect.stringContaining("Cargo.toml"),
        "utf8"
      );
      expect(mockReadFileSync).toHaveBeenCalledWith(
        expect.stringContaining("package.json"),
        "utf8"
      );
      expect(mockParse).toHaveBeenCalledWith("cargo toml content");

      const expectedPackageJson = {
        name: "new-name",
        version: "0.2.0",
        description: "New description",
      };

      expect(mockWriteFileSync).toHaveBeenCalledWith(
        expect.stringContaining("package.json"),
        `${JSON.stringify(expectedPackageJson, null, 2)}\n`
      );

      expect(consoleLog).toHaveBeenCalledWith("✅ package.json updated:");
      expect(consoleLog).toHaveBeenCalledWith("  name: old-name → new-name");
      expect(consoleLog).toHaveBeenCalledWith("  version: 0.1.0 → 0.2.0");
      expect(consoleLog).toHaveBeenCalledWith(
        "  description: Old description → New description"
      );

      expect(result.updated).toBe(true);
      expect(result.packageJson).toEqual(expectedPackageJson);
    });

    it("should not update package.json when values are the same", async () => {
      const samePackageJson = {
        name: "new-name",
        version: "0.2.0",
        description: "New description",
      };

      mockReadFileSync
        .mockReturnValueOnce("cargo toml content")
        .mockReturnValueOnce(JSON.stringify(samePackageJson, null, 2));

      mockParse.mockReturnValue(mockCargoToml);

      const { syncTomlToPackageJson } = await import("./sync-toml-to-package");
      const result = syncTomlToPackageJson();

      expect(mockWriteFileSync).not.toHaveBeenCalled();
      expect(consoleLog).toHaveBeenCalledWith(
        "✅ package.json is already in sync with Cargo.toml"
      );
      expect(result.updated).toBe(false);
      expect(result.packageJson).toEqual(samePackageJson);
    });

    it("should handle missing package section in Cargo.toml", async () => {
      mockReadFileSync
        .mockReturnValueOnce("cargo toml content")
        .mockReturnValueOnce(
          JSON.stringify(
            { name: "test", version: "1.0.0", description: "test" },
            null,
            2
          )
        );
      mockParse.mockReturnValue({});

      const { syncTomlToPackageJson } = await import("./sync-toml-to-package");

      expect(() => syncTomlToPackageJson()).toThrow("process.exit called");
      expect(consoleError).toHaveBeenCalledWith(
        "No [package] section found in Cargo.toml"
      );
      expect(mockWriteFileSync).not.toHaveBeenCalled();
    });

    it("should update only changed fields", async () => {
      const partiallyDifferentPackageJson = {
        name: "new-name",
        version: "0.1.0",
        description: "New description",
      };

      mockReadFileSync
        .mockReturnValueOnce("cargo toml content")
        .mockReturnValueOnce(
          JSON.stringify(partiallyDifferentPackageJson, null, 2)
        );

      mockParse.mockReturnValue(mockCargoToml);

      const { syncTomlToPackageJson } = await import("./sync-toml-to-package");
      const result = syncTomlToPackageJson();

      expect(mockWriteFileSync).toHaveBeenCalled();
      expect(consoleLog).toHaveBeenCalledWith("✅ package.json updated:");
      expect(consoleLog).toHaveBeenCalledWith("  version: 0.1.0 → 0.2.0");
      expect(consoleLog).not.toHaveBeenCalledWith(
        expect.stringContaining("name:")
      );
      expect(consoleLog).not.toHaveBeenCalledWith(
        expect.stringContaining("description:")
      );
      expect(result.updated).toBe(true);
    });

    it("should handle missing fields in Cargo.toml", async () => {
      const incompleteCargoToml = {
        package: {
          name: "new-name",
          version: "0.2.0",
          // description is missing
        },
      };

      mockReadFileSync
        .mockReturnValueOnce("cargo toml content")
        .mockReturnValueOnce(JSON.stringify(mockPackageJson, null, 2));

      mockParse.mockReturnValue(incompleteCargoToml);

      const { syncTomlToPackageJson } = await import("./sync-toml-to-package");
      const result = syncTomlToPackageJson();

      const writtenContent = mockWriteFileSync.mock.calls[0][1] as string;
      const writtenJson = JSON.parse(writtenContent);

      expect(writtenJson.name).toBe("new-name");
      expect(writtenJson.version).toBe("0.2.0");
      expect(writtenJson.description).toBe("Old description"); // Should keep the old description
      expect(result.updated).toBe(true);
    });

    it("should preserve other fields in package.json", async () => {
      const extendedPackageJson = {
        name: "old-name",
        version: "0.1.0",
        description: "Old description",
        author: "Test Author",
        license: "MIT",
        scripts: {
          test: "vitest",
        },
      };

      mockReadFileSync
        .mockReturnValueOnce("cargo toml content")
        .mockReturnValueOnce(JSON.stringify(extendedPackageJson, null, 2));

      mockParse.mockReturnValue(mockCargoToml);

      const { syncTomlToPackageJson } = await import("./sync-toml-to-package");
      const result = syncTomlToPackageJson();

      const writtenContent = mockWriteFileSync.mock.calls[0][1] as string;
      const writtenJson = JSON.parse(writtenContent);

      expect(writtenJson.author).toBe("Test Author");
      expect(writtenJson.license).toBe("MIT");
      expect(writtenJson.scripts).toEqual({ test: "vitest" });
      expect(result.updated).toBe(true);
    });
  });
});
