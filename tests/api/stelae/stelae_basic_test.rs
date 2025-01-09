use actix_web::test;

use crate::api::stelae::test_stelae_paths;
use crate::archive_testtools::config::{ArchiveType, Jurisdiction};
use crate::archive_testtools::get_repository;
use crate::common;

use super::test_stelae_paths_with_head_method;

#[actix_web::test]
async fn test_stele_api_on_all_repositories_with_full_path_expect_success() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    test_stelae_paths(
        "test_org",
        "law-html",
        vec!["", "a/", "a/b/", "a/d/", "a/b/c.html", "a/b/c/"],
        "HEAD",
        &app,
        true,
    )
    .await;

    test_stelae_paths(
        "test_org",
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

    test_stelae_paths(
        "test_org",
        "law-pdf",
        vec!["/example.pdf", "/a/example.pdf", "/a/b/example.pdf"],
        "HEAD",
        &app,
        true,
    )
    .await;

    test_stelae_paths(
        "test_org",
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

    test_stelae_paths(
        "test_org",
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

    test_stelae_paths(
        "test_org",
        "law-xml-codified",
        vec!["index.xml", "e/index.xml", "e/f/index.xml", "e/g/index.xml"],
        "HEAD",
        &app,
        true,
    )
    .await;
}

#[actix_web::test]
async fn test_stele_api_on_all_repositories_with_head_method_expect_success() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    test_stelae_paths_with_head_method(
        "test_org",
        "law-html",
        vec!["", "a/", "a/b/", "a/d/", "a/b/c.html", "a/b/c/"],
        "HEAD",
        &app,
        true,
    )
    .await;

    test_stelae_paths_with_head_method(
        "test_org",
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

    test_stelae_paths_with_head_method(
        "test_org",
        "law-pdf",
        vec!["/example.pdf", "/a/example.pdf", "/a/b/example.pdf"],
        "HEAD",
        &app,
        true,
    )
    .await;

    test_stelae_paths_with_head_method(
        "test_org",
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

    test_stelae_paths_with_head_method(
        "test_org",
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

    test_stelae_paths_with_head_method(
        "test_org",
        "law-xml-codified",
        vec!["index.xml", "e/index.xml", "e/f/index.xml", "e/g/index.xml"],
        "HEAD",
        &app,
        true,
    )
    .await;
}

#[actix_web::test]
async fn test_stele_api_on_law_html_repository_with_missing_branch_name_expect_client_error() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    test_stelae_paths(
        "test_org",
        "law-html",
        vec!["", "a/index.html"],
        "",
        &app,
        false,
    )
    .await;
}

#[actix_web::test]
async fn test_stele_api_on_law_html_repository_with_invalid_branch_name_expect_client_error() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    test_stelae_paths(
        "test_org",
        "law-html",
        vec!["", "a/index.html"],
        "notExistingBranch",
        &app,
        false,
    )
    .await;
}

#[actix_web::test]
async fn test_stele_api_on_law_html_repository_with_invalid_org_name_expect_client_error() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    test_stelae_paths(
        "not_test_org",
        "law-html",
        vec!["", "a/index.html"],
        "HEAD",
        &app,
        false,
    )
    .await;
}

#[actix_web::test]
async fn test_stele_api_on_law_html_repository_with_invalid_repo_name_expect_client_error() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    test_stelae_paths(
        "test_org",
        "not_law-html",
        vec!["", "a/index.html"],
        "HEAD",
        &app,
        false,
    )
    .await;
}

