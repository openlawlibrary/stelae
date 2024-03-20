//! Central place for database queries in Stelae
use sqlx::types::chrono::NaiveDate;
use sqlx::Row;

use crate::db::DatabaseConnection;

use crate::db::DatabaseKind;


/// Find a stele by `name`.
///
/// # Errors
/// Errors if the stele cannot be found in the database.
pub async fn find_stele_by_name(
    conn: &DatabaseConnection,
    name: &str,
) -> anyhow::Result<Option<i32>> {
    let statement: &'static str = r#"
        SELECT id
        FROM stele
        WHERE name = $1
    "#;
    let row: Option<i32> = match &conn.kind {
        &DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            let row = sqlx::query(statement)
                .bind(name)
                .fetch_one(&mut *connection)
                .await
                .ok();
            row.map(|r| r.get(0))
        }
        DatabaseKind::Postgres => {
            let mut connection = conn.pool.acquire().await?;
            let row = sqlx::query(statement)
                .bind(name)
                .fetch_one(&mut *connection)
                .await
                .ok();
            row.map(|r| r.get(0))
        }
    };
    Ok(row)
}

/// Find a publication by `name` and `date` and `stele_id`.
///
/// # Errors
/// Errors if the publication cannot be found in the database.
pub async fn find_publication_by_name_and_date_and_stele_id(conn: &DatabaseConnection, name: &str, date: &NaiveDate, stele_id: i32) -> anyhow::Result<Option<i32>> {
    let statement: &'static str = r#"
        SELECT id
        FROM publication
        WHERE name = $1 AND date = $2 AND stele_id = $3
    "#;
    let row: Option<i32> = match &conn.kind {
        &DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            let row = sqlx::query(statement)
                .bind(name)
                .bind(date)
                .bind(stele_id)
                .fetch_one(&mut *connection)
                .await
                .ok();
            row.map(|r| r.get(0))
        }
        DatabaseKind::Postgres => {
            let mut connection = conn.pool.acquire().await?;
            let row = sqlx::query(statement)
                .bind(name)
                .bind(date)
                .bind(stele_id)
                .fetch_one(&mut *connection)
                .await
                .ok();
            row.map(|r| r.get(0))
        }
    };
    Ok(row)
}