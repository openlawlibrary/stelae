use crate::common;
use crate::common::ArchiveType;
use actix_web::test;
use actix_web::web::Bytes;
use serde_json::Value;

#[actix_web::test]
async fn test_resolve_law_html_request_with_full_path_expect_success() {
    let archive_path = common::initialize_archive(ArchiveType::Basic).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let req = test::TestRequest::get().uri("/a/b/c.html").to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_success();
    let expected = true;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_resolve_law_html_request_with_empty_path_expect_success() {
    let archive_path = common::initialize_archive(ArchiveType::Basic).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_success();
    let expected = true;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_resolve_request_with_incorrect_path_expect_client_error() {
    let archive_path = common::initialize_archive(ArchiveType::Basic).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let req = test::TestRequest::get().uri("/a/b/x").to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_client_error();
    let expected = true;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_law_html_request_content_expect_html_document_retrieved() {
    let archive_path = common::initialize_archive(ArchiveType::Basic).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let req = test::TestRequest::get().uri("/a/b/c.html").to_request();
    let actual = test::call_and_read_body(&app, req).await;
    let expected = "<!DOCTYPE html>";
    assert!(
        common::blob_to_string(actual.to_vec()).starts_with(expected),
        "doesn't start with {expected}"
    );
}
