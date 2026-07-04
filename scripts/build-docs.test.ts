import { afterAll, describe, expect, it } from "vitest";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "fs";
import { tmpdir } from "os";
import path from "path";
import {
  buildDocs,
  colorizeCodeLine,
  preprocess,
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

  describe("preprocess", () => {
    it("extracts site:sub, heading notes, and drops skipped sections", () => {
      const md = [
        "# Title",
        "",
        "Intro.",
        "",
        "<!-- site:sub A one-line intro. -->",
        "",
        "## Keep <!-- note: kept things -->",
        "",
        "content",
        "",
        "## Drop <!-- site:skip -->",
        "",
        "hidden",
        "",
        "## Also Keep",
        "",
        "more",
      ].join("\n");

      const { md: out, notes, sub } = preprocess(md);
      expect(sub).toBe("A one-line intro.");
      expect(notes.get("keep")).toBe("kept things");
      expect(out).toContain("## Keep");
      expect(out).not.toContain("hidden");
      expect(out).not.toContain("site:skip");
      expect(out).toContain("## Also Keep");
    });

    it("drops standalone image lines", () => {
      const { md } = preprocess("![demo](docs/demo.gif)\n\ntext");
      expect(md).not.toContain("demo.gif");
      expect(md).toContain("text");
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
      expect(
        toComponents('```bash title="shell — install"\nnpx lintp-cli\n```').html
      ).toContain('<div class="cbar">shell — install</div>');
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
      ]) {
        expect(existsSync(path.join(outDir, file)), file).toBe(true);
      }
      expect(existsSync(path.join(outDir, "assets", "docs.css"))).toBe(true);
      expect(existsSync(path.join(outDir, "assets", "demo.gif"))).toBe(true);
      expect(written.length).toBeGreaterThanOrEqual(6);
    });

    it("derives getting-started from the README with metadata applied", () => {
      const html = readFileSync(
        path.join(outDir, "getting-started.html"),
        "utf8"
      );
      expect(html).toContain("Install lintp, write your first lintp.yml");
      expect(html).toContain('<div class="cbar">shell — install via npm</div>');
      expect(html).toContain("# npm, from source");
      expect(html).not.toContain("contributing");
      expect(html).not.toContain("dsl-at-a-glance");
      expect(html).not.toContain("demo.gif");
      expect(html).not.toContain("site:");
    });

    it("derives the homepage nav tree from the page registry", () => {
      const html = readFileSync(path.join(outDir, "index.html"), "utf8");
      expect(html).not.toContain("<!-- nav-tree -->");
      for (const link of [
        "getting-started.html",
        "DSL_REFERENCE.html",
        "COMMON_PATTERNS.html",
        "EXAMPLES.html",
      ]) {
        expect(html).toContain(`href="${link}"`);
      }
      expect(html).toContain("# install, CLI, config structure");
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
