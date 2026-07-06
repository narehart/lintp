import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    ignores: [
      "dist/**",
      "node_modules/**",
      "bin/**",
      "*.d.ts",
      ".wireit/**",
      "target/**",
      "coverage/**",
      "_site/**",
    ],
  },
  {
    files: ["**/*.ts", "**/*.mts"],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "module",
      parserOptions: {
        project: "./tsconfig.json",
      },
    },
    rules: {
      "@typescript-eslint/explicit-function-return-type": "off",
      "@typescript-eslint/no-explicit-any": "warn",
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
      "no-console": "warn",
      "prefer-const": "error",
      "no-var": "error",
      "object-shorthand": "error",
      "prefer-template": "error",
      "prefer-destructuring": [
        "error",
        {
          array: false,
          object: true,
        },
      ],
      "prefer-arrow-callback": "error",
      "arrow-body-style": ["error", "as-needed"],
      "no-duplicate-imports": "error",
      "sort-imports": [
        "error",
        {
          ignoreDeclarationSort: true,
          ignoreCase: true,
        },
      ],
    },
  },
  {
    files: ["scripts/**/*.ts", "index.ts", "src/index.ts"],
    rules: {
      "no-console": "off",
    },
  }
);
