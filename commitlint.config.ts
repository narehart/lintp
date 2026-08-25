import type { UserConfig } from "@commitlint/types";

const config: UserConfig = {
  extends: ["@commitlint/config-conventional"],
  // Dependabot writes its own commit bodies, and they carry compare links that
  // run past the 100-character body limit below (140 for a checkout bump).
  // Nothing can shorten them, so its commits are exempt rather than the rule
  // being relaxed for everyone. The header still ends up conventional: the
  // type prefix is pinned per ecosystem in .github/dependabot.yml.
  ignores: [(message) => message.includes("Signed-off-by: dependabot[bot]")],
  rules: {
    "type-enum": [
      2,
      "always",
      [
        "feat",
        "fix",
        "docs",
        "style",
        "refactor",
        "perf",
        "test",
        "build",
        "ci",
        "chore",
        "revert",
      ],
    ],
    "subject-case": [
      2,
      "never",
      ["sentence-case", "start-case", "pascal-case", "upper-case"],
    ],
    "subject-full-stop": [2, "never", "."],
    "subject-empty": [2, "never"],
    "type-case": [2, "always", "lower-case"],
    "type-empty": [2, "never"],
    "scope-case": [2, "always", "lower-case"],
    "header-max-length": [2, "always", 100],
  },
};

export default config;
