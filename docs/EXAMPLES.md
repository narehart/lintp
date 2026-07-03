# lintp Configuration Examples

This document provides practical, real-world examples of lintp configurations for different project types and scenarios.

## Table of Contents

- [Basic Examples](#basic-examples)
- [Framework-Specific Examples](#framework-specific-examples)
  - [React Application](#react-application)
  - [Next.js Application](#nextjs-application)
  - [Vue.js Application](#vuejs-application)
  - [Angular Application](#angular-application)
- [Backend Examples](#backend-examples)
  - [Express.js API](#expressjs-api)
  - [Microservices Architecture](#microservices-architecture)
- [Project Type Examples](#project-type-examples)
  - [Full-Stack Monorepo](#full-stack-monorepo)
  - [Library/Package Development](#librarypackage-development)
- [Specialized Configurations](#specialized-configurations)
  - [Documentation-Heavy Project](#documentation-heavy-project)
  - [Multi-Language Project](#multi-language-project)
  - [Configuration Management](#configuration-management)
- [Validation Examples](#validation-examples)
  - [Test Coverage Requirements](#test-coverage-requirements)
  - [Security Validation](#security-validation)

## Basic Examples

### Simple JavaScript Project

```yaml
lintp:
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"

  config:
    .js: "kebab-case"
    .json: "kebab-case"
    .dir: "kebab-case"

  ignore:
    - node_modules
    - dist
```

### TypeScript Project with Mixed Conventions

```yaml
lintp:
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"

  config:
    .ts: "PascalCase" # Classes && interfaces
    .js: "kebab-case" # Utilities && scripts
    .json: "kebab-case" # Config files
    .dir: "kebab-case" # All directories

  ignore:
    - node_modules
    - dist
    - coverage
```

## Framework-Specific Examples

### React Application

```yaml
lintp:
  custom-matchers:
    # Naming conventions
    PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    camelCase: "matches($BASENAME, /^[a-z][a-zA-Z0-9]*$/)"

    # File type identification
    component-file: 'in($EXT, ["tsx", "jsx"])'
    hook-file: 'startsWith($BASENAME, "use") && in($EXT, ["ts", "js"])'
    context-file: 'endsWith($BASENAME, "Context")'

    # Structure validation
    has-test: 'any(siblings("*.test.*"), contains($item, $BASENAME))'

  config:
    # React components must be PascalCase
    .tsx: "PascalCase && component-file"
    .jsx: "PascalCase && component-file"

    # Hooks must start with "use"
    .ts: "hook-file || kebab-case || PascalCase"
    .js: "kebab-case || camelCase"

    # Test files
    .test.tsx: "PascalCase || kebab-case"
    .test.ts: "PascalCase || kebab-case"
    .spec.tsx: "PascalCase || kebab-case"

    # Stories for Storybook
    .stories.tsx: "PascalCase"

    # Style files
    .css: "kebab-case || PascalCase"
    .scss: "kebab-case || PascalCase"

    # Config and other files
    .json: "kebab-case"
    .md: 'kebab-case || in($BASENAME, ["README", "CHANGELOG"])'

    # Directories
    .dir: "kebab-case || PascalCase"

  ignore:
    - node_modules
    - build
    - dist
    - .next
    - coverage
    - public
```

### Next.js Application

```yaml
lintp:
  custom-matchers:
    # Naming conventions
    PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"

    # Next.js specific patterns
    page-file: 'contains($PATH, "/pages/") || contains($PATH, "/app/")'
    api-route: 'contains($PATH, "/api/")'
    layout-file: 'in($BASENAME, ["layout", "_app", "_document"])'

    # File validations
    valid-page: "page-file && (kebab-case || layout-file)"
    valid-api: "api-route && kebab-case"

  config:
    # Pages directory
    .tsx: "PascalCase || valid-page"
    .ts: "kebab-case || valid-api || PascalCase"
    .js: "kebab-case || valid-page"

    # API routes
    .api.ts: "valid-api"
    .api.js: "valid-api"

    # Regular files
    .json: "kebab-case"
    .dir: 'kebab-case || in($BASENAME, ["pages", "components", "api", "lib", "styles"])'

  ignore:
    - node_modules
    - .next
    - out
    - dist
```

### Vue.js Application

```yaml
lintp:
  custom-matchers:
    PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"

    # Vue-specific patterns
    vue-component: '$EXT == "vue" && PascalCase'
    vue-page: 'contains($PATH, "/pages/") && $EXT == "vue"'
    vue-layout: 'contains($PATH, "/layouts/") && $EXT == "vue"'

  config:
    .vue: "PascalCase"
    .ts: "kebab-case || PascalCase"
    .js: "kebab-case"
    .json: "kebab-case"
    .dir: "kebab-case"

  ignore:
    - node_modules
    - dist
    - .nuxt
```

### Angular Application

```yaml
lintp:
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"

    # Angular file patterns
    component-file: 'endsWith($BASENAME, ".component")'
    service-file: 'endsWith($BASENAME, ".service")'
    module-file: 'endsWith($BASENAME, ".module")'
    directive-file: 'endsWith($BASENAME, ".directive")'
    pipe-file: 'endsWith($BASENAME, ".pipe")'
    guard-file: 'endsWith($BASENAME, ".guard")'

    # Angular structure validation
    angular-file: "component-file || service-file || module-file || directive-file || pipe-file || guard-file"

  config:
    .ts: 'kebab-case && (angular-file || contains($PATH, "/src/"))'
    .html: "kebab-case"
    .css: "kebab-case"
    .scss: "kebab-case"
    .spec.ts: "kebab-case"
    .json: "kebab-case"
    .dir: "kebab-case"

  ignore:
    - node_modules
    - dist
    - coverage
```

## Backend Examples

### Express.js API

```yaml
lintp:
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"

    # Backend patterns
    route-file: 'contains($PATH, "/routes/")'
    controller-file: 'contains($PATH, "/controllers/") && endsWith($BASENAME, "Controller")'
    model-file: 'contains($PATH, "/models/")'
    middleware-file: 'contains($PATH, "/middleware/")'
    util-file: 'contains($PATH, "/utils/")'

    # Structure validation
    has-test: 'any(siblings("*.test.js"), true) || any(find("./tests", "*"), contains($item, $BASENAME))'

  config:
    .js: "kebab-case || (PascalCase && model-file)"
    .ts: "kebab-case || (PascalCase && model-file)"
    .json: "kebab-case"
    .dir: "kebab-case"

  ignore:
    - node_modules
    - dist
    - logs
    - coverage
```

### Microservices Architecture

```yaml
lintp:
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"

    # Service structure
    service-dir: 'exists("package.json") && exists("src/index.js")'
    has-dockerfile: 'exists("Dockerfile")'
    has-readme: 'exists("README.md")'

    # Complete service validation
    valid-service: "service-dir && has-dockerfile && has-readme"

  config:
    .js: "kebab-case"
    .ts: "kebab-case"
    .json: "kebab-case"
    .yml: "kebab-case"
    .yaml: "kebab-case"
    .dir: "kebab-case && (! service-dir || valid-service)"

  ignore:
    - node_modules
    - "*/dist"
    - "*/coverage"
    - "*/logs"
```

## Project Type Examples

### Full-Stack Monorepo

```yaml
lintp:
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"

    # Package identification
    is-frontend: 'contains($PATH, "/frontend/") || contains($PATH, "/client/")'
    is-backend: 'contains($PATH, "/backend/") || contains($PATH, "/server/")'
    is-shared: 'contains($PATH, "/shared/") || contains($PATH, "/common/")'

    # Frontend rules
    frontend-component: 'is-frontend && in($EXT, ["tsx", "jsx"]) && PascalCase'
    frontend-util: "is-frontend && kebab-case"

    # Backend rules
    backend-file: "is-backend && kebab-case"

    # Shared library rules
    shared-file: "is-shared && kebab-case"

  config:
    .tsx: "frontend-component || shared-file"
    .jsx: "frontend-component"
    .ts: "backend-file || frontend-util || shared-file"
    .js: "backend-file || frontend-util || shared-file"
    .json: "kebab-case"
    .dir: "kebab-case"

  ignore:
    - node_modules
    - "*/dist"
    - "*/build"
    - "*/coverage"
```

### Library/Package Development

```yaml
lintp:
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"

    # Library structure
    is-source: 'contains($PATH, "/src/")'
    is-test: 'contains($PATH, "/test/") || contains($NAME, "test")'
    is-example: 'contains($PATH, "/example")'
    is-doc: 'contains($PATH, "/docs/")'

    # Export validation
    has-index: 'exists("src/index.ts") || exists("src/index.js")'
    has-types: 'exists("*.d.ts") || exists("types/*.d.ts")'
    has-package-json: 'exists("package.json")'

    # Complete library validation
    valid-library: "has-index && has-package-json"

  config:
    .ts: "PascalCase || kebab-case"
    .js: "kebab-case"
    .d.ts: "PascalCase || kebab-case"
    .json: "kebab-case"
    .md: 'kebab-case || in($BASENAME, ["README", "CHANGELOG", "CONTRIBUTING"])'
    .dir: "kebab-case"

  ignore:
    - node_modules
    - dist
    - lib
    - coverage
```

## Specialized Configurations

### Documentation-Heavy Project

```yaml
lintp:
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"

    # Documentation structure
    is-doc: 'in($EXT, ["md", "mdx", "txt"])'
    doc-file: 'is-doc && (kebab-case || in($BASENAME, ["README", "CHANGELOG", "LICENSE", "CONTRIBUTING"]))'

    # Image and asset validation
    is-image: 'in($EXT, ["png", "jpg", "jpeg", "gif", "svg", "webp"])'
    image-file: "is-image && kebab-case"

  config:
    .md: "doc-file"
    .mdx: "doc-file"
    .png: "image-file"
    .jpg: "image-file"
    .jpeg: "image-file"
    .svg: "image-file"
    .json: "kebab-case"
    .dir: "kebab-case"

  ignore:
    - node_modules
```

### Multi-Language Project

```yaml
lintp:
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    snake_case: "matches($BASENAME, /^[a-z0-9]+(?:_[a-z0-9]+)*$/)"
    PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"

    # Language-specific rules
    python-file: '$EXT == "py" && snake_case'
    rust-file: '$EXT == "rs" && snake_case'
    go-file: '$EXT == "go" && snake_case'
    java-file: '$EXT == "java" && PascalCase'
    js-file: 'in($EXT, ["js", "ts"]) && kebab-case'

  config:
    .py: "python-file"
    .rs: "rust-file"
    .go: "go-file"
    .java: "java-file"
    .js: "js-file"
    .ts: "js-file"
    .json: "kebab-case"
    .yml: "kebab-case"
    .yaml: "kebab-case"
    .dir: "kebab-case || snake_case"

  ignore:
    - node_modules
    - target
    - build
    - __pycache__
```

### Configuration Management

```yaml
lintp:
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"

    # Environment-specific configs
    env-config: 'matches($BASENAME, /^[a-z0-9-]+\.(dev|prod|test|staging)$/) || in($BASENAME, ["development", "production", "test"])'

    # Config file validation
    docker-file: 'in($BASENAME, ["Dockerfile", "docker-compose"]) || startsWith($BASENAME, "Dockerfile.")'
    ci-config: 'contains($PATH, "/.github/") || contains($PATH, "/.gitlab/") || in($BASENAME, ["Jenkinsfile"])'

  config:
    .json: "kebab-case || env-config"
    .yml: "kebab-case || env-config"
    .yaml: "kebab-case || env-config"
    .toml: "kebab-case"
    .env: "kebab-case || env-config"
    .dockerfile: "docker-file"
    .dir: "kebab-case"

  ignore:
    - node_modules
    - .git
```

## Validation Examples

### Test Coverage Requirements

```yaml
lintp:
  custom-matchers:
    kebab-case: 'matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)'

    # Test file patterns
    is-test: 'contains($NAME, "test") || contains($NAME, "spec")'

    # Source file that needs tests
    needs-test: 'in($EXT, ["js", "ts"]) && ! is-test && ! contains($PATH, "/node_modules/")'

    # Has corresponding test
    has-test: 'any(
      find(".", "*.test.*"),
      contains(without($item, ".test"), without($NAME, "." + $EXT))
    ) || any(
      find(".", "*.spec.*"),
      contains(without($item, ".spec"), without($NAME, "." + $EXT))
    )'

    # Test coverage validation
    covered-file: '! needs-test || has-test'

  config:
    .js: 'kebab-case && covered-file'
    .ts: 'kebab-case && covered-file'
    .dir: 'kebab-case'

  ignore:
    - node_modules
    - dist
    - coverage
```

### Security Validation

```yaml
lintp:
  custom-matchers:
    kebab-case: 'matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)'

    # Security-sensitive patterns
    no-secrets: '! contains($NAME, "secret") &&
                 ! contains($NAME, "password") &&
                 ! contains($NAME, "key") &&
                 ! contains($NAME, "token")'

    no-temp-files: '! endsWith($NAME, ".tmp") &&
                    ! endsWith($NAME, ".bak") &&
                    ! startsWith($BASENAME, "~")'

    # Safe file extensions
    safe-extensions: 'in($EXT, [
      "js", "ts", "jsx", "tsx", "vue",
      "json", "yml", "yaml", "toml",
      "md", "txt", "html", "css", "scss",
      "png", "jpg", "jpeg", "gif", "svg"
    ])'

  config:
    .js: 'kebab-case && no-secrets && safe-extensions'
    .ts: 'kebab-case && no-secrets && safe-extensions'
    .*: 'no-secrets && no-temp-files && safe-extensions'
    .dir: 'kebab-case && no-secrets'

  ignore:
    - node_modules
    - .git
    - "*.log"
```

These examples demonstrate various real-world scenarios and how to configure lintp for different project types and requirements.

## Cross-References

- **Getting Started**: See [README.md](../README.md#quick-start) for basic setup and usage
- **Complete Language Reference**: See [DSL_REFERENCE.md](DSL_REFERENCE.md) for all operators and functions
- **Reusable Patterns**: See [COMMON_PATTERNS.md](COMMON_PATTERNS.md) for standard naming conventions and validation patterns
