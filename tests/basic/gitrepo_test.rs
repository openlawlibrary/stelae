use stelae::utils::git::{Repo, GIT_REQUEST_NOT_FOUND};

use crate::common::{self, BASIC_MODULE_NAME};

const COMMIT: &str = "4ba432f61eec15194db527548be4cbc0105635b9";

#[test]
fn test_get_bytes_at_path_when_empty_path_expect_index_html() {
    common::initialize_git();
    let test_archive_path = common::get_test_archive_path(BASIC_MODULE_NAME);
    let repo = Repo::new(&test_archive_path, "test", "law-html").unwrap();
    let actual = repo.get_bytes_at_path(COMMIT, "").unwrap();
    let expected = "<!DOCTYPE html>";
    assert!(
        common::blob_to_string(actual.content).starts_with(expected),
        "doesn't start with {expected}"
    );
}

#[test]
fn test_get_bytes_at_path_when_full_path_expect_data() {
    common::initialize_git();
    let test_archive_path = common::get_test_archive_path(BASIC_MODULE_NAME);
    let repo = Repo::new(&test_archive_path, "test", "law-html").unwrap();
    let actual = repo.get_bytes_at_path(COMMIT, "a/b/c.html").unwrap();
    let expected = "<!DOCTYPE html>";
    assert!(
        common::blob_to_string(actual.content).starts_with(expected),
        "doesn't start with {expected}"
    );
}

#[test]
fn test_get_bytes_at_path_when_omit_html_expect_data() {
    common::initialize_git();
    let test_archive_path = common::get_test_archive_path(BASIC_MODULE_NAME);
    let repo = Repo::new(&test_archive_path, "test", "law-html").unwrap();
    let actual = repo.get_bytes_at_path(COMMIT, "a/b/c").unwrap();
    let expected = "<!DOCTYPE html>";
    assert!(
        common::blob_to_string(actual.content).starts_with(expected),
        "doesn't start with {expected}"
    );
}

#[test]
fn test_get_bytes_at_path_when_omit_index_expect_data() {
    common::initialize_git();
    let test_archive_path = common::get_test_archive_path(BASIC_MODULE_NAME);
    let repo = Repo::new(&test_archive_path, "test", "law-html").unwrap();
    let actual = repo.get_bytes_at_path(COMMIT, "a/b/d").unwrap();
    let expected = "<!DOCTYPE html>";
    assert!(
        common::blob_to_string(actual.content).starts_with(expected),
        "doesn't start with {expected}"
    );
}

#[test]
fn test_get_bytes_at_path_when_invalid_repo_namespace_expect_error() {
    common::initialize_git();
    let test_archive_path = common::get_test_archive_path(BASIC_MODULE_NAME);
    let actual = Repo::new(&test_archive_path, "xxx", "law-html").unwrap_err();
    let expected = "failed to resolve path";
    assert!(
        actual.to_string().contains(expected),
        "\"{actual}\" doesn't contain {expected}"
    );
}

#[test]
fn test_get_bytes_at_path_when_invalid_repo_name_expect_error() {
    common::initialize_git();
    let test_archive_path = common::get_test_archive_path(BASIC_MODULE_NAME);
    let actual = Repo::new(&test_archive_path, "test", "xxx").unwrap_err();
    let expected = "failed to resolve path";
    assert!(
        actual.to_string().contains(expected),
        "\"{actual}\" doesn't contain {expected}"
    );
}

#[test]
fn test_get_bytes_at_path_when_invalid_path_expect_error() {
    common::initialize_git();
    let test_archive_path = common::get_test_archive_path(BASIC_MODULE_NAME);
    let repo = Repo::new(&test_archive_path, "test", "law-html").unwrap();
    let actual = repo.get_bytes_at_path(COMMIT, "a/b/x").unwrap_err();
    let expected = GIT_REQUEST_NOT_FOUND;
    assert!(
        actual.to_string().contains(expected),
        "\"{actual}\" doesn't contain {expected}"
    );
}

#[test]
fn test_get_bytes_at_path_expect_etag() {
    common::initialize_git();
    let test_archive_path = common::get_test_archive_path(BASIC_MODULE_NAME);
    let repo = Repo::new(&test_archive_path, "test", "law-html").unwrap();
    let actual = repo.get_bytes_at_path(COMMIT, "").unwrap();
    assert!(!actual.blob_hash.to_string().is_empty(), "empty etag");
}
