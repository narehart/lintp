use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

use lintp::config::load_config;
use lintp::lint::run_lint;

const CONFIG: &str = r#"
lintp:
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    PascalCase: "matches($BASENAME, /^[A-Z][a-zA-Z0-9]*$/)"
    js-file: '$EXT == "js"'
    ts-file: '$EXT == "ts"'

  config:
    .js: "kebab-case && js-file"
    .ts: "PascalCase && ts-file"
    .dir: "kebab-case || PascalCase"

  ignore:
    - node_modules
"#;

/// Build a fixture tree: `dirs` directories, each containing `files_per_dir`
/// .js and .ts files, so run_lint visits dirs * files_per_dir * 2 files.
fn build_fixture(root: &Path, dirs: usize, files_per_dir: usize) {
    for d in 0..dirs {
        let dir = root.join(format!("module-{}", d));
        fs::create_dir_all(&dir).unwrap();
        for f in 0..files_per_dir {
            fs::write(dir.join(format!("some-file-{}.js", f)), "").unwrap();
            fs::write(dir.join(format!("SomeType{}.ts", f)), "").unwrap();
        }
    }
}

const SIBLINGS_CONFIG: &str = r#"
lintp:
  custom-matchers:
    kebab-case: "matches($BASENAME, /^[a-z0-9]+(?:-[a-z0-9]+)*$/)"
    has-siblings: 'count(siblings("*")) > 0'

  config:
    .js: "kebab-case && has-siblings"
    .ts: "has-siblings"

  ignore:
    - node_modules
"#;

fn bench_run_lint(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    build_fixture(temp.path(), 20, 10);

    let config_path = temp.path().join("lintp.yml");
    fs::write(&config_path, CONFIG).unwrap();
    let config = load_config(&config_path).unwrap();

    c.bench_function("run_lint 400 files", |b| {
        b.iter(|| run_lint(temp.path(), &config, false).unwrap())
    });
}

/// Exercises the per-run glob cache: every file's rule lists its directory,
/// which is O(n^2) filesystem reads per directory without caching.
fn bench_run_lint_siblings(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    build_fixture(temp.path(), 20, 10);

    let config_path = temp.path().join("lintp.yml");
    fs::write(&config_path, SIBLINGS_CONFIG).unwrap();
    let config = load_config(&config_path).unwrap();

    c.bench_function("run_lint 400 files with siblings() rule", |b| {
        b.iter(|| run_lint(temp.path(), &config, false).unwrap())
    });
}

criterion_group!(benches, bench_run_lint, bench_run_lint_siblings);
criterion_main!(benches);
