# Changelog

## 0.5.0 (2026-07-05)

## What's Changed
* feat: stricter dogfood config and root cleanup by @narehart in https://github.com/narehart/lintp/pull/54


**Full Changelog**: https://github.com/narehart/lintp/compare/v0.4.1...v0.5.0

## 0.4.1 (2026-07-05)

## What's Changed
* fix: expose path and parent dsl variables as strings by @narehart in https://github.com/narehart/lintp/pull/51
* fix: fail config load on silently inert shapes by @narehart in https://github.com/narehart/lintp/pull/52


**Full Changelog**: https://github.com/narehart/lintp/compare/v0.4.0...v0.4.1

## 0.4.0 (2026-07-04)

## What's Changed
* feat: dogfood lintp on its own repo in ci by @narehart in https://github.com/narehart/lintp/pull/49


**Full Changelog**: https://github.com/narehart/lintp/compare/v0.3.3...v0.4.0

## 0.3.3 (2026-07-04)

## What's Changed
* ci: run the release pr flow entirely under the github app token by @narehart in https://github.com/narehart/lintp/pull/46
* ci: gate release-pr auto-merge without fromjson by @narehart in https://github.com/narehart/lintp/pull/47


**Full Changelog**: https://github.com/narehart/lintp/compare/v0.3.2...v0.3.3

## 0.3.2 (2026-07-04)

## What's Changed
* fix: raise the rust floor to 1.85 and create release prs with the app token by @narehart in https://github.com/narehart/lintp/pull/43
* fix: kick release pr checks via app close/reopen by @narehart in https://github.com/narehart/lintp/pull/44


**Full Changelog**: https://github.com/narehart/lintp/compare/v0.3.1...v0.3.2

## 0.3.1 (2026-07-04)

## What's Changed
* fix: remove the prose measure cap so text aligns with panels by @narehart in https://github.com/narehart/lintp/pull/40
* ci: auto-merge release-please prs once checks pass by @narehart in https://github.com/narehart/lintp/pull/42


**Full Changelog**: https://github.com/narehart/lintp/compare/v0.3.0...v0.3.1

## 0.3.0 (2026-07-04)

## What's Changed
* fix: publish the npm wrapper as lintp-cli by @narehart in https://github.com/narehart/lintp/pull/36
* docs: add cargo install instructions to readme by @narehart in https://github.com/narehart/lintp/pull/38
* feat: redesign docs homepage and deduplicate readme by @narehart in https://github.com/narehart/lintp/pull/39


**Full Changelog**: https://github.com/narehart/lintp/compare/v0.2.2...v0.3.0

## 0.2.2 (2026-07-04)

## What's Changed
* docs: add terminal demo gif to readme by @narehart in https://github.com/narehart/lintp/pull/30
* fix: cross-compile linker, release-please config mode, and version sync by @narehart in https://github.com/narehart/lintp/pull/33
* fix: add front matter so the docs site renders README.html by @narehart in https://github.com/narehart/lintp/pull/34


**Full Changelog**: https://github.com/narehart/lintp/compare/v0.2.1...v0.2.2

## [0.2.1](https://github.com/narehart/lintp/compare/v0.2.0...v0.2.1) (2026-07-03)


### Bug Fixes

