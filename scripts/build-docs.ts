#!/usr/bin/env node

/**
 * Builds the docs site into _site/ — the exact artifact GitHub Pages
 * deploys. Run locally to preview and sign off on changes:
 *
 *   npm run docs:build && npm run docs:preview
 *
 * Every doc page is derived from markdown (single source of truth: the
 * README and docs/*.md), rendered into the design-system components in
 * docs/assets/docs.css. Three markdown-native conventions carry
 * site-only presentation without duplicating content (all invisible on
 * GitHub/npm):
 *
 *   ```bash title="shell — install via npm"   fence title → panel bar
 *   ## Installation <!-- note: npm, from source -->   ToC annotation
 *   ## Contributing <!-- site:skip -->        omit section from the site
 *   <!-- site:sub One-line page intro. -->    intro under the crumb
 *
 * llms.txt (https://llmstxt.org) is generated into the same output
 * directory from the MD_PAGES registry and README's opening paragraph —
 * see generateLlmsTxt() — so it's never hand-maintained separately.
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
    src: "README.md",
    out: "getting-started.html",
    name: "getting-started",
    note: "# install, CLI, config structure",
  },
  {
    src: "docs/DSL_REFERENCE.md",
    out: "dsl-reference.html",
    name: "dsl-reference",
    note: "# every operator, variable, function",
  },
  {
    src: "docs/COMMON_PATTERNS.md",
    out: "common-patterns.html",
    name: "common-patterns",
    note: "# reusable naming & structure rules",
  },
  {
    src: "docs/EXAMPLES.md",
    out: "examples.html",
    name: "examples",
    note: "# React, Node API, monorepo configs",
  },
];

const HOMEPAGE = "docs/index.html";
const STATIC_DIRS = ["docs/assets"];
const REPO_URL = "https://github.com/narehart/lintp";
const SITE_URL = "https://narehart.github.io/lintp";

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

/** Rewrite repo-relative markdown links to their built (kebab-case) pages */
export function rewriteLinks(md: string): string {
  const outFor = (name: string): string => {
    const page = MD_PAGES.find(
      (p) => p.src.endsWith(`/${name}.md`) || p.src === `${name}.md`
    );
    return page ? page.out : `${name.toLowerCase().replace(/_/g, "-")}.html`;
  };
  return md.replace(
    /\((?:\.\.\/|\.\/|docs\/)?([\w-]+)\.md(#[\w-]+)?\)/g,
    (_, name, hash) => `(${outFor(name)}${hash ?? ""})`
  );
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
  // troubleshooting fix lines ("  → do this instead")
  if (/^\s*→/.test(escaped)) return `<span class="cmt">${escaped}</span>`;
  // trailing comment after whitespace
  return escaped.replace(/(\s)(#\s[^#]*)$/, '$1<span class="cmt">$2</span>');
}

/** Render a fenced code block as the design system's terminal panel */
function codePanel(code: string, info: string | undefined): string {
  const body = code
    .replace(/\n$/, "")
    .split("\n")
    .map((line) => colorizeCodeLine(escapeHtml(line)))
    .join("\n");
  // fence info: `lang` or `lang title="panel bar text"`
  const titleMatch = (info ?? "").match(/title="([^"]+)"/);
  const lang = (info ?? "").trim().split(/\s+/)[0];
  const bar = titleMatch ? titleMatch[1] : lang || "code";
  return `<div class="cb"><div class="cbar">${escapeHtml(
    bar
  )}</div><pre class="cbody">${body}</pre></div>`;
}

interface Section {
  slug: string;
  name: string;
  note?: string;
}

interface Preprocessed {
  md: string;
  notes: Map<string, string>;
  sub?: string;
}

/**
 * Extract site-only metadata (invisible on GitHub) and strip content
 * that must not reach the site build.
 */
export function preprocess(md: string): Preprocessed {
  const notes = new Map<string, string>();
  let sub: string | undefined;

  // <!-- site:sub ... --> anywhere → page intro
  const subMatch = md.match(/<!--\s*site:sub\s+([\s\S]*?)-->/);
  if (subMatch) sub = subMatch[1].trim();
  let out = md.replace(/<!--\s*site:sub\s+[\s\S]*?-->\n?/g, "");

  // drop sections marked <!-- site:skip --> (until the next ## or EOF)
  out = out.replace(
    /^## [^\n]*<!--\s*site:skip\s*-->[\s\S]*?(?=^## |(?![\s\S]))/gm,
    ""
  );

  // ## Heading <!-- note: ... --> → ToC annotation
  out = out.replace(
    /^## (.*?)\s*<!--\s*note:\s*(.*?)\s*-->\s*$/gm,
    (_, title, note) => {
      notes.set(slugify(title), note);
      return `## ${title}`;
    }
  );

  // images stay in the README (the site homepage carries the demo)
  out = out.replace(/^!\[[^\]]*\]\([^)]*\)\s*$/gm, "");

  // badge lines (linked images) are README-only chrome
  out = out.replace(/^(?:\[!\[[^\]]*\]\([^)]*\)\]\([^)]*\)\s*)+$/gm, "");

  return { md: out, notes, sub };
}

