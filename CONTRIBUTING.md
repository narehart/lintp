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
