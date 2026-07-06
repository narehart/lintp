import { afterAll, describe, expect, it } from "vitest";
import { existsSync, mkdtempSync, readdirSync, readFileSync, rmSync } from "fs";
import { tmpdir } from "os";
import path from "path";
import {
  buildDocs,
  colorizeCodeLine,
  firstParagraph,
  generateLlmsTxt,
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
        "see [gs](getting-started.html#quick-start) and [ex](examples.html#react)"
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

  describe("firstParagraph", () => {
    it("skips the heading, blank lines, and comments to find the intro", () => {
      const md = [
        "# Title",
        "",
        "<!-- site:sub A page intro. -->",
        "",
        "The real description.",
        "",
        "## Next section",
      ].join("\n");
      expect(firstParagraph(md)).toBe("The real description.");
    });

    it("returns an empty string when there is no prose", () => {
      expect(firstParagraph("# Title\n\n<!-- comment -->\n")).toBe("");
    });
  });

  describe("generateLlmsTxt", () => {
    it("derives an llms.txt from the page registry and README's own intro, per https://llmstxt.org", () => {
      const txt = generateLlmsTxt();

      expect(txt).toMatch(/^# lintp\n/);
      expect(txt).toContain(
        "> A powerful file system linter that validates directory structures"
      );
      expect(txt).toContain("## Docs");
      for (const [page, description] of [
        ["getting-started.html", "install, CLI, config structure"],
        ["dsl-reference.html", "every operator, variable, function"],
        ["common-patterns.html", "reusable naming & structure rules"],
        ["examples.html", "React, Node API, monorepo configs"],
      ]) {
        expect(txt).toContain(
          `(https://narehart.github.io/lintp/${page}): ${description}`
        );
      }
      expect(txt).toContain("## Optional");
      expect(txt).toContain("https://github.com/narehart/lintp");
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
        "dsl-reference.html",
        "common-patterns.html",
        "examples.html",
        "index.html",
        "llms.txt",
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
        "dsl-reference.html",
        "common-patterns.html",
        "examples.html",
      ]) {
        expect(html).toContain(`href="${link}"`);
      }
      expect(html).toContain("# install, CLI, config structure");
    });

    it("renders md pages with crumb, toc tree, and continue tree", () => {
      const html = readFileSync(
        path.join(outDir, "dsl-reference.html"),
        "utf8"
      );
      expect(html).toContain('class="crumb"');
      expect(html).toContain("docs.css");
      expect(html).toContain('href="#variables"');
      expect(html).toContain(">continue/</span>");
      expect(html).toContain('href="getting-started.html"');
      expect(html).not.toContain(".md)");
    });

    it("has no broken internal links on any page", () => {
      const built = new Set(
        readdirSync(outDir).filter((f) => f.endsWith(".html"))
      );
      for (const page of built) {
        const html = readFileSync(path.join(outDir, page), "utf8");
        for (const m of html.matchAll(/href="([^"#]+)(?:#[\w-]*)?"/g)) {
          const href = m[1];
          if (href.startsWith("http") || href.startsWith("assets/")) continue;
          expect(built.has(href), `${page} -> ${href}`).toBe(true);
        }
      }
    });

    it("uses only design-system classes for code blocks", () => {
      const html = readFileSync(path.join(outDir, "examples.html"), "utf8");
      expect(html).toContain('class="cb"');
      expect(html).toContain('class="cbody"');
      expect(html).not.toContain("<pre><code");
    });
  });
});
