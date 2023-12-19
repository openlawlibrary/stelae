use crate::archive_testtools::{ArchiveType, Jurisdiction};
use crate::common;
use actix_web::test;

#[actix_web::test]
async fn test_resolve_root_stele_law_html_request_with_full_path_expect_success() {
    let archive_path = common::initialize_archive(ArchiveType::Basic(Jurisdiction::Multi)).unwrap();
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
async fn test_root_stele_fallback_request_expect_success() {
    let archive_path = common::initialize_archive(ArchiveType::Basic(Jurisdiction::Multi)).unwrap();
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
async fn test_dependent_stele_law_html_request_expect_success() {
    let archive_path = common::initialize_archive(ArchiveType::Basic(Jurisdiction::Multi)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    for request_uri in &[
        "/sub/scope/1/a/b/c.html",
        "/sub/scope/2/a/b/",
        "/sub/scope/3/a/b/c/",
        "/sub/scope/4/a/d/",
        "/sub/scope/1/a/b/c.html",
        "/sub/scope/2/a/b/",
        "/sub/scope/3/a/b/c/",
        "/sub/scope/4/a/d/",
    ] {
        let req = test::TestRequest::get().uri(request_uri).to_request();
        let resp = test::call_service(&app, req).await;
        let actual = resp.status().is_success();
        let expected = true;
        assert_eq!(actual, expected);
    }
}

#[actix_web::test]
async fn test_dependent_stele_fallback_request_when_only_root_fallback_is_supported_expect_not_found(
) {
    let archive_path = common::initialize_archive(ArchiveType::Basic(Jurisdiction::Multi)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    for request_uri in &[
        "/does-not-resolve.json",
        "/a/does-not-resolve.json",
        "/a/b/does-not-resolve.json",
        "/does-not-resolve.html",
        "/a/does-not-resolve.html",
        "/a/b/does-not-resolve.html",
    ] {
        let req = test::TestRequest::get().uri(request_uri).to_request();
        let resp = test::call_service(&app, req).await;
        let actual = resp.status().is_client_error();
        let expected = true;
        assert_eq!(actual, expected);
    }
}

#[actix_web::test]
async fn test_dependent_stele_law_html_request_where_path_does_not_exist_not_found() {
    let archive_path = common::initialize_archive(ArchiveType::Basic(Jurisdiction::Multi)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let req = test::TestRequest::get().uri("/sub/scope/x/").to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_client_error();
    let expected = true;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_root_stele_law_rdf_expect_rdf_document_retrieved() {
    let archive_path = common::initialize_archive(ArchiveType::Basic(Jurisdiction::Multi)).unwrap();
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
async fn test_dependent_stele_law_rdf_expect_not_found() {
    let archive_path = common::initialize_archive(ArchiveType::Basic(Jurisdiction::Multi)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    // Even though the dependent RDF data repository exists, serving underscore routes for dependent stele is not supported
    for request_uri in &[
        "/_rdf/sub/scope/1/index.rdf",
        "/_rdf/sub/scope/2/a/b/c.rdf",
        "/_rdf/sub/scope/3/a/b/index.rdf",
        "/_rdf/sub/scope/4/a/d/index.rdf",
        "/_rdf/sub/scope/1/a/b/c/index.rdf",
        "/_rdf/sub/scope/2/index.rdf",
        "/_rdf/sub/scope/3/a/b/c.rdf",
        "/_rdf/sub/scope/4/a/b/index.rdf",
    ] {
        let req = test::TestRequest::get().uri(request_uri).to_request();
        let resp = test::call_service(&app, req).await;
        let actual = resp.status().is_client_error();
        let expected = true;
        assert_eq!(actual, expected);
    }
}

#[actix_web::test]
async fn test_dependent_stele_law_other_with_full_path_when_request_matches_glob_pattern_expect_success(
) {
    let archive_path = common::initialize_archive(ArchiveType::Basic(Jurisdiction::Multi)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    for request_uri in &[
        "/sub/scope/2/a/_doc/e/index.html",
        "/sub/scope/4/a/e/_doc/f/index.html",
    ] {
        let req = test::TestRequest::get().uri(request_uri).to_request();
        let resp = test::call_service(&app, req).await;
        let actual = resp.status().is_success();
        let expected = true;
        assert_eq!(actual, expected);
    }
}

#[actix_web::test]
async fn test_dependent_stele_law_other_with_full_path_when_underscore_routing_is_not_supported_expect_not_found(
) {
    let archive_path = common::initialize_archive(ArchiveType::Basic(Jurisdiction::Multi)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    // Serving dependent stele routes that start with a `_` prefix glob pattern is only supported for root stele.
    for request_uri in &[
        "/sub/scope/1/_prefix/index.html",
        "/sub/scope/4/_prefix/a/index.html",
    ] {
        let req = test::TestRequest::get().uri(request_uri).to_request();
        let resp = test::call_service(&app, req).await;
        let actual = resp.status().is_client_error();
        let expected = true;
        assert_eq!(actual, expected);
    }
}
