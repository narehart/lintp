# Common Patterns Reference

This document defines reusable patterns that are used throughout the lintp documentation examples.

## Standard Naming Patterns

### Case Conventions

```yaml
# kebab-case: lowercase with hyphens
kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"

# PascalCase: uppercase first letter, camelCase thereafter
PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"

# camelCase: lowercase first letter, PascalCase thereafter
camelCase: "matches($BASENAME, /^[a-z][a-zA-Z0-9]*$/)"

# snake_case: lowercase with underscores
snake_case: "matches($BASENAME, /^[a-z0-9]+(?:_[a-z0-9]+)*$/)"
```

### File Type Patterns

```yaml
# JavaScript/TypeScript files
js-file: '$EXT == "js"'
ts-file: '$EXT == "ts"'
jsx-file: '$EXT == "jsx"'
tsx-file: '$EXT == "tsx"'

# Script files (any JS/TS variant)
script-file: 'in($EXT, ["js", "ts", "jsx", "tsx"])'

# Test files
test-file: 'matches($NAME, /\.test\.(js|ts|jsx|tsx)$/)'
spec-file: 'matches($NAME, /\.spec\.(js|ts|jsx|tsx)$/)'

# Config files
config-file: 'matches($NAME, /\.config\.(js|ts|json)$/)'
```

## Framework-Specific Patterns

### React Patterns

```yaml
# React component files
react-component: 'in($EXT, ["tsx", "jsx"]) && PascalCase'

# React hooks (must start with "use")
react-hook: 'startsWith($BASENAME, "use") && camelCase'

# Story files for Storybook
story-file: 'endsWith($BASENAME, ".stories") && PascalCase'
```

### Node.js Backend Patterns

```yaml
# API route files
route-file: 'contains($PATH, "/routes/") && kebab-case'

# Controller files
controller-file: 'endsWith($BASENAME, "Controller") && PascalCase'

# Model files
model-file: 'contains($PATH, "/models/") && PascalCase'

# Middleware files
middleware-file: 'endsWith($BASENAME, "Middleware") && PascalCase'

# Utility files
util-file: 'contains($PATH, "/utils/") && kebab-case'
```

## Validation Patterns

### Structure Requirements

```yaml
# Has corresponding test file
has-test: 'any(siblings("*.test.*"), contains($item, $BASENAME))'

# Has package.json (for packages)
is-package: 'exists("package.json")'

# Has README documentation
has-docs: 'exists("README.*")'

# Has proper project structure
has-src: 'exists("src/")'
```

### Security Patterns

```yaml
# No sensitive information in filenames
no-secrets: '!contains($NAME, "secret") &&
             !contains($NAME, "password") &&
             !contains($NAME, "key") &&
             !contains($NAME, "token")'

# No temporary files
no-temp-files: '!endsWith($NAME, ".tmp") &&
                !endsWith($NAME, ".bak") &&
                !startsWith($BASENAME, "~")'

# Approved file extensions only
safe-extensions: 'in($EXT, [
  "js", "ts", "jsx", "tsx", "vue",
  "json", "yml", "yaml", "toml",
  "md", "txt", "html", "css", "scss",
  "png", "jpg", "jpeg", "gif", "svg"
])'
```

## Location-Based Architecture

For repos where each directory has its own convention, structure the config as **default-deny plus directory scopes** instead of one rule with many `$PARENT ==` branches:

```yaml
lintp:
  custom-matchers:
    camelCase: "matches($BASENAME, /^[a-z][a-zA-Z0-9]*$/)"
    # a source file must have a same-basename test next to it
    # (exists() patterns are relative to the file's directory)
    has-sibling-test: 'exists("${$BASENAME}.test.ts")'

  config:
    .test.ts: "camelCase || true"
    .ts: "false" # not listed below → not allowed

    "src/ecs/systems/*":
      .ts:
        rule: 'endsWith($BASENAME, "System") && camelCase && has-sibling-test'
        message: "systems are camelCase, end in System, and need a sibling test"

    "src/hooks/*":
      .ts:
        rule: "matches($BASENAME, /^use[A-Z][a-zA-Z0-9]*$/)"
        message: "hooks are named useX"

    "src/constants/*":
      .ts:
        rule: 'in($BASENAME, ["character", "inventory", "combat", "settings"])'
        message: "constants files are named after a game domain"
```

Three idioms doing the work:

- **Default-deny**: the global `.ts: "false"` makes every location a deliberate decision; a scoped rule overrides it where files belong
- **Per-scope messages**: failures cite the one convention that applies, not a wall of `||` branches
- **`in()` allowlists**: exact-name sets read (and diff) better as lists than as `/^(a|b|c)$/` alternations

For directory sets, the same shape applies to `.dir`:

```yaml
"src/*":
  .dir:
    rule: 'in($BASENAME, ["ecs", "hooks", "constants", "ui"])'
    message: "new src/ directories must be added to lintp.yml"
```

## Usage in Configuration

To use these patterns in your `lintp.yml`, simply reference them in your `custom-matchers` section:

```yaml
lintp:
  custom-matchers:
    # Import patterns you need
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"
    js-file: '$EXT == "js"'
    react-component: 'in($EXT, ["tsx", "jsx"]) && PascalCase'

    # Define your specific rules
    my-custom-rule: "kebab-case && js-file"

  config:
    .js: "my-custom-rule"
    .tsx: "react-component"
```

## Cross-References <!-- site:skip -->

- **Basic Usage**: See [README.md](../README.md#quick-start)
- **Complete Function Reference**: See [DSL_REFERENCE.md](DSL_REFERENCE.md#functions)
- **Real-World Examples**: See [EXAMPLES.md](EXAMPLES.md)
