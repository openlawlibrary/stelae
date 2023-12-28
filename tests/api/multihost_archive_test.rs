use crate::{archive_testtools::config::ArchiveType, common};
use actix_web::test;

#[actix_web::test]
async fn test_resolve_both_guarded_stele_law_html_request_with_full_path_expect_success() {
    let archive_path = common::initialize_archive(ArchiveType::Multihost).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    for guard_value in ["stele_1/law", "stele_2/law"] {
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
    let archive_path = common::initialize_archive(ArchiveType::Multihost).unwrap();
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
    let archive_path = common::initialize_archive(ArchiveType::Multihost).unwrap();
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
    let archive_path = common::initialize_archive(ArchiveType::Multihost).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    for guard_value in ["stele_1/law", "stele_2/law"] {
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
    let archive_path = common::initialize_archive(ArchiveType::Multihost).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    for guard_value in ["stele_1/law", "stele_2/law"] {
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
