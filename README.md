# lintp - File System Linter with DSL

A powerful file system linter that validates directory structures and file naming conventions using a custom Domain-Specific Language (DSL) defined in YAML configuration files.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Built-in Variables](#built-in-variables)
- [Basic Concepts](#basic-concepts)
  - [DSL Reference](#dsl-reference)
  - [Built-in Functions](#built-in-functions)
- [Configuration](#configuration)
- [CLI Reference](#cli-reference)
- [Examples](#examples)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)
- [Advanced Examples](#advanced-examples)
- [Additional Resources](#additional-resources)

## Installation

### From npm (recommended)

```bash
# Run without installing
npx lintp

# Or install globally
npm install -g lintp
```

npm installs a prebuilt binary for your platform (macOS, Linux, and Windows on x64/arm64) through `optionalDependencies`. If no prebuilt package matches your platform, the launcher falls back to downloading a checksum-verified binary from the GitHub release.

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

## DSL Reference

### Literals

```yaml
# String literals
rule: '$NAME == "exact-match"'
rule: "$NAME == 'single-quotes'"

# Integer literals
rule: 'count($NAME) == 42'
rule: 'count($BASENAME) > 5'

# Boolean literals
rule: 'true'
rule: 'false'

# Regular expressions
rule: 'matches($NAME, /^[a-z-]+$/)'
rule: 'matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)'

# Lists
rule: 'in($EXT, ["js", "ts", "jsx", "tsx"])'
rule: 'any(["js", "ts"], $EXT == $item)'
```

### Operators

#### Logical Operators

```yaml
# AND - both conditions must be true
rule: 'kebab-case && js-file'
rule: '$EXT == "js" && count($BASENAME) > 3'

# OR - either condition must be true
rule: 'kebab-case || PascalCase'
rule: '$EXT == "js" || $EXT == "ts"'

# NOT - negates condition
rule: '!matches($NAME, /test/)'
rule: 'js-file && !matches($BASENAME, /\.temp$/)'
```

#### Comparison Operators

```yaml
# Equality
rule: '$EXT == "js"'
rule: '$BASENAME != "index"'

# Numeric comparisons
rule: 'count($NAME) > 5'
rule: 'count($NAME) >= 10'
rule: 'count($BASENAME) < 20'
rule: 'count($BASENAME) <= 15'

# String comparisons (lexicographic)
rule: '$NAME > "a"'
rule: '$BASENAME <= "zzz"'
```

### String Templates

```yaml
# Embed expressions in strings
rule: 'matches($NAME, ${find(".", "*.pattern")[0]})'

# Complex template usage
custom-matchers:
  dynamic-pattern: 'matches($NAME, ${siblings("*.config")[0]})'
```

## Built-in Functions

### String Functions

#### matches(string, pattern)

Test if string matches a regex or glob pattern.

```yaml
custom-matchers:
  # Regex patterns
  kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
  PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"
  camelCase: "matches($BASENAME, /^[a-z][a-zA-Z0-9]*$/)"
  snake_case: "matches($BASENAME, /^[a-z0-9]+(?:_[a-z0-9]+)*$/)"

  # Glob patterns
  js-like: 'matches($NAME, "*.js")'
  test-file: 'matches($NAME, "*.test.*")'
  config-file: 'matches($NAME, "*.config.*")'
```

#### contains(haystack, needle)

Check if a string contains a substring. For list membership, use `in(item, list)` instead.

```yaml
custom-matchers:
  # String contains
  has-test: 'contains($NAME, "test")'
  in-src: 'contains($PATH, "/src/")'

  # List membership uses in(), not contains()
  is-script: 'in($EXT, ["js", "ts", "jsx", "tsx"])'
  valid-ext: 'in($EXT, ["jpg", "png", "gif", "svg"])'
```

#### startsWith(string, prefix)

Check if string starts with prefix.

```yaml
custom-matchers:
  # Component files
  is-component: 'startsWith($BASENAME, "Component")'

  # Test files
  test-prefix: 'startsWith($BASENAME, "test-")'

  # Private files
  is-private: 'startsWith($BASENAME, "_")'
```

#### endsWith(string, suffix)

Check if string ends with suffix.

```yaml
custom-matchers:
  # Different file types
  is-test: 'endsWith($BASENAME, ".test")'
  is-spec: 'endsWith($BASENAME, ".spec")'
  is-story: 'endsWith($BASENAME, ".stories")'

  # Naming patterns
  is-util: 'endsWith($BASENAME, "-util")'
  is-helper: 'endsWith($BASENAME, "-helper")'
```

#### without(string, suffix)

Remove suffix from string if present.

```yaml
custom-matchers:
  # Get base name without common suffixes
  clean-name: 'without(without($BASENAME, ".test"), ".spec")'

  # Remove file extensions manually
  name-without-ext: 'without($NAME, ".js")'

  # Chain multiple removals
  clean-basename: 'without(without(without($BASENAME, "-test"), "-spec"), "-util")'
```

#### count(string_or_list)

Get the length of a string (in characters) or a list.

```yaml
custom-matchers:
  # String length constraints
  reasonable-length: "count($NAME) >= 3 && count($NAME) <= 50"
  short-basename: "count($BASENAME) <= 20"

  # List size constraints
  few-siblings: 'count(siblings("*")) < 10'
  has-children: 'count(children("*")) > 0'
```

### Collection Functions

#### in(item, list)

Check if item exists in list.

```yaml
custom-matchers:
  # Extension allowlists
  allowed-ext: 'in($EXT, ["js", "ts", "jsx", "tsx"])'
  image-file: 'in($EXT, ["jpg", "jpeg", "png", "gif", "svg", "webp"])'

  # Name allowlists
  special-files: 'in($BASENAME, ["index", "main", "app"])'
  config-names: 'in($NAME, ["package.json", "tsconfig.json", ".gitignore"])'
```

#### map(collection, expression)

Transform each item in collection using expression.

```yaml
custom-matchers:
  # Get extensions of all siblings
  sibling-exts: 'map(siblings("*"), $EXT)'

  # Get basenames without extensions
  clean-names: 'map(children("*.js"), without($item, ".js"))'

  # Transform file paths
  relative-paths: 'map(find(".", "*"), without($item, "./"))'
```

#### filter(collection, expression)

Filter collection items by condition.

```yaml
custom-matchers:
  # Filter by extension
  js-siblings: 'filter(siblings("*"), endsWith($item, ".js"))'

  # Filter by naming pattern
  test-files: 'filter(children("*"), contains($item, "test"))'

  # Complex filtering
  valid-components: 'filter(
    siblings("*.tsx"),
    matches($item, /^[A-Z][a-zA-Z0-9]*\.tsx$/)
  )'
```

#### any(collection, expression)

Test if any item in collection matches condition.

```yaml
custom-matchers:
  # Has test files
  has-tests: 'any(siblings("*"), contains($item, "test"))'

  # Has configuration
  has-config: 'any(children("*"), endsWith($item, ".config.js"))'

  # Has documentation
  has-docs: 'any(find(".", "*"), matches($item, /README\.(md|txt)/))'
```

#### all(collection, expression)

Test if all items in collection match condition.

```yaml
custom-matchers:
  # All files follow naming convention
  all-kebab: 'all(siblings("*.js"), matches($item, /^[a-z-]+\.js$/))'

  # All components are PascalCase
  all-pascal-components: 'all(
    filter(children("*"), endsWith($item, "Component.tsx")),
    matches($item, /^[A-Z][a-zA-Z0-9]*Component\.tsx$/)
  )'
```

### File System Functions

#### exists(pattern)

Check if files matching pattern exist.

```yaml
custom-matchers:
  # Required files
  has-package-json: 'exists("package.json")'
  has-readme: 'exists("README.*")'
  has-tests: 'exists("**/*test*")'

  # Conditional requirements
  has-types-if-ts: '$EXT != "ts" || exists("types.d.ts")'
```

#### siblings(pattern)

Get list of files in same directory matching pattern.

```yaml
custom-matchers:
  # Check sibling files
  has-test-sibling: 'any(siblings("*.test.js"), true)'

  # Count siblings
  not-too-many-files: 'count(siblings("*")) <= 20'

  # Naming consistency with siblings
  consistent-with-siblings: 'all(
    siblings("*.js"),
    matches($item, /^[a-z-]+\.js$/)
  )'
```

#### children(pattern)

Get list of files in subdirectories matching pattern.

```yaml
custom-matchers:
  # Check for specific child files
  has-index: 'any(children("index.*"), true)'

  # Validate child structure
  valid-structure: 'all(
    children("*.js"),
    matches($item, /^[a-z-]+\.js$/)
  )'

  # Count children
  reasonable-size: 'count(children("*")) <= 100'
```

#### find(path, pattern)

Find files recursively from path matching pattern.

```yaml
custom-matchers:
  # Global checks
  no-temp-files: 'count(find(".", "*.tmp")) == 0'

  # Required files exist somewhere
  has-license: 'count(find(".", "LICENSE*")) > 0'

  # Validate all files of type
  all-js-valid: 'all(
    find(".", "*.js"),
    matches($item, /^[a-z-]+\.js$/)
  )'
```

## Advanced Examples

### React Project Structure

```yaml
lintp:
  custom-matchers:
    # Naming conventions
    PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    camelCase: "matches($BASENAME, /^[a-z][a-zA-Z0-9]*$/)"

    # File types
    component-file: 'in($EXT, ["tsx", "jsx"]) && PascalCase'
    hook-file: 'startsWith($BASENAME, "use") && camelCase'
    test-file: 'endsWith($BASENAME, ".test") || endsWith($BASENAME, ".spec")'
    story-file: 'endsWith($BASENAME, ".stories")'

    # Structure rules
    has-test: 'any(siblings("*.test.*"), contains($item, without($BASENAME, ".stories")))'
    component-structure: "component-file && has-test"

  config:
    # Components must be PascalCase
    .tsx: "component-file"
    .jsx: "component-file"

    # Hooks must start with "use"
    .ts: "hook-file || kebab-case"
    .js: "kebab-case || camelCase"

    # Test files
    .test.ts: "test-file"
    .test.tsx: "test-file"
    .spec.ts: "test-file"

    # Story files
    .stories.tsx: "story-file && PascalCase"

    # Directories
    .dir: "kebab-case || PascalCase"

  ignore:
    - node_modules
    - build
    - dist
    - .next
    - coverage
```

### Node.js API Project

```yaml
lintp:
  custom-matchers:
    # Naming patterns
    kebab-case: 'matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)'
    PascalCase: 'matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)'
    camelCase: 'matches($BASENAME, /^[a-z][a-zA-Z0-9]*$/)'

    # File types
    route-file: 'contains($PATH, "/routes/") && kebab-case'
    controller-file: 'endsWith($BASENAME, "Controller") && PascalCase'
    model-file: 'contains($PATH, "/models/") && PascalCase'
    middleware-file: 'endsWith($BASENAME, "Middleware") && PascalCase'
    util-file: 'contains($PATH, "/utils/") && kebab-case'

    # API structure
    has-controller: 'route-file && any(
      find("./controllers", "*.js"),
      contains($item, $BASENAME)
    )'

    # Test requirements
    has-unit-test: 'any(
      siblings("*.test.js"),
      contains($item, $BASENAME)
    )'

  config:
    .js: 'kebab-case || PascalCase || camelCase'
    .ts: 'kebab-case || PascalCase || camelCase'
    .json: 'kebab-case'
    .dir: 'kebab-case'

  ignore:
    - node_modules
    - dist
    - coverage
    - logs
```

### Monorepo Structure

```yaml
lintp:
  custom-matchers:
    # Package naming
    package-name: 'matches($BASENAME, /^[a-z][a-z0-9-]*$/)'
    scope-package: 'matches($BASENAME, /^@[a-z][a-z0-9-]*\/[a-z][a-z0-9-]*$/)'

    # Workspace structure
    is-package: 'exists("package.json") && exists("src/index.*")'
    has-tests: 'exists("src/**/*.test.*") || exists("tests/**/*")'
    has-docs: 'exists("README.md")'

    # Valid package
    valid-package: 'is-package && has-tests && has-docs'

    # Consistent structure across packages
    standard-structure: 'all([
      "src",
      "tests",
      "docs"
    ], exists($item))'

  config:
    .dir: 'package-name && (! is-package || valid-package)'
    .json: 'kebab-case'
    .md: 'kebab-case || in($BASENAME, ["README", "CHANGELOG"])'

  ignore:
    - node_modules
    - "*/dist"
    - "*/build"
    - "*/coverage"
```

### Strict Corporate Standards

```yaml
lintp:
  custom-matchers:
    # Strict naming conventions
    strict-kebab: 'matches($BASENAME, /^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/)'
    strict-pascal: 'matches($BASENAME, /^[A-Z][A-Z0-9]*(?:[A-Z][a-z0-9]*)*$/)'

    # File size limits
    reasonable-length: 'count($NAME) >= 5 && count($NAME) <= 50'
    short-basename: 'count($BASENAME) >= 3 && count($BASENAME) <= 30'

    # Required documentation
    has-header-comment: 'matches(
      find(".", $NAME)[0],
      /^\/\*\*[\s\S]*@author[\s\S]*@date[\s\S]*\*\//
    )'

    # Security requirements
    no-secrets: '! contains($NAME, "secret") &&
                 ! contains($NAME, "password") and
                 ! contains($NAME, "key")'

    # Approved file types only
    approved-ext: 'in($EXT, [
      "js", "ts", "jsx", "tsx",
      "json", "yml", "yaml",
      "md", "txt",
      "css", "scss", "less"
    ])'

    # Directory structure
    approved-dirs: 'in($BASENAME, [
      "src", "lib", "tests", "docs", "scripts",
      "assets", "public", "config", "types"
    ])'

  config:
    .js: 'strict-kebab && reasonable-length && no-secrets'
    .ts: 'strict-kebab && reasonable-length && no-secrets'
    .tsx: 'strict-pascal && reasonable-length && no-secrets'
    .jsx: 'strict-pascal && reasonable-length && no-secrets'
    .json: 'strict-kebab && approved-ext'
    .dir: 'approved-dirs && strict-kebab'
    .*: 'approved-ext && no-secrets'

  ignore:
    - node_modules
    - .git
    - dist
    - build
    - coverage
    - "*.log"
```

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
