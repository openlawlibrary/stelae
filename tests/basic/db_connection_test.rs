use crate::common;
use entity::sea_orm::DatabaseConnection;
use std::matches;
use stelae::db::init::connect;
use tempfile::tempdir;

#[actix_web::test]
async fn test_get_db_connection_when_no_env_var_expect_sqlite_connection() {
    common::initialize();
    let test_library_path = common::get_test_library_path();
    let actual = connect(&test_library_path).await.unwrap();
    let _expected = DatabaseConnection::SqlxSqlitePoolConnection;
    assert!(matches!(actual, _expected));
}

#[actix_web::test]
async fn test_get_db_connection_when_no_env_var_and_no_sqlite_file_expect_error() {
    let dir = tempdir().unwrap();
    let test_library_path = dir.path();
    let actual = connect(test_library_path).await.unwrap_err();
    let expected = "unable to open database file";
    assert!(
        actual.to_string().contains(expected),
        "\"{actual}\" doesn't contain {expected}"
    );
}
