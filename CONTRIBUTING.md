# Contributing to lintp

## Commit Message Guidelines

This project enforces [Conventional Commits](https://www.conventionalcommits.org/) for all commit messages.

### Commit Message Format

Each commit message consists of a **header**, an optional **body**, and an optional **footer**.

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes that affect the build system or external dependencies
- **ci**: Changes to CI configuration files and scripts
- **chore**: Other changes that don't modify src or test files
- **revert**: Reverts a previous commit

### Making Commits

You have two options for creating commits:

1. **Interactive mode** (recommended for beginners):

   ```bash
   npm run commit
   ```

   This will guide you through creating a properly formatted commit message.

2. **Manual mode**:
   ```bash
   git commit -m "type: subject"
   ```
   Example: `git commit -m "feat: add file pattern validation"`

### Enforcement

- **Local**: Commit messages are validated by commitlint via husky git hooks
- **CI**: Pull requests are checked for conventional commits in GitHub Actions

If your commit message doesn't follow the convention, the commit will be rejected with helpful error messages.

## Release Process

This project uses [Release Please](https://github.com/googleapis/release-please) to automate releases. The release process is fully automated based on conventional commit messages:

### How It Works

1. **Automatic PR Creation**: When you merge commits to `main`, Release Please will:

   - Analyze the commit messages since the last release
   - Determine the appropriate version bump (major, minor, or patch)
   - Create or update a release PR with:
     - Updated version in `Cargo.toml` and `package.json`
     - Generated CHANGELOG entries
     - Release notes

2. **Version Bumping Rules**:

   - `fix:` commits → patch version bump (0.0.X)
   - `feat:` commits → minor version bump (0.X.0)
   - `feat!:` or `fix!:` commits (breaking changes) → major version bump (X.0.0)

3. **Release Creation**: When the release PR is merged:
   - A GitHub release is created with the new tag, with binaries and
     SHA256 checksums for all 5 platforms attached
   - Platform binary packages (`lintp-darwin-arm64` etc.) and the main
     wrapper package (`lintp-cli` — npm reserves the bare name; the
     installed command is still `lintp`) are published to npm
   - The `lintp` crate is published to crates.io

### Manual Release (Emergency Only)

The release workflow triggers on pushes to `main`, not on tags — pushing
a tag by hand does nothing. If the automated process fails partway:

1. Fix the cause, land the fix on `main` as a `fix:` commit
2. Merge the release PR that Release Please opens; the next release
   re-publishes everything consistently
3. For a stuck npm publish only, the `Publish to NPM` workflow can be
   run manually from the Actions tab (workflow_dispatch)

### Notes

- Never manually edit the CHANGELOG.md - it's automatically generated
- Version numbers are managed by Release Please - don't update them manually
- All releases follow [Semantic Versioning](https://semver.org/)
