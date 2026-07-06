import {
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from "vitest";
import { type ChildProcess, spawn } from "child_process";
import { arch, homedir, platform } from "os";
import { existsSync } from "fs";
import { createHash } from "crypto";
import path from "path";

vi.mock("child_process");
vi.mock("os");
vi.mock("fs", async (importOriginal) => {
  const actual = await importOriginal<typeof import("fs")>();
  return {
    ...actual,
    existsSync: vi.fn(actual.existsSync),
  };
});

let actualFs: typeof import("fs");

beforeAll(async () => {
  actualFs = await vi.importActual<typeof import("fs")>("fs");
});

function realPackageVersion(): string {
  const pkg = JSON.parse(
    actualFs.readFileSync(path.join(process.cwd(), "package.json"), "utf8")
  ) as { version: string };
  return pkg.version;
}

describe("index.ts", () => {
  const mockSpawn = vi.mocked(spawn);
  const mockPlatform = vi.mocked(platform);
  const mockArch = vi.mocked(arch);
  const mockHomedir = vi.mocked(homedir);
  const mockExistsSync = vi.mocked(existsSync);

  function mockProcessReport(header: Record<string, unknown>) {
    return vi.spyOn(process, "report", "get").mockReturnValue({
      getReport: () => ({ header }),
    } as unknown as NodeJS.ProcessReport);
  }

  beforeEach(() => {
    vi.resetAllMocks();
    vi.clearAllMocks();
    vi.resetModules();
    // Default: real filesystem behavior (package.json discovery etc.)
    mockExistsSync.mockImplementation(actualFs.existsSync);
    // Default: a glibc-reporting host, so tests that mock os.platform() as
    // "linux" get consistent isMusl()=false behavior regardless of what the
    // test runner's actual host libc is. Individual musl tests override this.
    mockProcessReport({ glibcVersionRuntime: "2.31" });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("isMusl", () => {
    it("returns false on non-linux platforms regardless of process.report", async () => {
      mockPlatform.mockReturnValue("darwin");

      const { isMusl } = await import("./index");
      expect(isMusl()).toBe(false);
    });

    it("returns false on linux when glibcVersionRuntime is present", async () => {
      mockPlatform.mockReturnValue("linux");
      mockProcessReport({ glibcVersionRuntime: "2.31" });

      const { isMusl } = await import("./index");
      expect(isMusl()).toBe(false);
    });

    it("returns true on linux when glibcVersionRuntime is absent (musl)", async () => {
      mockPlatform.mockReturnValue("linux");
      mockProcessReport({});

      const { isMusl } = await import("./index");
      expect(isMusl()).toBe(true);
    });

    it("returns false when process.report is unavailable", async () => {
      mockPlatform.mockReturnValue("linux");
      vi.spyOn(process, "report", "get").mockReturnValue(
        undefined as unknown as NodeJS.ProcessReport
      );

      const { isMusl } = await import("./index");
      expect(isMusl()).toBe(false);
    });

    it("returns false when reading the report throws", async () => {
      mockPlatform.mockReturnValue("linux");
      vi.spyOn(process, "report", "get").mockReturnValue({
        getReport: () => {
          throw new Error("boom");
        },
      } as unknown as NodeJS.ProcessReport);

      const { isMusl } = await import("./index");
      expect(isMusl()).toBe(false);
    });
  });

  describe("getPlatformTarget", () => {
    it.each([
      ["win32", "x64", "x86_64-pc-windows-msvc"],
      ["win32", "arm64", "x86_64-pc-windows-msvc"],
      ["darwin", "x64", "x86_64-apple-darwin"],
      ["darwin", "arm64", "aarch64-apple-darwin"],
      ["linux", "x64", "x86_64-unknown-linux-gnu"],
      ["linux", "arm64", "aarch64-unknown-linux-gnu"],
    ])("maps %s/%s to %s", async (plat, architecture, expected) => {
      mockPlatform.mockReturnValue(plat as NodeJS.Platform);
      mockArch.mockReturnValue(architecture as NodeJS.Architecture);

      const { getPlatformTarget } = await import("./index");
      expect(getPlatformTarget()).toBe(expected);
    });

    it.each([
      ["x64", "x86_64-unknown-linux-musl"],
      ["arm64", "aarch64-unknown-linux-musl"],
    ])("maps musl linux/%s to %s", async (architecture, expected) => {
      mockPlatform.mockReturnValue("linux");
      mockArch.mockReturnValue(architecture as NodeJS.Architecture);
      mockProcessReport({});

      const { getPlatformTarget } = await import("./index");
      expect(getPlatformTarget()).toBe(expected);
    });

    it("throws for unsupported platforms", async () => {
      mockPlatform.mockReturnValue("freebsd" as NodeJS.Platform);
      mockArch.mockReturnValue("x64");

      const { getPlatformTarget } = await import("./index");
      expect(() => getPlatformTarget()).toThrow("Unsupported platform");
    });

    it.each([
      ["darwin", "ia32"],
      ["darwin", "arm"],
      ["linux", "ia32"],
      ["linux", "arm"],
    ])(
      "throws for unsupported %s/%s architectures instead of falling back to x64",
      async (plat, architecture) => {
        mockPlatform.mockReturnValue(plat as NodeJS.Platform);
        mockArch.mockReturnValue(architecture as NodeJS.Architecture);

        const { getPlatformTarget } = await import("./index");
        expect(() => getPlatformTarget()).toThrow("Unsupported platform");
      }
    );
  });

  describe("getBinaryName", () => {
    it("returns lintp.exe on windows", async () => {
      mockPlatform.mockReturnValue("win32");

      const { getBinaryName } = await import("./index");
      expect(getBinaryName()).toBe("lintp.exe");
    });

    it("returns lintp elsewhere", async () => {
      mockPlatform.mockReturnValue("darwin");

      const { getBinaryName } = await import("./index");
      expect(getBinaryName()).toBe("lintp");
    });
  });

  describe("getAssetName", () => {
    it("matches the release asset naming for unix targets", async () => {
      mockPlatform.mockReturnValue("darwin");
      mockArch.mockReturnValue("arm64");

      const { getAssetName } = await import("./index");
      expect(getAssetName()).toBe("lintp-aarch64-apple-darwin");
    });

    it("appends .exe for windows", async () => {
      mockPlatform.mockReturnValue("win32");
      mockArch.mockReturnValue("x64");

      const { getAssetName } = await import("./index");
      expect(getAssetName()).toBe("lintp-x86_64-pc-windows-msvc.exe");
    });

    it.each([
      ["x64", "lintp-x86_64-unknown-linux-musl"],
      ["arm64", "lintp-aarch64-unknown-linux-musl"],
    ])(
      "matches the musl release asset naming for linux/%s",
      async (architecture, expected) => {
        mockPlatform.mockReturnValue("linux");
        mockArch.mockReturnValue(architecture as NodeJS.Architecture);
        mockProcessReport({});

        const { getAssetName } = await import("./index");
        expect(getAssetName()).toBe(expected);
      }
    );
  });

  describe("getPlatformPackageName", () => {
    it.each([
      ["darwin", "arm64", "lintp-darwin-arm64"],
      ["darwin", "x64", "lintp-darwin-x64"],
      ["linux", "arm64", "lintp-linux-arm64"],
      ["linux", "x64", "lintp-linux-x64"],
      ["win32", "x64", "lintp-win32-x64"],
      // Windows on ARM falls back to the emulated x64 build
      ["win32", "arm64", "lintp-win32-x64"],
    ])("maps %s/%s to %s", async (plat, architecture, expected) => {
      mockPlatform.mockReturnValue(plat as NodeJS.Platform);
      mockArch.mockReturnValue(architecture as NodeJS.Architecture);

      const { getPlatformPackageName } = await import("./index");
      expect(getPlatformPackageName()).toBe(expected);
    });

    it("returns null for unsupported platforms", async () => {
      mockPlatform.mockReturnValue("freebsd" as NodeJS.Platform);
      mockArch.mockReturnValue("x64");

      const { getPlatformPackageName } = await import("./index");
      expect(getPlatformPackageName()).toBeNull();
    });

    it.each([
      ["x64", "lintp-linux-x64-musl"],
      ["arm64", "lintp-linux-arm64-musl"],
    ])("maps musl linux/%s to %s", async (architecture, expected) => {
      mockPlatform.mockReturnValue("linux");
      mockArch.mockReturnValue(architecture as NodeJS.Architecture);
      mockProcessReport({});

      const { getPlatformPackageName } = await import("./index");
      expect(getPlatformPackageName()).toBe(expected);
    });
  });

  describe("resolveInstalledBinary", () => {
    it("returns null when the platform package is not installed", async () => {
      mockPlatform.mockReturnValue("darwin");
      mockArch.mockReturnValue("arm64");

      const { resolveInstalledBinary } = await import("./index");
      expect(resolveInstalledBinary()).toBeNull();
    });

    it("returns null for unsupported platforms", async () => {
      mockPlatform.mockReturnValue("freebsd" as NodeJS.Platform);
      mockArch.mockReturnValue("x64");

      const { resolveInstalledBinary } = await import("./index");
      expect(resolveInstalledBinary()).toBeNull();
    });
  });

  describe("getPackageVersion", () => {
    it("finds the project package.json version", async () => {
      const { getPackageVersion } = await import("./index");
      expect(getPackageVersion()).toBe(realPackageVersion());
    });
  });

  describe("getBinaryPath", () => {
    it("builds a per-version path under the home directory", async () => {
      mockPlatform.mockReturnValue("linux");
      mockArch.mockReturnValue("x64");
      mockHomedir.mockReturnValue("/home/test");

      const { getBinaryPath } = await import("./index");
      expect(getBinaryPath()).toBe(
        path.join("/home/test", ".lintp", "bin", realPackageVersion(), "lintp")
      );
    });
  });

  describe("verifyChecksum", () => {
    const data = Buffer.from("lintp binary contents");
    const digest = createHash("sha256").update(data).digest("hex");

    it("accepts a matching digest", async () => {
      const { verifyChecksum } = await import("./index");
      expect(verifyChecksum(data, digest)).toBe(true);
    });

    it("accepts shasum-style 'digest  filename' output", async () => {
      const { verifyChecksum } = await import("./index");
      expect(
        verifyChecksum(data, `${digest}  lintp-x86_64-apple-darwin\n`)
      ).toBe(true);
    });

    it("rejects a mismatched digest", async () => {
      const { verifyChecksum } = await import("./index");
      expect(verifyChecksum(data, "0".repeat(64))).toBe(false);
    });
  });

  describe("spawnBinary", () => {
    it("spawns the child process with correct arguments", async () => {
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
    it("reports a missing binary and exits on ENOENT", async () => {
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
        "Error: Binary not found at: /path/to/binary"
      );
      expect(processExit).toHaveBeenCalledWith(1);

      consoleError.mockRestore();
      processExit.mockRestore();
    });

    it("rethrows other errors", async () => {
      const { handleBinaryError } = await import("./index");
      const error = new Error("Some other error");

      expect(() =>
        handleBinaryError(error as NodeJS.ErrnoException, "/path/to/binary")
      ).toThrow("Some other error");
    });

    it("reports a musl/glibc mismatch when the binary exists but ENOENT is thrown on Linux", async () => {
      const consoleError = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});
      const processExit = vi.spyOn(process, "exit").mockImplementation(() => {
        throw new Error("process.exit called");
      });
      const processPlatform = vi
        .spyOn(process, "platform", "get")
        .mockReturnValue("linux");
      mockExistsSync.mockReturnValue(true);

      const { handleBinaryError } = await import("./index");
      const error = new Error("File not found") as NodeJS.ErrnoException;
      error.code = "ENOENT";

      expect(() => handleBinaryError(error, "/path/to/binary")).toThrow(
        "process.exit called"
      );

      expect(consoleError).toHaveBeenCalledWith(
        "Error: Binary exists at /path/to/binary but could not be executed."
      );
      expect(consoleError).toHaveBeenCalledWith(
        expect.stringContaining("musl-based distro")
      );
      expect(consoleError).not.toHaveBeenCalledWith(
        expect.stringContaining("Binary not found at:")
      );
      expect(processExit).toHaveBeenCalledWith(1);

      consoleError.mockRestore();
      processExit.mockRestore();
      processPlatform.mockRestore();
    });

    it("keeps the generic not-found message when the binary is truly missing on Linux", async () => {
      const consoleError = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});
      const processExit = vi.spyOn(process, "exit").mockImplementation(() => {
        throw new Error("process.exit called");
      });
      const processPlatform = vi
        .spyOn(process, "platform", "get")
        .mockReturnValue("linux");
      mockExistsSync.mockReturnValue(false);

      const { handleBinaryError } = await import("./index");
      const error = new Error("File not found") as NodeJS.ErrnoException;
      error.code = "ENOENT";

      expect(() => handleBinaryError(error, "/path/to/binary")).toThrow(
        "process.exit called"
      );

      expect(consoleError).toHaveBeenCalledWith(
        "Error: Binary not found at: /path/to/binary"
      );
      expect(consoleError).not.toHaveBeenCalledWith(
        expect.stringContaining("musl-based distro")
      );
      expect(processExit).toHaveBeenCalledWith(1);

      consoleError.mockRestore();
      processExit.mockRestore();
      processPlatform.mockRestore();
    });
  });

  describe("handleBinaryExit", () => {
    it("exits with the child's exit code", async () => {
      const processExit = vi
        .spyOn(process, "exit")
        .mockImplementation(() => undefined as never);

      const { handleBinaryExit } = await import("./index");
      handleBinaryExit(42, null);

      expect(processExit).toHaveBeenCalledWith(42);
      processExit.mockRestore();
    });

    it("exits with 0 when the code is null", async () => {
      const processExit = vi
        .spyOn(process, "exit")
        .mockImplementation(() => undefined as never);

      const { handleBinaryExit } = await import("./index");
      handleBinaryExit(null, null);

      expect(processExit).toHaveBeenCalledWith(0);
      processExit.mockRestore();
    });

    it("re-raises the child's signal", async () => {
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
    it("spawns the cached binary without downloading", async () => {
      mockPlatform.mockReturnValue("darwin");
      mockArch.mockReturnValue("x64");
      mockHomedir.mockReturnValue("/home/test");
      process.argv = ["node", "index.js", "--help", "--version"];

      // package.json discovery stays real; the cached binary "exists"
      mockExistsSync.mockImplementation((p) =>
        String(p).endsWith("package.json") ? actualFs.existsSync(p) : true
      );

      const mockChild = {
        on: vi.fn(),
      };
      mockSpawn.mockReturnValue(mockChild as unknown as ChildProcess);

      const { main } = await import("./index");
      await main();

      const expectedBinary = path.join(
        "/home/test",
        ".lintp",
        "bin",
        realPackageVersion(),
        "lintp"
      );
      expect(mockSpawn).toHaveBeenCalledWith(
        expectedBinary,
        ["--help", "--version"],
        { stdio: "inherit", env: process.env }
      );
      expect(mockChild.on).toHaveBeenCalledWith("error", expect.any(Function));
      expect(mockChild.on).toHaveBeenCalledWith("exit", expect.any(Function));
    });

    it("prints only the error message (no stack trace) for an unsupported platform", async () => {
      mockPlatform.mockReturnValue("freebsd" as NodeJS.Platform);
      mockArch.mockReturnValue("x64");
      mockHomedir.mockReturnValue("/home/test");
      process.argv = ["node", "index.js"];

      // No cached binary and no installed platform package, so ensureBinary
      // falls through to the download path, which throws while building the
      // asset name (getPlatformTarget) for this unsupported platform.
      mockExistsSync.mockImplementation((p) =>
        String(p).endsWith("package.json") ? actualFs.existsSync(p) : false
      );

      const consoleError = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});
      const processExit = vi.spyOn(process, "exit").mockImplementation(() => {
        throw new Error("process.exit called");
      });

      const { main } = await import("./index");

      await expect(main()).rejects.toThrow("process.exit called");

      expect(consoleError).toHaveBeenCalledWith(
        "Failed to start lintp:",
        "Unsupported platform: freebsd x64"
      );
      expect(processExit).toHaveBeenCalledWith(1);

      consoleError.mockRestore();
      processExit.mockRestore();
    });

    it("wires error and exit handling to the child process", async () => {
      mockPlatform.mockReturnValue("linux");
      mockArch.mockReturnValue("arm64");
      mockHomedir.mockReturnValue("/home/test");
      process.argv = ["node", "index.js"];

      mockExistsSync.mockImplementation((p) =>
        String(p).endsWith("package.json") ? actualFs.existsSync(p) : true
      );

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
      await main();

      const errorCallback = mockChild.on.mock.calls.find(
        (call) => call[0] === "error"
      )?.[1];
      const exitCallback = mockChild.on.mock.calls.find(
        (call) => call[0] === "exit"
      )?.[1];

      const error = new Error("File not found") as NodeJS.ErrnoException;
      error.code = "ENOENT";

      expect(() => errorCallback(error)).toThrow("process.exit called");
      expect(consoleError).toHaveBeenCalledWith(
        expect.stringContaining("Error: Binary not found at:")
      );

      processExit.mockImplementation(() => undefined as never);

      exitCallback(0, null);
      expect(processExit).toHaveBeenCalledWith(0);

      consoleError.mockRestore();
      processExit.mockRestore();
    });
  });

  describe("installSignalForwarding", () => {
    function spyOnProcessOn() {
      // Capture the handlers without actually registering them on the real
      // process object: a real registration would leak SIGINT/SIGTERM
      // listeners across tests (and risk interacting with the test
      // runner's own signal handling).
      return vi.spyOn(process, "on").mockImplementation(() => process);
    }

    function findHandler(
      onSpy: ReturnType<typeof spyOnProcessOn>,
      signal: "SIGINT" | "SIGTERM"
    ): (signal: NodeJS.Signals) => void {
      const call = onSpy.mock.calls.find(([event]) => event === signal);
      expect(call).toBeDefined();
      return call?.[1] as (signal: NodeJS.Signals) => void;
    }

    it("forwards SIGINT to a still-running child instead of exiting", async () => {
      const onSpy = spyOnProcessOn();
      const processExit = vi.spyOn(process, "exit").mockImplementation(() => {
        throw new Error("process.exit called");
      });

      const mockChild = {
        exitCode: null,
        signalCode: null,
        kill: vi.fn(),
      } as unknown as ChildProcess;

      const { installSignalForwarding } = await import("./index");
      installSignalForwarding(mockChild);

      const sigint = findHandler(onSpy, "SIGINT");
      expect(() => sigint("SIGINT")).not.toThrow();

      expect(mockChild.kill).toHaveBeenCalledWith("SIGINT");
      expect(processExit).not.toHaveBeenCalled();

      onSpy.mockRestore();
      processExit.mockRestore();
    });

    it("forwards SIGTERM to a still-running child instead of exiting", async () => {
      const onSpy = spyOnProcessOn();
      const processExit = vi.spyOn(process, "exit").mockImplementation(() => {
        throw new Error("process.exit called");
      });

      const mockChild = {
        exitCode: null,
        signalCode: null,
        kill: vi.fn(),
      } as unknown as ChildProcess;

      const { installSignalForwarding } = await import("./index");
      installSignalForwarding(mockChild);

      const sigterm = findHandler(onSpy, "SIGTERM");
      expect(() => sigterm("SIGTERM")).not.toThrow();

      expect(mockChild.kill).toHaveBeenCalledWith("SIGTERM");
      expect(processExit).not.toHaveBeenCalled();

      onSpy.mockRestore();
      processExit.mockRestore();
    });

    it("does nothing once the child has already exited", async () => {
      const onSpy = spyOnProcessOn();

      const mockChild = {
        exitCode: 0,
        signalCode: null,
        kill: vi.fn(),
      } as unknown as ChildProcess;

      const { installSignalForwarding } = await import("./index");
      installSignalForwarding(mockChild);

      const sigint = findHandler(onSpy, "SIGINT");
      sigint("SIGINT");

      expect(mockChild.kill).not.toHaveBeenCalled();

      onSpy.mockRestore();
    });
  });

  describe("main + signal handling integration", () => {
    it("does not die on SIGINT before the child exits, and still forwards the child's exit code", async () => {
      mockPlatform.mockReturnValue("linux");
      mockArch.mockReturnValue("x64");
      mockHomedir.mockReturnValue("/home/test");
      process.argv = ["node", "index.js"];

      mockExistsSync.mockImplementation((p) =>
        String(p).endsWith("package.json") ? actualFs.existsSync(p) : true
      );

      const mockChild = {
        exitCode: null as number | null,
        signalCode: null as NodeJS.Signals | null,
        kill: vi.fn(),
        on: vi.fn(),
      };
      mockSpawn.mockReturnValue(mockChild as unknown as ChildProcess);

      const onSpy = vi.spyOn(process, "on").mockImplementation(() => process);
      const processExit = vi
        .spyOn(process, "exit")
        .mockImplementation(() => undefined as never);

      const { main } = await import("./index");
      await main();

      const sigintHandler = onSpy.mock.calls.find(
        (call) => call[0] === "SIGINT"
      )?.[1] as (signal: NodeJS.Signals) => void;
      expect(sigintHandler).toBeDefined();

      // Ctrl+C arrives while the child is still running: the wrapper must
      // not exit itself; it should only forward the signal to the child.
      sigintHandler("SIGINT");
      expect(mockChild.kill).toHaveBeenCalledWith("SIGINT");
      expect(processExit).not.toHaveBeenCalled();

      // The child (having received the forwarded signal) exits with its own
      // code; the wrapper's "exit" listener (handleBinaryExit) must still
      // forward that code, undisturbed by the earlier SIGINT.
      const exitCallback = mockChild.on.mock.calls.find(
        (call) => call[0] === "exit"
      )?.[1];
      mockChild.exitCode = 130;
      exitCallback(130, null);

      expect(processExit).toHaveBeenCalledWith(130);

      onSpy.mockRestore();
      processExit.mockRestore();
    });
  });

  describe("module exports", () => {
    it("exposes the public API", async () => {
      const module = await import("./index");

      expect(module.main).toBeDefined();
      expect(module.isMusl).toBeDefined();
      expect(module.getPlatformTarget).toBeDefined();
      expect(module.getBinaryName).toBeDefined();
      expect(module.getAssetName).toBeDefined();
      expect(module.getBinaryPath).toBeDefined();
      expect(module.verifyChecksum).toBeDefined();
      expect(module.spawnBinary).toBeDefined();
      expect(module.handleBinaryError).toBeDefined();
      expect(module.handleBinaryExit).toBeDefined();
      expect(module.installSignalForwarding).toBeDefined();
    });
  });
});
