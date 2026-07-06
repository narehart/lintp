# lintp - File System Linter with DSL

A powerful file system linter that validates directory structures and file naming conventions using a custom Domain-Specific Language (DSL) defined in YAML configuration files.

<!-- site:sub Install lintp, write your first lintp.yml, and run it against a project — plus configuration, CLI flags, and troubleshooting. -->

![lintp demo: failing files are flagged with the exact rule condition that failed, then pass after renaming](https://raw.githubusercontent.com/narehart/lintp/main/docs/assets/demo.gif)

## Installation <!-- note: npm, from source -->

npm ships a prebuilt binary for macOS, Linux, and Windows (x64/arm64) via `optionalDependencies`. On Windows ARM64, the x64 binary runs through Windows' built-in emulation layer. If no prebuilt package matches your platform, the launcher falls back to a checksum-verified binary from the GitHub release (the checksum guards download integrity, not tamper-proof supply-chain verification). The npm package is named `lintp-cli` (npm reserves the bare name) — the installed command is `lintp`.

```bash title="shell — install via npm"
# run without installing
npx lintp-cli

# or install globally — the command is `lintp`
npm install -g lintp-cli
```

### From crates.io

```bash title="shell — install via cargo"
cargo install lintp            # compiles with your Rust toolchain (1.85+)
cargo install lintp --locked   # exact tested dependency versions
```

### From source

The project uses [asdf](https://asdf-vm.com/) to pin Node.js and Rust versions.

```bash title="shell — build from source"
git clone https://github.com/narehart/lintp.git
cd lintp
asdf install          # required Node.js + Rust versions
cargo build --release
./target/release/lintp --help
```

## Quick Start <!-- note: first config, first run -->

Create `lintp.yml` in your project root. Define reusable patterns under `custom-matchers`, then assign a rule to each file type under `config`.

```yaml title="lintp.yml"
lintp:
  # define reusable patterns
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"
    js-file: '$EXT == "js"'
    ts-file: '$EXT == "ts"'

  # rules per file type
  config:
    .js: "kebab-case && js-file"
    .ts: "PascalCase && ts-file"
    .dir: "kebab-case || PascalCase"

  ignore:
    - node_modules
    - .git
    - dist
```

Run it. Every file and directory is checked against the longest-matching suffix rule; failures name the exact condition that failed.

```text title="shell — first run"
$ lintp
✓ ./lintp.yml
✓ ./tests
✓ ./tests/user-tests.js
✓ ./src
✗ ./src/badFile.js - .js - Does not match rule: kebab-case && js-file (failed: kebab-case)
✓ ./src/UserManager.ts
✓ ./src/utils.js
Some files or directories do not match the configured rules.
```

Rules combine **variables** ($NAME, $EXT…), **operators** (&&, ||, !, ==…), **functions** (matches, contains, startsWith…) and **collections** (siblings, children, find). The complete language lives in the [dsl-reference](docs/DSL_REFERENCE.md); reusable recipes in [common-patterns](docs/COMMON_PATTERNS.md).

## Built-in Variables <!-- note: $NAME, $EXT, $PATH, $item… -->

Every DSL expression has access to the file being checked:

```text title="variables — file & context"
$NAME      # full filename incl. extension   "index.test.js"
$BASENAME  # filename without extension      "index.test"
$EXT       # extension without the dot       "js"
$PATH      # full file path                  "./src/index.test.js"
$PARENT    # parent directory path           "./src"
$item      # current item inside any(), all(), map(), filter()
```

```yaml title="lintp.yml — variables in matchers"
lintp:
  custom-matchers:
    js-file: '$EXT == "js"'
    in-src: 'contains($PATH, "/src/")'
    has-js-sibling: 'any(siblings("*"), endsWith($item, ".js"))'
```

## Configuration <!-- note: rules, messages, ignore -->

Rule keys are **suffix patterns**, not just extensions: a file matches every key its path ends with, and the longest matching suffix wins. `Button.test.tsx` matches both `.tsx` and `.test.tsx`, and the `.test.tsx` rule applies. `.*` applies only when no other key matches; `.dir` targets directories.

A key can group several suffixes with brace alternation — each expansion gets the same rule and message:

```yaml title="lintp.yml — grouped suffixes"
lintp:
  config:
    ".{png,jpg,jpeg,gif,webp,svg}":
      rule: "camelCase"
      message: "image files are camelCase"
```

Suffix matching has one subtlety with dotfiles: a file literally named `.rules` also matches a `.rules:` key, but as a dotfile its `$EXT` is `""` and its `$BASENAME` is the full dotted name. Write `$EXT == "rules"` when a rule should apply only to real `.rules` extensions — or use the behavior deliberately: `.gitignore:` is a valid key for targeting that exact file.

### Directory-scoped rules

A top-level key that is a glob pattern holds its own suffix→rule map, applied only to matching paths — and it **overrides** the global rule for the same suffix there. Globs match the path relative to the linted directory, and `*` crosses `/`, so `src/ui/*` covers the whole subtree. When several scopes match the same path, the most specific (longest pattern) wins: `src/ui/*` beats `src/*` for files under `src/ui/`. Braces expand in scope keys too: `"api/{auth,billing}/*"` is two scopes sharing one rule map.

```yaml title="lintp.yml — per-directory conventions"
lintp:
  config:
    .ts: "false" # default-deny: a .ts file must live in a listed location
    "src/ecs/systems/*":
      .ts:
        rule: 'matches($BASENAME, /^[a-z][a-zA-Z0-9]*System$/) && exists("${$BASENAME}.test.ts")'
        message: "systems are camelCase, end in System, and need a sibling test"
    "src/hooks/*":
      .ts:
        rule: "matches($BASENAME, /^use[A-Z][a-zA-Z0-9]*$/)"
        message: "hooks are named useX"
```

Prefer this over one global rule that chains `($PARENT == "./src/x" && ...) || ...` branches: each location gets its own message, and failures point at the one rule that applies instead of printing the whole chain.

```yaml title="lintp.yml — full structure"
lintp:
  custom-matchers: # reusable pattern definitions
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    pascal-case: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"
  config: # suffix pattern → rule
    .test.js: 'matches($BASENAME, /^[a-z0-9-]+\.test$/)'
    .js: "kebab-case"
    .dir: "kebab-case || pascal-case"
    .*: '!contains($NAME, " ")'
  ignore: # glob patterns to skip
    - node_modules
    - "build/**"
    - "*.tmp"
```

### Custom failure messages

Any rule can be a map with a `message` that replaces the raw expression in failure output — point teammates at your conventions doc instead of a regex.

```yaml title="lintp.yml — custom message"
lintp:
  config:
    .tsx:
      rule: "component-file"
      message: "Components must be PascalCase (see CONTRIBUTING.md)"
```

```text title="shell — failure output"
✗ ./src/badName.tsx - .tsx - Components must be PascalCase (see CONTRIBUTING.md)
```

## DSL at a Glance <!-- site:skip -->

The built-in functions (full documentation with examples in the [DSL Reference](docs/DSL_REFERENCE.md)):

| Function                   | Purpose                                           |
| -------------------------- | ------------------------------------------------- |
| `matches(s, /re/ or glob)` | Test against a regex or glob pattern              |
| `contains(s, sub)`         | String contains a substring                       |
| `startsWith(s, prefix)`    | String starts with prefix                         |
| `endsWith(s, suffix)`      | String ends with suffix                           |
| `without(s, suffix)`       | Remove a suffix if present                        |
| `count(s or list)`         | Length of a string (characters) or list           |
| `in(item, list)`           | List membership                                   |
| `any(list, expr)`          | True if any item satisfies `expr` (binds `$item`) |
| `all(list, expr)`          | True if all items satisfy `expr` (binds `$item`)  |
| `map(list, expr)`          | Transform each item                               |
| `filter(list, expr)`       | Keep items satisfying `expr`                      |
| `siblings(glob)`           | Files in the same directory                       |
| `children(glob)`           | Files inside this directory                       |
| `find(path, glob)`         | Files matching glob under a path                  |
| `exists(glob[, min, max])` | Files matching glob exist (optionally bounded)    |

## CLI Reference <!-- note: flags, exit codes, output -->

```text title="shell — usage"
lintp                          # lint cwd with ./lintp.yml
lintp /path/to/project         # lint a specific directory
lintp --config custom.yml      # custom config file
lintp --verbose                # show every file checked
```

Exit code `0` when everything passes, `1` on any violation or configuration error — a one-line CI gate. When a rule is a chain of `&&` conditions, the failing condition(s) are listed in the `(failed: …)` suffix so you don't have to bisect composed rules by hand.

**Symlinks:** lintp does not follow symlinks. A symlinked directory's name IS checked against `.dir` rules, but its contents are not traversed.

## Best Practices <!-- note: composing rules that scale -->

- **Start simple.** One kebab-case matcher and two config keys; add complexity as conventions solidify.
- **Name matchers descriptively.** `react-component`, not `rule1`.
- **Compose.** Build base patterns (`kebab-case`, `js-file`), then combine them: `"kebab-case && js-file"`.
- **Test your rules.** Touch a good file and a bad file, run `lintp --verbose`, verify both outcomes.
- **Ignore aggressively.** `node_modules`, build output, generated files — specific names beat broad wildcards for speed.
- **Keep checks local.** `siblings()`/`children()` are cheap; `find(".", "**/*")` re-scans the project for every file.

## Troubleshooting <!-- note: common errors, debug tips -->

```text title="common errors — cause → fix"
✗ No config file found
  → create lintp.yml or pass --config path/to/config.yml

✗ Invalid YAML in config file: expected ':' at line 5
  → quote DSL expressions:  rule: '$NAME == "test"'

✗ Failed to parse rule: kebab-case && js file
  → DSL syntax error; the matcher name is missing its hyphen

✗ Unknown matcher 'keba-case' referenced by rule '.js'
  → define the matcher under custom-matchers first (checked at startup)

✗ Circular reference detected: rule-a
  → matchers may not reference each other in a cycle
```

Debugging a rule: run with `--verbose`, read the `(failed: …)` suffix first, and test expressions in isolation with a minimal single-rule config.

## Additional Resources <!-- site:skip -->

- **[Docs site](https://narehart.github.io/lintp/)** — this content plus the full reference, rendered
- **[DSL Reference](docs/DSL_REFERENCE.md)** — complete language reference with all operators, functions, and variables
- **[Common Patterns](docs/COMMON_PATTERNS.md)** — reusable patterns for naming conventions and validation rules
- **[Examples](docs/EXAMPLES.md)** — real-world configuration examples for different project types

## Contributing <!-- site:skip -->

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for commit conventions, the release process, and the docs-site workflow.

## License <!-- site:skip -->

MIT License - see LICENSE file for details.
