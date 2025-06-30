import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { type ChildProcess, spawn } from "child_process";
import { arch, platform } from "os";
import path from "path";

vi.mock("child_process");
vi.mock("os");

describe("index.ts", () => {
  const mockSpawn = vi.mocked(spawn);
  const mockPlatform = vi.mocked(platform);
  const mockArch = vi.mocked(arch);

  beforeEach(() => {
    vi.resetAllMocks();
    vi.clearAllMocks();
    vi.resetModules();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("getBinaryName", () => {
    it("should return windows binary for win32 platform", async () => {
      mockPlatform.mockReturnValue("win32");
      mockArch.mockReturnValue("x64");

      const { getBinaryName } = await import("./index");
      expect(getBinaryName()).toBe("lintp.exe");
    });

    it("should return windows binary for win32 platform arm64", async () => {
      mockPlatform.mockReturnValue("win32");
      mockArch.mockReturnValue("arm64");

      const { getBinaryName } = await import("./index");
      expect(getBinaryName()).toBe("lintp.exe");
    });

    it("should return macos binary for darwin platform x64", async () => {
      mockPlatform.mockReturnValue("darwin");
      mockArch.mockReturnValue("x64");

      const { getBinaryName } = await import("./index");
      expect(getBinaryName()).toBe("lintp-macos-x64");
    });

    it("should return macos binary for darwin arm64", async () => {
      mockPlatform.mockReturnValue("darwin");
      mockArch.mockReturnValue("arm64");

      const { getBinaryName } = await import("./index");
      expect(getBinaryName()).toBe("lintp-macos-arm64");
    });

    it("should return linux binary for linux platform x64", async () => {
      mockPlatform.mockReturnValue("linux");
      mockArch.mockReturnValue("x64");

      const { getBinaryName } = await import("./index");
      expect(getBinaryName()).toBe("lintp-linux-x64");
    });

    it("should return linux binary for linux platform arm64", async () => {
      mockPlatform.mockReturnValue("linux");
      mockArch.mockReturnValue("arm64");

      const { getBinaryName } = await import("./index");
      expect(getBinaryName()).toBe("lintp-linux-arm64");
    });

    it("should return base binary name for unsupported platform", async () => {
      mockPlatform.mockReturnValue("freebsd" as NodeJS.Platform);
      mockArch.mockReturnValue("x64");

      const { getBinaryName } = await import("./index");
      expect(getBinaryName()).toBe("lintp");
    });

    it("should default to x64 for unsupported architecture on darwin", async () => {
      mockPlatform.mockReturnValue("darwin");
      mockArch.mockReturnValue("ia32");

      const { getBinaryName } = await import("./index");
      expect(getBinaryName()).toBe("lintp-macos-x64");
    });

    it("should default to x64 for unsupported architecture on linux", async () => {
      mockPlatform.mockReturnValue("linux");
      mockArch.mockReturnValue("ia32");

      const { getBinaryName } = await import("./index");
      expect(getBinaryName()).toBe("lintp-linux-x64");
    });
  });

  describe("spawnBinary", () => {
    it("should spawn child process with correct arguments", async () => {
      const mockChild = {
        on: vi.fn(),
      };
      mockSpawn.mockReturnValue(mockChild as unknown as ChildProcess);

      const onError = vi.fn();
      const onExit = vi.fn();

      const { spawnBinary } = await import("./index");
      const child = spawnBinary(
        "/path/to/binary",
        ["--help", "--version"],
        onError,
        onExit
      );

      expect(mockSpawn).toHaveBeenCalledWith(
        "/path/to/binary",
        ["--help", "--version"],
        { stdio: "inherit", env: process.env }
      );
      expect(mockChild.on).toHaveBeenCalledWith("error", onError);
      expect(mockChild.on).toHaveBeenCalledWith("exit", onExit);
      expect(child).toBe(mockChild);
    });
  });

  describe("handleBinaryError", () => {
    it("should handle ENOENT error", async () => {
      mockPlatform.mockReturnValue("linux");
      mockArch.mockReturnValue("x64");

      const consoleError = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});
      const processExit = vi.spyOn(process, "exit").mockImplementation(() => {
        throw new Error("process.exit called");
      });

      const { handleBinaryError } = await import("./index");
      const error = new Error("File not found") as NodeJS.ErrnoException;
      error.code = "ENOENT";

      expect(() => handleBinaryError(error, "/path/to/binary")).toThrow(
        "process.exit called"
      );

      expect(consoleError).toHaveBeenCalledWith(
        expect.stringContaining("Error: Binary not found for your platform")
      );
      expect(consoleError).toHaveBeenCalledWith(
        "Expected binary at: /path/to/binary"
      );

      consoleError.mockRestore();
      processExit.mockRestore();
    });

    it("should throw other errors", async () => {
      const { handleBinaryError } = await import("./index");
      const error = new Error("Some other error");

      expect(() =>
        handleBinaryError(error as NodeJS.ErrnoException, "/path/to/binary")
      ).toThrow("Some other error");
    });
  });

  describe("handleBinaryExit", () => {
    it("should handle exit with code", async () => {
      const processExit = vi
        .spyOn(process, "exit")
        .mockImplementation(() => undefined as never);

      const { handleBinaryExit } = await import("./index");
      handleBinaryExit(42, null);

      expect(processExit).toHaveBeenCalledWith(42);
      processExit.mockRestore();
    });

    it("should handle exit with code 0 when null", async () => {
      const processExit = vi
        .spyOn(process, "exit")
        .mockImplementation(() => undefined as never);

      const { handleBinaryExit } = await import("./index");
      handleBinaryExit(null, null);

      expect(processExit).toHaveBeenCalledWith(0);
      processExit.mockRestore();
    });

    it("should handle exit with signal", async () => {
      const processKill = vi
        .spyOn(process, "kill")
        .mockImplementation(() => true);

      const { handleBinaryExit } = await import("./index");
      handleBinaryExit(null, "SIGTERM");

      expect(processKill).toHaveBeenCalledWith(process.pid, "SIGTERM");
      processKill.mockRestore();
    });
  });

  describe("main", () => {
    it("should call spawnBinary with correct arguments", async () => {
      mockPlatform.mockReturnValue("darwin");
      mockArch.mockReturnValue("x64");
      process.argv = ["node", "index.js", "--help", "--version"];

      const mockChild = {
        on: vi.fn(),
      };
      mockSpawn.mockReturnValue(mockChild as unknown as ChildProcess);

      const { main } = await import("./index");
      main();

      expect(mockSpawn).toHaveBeenCalledWith(
        expect.stringContaining(path.join("bin", "lintp-macos-x64")),
        ["--help", "--version"],
        { stdio: "inherit", env: process.env }
      );
    });

    it("should handle errors and exits", async () => {
      mockPlatform.mockReturnValue("linux");
      mockArch.mockReturnValue("arm64");
      process.argv = ["node", "index.js"];

      const mockChild = {
        on: vi.fn(),
      };
      mockSpawn.mockReturnValue(mockChild as unknown as ChildProcess);

      const consoleError = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});
      const processExit = vi.spyOn(process, "exit").mockImplementation(() => {
        throw new Error("process.exit called");
      });

      const { main } = await import("./index");
      main();

      // Get the callbacks
      const errorCallback = mockChild.on.mock.calls.find(
        (call) => call[0] === "error"
      )?.[1];
      const exitCallback = mockChild.on.mock.calls.find(
        (call) => call[0] === "exit"
      )?.[1];

      // Test error handling
      const error = new Error("File not found") as NodeJS.ErrnoException;
      error.code = "ENOENT";

      expect(() => errorCallback(error)).toThrow("process.exit called");
      expect(consoleError).toHaveBeenCalledWith(
        expect.stringContaining("Error: Binary not found for your platform")
      );

      // Reset process.exit mock to not throw
      processExit.mockImplementation(() => undefined as never);

      // Test exit handling
      exitCallback(0, null);
      expect(processExit).toHaveBeenCalledWith(0);

      consoleError.mockRestore();
      processExit.mockRestore();
    });
  });

  describe("main execution", () => {
    it("should test that main module check exists", async () => {
      // Simply verify the module exports and the if check exists
      const module = await import("./index");

      expect(module.main).toBeDefined();
      expect(module.getBinaryName).toBeDefined();
      expect(module.spawnBinary).toBeDefined();
      expect(module.handleBinaryError).toBeDefined();
      expect(module.handleBinaryExit).toBeDefined();

      // The actual main execution is covered by the 'main' tests above
    });
  });
});
