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
      // half is gated the equivalent way, by fail-under in tarpaulin.toml.
      // Branches and functions sit lower than lines because the npm
      // launcher's platform-detection paths can't all run on one host.
      thresholds: {
        lines: 70,
        statements: 70,
        functions: 60,
        branches: 60,
      },
    },
  },
});
