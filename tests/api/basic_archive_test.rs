use crate::archive_testtools::{ArchiveType, Jurisdiction};
use crate::common;
use actix_web::test;

// TODO: Implement TestContext

// let context = TestContext {
//     stele: vec![TestSteleEntry {
//         name: "law".into(),
//         org_name: "test_org".into(),
//         scopes: None,
//         data_repositories: vec![
//             TestDataRepositoryEntry {
//                 kind: DataRepositoryType::Html("html".into()),
//                 routes: vec![
//                     "index.html".into(),
//                     "a/b/c.html".into(),
//                     "a/b/index.html".into(),
//                     "a/b/c/index.html".into(),
//                     "a/b/d/index.html".into(),
//                 ],
//                 route_glob_patterns: Some(vec![".*".into()]),
//                 is_fallback: false,
//                 serve_prefix: None,
//             },
//             TestDataRepositoryEntry {
//                 kind: DataRepositoryType::Xml("xml".into()),
//                 routes: vec!["a/b/c.xml".into()],
//                 route_glob_patterns: None,
//                 is_fallback: false,
//                 serve_prefix: "_uncodified_xml".into(),
//             },
//         ],
//     }],
// };

#[actix_web::test]
async fn test_resolve_law_html_request_with_full_path_expect_success() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;
    let req = test::TestRequest::get().uri("/a/b/c.html").to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_success();
    let expected = true;
    assert_eq!(actual, expected);
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
    let req = test::TestRequest::get().uri("/a/b/c.html").to_request();
    let actual = test::call_and_read_body(&app, req).await;
    let expected = "<!DOCTYPE html>";
    assert!(
        common::blob_to_string(actual.to_vec()).starts_with(expected),
        "doesn't start with {expected}"
    );
}
