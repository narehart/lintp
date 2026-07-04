import { afterAll, describe, expect, it } from "vitest";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "fs";
import { tmpdir } from "os";
import path from "path";
import { addHeadingIds, buildDocs, rewriteLinks, slugify } from "./build-docs";

describe("build-docs", () => {
  describe("slugify", () => {
    it("matches GitHub-style anchors", () => {
      expect(slugify("DSL at a Glance")).toBe("dsl-at-a-glance");
      expect(slugify("From npm (recommended)")).toBe("from-npm-recommended");
      expect(slugify("matches(string, pattern)")).toBe("matchesstring-pattern");
    });
  });

  describe("addHeadingIds", () => {
    it("adds ids to h1-h4", () => {
      expect(addHeadingIds("<h2>Quick Start</h2>")).toBe(
        '<h2 id="quick-start">Quick Start</h2>'
      );
    });
  });

  describe("rewriteLinks", () => {
    it("rewrites .md links to .html and strips docs/ when asked", () => {
      const md = "see [ref](docs/DSL_REFERENCE.md) and [ex](EXAMPLES.md#react)";
      expect(rewriteLinks(md, true)).toBe(
        "see [ref](DSL_REFERENCE.html) and [ex](EXAMPLES.html#react)"
      );
    });

    it("leaves external links alone", () => {
      const md = "[repo](https://github.com/narehart/lintp)";
      expect(rewriteLinks(md, true)).toBe(md);
    });
  });

  describe("buildDocs", () => {
    const outDir = mkdtempSync(path.join(tmpdir(), "lintp-docs-"));

    afterAll(() => {
      rmSync(outDir, { recursive: true, force: true });
    });

    it("builds all pages and copies static assets", () => {
      const written = buildDocs(outDir);

      for (const file of [
        "README.html",
        "DSL_REFERENCE.html",
        "COMMON_PATTERNS.html",
        "EXAMPLES.html",
        "index.html",
        "demo.gif",
      ]) {
        expect(existsSync(path.join(outDir, file)), file).toBe(true);
      }
      expect(existsSync(path.join(outDir, "assets", "docs.css"))).toBe(true);
      expect(written.length).toBeGreaterThanOrEqual(7);
    });

    it("produces working ToC anchors and rewritten links in README", () => {
      const html = readFileSync(path.join(outDir, "README.html"), "utf8");
      expect(html).toContain('id="installation"');
      expect(html).toContain('href="DSL_REFERENCE.html"');
      expect(html).not.toContain("docs/DSL_REFERENCE.md");
      expect(html).toContain('class="prose"');
      expect(html).toContain("assets/docs.css");
    });
  });
});
