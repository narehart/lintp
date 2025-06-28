module.exports = {
  extends: ["@commitlint/config-conventional"],
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
    "scope-case": [2, "always", "lower-case"],
    "subject-case": [2, "never", ["start-case", "pascal-case", "upper-case"]],
    "subject-full-stop": [2, "never", "."],
    "subject-empty": [2, "never"],
    "type-case": [2, "always", "lower-case"],
    "type-empty": [2, "never"],
  },
};
