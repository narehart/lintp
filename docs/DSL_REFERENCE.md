# lintp DSL Reference

Complete reference for the lintp Domain-Specific Language (DSL) used in configuration files.

## Overview

The lintp DSL is a powerful expression language for defining file and directory validation rules. It supports variables, operators, functions, and complex expressions that can be composed to create sophisticated validation logic.

## Variables

### Built-in File Variables

| Variable    | Description                       | Example Value                  |
| ----------- | --------------------------------- | ------------------------------ |
| `$NAME`     | Full filename including extension | `"Button.tsx"`                 |
| `$BASENAME` | Filename without extension        | `"Button"`                     |
| `$EXT`      | File extension (without dot)      | `"tsx"`                        |
| `$PATH`     | Full file path                    | `"/src/components/Button.tsx"` |
| `$PARENT`   | Parent directory path             | `"/src/components"`            |

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
# Double quotes (supports string templates)
rule: '$NAME == "exact-match.js"'
rule: "matches($NAME, ${pattern_var})"

# Single quotes (literal strings only)
rule: '$NAME == \'literal-string.js\''
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

```yaml
# Basic regex
rule: 'matches($NAME, /^[a-z-]+$/)'

# Complex regex with escaping
rule: 'matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)'

# Regex with special characters
rule: 'matches($NAME, /\.test\.(js|ts)$/)'
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

### Operator Precedence

From highest to lowest precedence:

1. **Unary operators**: `!`, `-` (unary minus)
2. **Comparison**: `==`, `!=`, `<`, `<=`, `>`, `>=`
3. **Logical AND**: `&&`
4. **Logical OR**: `||`

```yaml
# Examples of precedence
rule: '!a == b'        # Equivalent to: (!a) == b
rule: 'a && b || c'    # Equivalent to: (a && b) || c
rule: 'a || b && c'    # Equivalent to: a || (b && c)
rule: 'a == b && c'    # Equivalent to: (a == b) && c
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

Check if string contains substring or list contains item.

```yaml
# String contains substring
contains($NAME, "test")                     # Filename contains "test"
contains($PATH, "/src/")                    # Path contains "/src/"
contains($PARENT, "components")             # Parent contains "components"

# List contains item
contains(["js", "ts", "jsx"], $EXT)         # Extension in list
contains(siblings("*.js"), "index.js")      # Sibling list contains file
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

Get length of string or list.

```yaml
count($NAME)                                # Filename length
count($BASENAME)                            # Basename length
count(siblings("*.js"))                     # Number of JS siblings
count(["a", "b", "c"])                      # List length (3)
```

### Collection Functions

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

Test if any item matches condition.

```yaml
# Check if any sibling is a test file
any(siblings("*"), contains($item, "test"))

# Check if any child is an index file
any(children("*"), startsWith($item, "index"))

# Check for configuration files
any(find(".", "*"), endsWith($item, ".config.js"))
```

#### `all(collection, expression)`

Test if all items match condition.

```yaml
# All siblings follow naming convention
all(siblings("*.js"), matches($item, /^[a-z-]+\.js$/))

# All children are properly named
all(children("*"), !contains($item, " "))

# All TypeScript files have proper extensions
all(
  filter(find(".", "*"), contains($item, "typescript")),
  endsWith($item, ".ts")
)
```

### File System Functions

#### `exists(pattern)`

Check if files matching pattern exist.

```yaml
exists("package.json")                      # Package file exists
exists("README.*")                          # Any README file
exists("src/index.*")                       # Index file in src
exists("**/*.test.*")                       # Any test files recursively
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

Get files in subdirectories matching pattern.

```yaml
children("*")                               # All children
children("*.ts")                            # TypeScript children
children("src/*")                           # Files in src subdirectory
```

#### `find(path, pattern)`

Find files recursively from path.

```yaml
find(".", "*")                              # All files from current dir
find("./src", "*.ts")                       # TypeScript files in src
find(".", "*.test.*")                       # All test files
find("/project", "package.json")            # Package files in project
```

## String Templates

String templates allow embedding expressions within strings using `${...}` syntax.

### Basic Templates

```yaml
# Simple variable substitution
rule: 'matches($NAME, ${pattern_variable})'

# Expression in template
rule: 'contains($PATH, ${$PARENT + "/tests"})'
```

### Advanced Templates

```yaml
# Function result in template
rule: 'matches($NAME, ${siblings("*.config")[0]})'

# Complex expression
rule: 'equals($BASENAME, ${without(find(".", "*.main")[0], ".main")})'
```

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
rule: '!"string"'             # Error: NOT requires boolean

# Invalid function arguments
rule: 'matches($NAME)'        # Error: matches requires 2 arguments

# Unknown variable
rule: '$UNKNOWN == "test"'    # Error: Unknown variable

# Unknown function
rule: 'unknown_func($NAME)'   # Error: Unknown function
```

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

## Cross-References

- **Getting Started**: See [README.md](../README.md#quick-start) for basic usage examples
- **Common Patterns**: See [COMMON_PATTERNS.md](COMMON_PATTERNS.md) for reusable validation patterns
- **Real-World Examples**: See [EXAMPLES.md](EXAMPLES.md) for complete project configurations
