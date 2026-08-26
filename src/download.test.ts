import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { EventEmitter } from "events";
import { createHash } from "crypto";

vi.mock("https");
vi.mock("os");
vi.mock("fs", async (importOriginal) => {
  const actual = await importOriginal<typeof import("fs")>();
  return {
    ...actual,
    existsSync: vi.fn(),
    mkdirSync: vi.fn(),
    writeFileSync: vi.fn(),
    chmodSync: vi.fn(),
  };
});

import https from "https";
import { chmodSync, existsSync, mkdirSync, writeFileSync } from "fs";
import { platform } from "os";

import { downloadBinary, verifyChecksum } from "./index";

/**
 * A stand-in for the response `https.get` yields, driven manually so a test
 * can decide the status, the body, and whether it errors part way through.
 */
class FakeResponse extends EventEmitter {
  statusCode: number;
  headers: Record<string, string>;
  resume = vi.fn();

  constructor(statusCode: number, headers: Record<string, string> = {}) {
    super();
    this.statusCode = statusCode;
    this.headers = headers;
  }
}

class FakeRequest extends EventEmitter {
  destroy = vi.fn((err?: Error) => this.emit("error", err));
  setTimeout = vi.fn();
}

describe("binary download", () => {
  const mockGet = vi.mocked(https.get);
  const mockExistsSync = vi.mocked(existsSync);
  const mockPlatform = vi.mocked(platform);

  /** One queued reply per https.get call, in order. */
  interface Reply {
    status: number;
    headers?: Record<string, string>;
    body?: string;
  }

  /**
   * The body is emitted only after `https.get`'s callback has run, because
   * that is when the real caller attaches its "data" and "end" listeners.
   */
  function respondWith(replies: Reply[]): void {
    let call = 0;
    mockGet.mockImplementation(((
      _url: string,
      cb: (r: FakeResponse) => void
    ) => {
      const request = new FakeRequest();
      const reply = replies[Math.min(call, replies.length - 1)];
      call += 1;

      const response = new FakeResponse(reply.status, reply.headers ?? {});
      queueMicrotask(() => {
        cb(response);
        if (reply.body !== undefined) {
          queueMicrotask(() => {
            response.emit("data", Buffer.from(reply.body as string));
            response.emit("end");
          });
        }
      });

      return request;
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    }) as any);
  }

  /** A 200 reply carrying `body`. */
  function ok(body: string): Reply {
    return { status: 200, body };
  }

  function sha256(data: string): string {
    return createHash("sha256").update(Buffer.from(data)).digest("hex");
  }

  beforeEach(() => {
    vi.clearAllMocks();
    mockPlatform.mockReturnValue("linux");
    mockExistsSync.mockReturnValue(true);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("verifyChecksum", () => {
    it("accepts a digest that matches the data", () => {
      expect(verifyChecksum(Buffer.from("binary"), sha256("binary"))).toBe(
        true
      );
    });

    it("accepts the two-column '<digest>  <filename>' asset format", () => {
      const line = `${sha256("binary")}  lintp-linux-x64`;
      expect(verifyChecksum(Buffer.from("binary"), line)).toBe(true);
    });

    it("is case-insensitive and tolerates surrounding whitespace", () => {
      const line = `\n  ${sha256("binary").toUpperCase()}  lintp\n`;
      expect(verifyChecksum(Buffer.from("binary"), line)).toBe(true);
    });

    it("rejects a digest that does not match", () => {
      expect(verifyChecksum(Buffer.from("binary"), sha256("tampered"))).toBe(
        false
      );
    });
  });

  describe("downloadBinary", () => {
    it("writes the binary when the checksum matches", async () => {
      respondWith([ok("payload"), ok(`${sha256("payload")}  lintp`)]);

      await downloadBinary("https://example.test/lintp", "/tmp/bin/lintp");

      expect(vi.mocked(writeFileSync)).toHaveBeenCalledWith(
        "/tmp/bin/lintp",
        Buffer.from("payload")
      );
    });

    /** The whole point of the fallback path: a tampered binary is not run. */
    it("refuses to write a binary whose checksum does not match", async () => {
      respondWith([ok("payload"), ok(sha256("something else"))]);

      await expect(
        downloadBinary("https://example.test/lintp", "/tmp/bin/lintp")
      ).rejects.toThrow(/Checksum verification failed/);

      expect(vi.mocked(writeFileSync)).not.toHaveBeenCalled();
    });

    it("fetches the digest from the binary's .sha256 sibling", async () => {
      respondWith([ok("payload"), ok(sha256("payload"))]);

      await downloadBinary("https://example.test/lintp", "/tmp/bin/lintp");

      const urls = mockGet.mock.calls.map((c) => c[0]);
      expect(urls).toEqual([
        "https://example.test/lintp",
        "https://example.test/lintp.sha256",
      ]);
    });

    it("creates the target directory when it is missing", async () => {
      mockExistsSync.mockReturnValue(false);
      respondWith([ok("payload"), ok(sha256("payload"))]);

      await downloadBinary("https://example.test/lintp", "/tmp/bin/lintp");

      expect(vi.mocked(mkdirSync)).toHaveBeenCalledWith("/tmp/bin", {
        recursive: true,
      });
    });

    it("marks the binary executable on unix", async () => {
      respondWith([ok("payload"), ok(sha256("payload"))]);

      await downloadBinary("https://example.test/lintp", "/tmp/bin/lintp");

      expect(vi.mocked(chmodSync)).toHaveBeenCalledWith(
        "/tmp/bin/lintp",
        0o755
      );
    });

    it("does not chmod on windows", async () => {
      mockPlatform.mockReturnValue("win32");
      respondWith([ok("payload"), ok(sha256("payload"))]);

      await downloadBinary("https://example.test/lintp", "/tmp/bin/lintp.exe");

      expect(vi.mocked(chmodSync)).not.toHaveBeenCalled();
    });

    it("follows a redirect to the final asset", async () => {
      respondWith([
        { status: 302, headers: { location: "https://cdn.test/lintp" } },
        ok("payload"),
        ok(sha256("payload")),
      ]);

      await downloadBinary("https://example.test/lintp", "/tmp/bin/lintp");

      expect(mockGet.mock.calls[1][0]).toBe("https://cdn.test/lintp");
    });

    it("gives up rather than following redirects forever", async () => {
      respondWith([
        { status: 302, headers: { location: "https://loop.test/lintp" } },
      ]);

      await expect(
        downloadBinary("https://example.test/lintp", "/tmp/bin/lintp")
      ).rejects.toThrow(/Too many redirects/);
    });

    it("treats a response with no status code as a failure", async () => {
      // statusCode is optional on the node type; a missing one must not be
      // read as a success.
      respondWith([{ status: undefined as unknown as number }]);

      await expect(
        downloadBinary("https://example.test/lintp", "/tmp/bin/lintp")
      ).rejects.toThrow(/HTTP 0/);
    });

    it("reports the status code when the download fails", async () => {
      respondWith([{ status: 404 }]);

      await expect(
        downloadBinary("https://example.test/lintp", "/tmp/bin/lintp")
      ).rejects.toThrow(/HTTP 404/);
    });

    it("propagates a transport error", async () => {
      mockGet.mockImplementation(((_url: string) => {
        const request = new FakeRequest();
        queueMicrotask(() => request.emit("error", new Error("ECONNRESET")));
        return request;
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
      }) as any);

      await expect(
        downloadBinary("https://example.test/lintp", "/tmp/bin/lintp")
      ).rejects.toThrow(/ECONNRESET/);
    });

    it("arms a request timeout so a stalled download cannot hang forever", async () => {
      const requests: FakeRequest[] = [];
      mockGet.mockImplementation(((
        _url: string,
        cb: (r: FakeResponse) => void
      ) => {
        const request = new FakeRequest();
        requests.push(request);
        queueMicrotask(() => {
          const response = new FakeResponse(200);
          cb(response);
          queueMicrotask(() => {
            response.emit("data", Buffer.from("payload"));
            response.emit("end");
          });
        });
        return request;
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
      }) as any);

      await downloadBinary(
        "https://example.test/lintp",
        "/tmp/bin/lintp"
      ).catch(() => undefined);

      expect(requests[0].setTimeout).toHaveBeenCalledWith(
        expect.any(Number),
        expect.any(Function)
      );
    });
  });
});
