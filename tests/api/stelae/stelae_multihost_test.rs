use crate::{archive_testtools::config::ArchiveType, common};
use actix_web::test;

use super::test_stelae_paths;

#[actix_web::test]
async fn test_stelae_api_with_multiple_repositories_expect_success() {
    let archive_path = common::initialize_archive(ArchiveType::Multihost).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    test_stelae_paths(
        "stele_1",
        "law-html",
        vec!["/a/b/c.html"],
        "HEAD",
        &app,
        true,
    )
    .await;

    test_stelae_paths(
        "stele_1_1",
        "law-pdf",
        vec!["/a/b/example.pdf"],
        "HEAD",
        &app,
        true,
    )
    .await;

    test_stelae_paths(
        "stele_1_2",
        "law-xml",
        vec!["/a/b/c/index.xml"],
        "HEAD",
        &app,
        true,
    )
    .await;

    test_stelae_paths("stele_2", "law-rdf", vec!["/a/b/c.rdf"], "HEAD", &app, true).await;
}

#[actix_web::test]
async fn test_stelae_api_where_header_is_not_present_for_root_stele_expect_success() {
    let archive_path = common::initialize_archive(ArchiveType::Multihost).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    let req = test::TestRequest::get()
        .uri("/_stelae/root_stele/law-html?commitish=HEAD&remainder=/index.html")
        .to_request();
    let actual = test::call_and_read_body(&app, req).await;
    let expected = "<!DOCTYPE html>";
    assert!(
        common::blob_to_string(actual.to_vec()).starts_with(expected),
        "doesn't start with {expected}"
    );
}

#[actix_web::test]
async fn test_stelae_api_where_header_is_not_present_for_non_root_stele_expect_error() {
    let archive_path = common::initialize_archive(ArchiveType::Multihost).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    let req = test::TestRequest::get()
        .uri("/_stelae/stele_1/law-html?commitish=HEAD&remainder=/index.html")
        .to_request();
    let actual = test::call_and_read_body(&app, req).await;
    let expected = "Organization name is different from namespace path segment";
    assert!(
        common::blob_to_string(actual.to_vec()).starts_with(expected),
        "doesn't start with {expected}"
    );
}
