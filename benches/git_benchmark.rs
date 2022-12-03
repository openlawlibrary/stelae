//! benchmark for git utils

use criterion::{criterion_group, criterion_main, Criterion};
use std::env::current_exe;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Once;
use stele::utils::git::Repo;

/// get the path to the test library at $REPO_ROOT/test/library
#[allow(clippy::expect_used)]
fn get_test_library_path() -> PathBuf {
    let mut library_path = current_exe()
        .expect("Something went wrong getting the library path")
        .parent()
        .expect("Something went wrong getting the library path")
        .parent()
        .expect("Something went wrong getting the library path")
        .parent()
        .expect("Something went wrong getting the library path")
        .parent()
        .expect("Something went wrong getting the library path")
        .to_owned();
    library_path.push("test");
    library_path.push("library");
    library_path
}

/// ensure `initialize` function, below, is only called once
static INIT: Once = Once::new();

/// Bare git repo(s) in test library must have `refs/heads` and
/// `refs/tags` folders. They are empty, so not stored in git,
/// so must be created
#[allow(clippy::expect_used)]
pub fn initialize() {
    INIT.call_once(|| {
        let repo_path = get_test_library_path().join(PathBuf::from("test/law-html"));
        let heads_path = repo_path.join(PathBuf::from("refs/heads"));
        create_dir_all(heads_path).expect("Something went wrong creating the refs/heads folder");
        let tags_path = repo_path.join(PathBuf::from("refs/tags"));
        create_dir_all(tags_path).expect("Something went wrong getting the ref/tags folder");
    });
}

/// Measure the speed of the git utils
#[allow(clippy::expect_used)]
fn bench_repo() {
    initialize();
    let test_library_path = get_test_library_path();
    let repo = Repo::new(test_library_path.to_str().unwrap(), "test", "law-html")
        .expect("Something went wrong creating the repo");
    repo.get_bytes_at_path("ed782e08d119a580baa3067e2ea5df06f3d1cd05", "a/b/c.html")
        .expect("Something went wrong calling `get_bytes_at_path`");
}

/// Initialize criterion benchmarks
fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("get_bytes_at_path", |b| b.iter(|| bench_repo()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
