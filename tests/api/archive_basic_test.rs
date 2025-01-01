use crate::archive_testtools::config::{ArchiveType, Jurisdiction};
use crate::archive_testtools::get_repository;
use crate::common::{self};
use actix_http::Request;
use actix_service::Service;
use actix_web::body::MessageBody;
use actix_web::dev::ServiceResponse;
use actix_web::{test, Error};

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
async fn test_resolve_root_stele_all_repositories_request_with_full_path_expect_success() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    let mut path = archive_path.path().to_path_buf();
    path.push("test_org");

    // let git_repo = get_repository(&path, "law-html");
    // let _ = git_repo.create_branch("test_branch");
    println!("Testing law-html");
    test_paths(
        "law-html",
        vec!["", "a/", "a/b/", "a/d/", "a/b/c.html", "a/b/c/"],
        "HEAD",
        &app,
        true,
    )
    .await;

    println!("Testing law-other");
    test_paths(
        "law-other",
        vec![
            "",
            "example.json",
            "a/",
            "a/e/_doc/f/",
            "a/d/",
            "a/b/",
            "a/b/c.html",
            "a/_doc/e/",
            "_prefix/",
            "_prefix/a/",
        ],
        "HEAD",
        &app,
        true,
    )
    .await;

    println!("Testing law-pdf");
    test_paths(
        "law-pdf",
        vec!["/example.pdf", "/a/example.pdf", "/a/b/example.pdf"],
        "HEAD",
        &app,
        true,
    )
    .await;

    println!("Testing law-rdf");
    test_paths(
        "law-rdf",
        vec![
            "index.rdf",
            "a/index.rdf",
            "a/b/index.rdf",
            "a/d/index.rdf",
            "a/b/c.rdf",
            "a/b/c/index.rdf",
        ],
        "HEAD",
        &app,
        true,
    )
    .await;

    println!("Testing law-xml");
    test_paths(
        "law-xml",
        vec![
            "index.xml",
            "a/index.xml",
            "a/b/index.xml",
            "a/d/index.xml",
            "a/b/c.xml",
            "a/b/c/index.xml",
        ],
        "HEAD",
        &app,
        true,
    )
    .await;

    println!("Testing law-xml-codified");
    test_paths(
        "law-xml-codified",
        vec!["index.xml", "e/index.xml", "e/f/index.xml", "e/g/index.xml"],
        "master",
        &app,
        true,
    )
    .await;
}

#[actix_web::test]
async fn test_resolve_root_stele_all_repositories_request_with_incorrect_path_expect_client_error()
{
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    let mut path = archive_path.path().to_path_buf();
    path.push("test_org");

    println!("Testing law-html");
    test_paths(
        "law-html",
        vec!["a/b/c/d", "a/index.css"],
        "HEAD",
        &app,
        false,
    )
    .await;

    println!("Testing law-other");
    test_paths(
        "law-other",
        vec!["a/b/c/d", "example1.json"],
        "HEAD",
        &app,
        false,
    )
    .await;

    println!("Testing law-pdf");
    test_paths(
        "law-pdf",
        vec!["/example1.pdf", "/c/example.pdf"],
        "HEAD",
        &app,
        false,
    )
    .await;

    println!("Testing law-rdf");
    test_paths(
        "law-rdf",
        vec!["index1.rdf", "z/index.rdf"],
        "HEAD",
        &app,
        false,
    )
    .await;

    println!("Testing law-xml");
    test_paths(
        "law-xml",
        vec!["index1.xml", "t/index.xml"],
        "HEAD",
        &app,
        false,
    )
    .await;

    println!("Testing law-xml-codified");
    test_paths(
        "law-xml-codified",
        vec!["index1.xml", "x/index.xml"],
        "HEAD",
        &app,
        false,
    )
    .await;
}

async fn test_paths(
    repo_name: &str,
    file_paths: Vec<&str>,
    branch_name: &str,
    app: &impl Service<Request, Response = ServiceResponse<impl MessageBody>, Error = Error>,
    expected: bool,
) {
    for request_uri in file_paths {
        let req = test::TestRequest::get()
            .uri(&format!(
                "/_stelae/test_org/{}?commitish={}&remainder={}",
                repo_name, branch_name, request_uri
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let actual = if expected {
            resp.status().is_success()
        } else {
            resp.status().is_client_error()
        };
        let expected = true;
        assert_eq!(actual, expected);
    }
}

#[actix_web::test]
async fn test_resolve_root_stele_law_html_different_files_with_different_branches() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    let mut path = archive_path.path().to_path_buf();
    path.push("test_org");
    let git_repo = get_repository(&path, "law-html");
    path.push("law-html");
    let _ = git_repo.create_branch("test_branch");

    let _ = git_repo.checkout("master");
    let _ = git_repo.add_file(&path, "test.txt", "Content for master branch");
    let _ = git_repo.commit(None, "Adding data for master branch");

    let _ = git_repo.checkout("test_branch");
    let _ = git_repo.add_file(&path, "test1.txt", "Content for test branch");
    let _ = git_repo.commit(None, "Adding data for master branch");

    let req = test::TestRequest::get()
        .uri(&format!(
            "/_stelae/test_org/law-html?commitish=master&remainder=/test.txt"
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_success();
    assert_eq!(actual, true);

    let req = test::TestRequest::get()
        .uri(&format!(
            "/_stelae/test_org/law-html?commitish=master&remainder=/test1.txt"
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_success();
    assert_eq!(actual, false);

    let req = test::TestRequest::get()
        .uri(&format!(
            "/_stelae/test_org/law-html?commitish=test_branch&remainder=/test.txt"
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_success();
    assert_eq!(actual, false);

    let req = test::TestRequest::get()
        .uri(&format!(
            "/_stelae/test_org/law-html?commitish=test_branch&remainder=/test1.txt"
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let actual = resp.status().is_success();
    assert_eq!(actual, true);
}

#[actix_web::test]
async fn test_resolve_root_stele_law_html_file_content_with_different_branches_expect_success() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    let mut path = archive_path.path().to_path_buf();
    path.push("test_org");
    let git_repo = get_repository(&path, "law-html");
    path.push("law-html");

    let _ = git_repo.create_branch("test_branch");

    let _ = git_repo.checkout("master");
    let _ = git_repo.add_file(&path, "test.txt", "Content for master branch");
    let _ = git_repo.commit(None, "Adding data for master branch");

    let _ = git_repo.checkout("test_branch");
    let _ = git_repo.add_file(&path, "test.txt", "Content for test branch");
    let _ = git_repo.commit(None, "Adding data for test branch");

    let req = test::TestRequest::get()
        .uri(&format!(
            "/_stelae/test_org/law-html?commitish=master&remainder=/test.txt"
        ))
        .to_request();
    let actual = test::call_and_read_body(&app, req).await;
    let expected = "Content for master branch";
    assert!(
        common::blob_to_string(actual.to_vec()).starts_with(expected),
        "doesn't start with {expected}"
    );

    let req = test::TestRequest::get()
        .uri(&format!(
            "/_stelae/test_org/law-html?commitish=test_branch&remainder=test.txt"
        ))
        .to_request();
    let actual = test::call_and_read_body(&app, req).await;
    let expected = "Content for test branch";
    println!("{}", common::blob_to_string(actual.to_vec()));
    assert!(
        common::blob_to_string(actual.to_vec()).starts_with(expected),
        "doesn't start with {expected}"
    );
}
