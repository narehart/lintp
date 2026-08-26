import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    globals: true,
    environment: "node",
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html"],
      all: true,
      include: ["src/**/*.ts", "scripts/**/*.ts"],
      exclude: [
        "src/**/*.d.ts",
        "src/**/*.test.ts",
        "src/**/*.spec.ts",
        "**/*.config.ts",
        "**/vitest.config.mts",
        "**/commitlint.config.ts",
      ],
      // The gate for the TypeScript half, enforced by vitest itself. The Rust
      // half is gated the equivalent way, by --fail-under-lines in the
      // coverage:rust task.
      //
      // The CLI entry guards (`require.main === module`) carry v8-ignore
      // comments: they run on every real invocation but can never be true
      // under the test runner, so counting them only dilutes the number.
      thresholds: {
        lines: 90,
        statements: 90,
        functions: 90,
        branches: 90,
      },
    },
  },
});
