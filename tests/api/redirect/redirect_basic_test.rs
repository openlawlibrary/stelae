use crate::archive_testtools::add_redirects_json_file;
use crate::{
    archive_testtools::config::{ArchiveType, Jurisdiction},
    common,
};
use actix_http::StatusCode;
use actix_web::test;
use std::path::PathBuf;

#[actix_web::test]
async fn test_redirect_law_html_request_with_correct_redirects_json_expect_success() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();

    let html_repo_path: PathBuf = archive_path.path().join("test_org/law-html");

    let file_content = r#"
    [
        [
            "/a/b/c.html",
            "/"
        ]
    ]
    "#
    .to_string();

    let _ = add_redirects_json_file(&html_repo_path, file_content);
    let app = common::initialize_app(archive_path.path()).await;

    let request_uri = "/a/b/c.html";
    let req = test::TestRequest::get().uri(request_uri).to_request();
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
    let req2 = test::TestRequest::get().uri(location).to_request();
    let resp2 = test::call_service(&app, req2).await;

    // assert final response
    assert!(resp2.status().is_success());
}

#[actix_web::test]
async fn test_redirect_law_html_request_with_incorrect_redirects_json_expect_fail() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();

    let html_repo_path: PathBuf = archive_path.path().join("test_org/law-html");

    let file_content = r#"
    [
        [
            "/a/b/c.html",
            "/"
        ],
        [
            "/a/b/index.html",
            "/"
        } // bad symbol
    ]
    "#
    .to_string();

    let _ = add_redirects_json_file(&html_repo_path, file_content);
    let app = common::initialize_app(archive_path.path()).await;

    let request_uri = "/a/b/c.html";
    let req = test::TestRequest::get().uri(request_uri).to_request();
    let resp = test::call_service(&app, req).await;
    assert_ne!(resp.status(), StatusCode::TEMPORARY_REDIRECT);
}
