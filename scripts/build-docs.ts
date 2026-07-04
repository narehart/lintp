#!/usr/bin/env node

/**
 * Builds the docs site into _site/ — the exact artifact GitHub Pages
 * deploys. Run locally to preview and sign off on changes:
 *
 *   npm run docs:build && npm run docs:preview
 *
 * Two kinds of pages, one design system (docs/assets/docs.css):
 * - hand-authored pages (docs/index.html, docs/pages/*.html) are copied
 *   through unchanged; they use the design-system classes directly
 * - markdown pages are rendered with a transform that emits the SAME
 *   design-system classes, so generated and hand-authored pages are
 *   built from identical components
 */

import fs from "fs";
import path from "path";
import { marked } from "marked";

const ROOT = path.join(__dirname, "..");

interface MdPage {
  /** Source markdown, relative to repo root */
  src: string;
  /** Output filename in the site root */
  out: string;
  /** Tree name shown in the crumb and continue/ tree */
  name: string;
  /** One-line annotation for continue/ trees on other pages */
  note: string;
}

const MD_PAGES: MdPage[] = [
  {
    src: "docs/DSL_REFERENCE.md",
    out: "DSL_REFERENCE.html",
    name: "dsl-reference",
    note: "# every operator, variable, function",
  },
  {
    src: "docs/COMMON_PATTERNS.md",
    out: "COMMON_PATTERNS.html",
    name: "common-patterns",
    note: "# reusable naming & structure rules",
  },
  {
    src: "docs/EXAMPLES.md",
    out: "EXAMPLES.html",
    name: "examples",
    note: "# React, Node API, monorepo configs",
  },
];

const GETTING_STARTED = {
  out: "getting-started.html",
  name: "getting-started",
  note: "# install, CLI, config structure",
};

const CUSTOM_PAGES_DIR = "docs/pages";
const STATIC_FILES = ["docs/index.html", "docs/demo.gif"];
const STATIC_DIRS = ["docs/assets"];

/** GitHub-compatible heading slug, so intra-page anchors keep working */
export function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/<[^>]+>/g, "")
    .replace(/&[a-z]+;/g, "")
    .replace(/[^\w\- ]/g, "")
    .trim()
    .replace(/ +/g, "-");
}

