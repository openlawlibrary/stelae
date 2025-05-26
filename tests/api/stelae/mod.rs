use actix_http::{Method, Request};
use actix_service::Service;
use actix_web::body::MessageBody;
use actix_web::dev::ServiceResponse;
use actix_web::http::header;
use actix_web::{test, Error};
mod stelae_basic_test;
mod stelae_multihost_test;

/// Helper method which test all `file_paths`` in `org_name`/`repo_name` repository on `branch_name`` branch with `expected` result
async fn test_stelae_paths(
    org_name: &str,
    repo_name: &str,
    file_paths: Vec<&str>,
    branch_name: &str,
    app: &impl Service<Request, Response = ServiceResponse<impl MessageBody>, Error = Error>,
    is_success: bool,
) {
    for file_path in file_paths {
        let req = test::TestRequest::get()
            .uri(&format!(
                "/_stelae/{}/{}?commitish={}&remainder={}",
                org_name, repo_name, branch_name, file_path
            ))
            .insert_header((
                header::HeaderName::from_static("x-stelae"),
                format!("{org_name}/law"),
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;
        let actual = if is_success {
            resp.status().is_success()
        } else {
            resp.status().is_client_error()
        };
        let expected = true;
        assert_eq!(actual, expected);

        if is_success {
            let http_x_path = resp.headers().get("X-File-Path").unwrap().to_str().unwrap();
            // strip leading slash from filepath (if exists)
            let file_path = file_path.strip_prefix('/').unwrap_or(file_path);
            assert!(
                http_x_path.to_string().contains(file_path),
                "\"{http_x_path}\" doesn't contain {file_path}"
            );
        }
    }
}

/// Helper method which test all `fille_paths`` in `org_name`/`repo_name` repository on `branch_name`` branch with `expected` result
async fn test_stelae_paths_with_head_method(
    org_name: &str,
    repo_name: &str,
    file_paths: Vec<&str>,
    branch_name: &str,
    app: &impl Service<Request, Response = ServiceResponse<impl MessageBody>, Error = Error>,
    is_success: bool,
) {
    for file_path in file_paths {
        let req = test::TestRequest::default()
            .method(Method::HEAD)
            .uri(&format!(
                "/_stelae/{}/{}?commitish={}&remainder={}",
                org_name, repo_name, branch_name, file_path
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;
        let actual = if is_success {
            resp.status().is_success()
        } else {
            resp.status().is_client_error()
        };
        let expected = true;
        assert_eq!(actual, expected);
    }
}
