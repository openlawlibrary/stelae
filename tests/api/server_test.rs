use crate::common;
use actix_web::test;

#[actix_web::test]
async fn test_resolve_law_html_request_with_full_path_expect_success() {
    let app = common::initialize_app().await;
    let req = test::TestRequest::get().uri("/a/b/c.html").to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_success();
    let expected = true;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_resolve_law_html_request_with_empty_path_expect_success() {
    let app = common::initialize_app().await;
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_success();
    let expected = true;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_resolve_request_with_incorrect_path_expect_client_error() {
    let app = common::initialize_app().await;
    let req = test::TestRequest::get().uri("/a/b/x").to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_client_error();
    let expected = true;
    assert_eq!(actual, expected);
}
