use crate::{
    archive_testtools::{
        self,
        config::{ArchiveType, MultihostConfig},
        utils,
    },
    common,
};
use actix_http::header::IF_NONE_MATCH;
use actix_http::StatusCode;
use actix_web::test;
use stelae::server::headers::HTTP_E_TAG;

#[actix_web::test]
async fn test_resolve_both_guarded_stele_law_html_request_with_full_path_expect_success() {
    let archive_path =
        common::initialize_archive(ArchiveType::Multihost(MultihostConfig::Public)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    for guard_value in [
        "stele_1/law",
        "stele_2/law",
        "stele_1_1/law",
        "stele_1_2/law",
    ] {
        for request_uri in &["/a/b/c.html", "/a/b/", "/a/b/c/", "/a/d/"] {
            let req = test::TestRequest::get()
                .insert_header(("X-Current-Documents-Guard", guard_value))
                .uri(request_uri)
                .to_request();
            let resp = test::call_service(&app, req).await;
            let actual = resp.status().is_success();
            let expected = true;
            assert_eq!(actual, expected);
        }
    }
}

#[actix_web::test]
async fn test_resolve_guarded_stele_law_html_request_where_header_value_is_incorrect_expect_error()
{
    let archive_path =
        common::initialize_archive(ArchiveType::Multihost(MultihostConfig::Public)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let req = test::TestRequest::get()
        .insert_header(("X-Current-Documents-Guard", "xxx/xxx"))
        .uri("/a/b/c.html")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_client_error();
    let expected = true;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_resolve_guarded_stele_law_html_request_where_header_name_is_incorrect_expect_error() {
    let archive_path =
        common::initialize_archive(ArchiveType::Multihost(MultihostConfig::Public)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let req = test::TestRequest::get()
        .insert_header(("X-Incorrect-Header-Name", "stele_1/law"))
        .uri("/a/b/c.html")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_client_error();
    let expected = true;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_resolve_guarded_stele_law_rdf_request_content_expect_rdf_document_retrieved() {
    let archive_path =
        common::initialize_archive(ArchiveType::Multihost(MultihostConfig::Public)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    for guard_value in [
        "stele_1/law",
        "stele_2/law",
        "stele_1_1/law",
        "stele_1_2/law",
    ] {
        for request_uri in &[
            "/_rdf/index.rdf",
            "/_rdf/a/b/c.rdf",
            "/_rdf/a/b/index.rdf",
            "/_rdf/a/d/index.rdf",
            "/_rdf/a/b/c/index.rdf",
        ] {
            let req = test::TestRequest::get()
                .insert_header(("X-Current-Documents-Guard", guard_value))
                .uri(request_uri)
                .to_request();
            let actual = test::call_and_read_body(&app, req).await;
            let expected = "<?xml version=\"1.0\"?>";
            assert!(
                common::blob_to_string(actual.to_vec()).starts_with(expected),
                "doesn't start with {expected}"
            );
        }
    }
}

#[actix_web::test]
async fn test_law_other_data_request_content_expect_other_document_retrieved() {
    let archive_path =
        common::initialize_archive(ArchiveType::Multihost(MultihostConfig::Public)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    for guard_value in [
        "stele_1/law",
        "stele_2/law",
        "stele_1_1/law",
        "stele_1_2/law",
    ] {
        for request_uri in &[
            "/_prefix/a/index.html",
            "/a/_doc/e/index.html",
            "/a/e/_doc/f/index.html",
        ] {
            let req = test::TestRequest::get()
                .insert_header(("X-Current-Documents-Guard", guard_value))
                .uri(request_uri)
                .to_request();
            let actual = test::call_and_read_body(&app, req).await;
            let expected = "<!DOCTYPE html>";
            assert!(
                common::blob_to_string(actual.to_vec()).starts_with(expected),
                "doesn't start with {expected}"
            );
        }
    }
}

#[actix_web::test]
async fn test_law_other_data_request_content_with_cycle_expect_other_document_retrieved() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Multihost(MultihostConfig::Public))
            .unwrap();
    // Add a cycle
    // stele_1 -> stele_1_1 -> stele_1
    // Expect that the cycle is resolved
    archive_testtools::add_dependencies(archive_path.path(), "stele_1_1", vec!["stele_1"], None)
        .unwrap();
    utils::make_all_git_repos_bare_recursive(&archive_path).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    for guard_value in [
        "stele_1/law",
        "stele_2/law",
        "stele_1_1/law",
        "stele_1_2/law",
    ] {
        for request_uri in &[
            "/_prefix/a/index.html",
            "/a/_doc/e/index.html",
            "/a/e/_doc/f/index.html",
        ] {
            let req = test::TestRequest::get()
                .insert_header(("X-Current-Documents-Guard", guard_value))
                .uri(request_uri)
                .to_request();
            let actual = test::call_and_read_body(&app, req).await;
            let expected = "<!DOCTYPE html>";
            assert!(
                common::blob_to_string(actual.to_vec()).starts_with(expected),
                "doesn't start with {expected}"
            );
        }
    }
}

#[actix_web::test]
async fn get_law_html_request_with_no_if_no_match_header_expect_new_etag() {
    let archive_path =
        common::initialize_archive(ArchiveType::Multihost(MultihostConfig::Public)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    for guard_value in [
        "stele_1/law",
        "stele_2/law",
        "stele_1_1/law",
        "stele_1_2/law",
    ] {
        for request_uri in &["/a/b/c.html", "/a/b/", "/a/b/c/", "/a/d/"] {
            let req = test::TestRequest::get()
                .insert_header(("X-Current-Documents-Guard", guard_value))
                .uri(request_uri)
                .to_request();
            let resp = test::call_service(&app, req).await;
            let etag = resp.headers().get(HTTP_E_TAG);
            assert!(etag.is_some(), "ETag header is missing");
        }
    }
}

#[actix_web::test]
async fn get_law_html_request_with_if_no_match_header_expect_not_modified() {
    let archive_path =
        common::initialize_archive(ArchiveType::Multihost(MultihostConfig::Public)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let file_hash = "d8356a575732fe015dddcf3fa2f23f2ace98b712";
    for guard_value in [
        "stele_1/law",
        "stele_2/law",
        "stele_1_1/law",
        "stele_1_2/law",
    ] {
        for request_uri in &["/a/b/c.html", "/a/b/", "/a/b/c/", "/a/d/"] {
            let req = test::TestRequest::get()
                .insert_header(("X-Current-Documents-Guard", guard_value))
                .uri(request_uri)
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
    }
}

#[actix_web::test]
async fn get_law_html_request_with_old_if_no_match_header_expect_new_tag() {
    let archive_path =
        common::initialize_archive(ArchiveType::Multihost(MultihostConfig::Public)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let file_hash = "d8356a575732fe015dddcf3fa2f23f2ace98b712";
    let old_file_hash = "0000000000000000000000000000000000000000000";
    for guard_value in [
        "stele_1/law",
        "stele_2/law",
        "stele_1_1/law",
        "stele_1_2/law",
    ] {
        for request_uri in &["/a/b/c.html", "/a/b/", "/a/b/c/", "/a/d/"] {
            let req = test::TestRequest::get()
                .insert_header(("X-Current-Documents-Guard", guard_value))
                .uri(request_uri)
                .append_header((IF_NONE_MATCH, old_file_hash))
                .to_request();
            let resp = test::call_service(&app, req).await;
            let etag = resp.headers().get(HTTP_E_TAG);
            assert!(etag.is_some(), "ETag header is missing");

            assert_eq!(resp.status(), StatusCode::OK, "Expected 200 OK");
            assert_eq!(etag.expect("error"), file_hash);
        }
    }
}