/** Rewrite repo-relative markdown links to their built pages */
export function rewriteLinks(md: string): string {
  return md
    .replace(/\((\.\.\/)?README\.md(#[\w-]+)?\)/g, "(getting-started.html$2)")
    .replace(/\((\.\/)?([\w-]+)\.md(#[\w-]+)?\)/g, "($2.html$3)");
}

const ESCAPES: Record<string, string> = {
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
};

function escapeHtml(s: string): string {
  return s.replace(/[&<>]/g, (c) => ESCAPES[c]);
}

/** Colorize a code-panel line with the terminal accent classes */
export function colorizeCodeLine(escaped: string): string {
  if (/^\s*✓/.test(escaped)) return `<span class="pass">${escaped}</span>`;
  if (/^\s*✗/.test(escaped)) return `<span class="fail">${escaped}</span>`;
  // full-line comment
  if (/^\s*#(\s|$)/.test(escaped)) return `<span class="cmt">${escaped}</span>`;
  // trailing comment after whitespace
  return escaped.replace(/(\s)(#\s[^#]*)$/, '$1<span class="cmt">$2</span>');
}

/** Render a fenced code block as the design system's terminal panel */
function codePanel(code: string, lang: string | undefined): string {
  const body = code
    .replace(/\n$/, "")
    .split("\n")
    .map((line) => colorizeCodeLine(escapeHtml(line)))
    .join("\n");
  const bar = lang && lang.trim() ? lang.trim() : "code";
  return `<div class="cb"><div class="cbar">${escapeHtml(
    bar
  )}</div><pre class="cbody">${body}</pre></div>`;
}

interface Section {
  slug: string;
  name: string;
}

/**
 * Transform marked's plain output into design-system components.
 * Fenced code is pulled out before parsing so the paragraph and
 * inline-code transforms can't touch panel contents.
 */
export function toComponents(md: string): {
  html: string;
  sections: Section[];
} {
  const panels: string[] = [];
  const withPlaceholders = md.replace(
    /```([^\n]*)\n([\s\S]*?)```/g,
    (_, lang, code) => {
      panels.push(codePanel(code, lang));
      return `\n@@CODE_PANEL_${panels.length - 1}@@\n`;
    }
  );

  let html = marked.parse(withPlaceholders, {
    async: false,
    gfm: true,
  }) as string;

  const sections: Section[] = [];

  html = html
    // h1 is dropped: the crumb carries the page title
    .replace(/<h1>.*?<\/h1>\n?/, "")
    // h2 → lowercase-slug section heading with trailing slash
    .replace(/<h2>(.*?)<\/h2>/g, (_, inner) => {
      const slug = slugify(inner);
      sections.push({ slug, name: slug });
      return `<h2 class="h2" id="${slug}">${slug}<span class="slash">/</span></h2>`;
    })
    .replace(
      /<h[34]>(.*?)<\/h[34]>/g,
      (_, inner) => `<div class="h3" id="${slugify(inner)}">${inner}</div>`
    )
    .replace(/<p>/g, '<p class="p">')
    .replace(/<(ul|ol)>/g, '<$1 class="ul">')
    .replace(/<table>/g, '<table class="tbl">')
    .replace(/<code>/g, '<code class="ic">');

  // Restore code panels (marked wraps lone placeholders in <p>)
  html = html.replace(
    /<p class="p">@@CODE_PANEL_(\d+)@@<\/p>|@@CODE_PANEL_(\d+)@@/g,
    (_, a, b) => panels[Number(a ?? b)]
  );

  return { html, sections };
}

/** In-page table of contents as a directory tree */
export function tocTree(pageName: string, sections: Section[]): string {
  if (sections.length < 2) return "";
  const rows = sections
    .map((s, i) => {
      const guide = i === sections.length - 1 ? "└── " : "├── ";
      return `<div><span class="guide">${guide}</span><a class="tlink" href="#${s.slug}">${s.name}</a></div>`;
    })
    .join("\n");
  return `<div class="tree">\n<div><span class="dim">${pageName}/</span></div>\n${rows}\n</div>`;
}

/** continue/ tree linking to the other doc pages */
function continueTree(currentName: string): string {
  const others = [GETTING_STARTED, ...MD_PAGES].filter(
    (p) => p.name !== currentName
  );
  const width = Math.max(...others.map((p) => p.name.length)) + 1;
  const rows = others
    .map((p, i) => {
      const guide = i === others.length - 1 ? "└── " : "├── ";
      const pad = " ".repeat(width - p.name.length);
      return `<div><span class="guide">${guide}</span><a class="tlink" href="${p.out}">${p.name}</a><span class="note">${pad}${p.note}</span></div>`;
    })
    .join("\n");
  return `<div class="tree" style="margin-top: 56px">\n<div><span class="dim">continue/</span></div>\n${rows}\n</div>`;
}

function template(name: string, intro: string, bodyHtml: string): string {
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
      <div class="crumb">
        <a href="index.html">lintp</a><span class="dim"> / ${name}</span>
      </div>
${intro}
${bodyHtml}
${continueTree(name)}
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

  for (const page of MD_PAGES) {
    const md = fs.readFileSync(path.join(ROOT, page.src), "utf8");
    const { html, sections } = toComponents(rewriteLinks(md));

    // The first paragraph doubles as the intro under the crumb
    const introMatch = html.match(/<p class="p">[\s\S]*?<\/p>/);
    const intro = introMatch
      ? introMatch[0].replace('class="p"', 'class="sub"')
      : "";
    const body = introMatch ? html.replace(introMatch[0], "") : html;

    fs.writeFileSync(
      path.join(outDir, page.out),
      template(page.name, intro, `${tocTree(page.name, sections)}\n${body}`)
    );
    written.push(page.out);
  }

  const customDir = path.join(ROOT, CUSTOM_PAGES_DIR);
  for (const file of fs.readdirSync(customDir)) {
    if (!file.endsWith(".html")) continue;
    fs.copyFileSync(path.join(customDir, file), path.join(outDir, file));
    written.push(file);
  }

  for (const file of STATIC_FILES) {
    fs.copyFileSync(
      path.join(ROOT, file),
      path.join(outDir, path.basename(file))
    );
    written.push(path.basename(file));
  }

  for (const dir of STATIC_DIRS) {
    fs.cpSync(path.join(ROOT, dir), path.join(outDir, path.basename(dir)), {
      recursive: true,
    });
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
