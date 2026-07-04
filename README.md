# lintp - File System Linter with DSL

A powerful file system linter that validates directory structures and file naming conventions using a custom Domain-Specific Language (DSL) defined in YAML configuration files.

![lintp demo: failing files are flagged with the exact rule condition that failed, then pass after renaming](https://raw.githubusercontent.com/narehart/lintp/main/docs/demo.gif)

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Built-in Variables](#built-in-variables)
- [Basic Concepts](#basic-concepts)
- [Configuration](#configuration)
- [DSL at a Glance](#dsl-at-a-glance)
- [CLI Reference](#cli-reference)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)
- [Additional Resources](#additional-resources)

## Installation

### From npm (recommended)

```bash
# Run without installing
npx lintp-cli

# Or install globally — the installed command is `lintp`
npm install -g lintp-cli
lintp --help
```

The npm package is named `lintp-cli` (npm reserves the bare name), but the command it installs is `lintp`. npm installs a prebuilt binary for your platform (macOS, Linux, and Windows on x64/arm64) through `optionalDependencies`. If no prebuilt package matches your platform, the launcher falls back to downloading a checksum-verified binary from the GitHub release.

### From crates.io

```bash
cargo install lintp
```

Compiles from source with your Rust toolchain (1.82 or newer) — no Node.js required.

### Prerequisites (building from source)

This project uses [asdf](https://asdf-vm.com/) to manage tool versions. Make sure you have asdf installed, then run:

```bash
asdf plugin add nodejs https://github.com/asdf-vm/asdf-nodejs.git
asdf plugin add rust https://github.com/code-lever/asdf-rust.git
asdf install
```

### From Source

```bash
git clone https://github.com/narehart/lintp.git
cd lintp
asdf install  # Install required Node.js and Rust versions
cargo build --release
./target/release/lintp --help
```

### Binary Usage

```bash
# Basic usage with default config (lintp.yml)
lintp

# Specify custom config file
lintp --config my-rules.yml

# Lint specific directory
lintp /path/to/project

# Verbose output
lintp --verbose

# Combine options
lintp --config custom.yml --verbose /path/to/project
```

## Quick Start

### 1. Create a Configuration File

Create `lintp.yml` in your project root:

```yaml
lintp:
  # Define reusable patterns
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"
    camelCase: "matches($BASENAME, /^[a-z][a-zA-Z0-9]*$/)"
    js-file: '$EXT == "js"'
    ts-file: '$EXT == "ts"'

  # Define rules for different file types
  config:
    .js: "kebab-case && js-file"
    .ts: "PascalCase && ts-file"
    .dir: "kebab-case || PascalCase"

  # Files/directories to ignore
  ignore:
    - node_modules
    - .git
    - dist
    - build
```

### 2. Run the Linter

```bash
lintp
```

Output:

```
✓ ./src/utils.js
✓ ./src/UserManager.ts
✗ ./src/badFile.js - .js - Does not match rule: kebab-case && js-file (failed: kebab-case)
✓ ./tests/user-tests.js
Some files or directories do not match the configured rules.
```

## Built-in Variables

lintp provides built-in variables that give you access to file and path information within your DSL expressions.

### File Variables

```yaml
# $NAME - Full filename including extension
custom-matchers:
  full-name-check: '$NAME == "package.json"'

# $BASENAME - Filename without extension
custom-matchers:
  base-check: '$BASENAME == "index"'

# $EXT - File extension (without dot)
custom-matchers:
  js-file: '$EXT == "js"'
  script-file: 'in($EXT, ["js", "ts", "jsx", "tsx"])'

# $PATH - Full file path
custom-matchers:
  path-check: 'contains($PATH, "/src/")'

# $PARENT - Parent directory path
custom-matchers:
  in-tests: 'contains($PARENT, "tests")'
```

### Context Variables

```yaml
# $item - Current item in collection functions
custom-matchers:
  has-js-sibling: 'any(siblings("*"), endsWith($item, ".js"))'
  all-lowercase: 'all(children("*"), matches($item, /^[a-z-]+$/))'
```

## Basic Concepts

The lintp DSL allows you to create powerful validation rules using:

- **Variables** - Access file information (`$NAME`, `$EXT`, `$PATH`, etc.)
- **Operators** - Combine conditions (`&&`, `||`, `!`, `==`, `!=`, etc.)
- **Functions** - Perform operations (`matches()`, `contains()`, `startsWith()`, etc.)
- **Collections** - Work with lists of files (`siblings()`, `children()`, `find()`)

For detailed information, see our complete [DSL Reference](docs/DSL_REFERENCE.md) and [Common Patterns](docs/COMMON_PATTERNS.md).

## Configuration

### Configuration Structure

```yaml
lintp:
  custom-matchers: # Reusable pattern definitions
    pattern-name: "DSL expression"

  config: # Rules for file types
    .ext: "rule expression"
    .dir: "rule for directories"

  ignore: # Glob patterns to ignore
    - pattern1
    - pattern2
```

### File Type Rules

Rules are applied based on file extensions and special patterns:

```yaml
config:
  # JavaScript files
  .js: "kebab-case && js-file"

  # TypeScript files
  .ts: "PascalCase && ts-file"

  # Multiple extensions
  .test.js: "test-file-naming"
  .spec.ts: "spec-file-naming"

  # Directories
  .dir: "kebab-case || PascalCase"

  # All files (fallback)
  .*: "basic-naming-rules"
```

Rule keys are suffix patterns, not just extensions: a file matches every key its path ends with, and the **longest matching suffix wins**. `Button.test.tsx` matches both `.tsx` and `.test.tsx`, and the `.test.tsx` rule is applied. `.*` applies only when no other key matches.

### Custom Failure Messages

Any rule can be written as a map with a `message` that replaces the raw expression in failure output — useful for pointing teammates at your conventions doc:

```yaml
config:
  .tsx:
    rule: "component-file"
    message: "Component files must be PascalCase (see CONTRIBUTING.md)"
```

```
✗ ./src/badName.tsx - .tsx - Component files must be PascalCase (see CONTRIBUTING.md)
```

### Ignore Patterns

```yaml
ignore:
  # Exact directory names
  - node_modules
  - .git

  # Glob patterns
  - "*.tmp"
  - "build/**"
  - "dist/**"

  # Regex patterns (use glob syntax)
  - ".*.bak"
```

## DSL at a Glance

Rules are boolean expressions over built-in variables, combined with `&&`, `||`, and `!`. The built-in functions:

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

For literals, operators, precedence, string templates, and full function documentation with examples, see the **[DSL Reference](docs/DSL_REFERENCE.md)**. Reusable naming-convention snippets live in **[Common Patterns](docs/COMMON_PATTERNS.md)**.

## CLI Reference

### Basic Usage

```bash
# Lint current directory with default config
lintp

# Specify directory to lint
lintp /path/to/project

# Use custom config file
lintp --config custom-rules.yml

# Verbose output
lintp --verbose

# Combine options
lintp --config rules.yml --verbose /path/to/project
```

### Exit Codes

- `0` - All files pass linting rules
- `1` - Some files fail linting rules or configuration error

### Output Format

#### Success Output

```
✓ ./src/components/Button.tsx
✓ ./src/utils/format.js
✓ ./tests/button.test.tsx
All files and directories match the configured rules.
```

#### Failure Output

```
✓ ./src/components/Button.tsx
✗ ./src/badFile.js - .js - Does not match rule: kebab-case && js-file (failed: kebab-case)
✗ ./Invalid-Dir - .dir - Does not match rule: kebab-case || PascalCase
✓ ./tests/button.test.tsx
Some files or directories do not match the configured rules.
```

When a rule is a chain of `&&` conditions, the failing condition(s) are listed in the `(failed: ...)` suffix so you don't have to bisect composed rules by hand.

#### Verbose Output

```
Checking: src/components/Button.tsx
Checking: src/utils/format.js
Checking: src/badFile.js
Checking: tests/button.test.tsx
✓ ./src/components/Button.tsx
✓ ./src/utils/format.js
✗ ./src/badFile.js - .js - Does not match rule: kebab-case && js-file
✓ ./tests/button.test.tsx
Some files or directories do not match the configured rules.
```

## Best Practices

### 1. Start Simple

Begin with basic naming conventions and gradually add complexity:

```yaml
# Start with this
lintp:
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9-]+$/)"

  config:
    .js: "kebab-case"
    .dir: "kebab-case"

  ignore:
    - node_modules
```

### 2. Use Descriptive Matcher Names

```yaml
# Good - descriptive names
custom-matchers:
  react-component: 'matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/) && in($EXT, ["tsx", "jsx"])'
  test-file: 'contains($NAME, "test") || contains($NAME, "spec")'

# Bad - unclear names
custom-matchers:
  rule1: 'matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)'
  check: 'contains($NAME, "test")'
```

### 3. Compose Complex Rules

Break down complex rules into smaller, reusable pieces:

```yaml
custom-matchers:
  # Base patterns
  kebab-case: "matches($BASENAME, /^[a-z0-9-]+$/)"
  PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"

  # File types
  js-file: '$EXT == "js"'
  component-file: 'in($EXT, ["tsx", "jsx"])'

  # Composed rules
  js-utility: "kebab-case && js-file"
  react-component: "PascalCase && component-file"
```

### 4. Document Your Rules

Add comments to explain complex rules:

```yaml
custom-matchers:
  # Component files must be PascalCase and end with .tsx/.jsx
  react-component: "PascalCase && component-file"

  # Test files must contain "test" or "spec" and have corresponding source file
  valid-test: '(contains($NAME, "test") || contains($NAME, "spec")) &&
    any(siblings("*"), without($item, ".test") == without($NAME, ".test"))'
```

### 5. Test Your Rules

Create test files to verify your rules work as expected:

```bash
# Create test files
touch good-file.js
touch BadFile.js
touch test.spec.js

# Run linter
lintp --verbose

# Verify output matches expectations
```

### 6. Use Ignore Patterns Effectively

```yaml
ignore:
  # Always ignore these
  - node_modules
  - .git
  - dist
  - build

  # Temporary files
  - "*.tmp"
  - "*.bak"
  - ".*.swp"

  # Generated files
  - "generated/**"
  - "**/*.generated.*"
```

## Troubleshooting

### Common Errors

#### 1. Configuration File Not Found

```
Error: No config file found. Use --config to specify a config file path or create lintp.yml in the current directory.
```

**Solution:** Create `lintp.yml` or specify a config path:

```bash
lintp --config /path/to/config.yml
```

#### 2. Invalid YAML Syntax

```
Error: Invalid YAML in config file: expected ':' at line 5
```

**Solution:** Check YAML syntax, especially quotes and indentation:

```yaml
# Bad
custom-matchers:
  rule: $NAME == "test"  # Missing quotes around expression

# Good
custom-matchers:
  rule: '$NAME == "test"'
```

#### 3. Parse Errors in DSL Expressions

```
Error: Failed to parse rule: kebab-case && js file
                                           ^
```

**Solution:** Check DSL syntax, missing operators:

```yaml
# Bad
rule: 'kebab-case && js file'  # Missing hyphen

# Good
rule: 'kebab-case && js-file'
```

#### 4. Unknown Reference Error

```
Error: Unknown reference: kebab-case
```

**Solution:** Define referenced matchers:

```yaml
custom-matchers:
  kebab-case: "matches($BASENAME, /^[a-z0-9-]+$/)" # Define before using

config:
  .js: "kebab-case" # Now this works
```

#### 5. Circular Reference Error

```
Error: Circular reference detected in custom matcher: rule-a
```

**Solution:** Remove circular dependencies:

```yaml
# Bad - circular reference
custom-matchers:
  rule-a: 'rule-b && $EXT == "js"'
  rule-b: 'rule-a || $EXT == "ts"'

# Good - no circular dependency
custom-matchers:
  js-file: '$EXT == "js"'
  ts-file: '$EXT == "ts"'
  script-file: 'js-file || ts-file'
```

### Debug Tips

#### 1. Use Verbose Mode

```bash
lintp --verbose
```

This shows which files are being checked and their processing status.

#### 2. Read the (failed: ...) Suffix

When a composed rule fails, the failure line names the specific `&&` condition(s) that failed — start there before bisecting by hand.

#### 3. Test Individual Expressions

Create a minimal config to test specific rules:

```yaml
lintp:
  custom-matchers:
    test-rule: "YOUR_EXPRESSION_HERE"

  config:
    .js: "test-rule"

  ignore: []
```

#### 4. Check Variable Values

Use simple expressions to verify variable contents:

```yaml
custom-matchers:
  debug-name: '$NAME == "expected-name.js"'
  debug-basename: '$BASENAME == "expected-name"'
  debug-ext: '$EXT == "js"'
```

#### 5. Validate Regex Patterns

Test regex patterns in isolation:

```yaml
custom-matchers:
  test-regex: 'matches("test-input", /^your-pattern$/)'
```

### Performance Tips

#### 1. Avoid Expensive File System Operations

```yaml
# Expensive - searches entire project for each file
slow-rule: 'count(find(".", "**/*")) < 1000'

# Better - use siblings/children for local checks
fast-rule: 'count(siblings("*")) < 50'
```

#### 2. Use Specific Ignore Patterns

```yaml
# Good - specific ignores
ignore:
  - node_modules
  - dist
  - "*.log"

# Less efficient - broad wildcards
ignore:
  - "**/*.tmp"    # Checks every file
```

#### 3. Order Rules by Specificity

```yaml
config:
  # More specific first
  .test.js: "test-file-rules"
  .spec.js: "spec-file-rules"

  # General patterns last
  .js: "general-js-rules"
  .*: "fallback-rules"
```

## Additional Resources

- **[DSL Reference](docs/DSL_REFERENCE.md)** - Complete language reference with all operators, functions, and variables
- **[Common Patterns](docs/COMMON_PATTERNS.md)** - Reusable patterns for naming conventions and validation rules
- **[Examples](docs/EXAMPLES.md)** - Real-world configuration examples for different project types

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

## License

MIT License - see LICENSE file for details.
