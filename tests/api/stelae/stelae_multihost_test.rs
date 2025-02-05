//# def test_<method/code under test>_where_<conditions/inputs/state>_expect_<result>

use crate::{archive_testtools::config::ArchiveType, common};

use super::test_stelae_paths;

#[actix_web::test]
async fn test_stelae_api_with_multiple_repositories_expect_success() {
    let archive_path = common::initialize_archive(ArchiveType::Multihost).unwrap();
    let app = common::initialize_app(archive_path.path()).await;

    test_stelae_paths(
        "stele_1",
        "law-html",
        vec!["/a/b/c.html"],
        "HEAD",
        &app,
        true,
    )
    .await;

    test_stelae_paths(
        "stele_1_1",
        "law-pdf",
        vec!["/a/b/example.pdf"],
        "HEAD",
        &app,
        true,
    )
    .await;

    test_stelae_paths(
        "stele_1_2",
        "law-xml",
        vec!["/a/b/c/index.xml"],
        "HEAD",
        &app,
        true,
    )
    .await;

    test_stelae_paths("stele_2", "law-rdf", vec!["/a/b/c.rdf"], "HEAD", &app, true).await;
}