/**
 * Transform marked's plain output into design-system components.
 * Fenced code is pulled out before parsing so the paragraph and
 * inline-code transforms can't touch panel contents.
 */
export function toComponents(
  md: string,
  notes: Map<string, string> = new Map()
): {
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
      sections.push({ slug, name: slug, note: notes.get(slug) });
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
  const width = Math.max(...sections.map((s) => s.name.length)) + 1;
  const rows = sections
    .map((s, i) => {
      const guide = i === sections.length - 1 ? "└── " : "├── ";
      const note = s.note
        ? `<span class="note">${" ".repeat(width - s.name.length)}# ${
            s.note
          }</span>`
        : "";
      return `<div><span class="guide">${guide}</span><a class="tlink" href="#${s.slug}">${s.name}</a>${note}</div>`;
    })
    .join("\n");
  return `<div class="tree">\n<div><span class="dim">${pageName}/</span></div>\n${rows}\n</div>`;
}

/** Homepage nav tree, derived from the page registry */
export function navTree(): string {
  const rows = MD_PAGES.map((p, i) => {
    const guide = i === MD_PAGES.length - 1 ? "└── " : "├── ";
    return `        <div>
          <span class="guide">${guide}</span
          ><span class="tname"><a class="tlink" href="${p.out}">${p.name}</a></span
          ><span class="note">${p.note}</span>
        </div>`;
  }).join("\n");
  return `<div class="tree">
        <div><span class="dim">docs/</span></div>
${rows}
      </div>`;
}

/** First real line of body text in a markdown doc: skips blank lines,
 * headings, HTML comments (site:sub, etc.), and image/badge lines to find
 * the plain-prose description that opens the file. */
export function firstParagraph(md: string): string {
  for (const line of md.split("\n")) {
    const trimmed = line.trim();
    if (
      !trimmed ||
      trimmed.startsWith("#") ||
      trimmed.startsWith("<!--") ||
      trimmed.startsWith("![") ||
      trimmed.startsWith("[![")
    ) {
      continue;
    }
    return trimmed;
  }
  return "";
}

/**
 * Generates the llms.txt (https://llmstxt.org) served alongside the docs
 * site, so LLM tooling gets a curated index instead of having to crawl
 * rendered HTML. Entirely derived from the MD_PAGES registry and README's
 * own opening description — there's no hand-authored prose here to drift
 * out of sync with the real docs.
 */
export function generateLlmsTxt(): string {
  const readme = fs.readFileSync(path.join(ROOT, "README.md"), "utf8");
  const summary = firstParagraph(readme);

  const docs = MD_PAGES.map(
    (p) => `- [${p.name}](${SITE_URL}/${p.out}): ${p.note.replace(/^#\s*/, "")}`
  ).join("\n");

  return `# lintp

> ${summary}

## Docs

${docs}

## Optional

- [Docs site](${SITE_URL}/): the same pages above, rendered
- [Repository](${REPO_URL}): source, issues, and contribution guidelines
`;
}

/** continue/ tree linking to the other doc pages */
function continueTree(currentName: string): string {
  const others = MD_PAGES.filter((p) => p.name !== currentName);
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
    <link rel="icon" type="image/svg+xml" href="assets/favicon.svg" />
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
    const raw = fs.readFileSync(path.join(ROOT, page.src), "utf8");
    const { md, notes, sub } = preprocess(raw);
    const { html, sections } = toComponents(rewriteLinks(md), notes);

    // Intro under the crumb: site:sub metadata, else the first paragraph
    let intro: string;
    let body: string;
    if (sub) {
      intro = `<p class="sub">${sub}</p>`;
      const firstP = html.match(/<p class="p">[\s\S]*?<\/p>/);
      body = firstP ? html.replace(firstP[0], "") : html;
    } else {
      const firstP = html.match(/<p class="p">[\s\S]*?<\/p>/);
      intro = firstP ? firstP[0].replace('class="p"', 'class="sub"') : "";
      body = firstP ? html.replace(firstP[0], "") : html;
    }

    fs.writeFileSync(
      path.join(outDir, page.out),
      template(page.name, intro, `${tocTree(page.name, sections)}\n${body}`)
    );
    written.push(page.out);
  }

  const homepage = fs.readFileSync(path.join(ROOT, HOMEPAGE), "utf8");
  if (!homepage.includes("<!-- nav-tree -->")) {
    throw new Error(`${HOMEPAGE} is missing the <!-- nav-tree --> marker`);
  }
  fs.writeFileSync(
    path.join(outDir, "index.html"),
    homepage.replace("<!-- nav-tree -->", navTree())
  );
  written.push("index.html");

  fs.writeFileSync(path.join(outDir, "llms.txt"), generateLlmsTxt());
  written.push("llms.txt");

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
