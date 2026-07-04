#!/usr/bin/env node

/**
 * Builds the docs site into _site/ — the exact artifact GitHub Pages
 * deploys. Run locally to preview and sign off on changes:
 *
 *   npm run docs:build && npm run docs:preview
 *
 * Markdown pages are rendered into the shared design-system template
 * (docs/assets/docs.css); the hand-written homepage and static assets are
 * copied through unchanged.
 */

import fs from "fs";
import path from "path";
import { marked } from "marked";

const ROOT = path.join(__dirname, "..");

interface Page {
  /** Source markdown, relative to repo root */
  src: string;
  /** Output filename in the site root */
  out: string;
  /** Tree name shown in the masthead breadcrumb */
  name: string;
  /** Rewrite links that point into docs/ (for pages that live outside it) */
  stripDocsPrefix?: boolean;
}

const PAGES: Page[] = [
  {
    src: "README.md",
    out: "README.html",
    name: "getting-started",
    stripDocsPrefix: true,
  },
  {
    src: "docs/DSL_REFERENCE.md",
    out: "DSL_REFERENCE.html",
    name: "dsl-reference",
  },
  {
    src: "docs/COMMON_PATTERNS.md",
    out: "COMMON_PATTERNS.html",
    name: "common-patterns",
  },
  { src: "docs/EXAMPLES.md", out: "EXAMPLES.html", name: "examples" },
];

const STATIC_FILES = ["docs/index.html", "docs/demo.gif"];
const STATIC_DIRS = ["docs/assets"];

/** GitHub-compatible heading slug, so intra-page ToC anchors keep working */
export function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/<[^>]+>/g, "")
    .replace(/&[a-z]+;/g, "")
    .replace(/[^\w\- ]/g, "")
    .trim()
    .replace(/ +/g, "-");
}

/** Add ids to rendered headings (marked emits none by default) */
export function addHeadingIds(html: string): string {
  return html.replace(
    /<h([1-4])>(.*?)<\/h\1>/g,
    (_, level, inner) =>
      `<h${level} id="${slugify(inner)}">${inner}</h${level}>`
  );
}

/** Rewrite repo-relative markdown links to their built pages */
export function rewriteLinks(md: string, stripDocsPrefix: boolean): string {
  let out = md;
  if (stripDocsPrefix) {
    out = out.replace(/\(docs\//g, "(");
  }
  // (FILE.md) or (./FILE.md) → (FILE.html)
  out = out.replace(/\((\.\/)?([\w-]+)\.md(#[\w-]+)?\)/g, "($2.html$3)");
  return out;
}

function template(name: string, bodyHtml: string): string {
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>lintp — ${name}</title>
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
    <link
      href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;700;800&display=swap"
      rel="stylesheet"
    />
    <link rel="stylesheet" href="assets/docs.css" />
  </head>
  <body>
    <div class="page">
      <div class="masthead">
        <a href="index.html"><strong>lintp</strong></a><span class="dim"> — docs/${name}</span>
      </div>
      <div class="prose">
${bodyHtml}
      </div>
      <div class="foot">
        <span>MIT License</span>
        <a class="flink" href="https://github.com/narehart/lintp">github.com/narehart/lintp</a>
      </div>
    </div>
  </body>
</html>
`;
}

export function buildDocs(outDir: string): string[] {
  fs.rmSync(outDir, { recursive: true, force: true });
  fs.mkdirSync(outDir, { recursive: true });

  const written: string[] = [];

  for (const page of PAGES) {
    const md = fs.readFileSync(path.join(ROOT, page.src), "utf8");
    const rewritten = rewriteLinks(md, page.stripDocsPrefix ?? false);
    const body = addHeadingIds(
      marked.parse(rewritten, { async: false, gfm: true })
    );
    const outPath = path.join(outDir, page.out);
    fs.writeFileSync(outPath, template(page.name, body));
    written.push(page.out);
  }

  for (const file of STATIC_FILES) {
    const dest = path.join(outDir, path.basename(file));
    fs.copyFileSync(path.join(ROOT, file), dest);
    written.push(path.basename(file));
  }

  for (const dir of STATIC_DIRS) {
    const dest = path.join(outDir, path.basename(dir));
    fs.cpSync(path.join(ROOT, dir), dest, { recursive: true });
    written.push(`${path.basename(dir)}/`);
  }

  return written;
}

if (require.main === module) {
  const outIndex = process.argv.indexOf("--out");
  const outDir =
    outIndex >= 0
      ? path.resolve(process.argv[outIndex + 1])
      : path.join(ROOT, "_site");

  const written = buildDocs(outDir);
  console.log(`Built ${written.length} entries into ${outDir}:`);
  written.forEach((entry) => console.log(`  ${entry}`));
}
