use criterion::{criterion_group, criterion_main, Criterion};
use std::env::current_exe;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Once;
use stele::utils::git::Repo;

fn get_test_library_path() -> PathBuf {
    let mut library_path = current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    library_path.push("test");
    library_path.push("library");
    library_path
}

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        let repo_path = get_test_library_path().join(PathBuf::from("test/law-html"));
        let heads_path = repo_path.join(PathBuf::from("refs/heads"));
        create_dir_all(heads_path).unwrap();
        let tags_path = repo_path.join(PathBuf::from("refs/tags"));
        create_dir_all(tags_path).unwrap();
    });
}

fn bench_repo() {
    initialize();
    let test_library_path = get_test_library_path();
    let repo = Repo::new(test_library_path.to_str().unwrap(), "test", "law-html").unwrap();
    repo.get_bytes_at_path("ed782e08d119a580baa3067e2ea5df06f3d1cd05", "a/b/c.html")
        .unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("get_bytes_at_path", |b| b.iter(|| bench_repo()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
