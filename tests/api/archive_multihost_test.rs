use crate::{
    archive_testtools::{
        self, add_redirects_json_file,
        config::{ArchiveType, MultihostConfig},
        utils,
    },
    common,
};
use actix_http::StatusCode;
use actix_web::test;
use std::path::PathBuf;

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
async fn test_redirect_dependant_stele_law_html_request_with_correct_redirects_json_expect_success()
{
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Multihost(MultihostConfig::Public))
            .unwrap();

    let stelae_1_html_repo_path: PathBuf = archive_path.path().join("stele_1/law-html");

    let file_content = r#"
    [
        [
            "/not/a/good/path",
            "/"
        ]
    ]
    "#
    .to_string();

    let _ = add_redirects_json_file(&stelae_1_html_repo_path, file_content);
    let app = common::initialize_app(archive_path.path()).await;

    let request_uri = "/not/a/good/path";
    let req = test::TestRequest::get()
        .uri(request_uri)
        .insert_header(("X-Current-Documents-Guard", "stele_1/law"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);

    let location: &str = resp
        .headers()
        .get("Location")
        .expect("Location header missing")
        .to_str()
        .expect("Location header is not valid UTF-8");

    assert_eq!(location, "/");

    // follow redirect
    let req2 = test::TestRequest::get()
        .insert_header(("X-Current-Documents-Guard", "stele_1/law"))
        .uri(location)
        .to_request();
    let resp2 = test::call_service(&app, req2).await;

    // assert final response
    assert!(resp2.status().is_success());

    // test on other dependant stelae
    let request_uri = "/not/a/good/path";
    let req3 = test::TestRequest::get()
        .uri(request_uri)
        .insert_header(("X-Current-Documents-Guard", "stele_2/law"))
        .to_request();
    let resp = test::call_service(&app, req3).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_redirect_dependant_stelae_law_html_requests_with_correct_redirects_json_expect_success(
) {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Multihost(MultihostConfig::Public))
            .unwrap();

    let stelae_1_html_repo_path: PathBuf = archive_path.path().join("stele_1/law-html");
    let stelae_2_html_repo_path: PathBuf = archive_path.path().join("stele_2/law-html");
    let stelae_1_1_html_repo_path: PathBuf = archive_path.path().join("stele_1_1/law-html");

    let file_content = r#"
    [
        [
            "/not/a/good/path",
            "/"
        ]
    ]
    "#
    .to_string();

    let file_content1 = r#"
    [
        [
            "/not/a/good/path",
            "/a/"
        ]
    ]
    "#
    .to_string();

    let file_content2 = r#"
    [
        [
            "/not/a/good/path",
            "/a/b"
        ]
    ]
    "#
    .to_string();

    let _ = add_redirects_json_file(&stelae_1_html_repo_path, file_content);
    let _ = add_redirects_json_file(&stelae_2_html_repo_path, file_content1);
    let _ = add_redirects_json_file(&stelae_1_1_html_repo_path, file_content2);
    let app = common::initialize_app(archive_path.path()).await;

    let cases = [
        ("stele_1/law", "/"),
        ("stele_2/law", "/a/"),
        ("stele_1_1/law", "/a/b"),
    ];

    for (guard_value, expected_location) in cases {
        let request_uri = "/not/a/good/path";
        let req = test::TestRequest::get()
            .uri(request_uri)
            .insert_header(("X-Current-Documents-Guard", guard_value))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);

        let location: &str = resp
            .headers()
            .get("Location")
            .expect("Location header missing")
            .to_str()
            .expect("Location header is not valid UTF-8");

        assert_eq!(location, expected_location);
    }

    // test on other dependant stelae
    let request_uri = "/not/a/good/path";
    let req3 = test::TestRequest::get()
        .uri(request_uri)
        .insert_header(("X-Current-Documents-Guard", "stele_1_2/law"))
        .to_request();
    let resp = test::call_service(&app, req3).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
