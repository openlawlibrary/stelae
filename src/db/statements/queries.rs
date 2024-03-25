//! Central place for database queries
use sqlx::types::chrono::NaiveDate;

use crate::db::models::publication::Publication;
use crate::db::models::publication_version::PublicationVersion;
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
                .ok()
        }
    };
    Ok(row)
}

/// Find the last inserted publication by `stele_id`.
/// This function is then used to incrementally insert new change objects
///
/// # Errors
/// Errors if can't establish a connection to the database.
pub async fn find_last_inserted_publication(
    conn: &DatabaseConnection,
    stele_id: i32,
) -> anyhow::Result<Option<Publication>> {
    let statement: &'static str = r#"
        SELECT *
        FROM publication
        WHERE revoked = 0 AND stele_id = $1
        ORDER BY date DESC
        LIMIT 1
    "#;
    let row = match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, Publication>(statement)
                .bind(stele_id)
                .fetch_one(&mut *connection)
                .await
                .ok()
        }
        DatabaseKind::Postgres => {
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, Publication>(statement)
                .bind(stele_id)
                .fetch_one(&mut *connection)
                .await
                .ok()
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

/// Find a publication version by `publication_id` and `version`.
///
/// # Errors
/// Errors if can't establish a connection to the database.
pub async fn find_publication_version_by_publication_id_and_version(
    conn: &DatabaseConnection,
    publication_id: i32,
    codified_date: &str,
) -> anyhow::Result<Option<PublicationVersion>> {
    let statement: &'static str = r#"
        SELECT *
        FROM publication_version
        WHERE publication_id = $1 AND version = $2
    "#;
    let row = match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, PublicationVersion>(statement)
                .bind(publication_id)
                .bind(codified_date)
                .fetch_one(&mut *connection)
                .await
                .ok()
        }
        DatabaseKind::Postgres => {
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, PublicationVersion>(statement)
                .bind(publication_id)
                .bind(codified_date)
                .fetch_one(&mut *connection)
                .await
                .ok()
        }
    };
    Ok(row)
}
