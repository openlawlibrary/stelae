use stele::utils::git::{Repo, GIT_REQUEST_NOT_FOUND};

use crate::common;

const COMMIT: &str = "ed782e08d119a580baa3067e2ea5df06f3d1cd05";

fn blob_to_string(blob: Vec<u8>) -> String {
    core::str::from_utf8(blob.as_slice()).unwrap().into()
}

#[test]
fn test_get_bytes_at_path_when_empty_path_expect_index_html() {
    common::initialize();
    let test_library_path = common::get_test_library_path();
    let repo = Repo::new(&test_library_path, "test", "law-html").unwrap();
    let actual = repo.get_bytes_at_path(COMMIT, "").unwrap();
    let expected = "<!DOCTYPE html>";
    assert!(
        blob_to_string(actual).starts_with(expected),
        "doesn't start with {expected}"
    );
}

#[test]
fn test_get_bytes_at_path_when_full_path_expect_data() {
    common::initialize();
    let test_library_path = common::get_test_library_path();
    let repo = Repo::new(&test_library_path, "test", "law-html").unwrap();
    let actual = repo.get_bytes_at_path(COMMIT, "a/b/c.html").unwrap();
    let expected = "<!DOCTYPE html>";
    assert!(
        blob_to_string(actual).starts_with(expected),
        "doesn't start with {expected}"
    );
}

#[test]
fn test_get_bytes_at_path_when_omit_html_expect_data() {
    common::initialize();
    let test_library_path = common::get_test_library_path();
    let repo = Repo::new(&test_library_path, "test", "law-html").unwrap();
    let actual = repo.get_bytes_at_path(COMMIT, "a/b/c").unwrap();
    let expected = "<!DOCTYPE html>";
    assert!(
        blob_to_string(actual).starts_with(expected),
        "doesn't start with {expected}"
    );
}

#[test]
fn test_get_bytes_at_path_when_omit_index_expect_data() {
    common::initialize();
    let test_library_path = common::get_test_library_path();
    let repo = Repo::new(&test_library_path, "test", "law-html").unwrap();
    let actual = repo.get_bytes_at_path(COMMIT, "a/b/d").unwrap();
    let expected = "<!DOCTYPE html>";
    assert!(
        blob_to_string(actual).starts_with(expected),
        "doesn't start with {expected}"
    );
}

#[test]
fn test_get_bytes_at_path_when_invalid_repo_namespace_expect_error() {
    common::initialize();
    let test_library_path = common::get_test_library_path();
    let actual = Repo::new(&test_library_path, "xxx", "law-html").unwrap_err();
    let expected = "failed to resolve path";
    assert!(
        actual.to_string().contains(expected),
        "\"{actual}\" doesn't contain {expected}"
    );
}

#[test]
fn test_get_bytes_at_path_when_invalid_repo_name_expect_error() {
    common::initialize();
    let test_library_path = common::get_test_library_path();
    let actual = Repo::new(&test_library_path, "test", "xxx").unwrap_err();
    let expected = "failed to resolve path";
    assert!(
        actual.to_string().contains(expected),
        "\"{actual}\" doesn't contain {expected}"
    );
}

#[test]
fn test_get_bytes_at_path_when_invalid_path_expect_error() {
    common::initialize();
    let test_library_path = common::get_test_library_path();
    let repo = Repo::new(&test_library_path, "test", "law-html").unwrap();
    let actual = repo.get_bytes_at_path(COMMIT, "a/b/x").unwrap_err();
    let expected = GIT_REQUEST_NOT_FOUND;
    assert!(
        actual.to_string().contains(expected),
        "\"{actual}\" doesn't contain {expected}"
    );
}
