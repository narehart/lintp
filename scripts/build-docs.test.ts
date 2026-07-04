import { afterAll, describe, expect, it } from "vitest";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "fs";
import { tmpdir } from "os";
import path from "path";
import {
  buildDocs,
  colorizeCodeLine,
  rewriteLinks,
  slugify,
  toComponents,
} from "./build-docs";

describe("build-docs", () => {
  describe("slugify", () => {
    it("matches GitHub-style anchors", () => {
      expect(slugify("Complex Expression Examples")).toBe(
        "complex-expression-examples"
      );
      expect(slugify("matches(string, pattern)")).toBe("matchesstring-pattern");
    });
  });

  describe("rewriteLinks", () => {
    it("maps README links to getting-started and .md to .html", () => {
      const md =
        "see [gs](../README.md#quick-start) and [ex](EXAMPLES.md#react)";
      expect(rewriteLinks(md)).toBe(
        "see [gs](getting-started.html#quick-start) and [ex](EXAMPLES.html#react)"
      );
    });

    it("leaves external links alone", () => {
      const md = "[repo](https://github.com/narehart/lintp)";
      expect(rewriteLinks(md)).toBe(md);
    });
  });

  describe("colorizeCodeLine", () => {
    it("marks pass, fail, and comment lines with terminal classes", () => {
      expect(colorizeCodeLine("✓ ./src/utils.js")).toContain('class="pass"');
      expect(colorizeCodeLine("✗ ./bad.js - .js")).toContain('class="fail"');
      expect(colorizeCodeLine("# a comment")).toContain('class="cmt"');
      expect(colorizeCodeLine("node_modules   # skip deps")).toContain(
        'class="cmt"'
      );
      expect(colorizeCodeLine("plain code")).toBe("plain code");
    });
  });

  describe("toComponents", () => {
    it("emits design-system classes for markdown constructs", () => {
      const md = [
        "# Title",
        "",
        "Intro with `inline`.",
        "",
        "## Quick Start",
        "",
        "- item one",
        "",
        "```yaml",
        "# comment",
        "key: value",
        "```",
      ].join("\n");

      const { html, sections } = toComponents(md);
      expect(html).not.toContain("<h1>");
      expect(html).toContain(
        '<h2 class="h2" id="quick-start">quick-start<span class="slash">/</span></h2>'
      );
      expect(html).toContain('<p class="p">');
      expect(html).toContain('<code class="ic">');
      expect(html).toContain('<ul class="ul">');
      expect(html).toContain('<div class="cbar">yaml</div>');
      expect(html).toContain('<span class="cmt"># comment</span>');
      expect(sections).toEqual([{ slug: "quick-start", name: "quick-start" }]);
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
        "getting-started.html",
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

    it("renders md pages with crumb, toc tree, and continue tree", () => {
      const html = readFileSync(
        path.join(outDir, "DSL_REFERENCE.html"),
        "utf8"
      );
      expect(html).toContain('class="crumb"');
      expect(html).toContain("docs.css");
      expect(html).toContain('href="#variables"');
      expect(html).toContain(">continue/</span>");
      expect(html).toContain('href="getting-started.html"');
      expect(html).not.toContain(".md)");
    });

    it("uses only design-system classes for code blocks", () => {
      const html = readFileSync(path.join(outDir, "EXAMPLES.html"), "utf8");
      expect(html).toContain('class="cb"');
      expect(html).toContain('class="cbody"');
      expect(html).not.toContain("<pre><code");
    });
  });
});