* exclude release-please changelog from prettier ([566f27e](https://github.com/narehart/lintp/commit/566f27efbeb6e15ff88ef1e31a57c4cd01765725))
* use platform-native toolchain actions in binary build matrix ([2dbddbe](https://github.com/narehart/lintp/commit/2dbddbe47d20ea86e61b11a3f4e77f513265bf08))

## [0.2.0](https://github.com/narehart/lintp/compare/v0.1.3...v0.2.0) (2026-07-03)


### Features

* add comprehensive TypeScript testing with Vitest ([3bb0336](https://github.com/narehart/lintp/commit/3bb03364bc2f401b5c62ae680e7431ac27b9da52))
* add conventional commits enforcement with commitlint and husky ([6c423f4](https://github.com/narehart/lintp/commit/6c423f43bc1ca427f258c06987368716c011b2b1))
* add coverage check commands with failure thresholds ([465aff4](https://github.com/narehart/lintp/commit/465aff47a0297e22d35cf61b09a5804ddf877a43))
* add ESLint with modern flat config syntax ([ebc4d0b](https://github.com/narehart/lintp/commit/ebc4d0b015fb8e5ab0c92970b2c71ddf658f7677))
* add pre-push hook to run lint, type check, prettier, and tests ([929b352](https://github.com/narehart/lintp/commit/929b352b347c87561a4866f86386441d1b426e72))
* add Rust formatting and linting scripts to package.json ([9ad1a8b](https://github.com/narehart/lintp/commit/9ad1a8b625f9f66af2df19c1ca6a620dc74fb7a0))
* add wireit for enhanced build orchestration ([dffdba4](https://github.com/narehart/lintp/commit/dffdba41769e9f80a21d595832a4466bf31f2830))
* convert all JavaScript files to TypeScript ([cd00963](https://github.com/narehart/lintp/commit/cd0096338e15b94ca2378fe8b3d1d8864dd8414f))
* distribute prebuilt binaries via npm optionalDependencies ([00a751a](https://github.com/narehart/lintp/commit/00a751a7f3e03166ea1845d597793a3a4a80a071))
* fix lambda evaluation and add explain-mode, eager validation, per-rule messages ([83f6038](https://github.com/narehart/lintp/commit/83f60382bf688dc54d15173956df158ea38b03dd))
* improve TypeScript build configuration ([895d306](https://github.com/narehart/lintp/commit/895d306a95cb1d58c995a2cb765f9e047dabcc10))
* integrate google release-please for automated releases ([12fdbdc](https://github.com/narehart/lintp/commit/12fdbdc4d9e46997e5cebb3e9f61fa7925bac099))


### Bug Fixes

* add contents read permission to PR checks workflow ([80240c2](https://github.com/narehart/lintp/commit/80240c2ea9b6f62bf1c012adcc0a18ff26a1d075))
* add pull-requests read permission to workflow ([e913ace](https://github.com/narehart/lintp/commit/e913ace9dadc19e8fa85087491559643ec87f1d1))
* add type check for trace.stats.Line ([9705a8f](https://github.com/narehart/lintp/commit/9705a8f5769750e3412a02b6b5ebb564146aae8d))
* correct import order for eslint ([d6a046f](https://github.com/narehart/lintp/commit/d6a046f50b32a55f41211b81a9e37f287b3a07fc))
* install vite explicitly to resolve ESM compatibility issue ([00479d1](https://github.com/narehart/lintp/commit/00479d167a361bd940bc28224e00f1de190809bc))
* remove all 'any' types and enable strict TypeScript checking ([a08ea25](https://github.com/narehart/lintp/commit/a08ea257e5d1ac7002a68c76e870a9d07cf39aea))
* replace problematic fetch-gh-release-asset with gh CLI ([d70ba99](https://github.com/narehart/lintp/commit/d70ba9971ca9d4c29c4102b580582210ae9eeac5))
* resolve clippy and eslint warnings to pass pre-push hook ([b42f17e](https://github.com/narehart/lintp/commit/b42f17e2a5e52912367a84802f9ce2dd6740707b))
* resolve TypeScript errors and linting issues in code coverage utility ([60cf323](https://github.com/narehart/lintp/commit/60cf3237c7548587bb065696d7b91186c2fc7425))
* specify individual tools for asdf install in workflows ([52767ed](https://github.com/narehart/lintp/commit/52767edabd3bd417bf90aba2f34c76979f48292d))
* use correct wireit github action for caching ([0ac514e](https://github.com/narehart/lintp/commit/0ac514e9888cc066ba49052fee5a879471702b65))
* use full plugin URLs for asdf to avoid version conflicts ([1694e1c](https://github.com/narehart/lintp/commit/1694e1ccce79d87a26641f8fb4959dafa1d39982))
* use pull_request event instead of pull_request_target ([29bb2b5](https://github.com/narehart/lintp/commit/29bb2b58b39d64cad69d309e4dd56130fc04945c))


### Reverts

* remove PR checks workflow to create via PR instead ([83a17b1](https://github.com/narehart/lintp/commit/83a17b155c886757bba905449197fb8a351bfe11))

## Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
