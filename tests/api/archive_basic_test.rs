use crate::archive_testtools::config::{ArchiveType, Jurisdiction};
use crate::common::{self};
use actix_http::header::IF_NONE_MATCH;
use actix_http::StatusCode;
use actix_web::test;
use stelae::server::headers::HTTP_E_TAG;

#[actix_web::test]
async fn test_resolve_law_html_request_with_full_path_expect_success() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    for request_uri in &["/a/b/c.html", "/a/b/", "/a/b/c/", "/a/d/"] {
        let req = test::TestRequest::get().uri(request_uri).to_request();
        let resp = test::call_service(&app, req).await;
        let actual = resp.status().is_success();
        let expected = true;
        assert_eq!(actual, expected);
    }
}

#[actix_web::test]
async fn test_resolve_root_stele_law_html_request_with_full_path_no_trailing_slash_expect_success()
{
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    for request_uri in &["/a/b/c.html", "/a/b", "/a/b/c", "/a/d"] {
        let req = test::TestRequest::get().uri(request_uri).to_request();
        let resp = test::call_service(&app, req).await;
        let actual = resp.status().is_success();
        let expected = true;
        assert_eq!(actual, expected);
    }
}

#[actix_web::test]
async fn test_resolve_law_html_request_with_empty_path_expect_success() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_success();
    let expected = true;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_resolve_request_with_incorrect_path_expect_client_error() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let req = test::TestRequest::get().uri("/a/b/x").to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_client_error();
    let expected = true;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_law_html_request_content_expect_html_document_retrieved() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    for request_uri in &["/a/b/c.html", "/a/b/", "/a/b/c/", "/a/d/"] {
        let req = test::TestRequest::get().uri(request_uri).to_request();
        let actual = test::call_and_read_body(&app, req).await;
        let expected = "<!DOCTYPE html>";
        assert!(
            common::blob_to_string(actual.to_vec()).starts_with(expected),
            "doesn't start with {expected}"
        );
    }
}

#[actix_web::test]
async fn test_law_xml_request_content_expect_xml_document_retrieved() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    for request_uri in &[
        "/_xml/a/b/index.xml",
        "/_xml/a/d/index.xml",
        "/_xml/a/b/c.xml",
        "/_xml/a/b/c/index.xml",
    ] {
        let req = test::TestRequest::get().uri(request_uri).to_request();
        let actual = test::call_and_read_body(&app, req).await;
        let expected = "<?xml version='1.0' encoding='utf-8'?>";
        assert!(
            common::blob_to_string(actual.to_vec()).starts_with(expected),
            "doesn't start with {expected}"
        );
    }
}

#[actix_web::test]
async fn test_resolve_law_xml_request_without_serve_prefix_expect_client_error() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let req = test::TestRequest::get().uri("/a/b/c.xml").to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_client_error();
    let expected = true;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_law_rdf_request_content_expect_rdf_document_retrieved() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    for request_uri in &[
        "/_rdf/index.rdf",
        "/_rdf/a/b/c.rdf",
        "/_rdf/a/b/index.rdf",
        "/_rdf/a/d/index.rdf",
        "/_rdf/a/b/c/index.rdf",
    ] {
        let req = test::TestRequest::get().uri(request_uri).to_request();
        let actual = test::call_and_read_body(&app, req).await;
        let expected = "<?xml version=\"1.0\"?>";
        assert!(
            common::blob_to_string(actual.to_vec()).starts_with(expected),
            "doesn't start with {expected}"
        );
    }
}

#[actix_web::test]
async fn test_law_other_data_fallback_request_content_expect_document_retrieved() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let req = test::TestRequest::get().uri("/example.json").to_request();
    let actual = test::call_and_read_body(&app, req).await;
    let expected = "{ \"retrieved\": {\"json\": { \"key\": \"value\" } } }";
    assert!(
        common::blob_to_string(actual.to_vec()).starts_with(expected),
        "doesn't start with {expected}"
    );
}

#[actix_web::test]
async fn test_law_other_data_request_content_expect_other_document_retrieved() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    for request_uri in &[
        "/_prefix/a/index.html",
        "/a/_doc/e/index.html",
        "/a/e/_doc/f/index.html",
    ] {
        let req = test::TestRequest::get().uri(request_uri).to_request();
        let actual = test::call_and_read_body(&app, req).await;
        let expected = "<!DOCTYPE html>";
        assert!(
            common::blob_to_string(actual.to_vec()).starts_with(expected),
            "doesn't start with {expected}"
        );
    }
}

#[actix_web::test]
async fn get_law_pdf_request_content_expect_success() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    for request_uri in &["/example.pdf", "/a/example.pdf", "/a/b/example.pdf"] {
        let req = test::TestRequest::get().uri(request_uri).to_request();
        let resp = test::call_service(&app, req).await;
        let actual = resp.status().is_success();
        let expected = true;
        assert_eq!(actual, expected);
    }
}

#[actix_web::test]
async fn get_law_pdf_request_with_incorrect_path_expect_not_found() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let req = test::TestRequest::get()
        .uri("/does-not-exist.pdf")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_client_error();
    let expected = true;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn get_law_html_request_with_no_if_no_match_header_expect_new_etag() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let req = test::TestRequest::get().uri("/a/b/c.html").to_request();
    let resp = test::call_service(&app, req).await;

    let etag = resp.headers().get(HTTP_E_TAG);
    assert!(etag.is_some(), "ETag header is missing");
}

#[actix_web::test]
async fn get_law_html_request_with_if_no_match_header_expect_not_modified() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let file_hash = "d8356a575732fe015dddcf3fa2f23f2ace98b712";
    let req = test::TestRequest::get()
        .uri("/a/b/c.html")
        .append_header((IF_NONE_MATCH, file_hash))
        .to_request();
    let resp = test::call_service(&app, req).await;

    let etag = resp.headers().get(HTTP_E_TAG);
    assert!(etag.is_some(), "ETag header is missing");

    assert_eq!(
        resp.status(),
        StatusCode::NOT_MODIFIED,
        "Expected 304 Not Modified"
    );
    assert_eq!(etag.expect("error"), file_hash);
}

#[actix_web::test]
async fn get_law_html_request_with_old_if_no_match_header_expect_new_tag() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let file_hash = "d8356a575732fe015dddcf3fa2f23f2ace98b712";
    let old_file_hash = "0000000000000000000000000000000000000000000";
    let req = test::TestRequest::get()
        .uri("/a/d/")
        .append_header((IF_NONE_MATCH, old_file_hash))
        .to_request();
    let resp = test::call_service(&app, req).await;

    let etag = resp.headers().get(HTTP_E_TAG);
    assert!(etag.is_some(), "ETag header is missing");

    assert_eq!(resp.status(), StatusCode::OK, "Expected 200 OK");
    assert_eq!(etag.expect("error"), file_hash);
}
