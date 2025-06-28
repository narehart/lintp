# Contributing to lintp

## Commit Message Guidelines

This project uses [Conventional Commits](https://www.conventionalcommits.org/) to standardize commit messages and automate changelog generation.

### Commit Message Format

Each commit message consists of a **header**, an optional **body**, and an optional **footer**:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

Must be one of the following:

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes that affect the build system or external dependencies
- **ci**: Changes to our CI configuration files and scripts
- **chore**: Other changes that don't modify src or test files
- **revert**: Reverts a previous commit

### Scope

The scope should be the name of the module affected (as perceived by the person reading the changelog):

Examples: `parser`, `cli`, `dsl`, `docs`

### Subject

The subject contains a succinct description of the change:

- Use the imperative, present tense: "change" not "changed" nor "changes"
- Don't capitalize the first letter
- No dot (.) at the end

### Examples

```
feat(parser): add support for regex patterns in file matching
fix(cli): handle spaces in file paths correctly
docs: update DSL reference with new operators
refactor(core): simplify directory traversal logic
```

## Making Commits

### Interactive Commit Tool

For an interactive commit experience, use:

```bash
npm run commit
```

This will guide you through creating a properly formatted commit message.

### Manual Commits

You can also make commits manually using `git commit`. The commit message will be validated by commitlint.

## Releases

### Creating a Release

To create a new release:

1. **Dry run** (recommended first):

   ```bash
   npm run release:dry
   ```

2. **Create release**:
   ```bash
   npm run release        # Auto-detect version bump
   npm run release:patch  # Patch release (0.1.0 -> 0.1.1)
   npm run release:minor  # Minor release (0.1.0 -> 0.2.0)
   npm run release:major  # Major release (0.1.0 -> 1.0.0)
   ```

This will:

- Bump the version in `package.json` and `Cargo.toml`
- Update the CHANGELOG.md
- Create a commit with the new version
- Create a git tag

3. **Push changes**:
   ```bash
   git push --follow-tags origin main
   ```

## Local Development

### Git Hooks

This project uses Husky for Git hooks:

- **prepare-commit-msg**: Launches the interactive commit tool
- **commit-msg**: Validates commit messages with commitlint

### Running Tests

Before committing, ensure all tests pass:

```bash
cargo test
```

### Code Formatting

Format your code before committing:

```bash
npm run format
```

Check formatting:

```bash
npm run format:check
```