#[actix_web::test]
async fn test_stele_api_on_law_html_repository_with_incorrect_paths_expect_client_error() {
    let archive_path =
        common::initialize_archive(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    test_stelae_paths(
        "test_org",
        "law-html",
        vec!["a/b/c/d", "a/index.css"],
        "HEAD",
        &app,
        false,
    )
    .await;
}

#[actix_web::test]
async fn test_stele_api_on_law_html_repository_with_different_files_on_different_branches_expect_success(
) {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    let mut path = archive_path.path().to_path_buf();
    path.push("test_org");
    let git_repo = get_repository(&path, "law-html");
    path.push("law-html");
    let _ = git_repo.create_branch("test_branch");
    let _ = git_repo.create_branch("default_branch");

    let _ = git_repo.checkout("default_branch");
    let _ = git_repo.add_file(&path, "test.txt", "Content for default branch");
    let _ = git_repo.commit(None, "Adding data for default branch");

    let _ = git_repo.checkout("test_branch");
    let _ = git_repo.add_file(&path, "test1.txt", "Content for test branch");
    let _ = git_repo.commit(None, "Adding data for test branch");

    test_stelae_paths(
        "test_org",
        "law-html",
        vec!["/test.txt"],
        "default_branch",
        &app,
        true,
    )
    .await;

    test_stelae_paths(
        "test_org",
        "law-html",
        vec!["/test1.txt"],
        "default_branch",
        &app,
        false,
    )
    .await;

    test_stelae_paths(
        "test_org",
        "law-html",
        vec!["/test.txt"],
        "test_branch",
        &app,
        false,
    )
    .await;

    test_stelae_paths(
        "test_org",
        "law-html",
        vec!["/test1.txt"],
        "test_branch",
        &app,
        true,
    )
    .await;
}

#[actix_web::test]
async fn test_stele_api_with_same_file_on_different_branches_expect_different_file_content_on_different_branches(
) {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    let mut path = archive_path.path().to_path_buf();
    path.push("test_org");
    let git_repo = get_repository(&path, "law-html");
    path.push("law-html");

    let _ = git_repo.create_branch("test_branch");
    let _ = git_repo.create_branch("default_branch");

    let _ = git_repo.checkout("default_branch");
    let _ = git_repo.add_file(&path, "test.txt", "Content for default branch");
    let _ = git_repo.commit(None, "Adding data for default branch");

    let _ = git_repo.checkout("test_branch");
    let _ = git_repo.add_file(&path, "test.txt", "Content for test branch");
    let _ = git_repo.commit(None, "Adding data for test branch");

    let req = test::TestRequest::get()
        .uri(&format!(
            "/_stelae/test_org/law-html?commitish=default_branch&remainder=/test.txt"
        ))
        .to_request();
    let actual = test::call_and_read_body(&app, req).await;
    let expected = "Content for default branch";
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

#[actix_web::test]
async fn test_stelae_api_where_branch_contains_slashs_expect_resolved_content() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    let mut path = archive_path.path().to_path_buf();
    path.push("test_org");
    let git_repo = get_repository(&path, "law-html");
    path.push("law-html");
    let branch_name = "test/branch/with/slash";
    let _ = git_repo.create_branch(branch_name);

    let _ = git_repo.checkout(branch_name);
    let _ = git_repo.add_file(&path, "test.txt", "Content for test branch");
    let _ = git_repo.commit(None, "Adding data for test branch");

    let req = test::TestRequest::get()
        .uri(&format!(
            "/_stelae/test_org/law-html?commitish={}&remainder=/test.txt",
            branch_name
        ))
        .to_request();
    let actual = test::call_and_read_body(&app, req).await;
    let expected = "Content for test branch";
    assert!(
        common::blob_to_string(actual.to_vec()).starts_with(expected),
        "doesn't start with {expected}"
    );
}

#[actix_web::test]
async fn test_stelae_api_where_branch_is_commit_sha_expect_resolved_content() {
    let archive_path =
        common::initialize_archive_without_bare(ArchiveType::Basic(Jurisdiction::Single)).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    let mut path = archive_path.path().to_path_buf();
    path.push("test_org");
    let git_repo = get_repository(&path, "law-html");
    path.push("law-html");
    let branch_name = "test/branch/with/slash";
    let _ = git_repo.create_branch(branch_name);

    let _ = git_repo.checkout(branch_name);
    let _ = git_repo.add_file(&path, "test.txt", "Content for test branch");
    let commit_hash = git_repo.commit(None, "Adding data for test branch");
    let sha_string = commit_hash.unwrap().to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/_stelae/test_org/law-html?commitish={}&remainder=/test.txt",
            sha_string
        ))
        .to_request();
    let actual = test::call_and_read_body(&app, req).await;
    let expected = "Content for test branch";
    assert!(
        common::blob_to_string(actual.to_vec()).starts_with(expected),
        "doesn't start with {expected}"
    );
}
