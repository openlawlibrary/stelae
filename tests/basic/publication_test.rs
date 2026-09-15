use crate::common::db_data::{insert_publication, insert_stele};
use crate::common::get_db;
use stelae::db::models::publication;
use stelae::db::{DatabaseTransaction, Tx as _};
use tempfile::TempDir;

const STELE: &str = "test_org/law";

/// A bare directory holding only a migrated database.
///
/// These are database tests - they need no git repositories, so they do not build
/// an archive fixture. `get_db` puts the SQLite file at `.taf/db.sqlite3` and does
/// not create that directory itself.
fn db_dir() -> TempDir {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join(".taf")).unwrap();
    dir
}

async fn count_missing(db: &stelae::db::DatabaseConnection, stele: &str) -> usize {
    let mut tx = DatabaseTransaction::begin(db.pool.clone()).await.unwrap();
    let count =
        publication::TxManager::count_non_revoked_missing_html_data_repo_name(&mut tx, stele)
            .await
            .unwrap();
    tx.commit().await.unwrap();
    count
}

#[actix_web::test]
async fn test_count_missing_html_data_repo_name_where_all_recorded_expect_zero() {
    let dir = db_dir();
    let db = get_db(dir.path()).await;
    insert_stele(&db, STELE).await;

    insert_publication(
        &db,
        "pub-1",
        "2024-01-01",
        "2024-01-01",
        STELE,
        false,
        Some("test_org/law-html"),
    )
    .await;

    assert_eq!(count_missing(&db, STELE).await, 0);
    drop(db);
}

#[actix_web::test]
async fn test_count_missing_html_data_repo_name_where_row_predates_column_expect_counted() {
    // A publication written before `html_data_repo_name` existed carries NULL.
    // `create` is INSERT OR IGNORE and the publication walk skips dates already
    // recorded, so an incremental run can never fill it in - which is what makes
    // this an inconsistency worth rebuilding for.
    let dir = db_dir();
    let db = get_db(dir.path()).await;
    insert_stele(&db, STELE).await;

    insert_publication(
        &db,
        "pub-old",
        "2023-01-01",
        "2023-01-01",
        STELE,
        false,
        None,
    )
    .await;
    insert_publication(
        &db,
        "pub-new",
        "2024-01-01",
        "2024-01-01",
        STELE,
        false,
        Some("test_org/law-html"),
    )
    .await;

    // the mixed state an existing database is most likely to be in: recent
    // publications recorded, older ones not
    assert_eq!(count_missing(&db, STELE).await, 1);
    drop(db);
}

#[actix_web::test]
async fn test_count_missing_html_data_repo_name_where_publication_revoked_expect_ignored() {
    // Revoked publications are not served, so a missing name on one is not a
    // reason to rebuild the stele.
    let dir = db_dir();
    let db = get_db(dir.path()).await;
    insert_stele(&db, STELE).await;

    insert_publication(
        &db,
        "pub-revoked",
        "2023-01-01",
        "2023-01-01",
        STELE,
        true,
        None,
    )
    .await;

    assert_eq!(count_missing(&db, STELE).await, 0);
    drop(db);
}

#[actix_web::test]
async fn test_count_missing_html_data_repo_name_where_other_stele_expect_not_counted() {
    // The count is per stele: one stele's gap must not trigger a rebuild of another.
    let dir = db_dir();
    let db = get_db(dir.path()).await;
    insert_stele(&db, STELE).await;
    insert_stele(&db, "other_org/law").await;

    insert_publication(
        &db,
        "pub-other",
        "2023-01-01",
        "2023-01-01",
        "other_org/law",
        false,
        None,
    )
    .await;

    assert_eq!(count_missing(&db, STELE).await, 0);
    assert_eq!(count_missing(&db, "other_org/law").await, 1);
    drop(db);
}
