use std::path::PathBuf;

use super::test_stelae_paths;
use crate::{
    archive_testtools::{
        config::{ArchiveType, MultihostConfig},
        init_secret_repository,
    },
    common,
};
use actix_http::StatusCode;
use actix_web::http::header;
use actix_web::test;

#[actix_web::test]
async fn test_stelae_api_with_multiple_non_root_repositories_expect_success() {
    let archive_path =
        common::initialize_archive(ArchiveType::Multihost(MultihostConfig::Private)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    test_stelae_paths(
        "root_stele",
        "law-html",
        vec!["/a/b/c.html"],
        "HEAD",
        &app,
        true,
        "x-current-documents-guard",
        "root_stele/law-private",
    )
    .await;

    test_stelae_paths(
        "stele_1",
        "law-html",
        vec!["/a/b/c.html"],
        "HEAD",
        &app,
        true,
        "x-current-documents-guard",
        "root_stele/law-private",
    )
    .await;

    test_stelae_paths(
        "stele_1_1",
        "law-pdf",
        vec!["/a/b/example.pdf"],
        "HEAD",
        &app,
        true,
        "x-current-documents-guard",
        "root_stele/law-private",
    )
    .await;

    test_stelae_paths(
        "stele_1_2",
        "law-xml",
        vec!["/a/b/c/index.xml"],
        "HEAD",
        &app,
        true,
        "x-current-documents-guard",
        "root_stele/law-private",
    )
    .await;

    test_stelae_paths(
        "stele_2",
        "law-rdf",
        vec!["/a/b/c.rdf"],
        "HEAD",
        &app,
        true,
        "x-current-documents-guard",
        "root_stele/law-private",
    )
    .await;
}

#[actix_web::test]
async fn test_stelae_api_with_root_repositorie_expect_forbiden() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Multihost(MultihostConfig::Private))
            .unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    let req = test::TestRequest::get()
        .uri("/_archive/root_stele/law-reader-assets-private?path=/password.txt")
        .insert_header((
            header::HeaderName::from_static("x-current-documents-guard"),
            "root_stele/law-private",
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "Expected 403 Forbidden"
    );

    let actual = test::read_body(resp).await;
    let expected = "Forbidden repository";
    assert!(
        common::blob_to_string(actual.to_vec()).starts_with(expected),
        "doesn't start with {expected}"
    );
}

#[actix_web::test]
async fn test_stelae_api_with_wrong_guard_expect_failure() {
    let archive_path =
        common::initialize_archive(ArchiveType::Multihost(MultihostConfig::Private)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    test_stelae_paths(
        "root_stele",
        "law-html",
        vec!["/a/b/c.html"],
        "HEAD",
        &app,
        false,
        "x-current-documents-guard",
        "xxx/xxx",
    )
    .await;
}

#[actix_web::test]
async fn test_stelae_api_with_wrong_header_expect_failure() {
    let archive_path =
        common::initialize_archive(ArchiveType::Multihost(MultihostConfig::Private)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    test_stelae_paths(
        "root_stele",
        "law-html",
        vec!["/a/b/c.html"],
        "HEAD",
        &app,
        false,
        "xxx",
        "root_stele/law-private",
    )
    .await;
}

#[actix_web::test]
async fn test_stelae_api_where_repo_name_is_not_in_repository_json_file_expect_error() {
    let archive_path =
        common::initialize_archive(ArchiveType::Multihost(MultihostConfig::Private)).unwrap();
    let stele_path: PathBuf = archive_path.path().join("stele_1");
    let _ = init_secret_repository(&stele_path);
    let app = common::initialize_app(archive_path.path()).await;

    let req = test::TestRequest::get()
        .uri("/_archive/stele_1/secret_repo?path=/password.txt")
        .insert_header((
            header::HeaderName::from_static("x-current-documents-guard"),
            "root_stele/law-private",
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "Expected 400 Bad Request"
    );

    let actual = test::read_body(resp).await;
    let expected = "Repository is not in list of allowed repositories";
    assert!(
        common::blob_to_string(actual.to_vec()).starts_with(expected),
        "doesn't start with {expected}"
    );
}
