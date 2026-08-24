# Contributing to lintp

## Local setup

Node.js and Rust versions are pinned in `.tool-versions`; `asdf install` picks
them up. The coverage gate additionally needs tarpaulin:

```bash
asdf install
npm ci
cargo install cargo-tarpaulin   # required by npm run coverage and the pre-push hook
```

## Quality gates

Each language is gated by its own toolchain, so the same command fails the
same way locally and in CI:

| Command                | What it enforces                                                                     |
| ---------------------- | ------------------------------------------------------------------------------------ |
| `npm run check`        | `tsc --noEmit` and `cargo check`                                                     |
| `npm run lint`         | `eslint --max-warnings 0` and `cargo clippy --all-targets -D warnings`               |
| `npm run format:check` | `prettier --check` and `cargo fmt --check`                                           |
| `npm test`             | vitest and `cargo test`                                                              |
| `npm run coverage`     | vitest thresholds (`vitest.config.ts`) and tarpaulin `fail-under` (`tarpaulin.toml`) |

Coverage thresholds live with the tool that enforces them — raise the
TypeScript gate in `vitest.config.ts`, the Rust gate in `tarpaulin.toml`.

Clippy runs at the `pedantic` tier and the library denies `missing_docs`: a
new public item needs a doc comment, because the crate renders on docs.rs.

CI runs [similarity](https://github.com/mizchi/similarity) over both halves of
the repo to catch copy-pasted logic, and both checks fail the build.

`similarity-ts` runs at its default threshold and the TypeScript sources are
clean at it. For Rust, `scripts/check-similarity.sh` runs `similarity-rs` at
the same default sensitivity and compares every reported pair against
`scripts/similarity-baseline.txt`; anything not listed there fails. Raising the
threshold instead would not work — two functions differing only in
prefix/suffix score 91.78%, so a gate set above the known pairs would miss real
copy-paste.

The baseline is a list to shrink. Add to it only when two functions genuinely
share a shape rather than an implementation (caller and callee, or two
dispatchers), and write down why. Run it locally with:

```bash
cargo install similarity-rs similarity-ts --locked
./scripts/check-similarity.sh
```

The pre-push hook runs lint, type check, format check and coverage. Wireit
caches each task, so an unchanged tree re-runs almost nothing.

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

## Docs Site

The docs site (https://narehart.github.io/lintp/) is built by `scripts/build-docs.ts` — the same script locally and in CI, so what you preview is byte-for-byte what deploys. Sources are the repo README plus the markdown files in `docs/`; the design system (tokens and components) lives in `docs/assets/docs.css`.

To preview changes for sign-off before merging:

```bash
npm run docs:build     # renders everything into _site/ (gitignored)
npm run docs:preview   # serves it at http://localhost:8931
```

The site deploys automatically on merge to `main` when docs sources change.

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
     SHA256 checksums for all 7 platforms attached
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
