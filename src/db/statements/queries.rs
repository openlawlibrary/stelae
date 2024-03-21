//! Central place for database queries
use sqlx::types::chrono::NaiveDate;

use crate::db::models::publication::Publication;
use crate::db::models::stele::Stele;
use crate::db::DatabaseConnection;

use crate::db::DatabaseKind;

/// Find a stele by `name`.
///
/// # Errors
/// Errors if can't establish a connection to the database.
pub async fn find_stele_by_name(
    conn: &DatabaseConnection,
    name: &str,
) -> anyhow::Result<Option<Stele>> {
    let statement: &'static str = r#"
        SELECT *
        FROM stele
        WHERE name = $1
    "#;
    let row = match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, Stele>(statement)
                .bind(name)
                .fetch_one(&mut *connection)
                .await
                .ok()
        }
        DatabaseKind::Postgres => {
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, Stele>(statement)
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
/// Errors if can't establish a connection to the database.
pub async fn find_publication_by_name_and_date_and_stele_id(
    conn: &DatabaseConnection,
    name: &str,
    date: &NaiveDate,
    stele_id: i32,
) -> anyhow::Result<Option<Publication>> {
    let statement: &'static str = r#"
        SELECT *
        FROM publication
        WHERE name = $1 AND date = $2 AND stele_id = $3
    "#;
    let row = match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, Publication>(statement)
                .bind(name)
                .bind(date)
                .bind(stele_id)
                .fetch_one(&mut *connection)
                .await
                .ok()
        }
        DatabaseKind::Postgres => {
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, Publication>(statement)
                .bind(name)
                .bind(date)
                .bind(stele_id)
                .fetch_one(&mut *connection)
                .await
                .ok()
        }
    };
    Ok(row)
}
