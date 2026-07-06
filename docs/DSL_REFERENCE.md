# lintp DSL Reference

Complete reference for the lintp Domain-Specific Language (DSL) used in configuration files.

## Overview

The lintp DSL is a powerful expression language for defining file and directory validation rules. It supports variables, operators, functions, and complex expressions that can be composed to create sophisticated validation logic.

## Variables

### Built-in File Variables

| Variable    | Description                       | Example Value                   |
| ----------- | --------------------------------- | ------------------------------- |
| `$NAME`     | Full filename including extension | `"Button.tsx"`                  |
| `$BASENAME` | Filename without extension        | `"Button"`                      |
| `$EXT`      | File extension (without dot)      | `"tsx"`                         |
| `$PATH`     | Full file path                    | `"./src/components/Button.tsx"` |
| `$PARENT`   | Parent directory path             | `"./src/components"`            |

`$PATH` and `$PARENT` reflect however the CLI was invoked, not a fixed
absolute form — running `lintp` against the default `.` directory (as the
README's examples do) produces `./`-prefixed values like the ones above;
passing an absolute directory argument produces absolute paths instead.

`$EXT` and `$BASENAME` follow the same rule as Rust's `Path::file_stem` /
`Path::extension`: a name that starts with a dot and has no other dot (like
`.gitignore`) has no extension, so `$EXT` is `""` and `$BASENAME` is the
whole `".gitignore"`. A name with more than one dot (like `file.test.js`)
splits on the _last_ dot only, so `$EXT` is `"js"` and `$BASENAME` is
`"file.test"`.

### Context Variables

| Variable | Description                           | Usage                                 |
| -------- | ------------------------------------- | ------------------------------------- |
| `$item`  | Current item in collection operations | Used in `map`, `filter`, `any`, `all` |

### Variable Usage Examples

```yaml
# Check exact filename
rule: '$NAME == "package.json"'

# Check basename pattern
rule: 'matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)'

# Check file extension
rule: '$EXT == "tsx"'

# Check path contains directory
rule: 'contains($PATH, "/components/")'

# Check parent directory
rule: 'endsWith($PARENT, "/tests")'
```

## Literals

### String Literals

```yaml
# Double quotes (supports ${...} string templates)
rule: '$NAME == "exact-match.js"'
rule: 'endsWith($PATH, "${$NAME}")'

# Single quotes (literal strings, no templates)
rule: "$NAME == 'literal-string.js'"
```

### Integer Literals

```yaml
# Positive integers
rule: 'count($NAME) == 42'

# Negative integers (using unary minus)
rule: 'count($BASENAME) > -5'

# Zero
rule: 'count(siblings("*")) == 0'
```

### Boolean Literals

```yaml
# True/false
rule: 'true'
rule: 'false'
rule: '$EXT == "js" || false'
```

### Regular Expression Literals

Regex literals are compiled with Rust's [`regex`](https://docs.rs/regex)
crate, which guarantees linear-time matching by excluding backtracking
features. Lookahead, lookbehind, and backreferences aren't supported —
`matches($NAME, /^(?=x)/)` fails at evaluation time with:

```
regex parse error:
    ^(?=x)
     ^^^
error: look-around, including look-ahead and look-behind, is not supported
```

```yaml
# Basic regex
rule: 'matches($NAME, /^[a-z-]+$/)'

# Complex regex with escaping
rule: 'matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)'

# Regex with special characters
rule: 'matches($NAME, /\.test\.(js|ts)$/)'
```

### Comments

`#` starts a comment that runs to the end of the line, anywhere inside a
DSL expression.

```yaml
# Everything from # to end-of-line is ignored, including "&& false" below —
# this rule evaluates to plain `true`
rule: "true # this is a comment && false"
```

### List Literals

```yaml
# String list
rule: 'in($EXT, ["js", "ts", "jsx", "tsx"])'

# Mixed type list
rule: 'in($NAME, ["index.js", "main.ts", 42])'

# Empty list
rule: 'count([]) == 0'
```

## Operators

### Logical Operators

#### AND (`&&`)

Both operands must be true.

```yaml
# Basic AND
rule: '$EXT == "js" && count($BASENAME) > 3'

# Chained AND
rule: 'kebab-case && js-file && has-test'

# Precedence (AND has higher precedence than OR)
rule: 'a || b && c'  # Equivalent to: a || (b && c)
```

#### OR (`||`)

Either operand must be true.

```yaml
# Basic OR
rule: '$EXT == "js" || $EXT == "ts"'

# Chained OR
rule: 'kebab-case || PascalCase || camelCase'

# Mixed with AND
rule: '(kebab-case || PascalCase) && js-file'
```

#### NOT (`!`)

Negates the operand.

```yaml
# Basic NOT
rule: '!contains($NAME, "temp")'

# NOT with complex expression
rule: '!(matches($NAME, /test/) && $EXT == "js")'

# Double negation
rule: '!!true'  # Equivalent to: true
```

### Comparison Operators

#### Equality (`==`, `!=`)

```yaml
# String equality
rule: '$EXT == "js"'
rule: '$BASENAME != "index"'

# Integer equality
rule: 'count($NAME) == 10'
rule: 'count($BASENAME) != 0'

# Boolean equality
rule: 'true == true'
rule: 'false != true'
```

#### Ordering (`<`, `<=`, `>`, `>=`)

```yaml
# Integer comparison
rule: 'count($NAME) > 5'
rule: 'count($NAME) >= 10'
rule: 'count($BASENAME) < 20'
rule: 'count($BASENAME) <= 15'

# String comparison (lexicographic)
rule: '$NAME > "a"'
rule: '$BASENAME <= "zzz"'
rule: '"apple" < "banana"'
```

### Arithmetic Operators (not supported)

The grammar recognizes `+`, `-`, `*`, `/`, and `%` between two expressions,
but the evaluator rejects them rather than silently doing arithmetic (or
letting `+` slide as string concatenation). Compare `count()` results, or
build strings with `${...}` templates, instead:

```yaml
# Fails to parse — arithmetic is not supported
rule: "count($NAME) + 1 == 2"
```

```
Arithmetic operator '+' is not supported; compare count() results or build strings with ${...} templates instead
```

### Operator Precedence

From loosest to tightest binding — the grammar nests each level inside the
next, so `||` groups last and function calls / literals / indexing bind
first:

1. **Logical OR**: `||`
2. **Logical AND**: `&&`
3. **Logical NOT**: `!`
4. **Comparison**: `==`, `!=`, `<`, `<=`, `>`, `>=`
5. **Additive** (unsupported): `+`, `-`
6. **Multiplicative** (unsupported): `*`, `/`, `%`
7. **Unary minus**: `-`
8. **Primary**: literals, variables, function calls, indexing, `(...)`

Note that `!` binds _looser_ than comparison — it wraps the whole
comparison instead of just its left operand — while unary minus binds
_tighter_ than comparison, since it applies directly to a primary
expression:

```yaml
# Examples of precedence
rule: '!a == b'        # Equivalent to: !(a == b), NOT (!a) == b
rule: 'a && b || c'    # Equivalent to: (a && b) || c
rule: 'a || b && c'    # Equivalent to: a || (b && c)
rule: 'a == b && c'    # Equivalent to: (a == b) && c
```

`!$EXT == "zzz"` evaluates cleanly on any file — it never raises "NOT
requires boolean" — because `!` wraps the comparison rather than negating
`$EXT` on its own:

```yaml
rule: '!$EXT == "zzz"' # Equivalent to: !($EXT == "zzz")
```

## Functions

### String Functions

#### `matches(string, pattern)`

Test if string matches a regex or glob pattern.

```yaml
# Regex patterns
matches($BASENAME, /^[a-z0-9-]+$/)          # kebab-case
matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)   # PascalCase
matches($BASENAME, /^[a-z][a-zA-Z0-9]*$/)   # camelCase
matches($NAME, /\.test\.(js|ts)$/)          # test files

# Glob patterns
matches($NAME, "*.js")                      # JavaScript files
matches($NAME, "test-*")                    # Files starting with "test-"
matches($PATH, "*/components/*")            # Files in components directory
```

#### `contains(haystack, needle)`

Check if a string contains a substring. For list membership, use `in(item, list)` instead.

```yaml
# String contains substring
contains($NAME, "test")                     # Filename contains "test"
contains($PATH, "/src/")                    # Path contains "/src/"
contains($PARENT, "components")             # Parent contains "components"

# For list membership, use in(item, list):
in($EXT, ["js", "ts", "jsx"])               # Extension in list
in("index.js", siblings("*.js"))            # Sibling list contains file
```

#### `startsWith(string, prefix)`

Check if string starts with prefix.

```yaml
startsWith($BASENAME, "Component")          # Component files
startsWith($BASENAME, "use")                # React hooks
startsWith($NAME, "test-")                  # Test files with prefix
startsWith($PATH, "/src/")                  # Files in src directory
```

#### `endsWith(string, suffix)`

Check if string ends with suffix.

```yaml
endsWith($BASENAME, "Controller")           # Controller files
endsWith($BASENAME, ".test")                # Test files with suffix
endsWith($NAME, ".config.js")               # Config files
endsWith($PATH, "/index.ts")                # Index files
```

#### `without(string, suffix)`

Remove suffix from string if present.

```yaml
without($BASENAME, ".test")                      # Remove .test suffix
without($NAME, ".js")                            # Remove .js extension
without(without($BASENAME, "-util"), "-helper")  # Chain removals
```

#### `count(string_or_list)`

Get the length of a string (in characters) or a list.

```yaml
count($NAME)                                # Filename length
count($BASENAME)                            # Basename length
count(siblings("*.js"))                     # Number of JS siblings
count(["a", "b", "c"])                      # List length (3)
```

### Collection Functions

#### List and string indexing (`list[n]`, `string[n]`)

Any list expression can be indexed (zero-based). Out-of-range indexes are
an error, so guard with `count()` when the list may be empty.

```yaml
siblings("*.config")[0]                     # First config sibling
count(children("*")) > 0 && children("*")[0] == "index.ts"
```

Strings can be indexed the same way — `$BASENAME[0]` returns the character
at that index, as a one-character string. Negative indexes and indexes
past the end of the string are both errors, the same as list indexing:

```yaml
$BASENAME[0] == "h" # First character of the basename
```

#### `in(item, list)`

Check if item exists in list.

```yaml
in($EXT, ["js", "ts", "jsx", "tsx"])       # Extension allowlist
in($BASENAME, ["index", "main", "app"])    # Basename allowlist
in("test", siblings("*.js"))               # Check if file in siblings
```

#### `map(collection, expression)`

Transform each item in collection.

```yaml
# Get extensions of all siblings
map(siblings("*"), $EXT)

# Remove extension from sibling names
map(siblings("*.js"), without($item, ".js"))

# Transform paths
map(find(".", "*.ts"), without($item, "./"))

# Get basenames of children
map(children("*"), $BASENAME)
```

#### `filter(collection, expression)`

Filter collection by condition.

```yaml
# Filter siblings by extension
filter(siblings("*"), endsWith($item, ".js"))

# Filter by naming pattern
filter(children("*"), matches($item, /^[a-z-]+$/))

# Complex filtering
filter(
  siblings("*.tsx"),
  matches($item, /^[A-Z][a-zA-Z0-9]*\.tsx$/)
)
```

#### `any(collection, expression)`

Test if any item matches condition. `any([], ...)` is always `false` —
there's nothing in an empty collection that could match (vacuous truth).

```yaml
# Check if any sibling is a test file
any(siblings("*"), contains($item, "test"))

# Check if any child is an index file
any(children("*"), startsWith($item, "index"))

# Check for configuration files
any(find(".", "**/*"), endsWith($item, ".config.js"))

# Always false on an empty collection
any([], $item == "x") == false
```

#### `all(collection, expression)`

Test if all items match condition. `all([], ...)` is always `true` —
every item in an empty collection (there are none) trivially matches
(vacuous truth).

```yaml
# All siblings follow naming convention
all(siblings("*.js"), matches($item, /^[a-z-]+\.js$/))

# All children are properly named
all(children("*"), !contains($item, " "))

# All TypeScript files have proper extensions
all(
  filter(find(".", "**/*"), contains($item, "typescript")),
  endsWith($item, ".ts")
)

# Always true on an empty collection
all([], $item == "x") == true
```

### File System Functions

#### `exists(pattern)`, `exists(pattern, min)`, `exists(pattern, min, max)`

Check if the number of files matching `pattern` falls within `[min, max]`.
With one argument, `min` defaults to `1` and `max` is unbounded, so
`exists(pattern)` just checks that at least one match exists. Patterns are
resolved relative to the entry being linted — the file's own directory, or
the directory itself when the rule targets a directory — not the project
root. Pass `min` and/or `max` to require an exact count or a range:

```yaml
exists("package.json")                      # A package.json in this entry's directory
exists("README.*")                          # Any README next to this entry
exists("src/index.*")                       # An index file in a src/ subdirectory
exists("**/*.test.*")                       # Any test files recursively below here

# Exactly 3 matching files in the current directory
exists("*.sql", 3, 3)

# Between 5 and 10 matches
exists("*.log", 5, 10)
```

#### `siblings(pattern)`

Get files in same directory matching pattern.

```yaml
siblings("*")                               # All siblings
siblings("*.js")                            # JavaScript siblings
siblings("*.test.*")                        # Test file siblings
siblings("index.*")                         # Index files
```

#### `children(pattern)`

Get files in subdirectories matching pattern. `children()` only makes
sense against a directory; called against a file, it returns an empty
list rather than an error.

```yaml
children("*")                               # All children
children("*.ts")                            # TypeScript children
children("src/*")                           # Files in src subdirectory

# On a file (not a directory), children() is always empty
count(children("*")) == 0
```

#### `find(path, pattern)`

Find files under `path` matching a glob pattern. The pattern is joined to
the path, so `*` matches one directory level and `**` recurses.

```yaml
find("./src", "*.ts")                       # TypeScript files directly in src
find("./src", "**/*.ts")                    # TypeScript files anywhere under src
find(".", "**/*.test.*")                    # All test files, recursively
find(".", "*")                              # Entries at the top level only
```

## String Templates

String templates embed an expression inside a string literal using
`${...}`; the expression result is substituted into the string.

```yaml
# Every component must have a test named after it
rule: 'in("${$BASENAME}.test.tsx", siblings("*.test.tsx"))'

# A module entry file must sit in a directory of the same name
# (single quotes inside a template inside a double-quoted string)
rule: |-
  endsWith($PARENT, "${without($NAME, '.mod.ts')}")
```

Any expression works inside `${...}` — variables, function calls, or
compositions of both. Templates are for building strings; comparisons and
concatenation-like logic belong in the expression itself (`==`,
`endsWith`, `contains`), not inside the template.

## Complex Expression Examples

### Conditional Logic

```yaml
# If-then-else pattern using logical operators
rule: '$EXT == "ts" && PascalCase || $EXT != "ts"'

# Multiple conditions
rule: '($EXT == "tsx" || $EXT == "jsx") && PascalCase && has-test'
```

### Nested Function Calls

```yaml
# Filter then check
rule: 'count(filter(siblings("*.js"), contains($item, "test"))) > 0'

# Map then validate
rule: 'all(
  map(children("*.ts"), without($item, ".ts")),
  matches($item, /^[A-Z][a-zA-Z0-9]*$/)
)'
```

### Pattern Composition

```yaml
# Combine multiple patterns
rule: '(kebab-case || PascalCase) &&
       (js-file || ts-file) &&
       ! temp-file'

# Complex validation
rule: 'valid-extension &&
       reasonable-length &&
       no-special-chars &&
       (has-test || is-test-file)'
```

## Error Handling

### Common Expression Errors

```yaml
# Type mismatch - NOT operator with non-boolean
rule: '!"string"'             # Error: NOT operator requires a boolean operand

# Invalid function arguments
rule: 'matches($NAME)'        # Error: matches() requires 2 arguments

# Unknown variable
rule: '$UNKNOWN == "test"'    # Error: Unknown variable: UNKNOWN

# Unknown function
rule: 'unknown_func($NAME)'   # Error: Unknown function: unknown_func
```

Not every error surfaces at the same time. Config loading eagerly parses
every rule and every custom matcher and checks that all matcher references
resolve, so a reference to a matcher that doesn't exist — a typo, usually —
fails immediately with the offending rule and its location, before any
file is linted. Function arity and argument-type errors (like the ones
above) can only be caught by actually running the expression, so they
surface the first time a file matching that rule is evaluated — if no file
ever matches, a broken rule like `matches($NAME)` can sit unnoticed.

### Best Practices

```yaml
# Use parentheses for clarity
rule: '(kebab-case || PascalCase) && js-file'

# Break complex expressions into custom matchers
custom-matchers:
  valid-naming: 'kebab-case || PascalCase'
  valid-type: 'js-file || ts-file'
  complete-rule: 'valid-naming && valid-type'

# Validate inputs to functions
rule: 'count($NAME) > 0 && matches($NAME, /pattern/)'
```

Custom matcher names can't be `true` or `false` — the parser resolves
those two words to boolean literals before it looks up matcher
references, so a matcher named either one could never be reached, and
config loading rejects it outright:

```yaml
custom-matchers:
  true: "kebab-case" # Error: Invalid matcher name 'true': shadowed by the boolean literal
```

## Performance Considerations

### Efficient Patterns

```yaml
# Good - local operations
rule: 'count(siblings("*.js")) < 10'

# Avoid - expensive recursive operations
rule: 'count(find(".", "**/*")) < 1000'
```

### Optimization Tips

1. **Use specific patterns**: `siblings("*.js")` vs `siblings("*")`
2. **Avoid deep recursion**: Use `children()` over `find()` when possible
3. **Cache complex expressions**: Use custom matchers for reuse
4. **Order conditions**: Put cheap checks first in AND expressions

```yaml
# Optimized - cheap check first
rule: '$EXT == "js" && count(siblings("*")) < 100'

# Less optimal - expensive check first
rule: 'count(siblings("*")) < 100 && $EXT == "js"'
```

## Cross-References <!-- site:skip -->

- **Getting Started**: See [README.md](../README.md#quick-start) for basic usage examples
- **Common Patterns**: See [COMMON_PATTERNS.md](COMMON_PATTERNS.md) for reusable validation patterns
- **Real-World Examples**: See [EXAMPLES.md](EXAMPLES.md) for complete project configurations
