//! Central place for database queries in Stelae
use sqlx::Row;

use crate::db::DatabaseConnection;

/// Inserts a new document into the database.
///
/// # Errors
/// Errors if the document cannot be inserted into the database.
pub async fn insert_new_document(conn: &DatabaseConnection, doc_id: &str) -> anyhow::Result<()> {
    let statement: &'static str = r#"
        INSERT OR IGNORE INTO document ( doc_id )
        VALUES ( $1 )
    "#;
    match &conn {
        DatabaseConnection::Sqlite(ref pool) => {
            let mut connection = pool.acquire().await?;
            sqlx::query(statement)
                .bind(doc_id)
                .execute(&mut *connection)
                .await?;
        }
        DatabaseConnection::Postgres(pool) => {
            let mut connection = pool.acquire().await?;
            sqlx::query(statement)
                .bind(doc_id)
                .execute(&mut *connection)
                .await?;
        }
    };
    Ok(())
}

/// Inserts a new stele into the database.
///
/// # Errors
/// Errors if the stele cannot be inserted into the database.
pub async fn insert_new_stele(conn: &DatabaseConnection, stele_id: &str) -> anyhow::Result<()> {
    let statement: &'static str = r#"
        INSERT OR IGNORE INTO stele ( name )
        VALUES ( $1 )
    "#;
    match &conn {
        DatabaseConnection::Sqlite(ref pool) => {
            let mut connection = pool.acquire().await?;
            sqlx::query(statement)
                .bind(stele_id)
                .execute(&mut *connection)
                .await?;
        }
        DatabaseConnection::Postgres(pool) => {
            let mut connection = pool.acquire().await?;
            sqlx::query(statement)
                .bind(stele_id)
                .execute(&mut *connection)
                .await?;
        }
    };
    Ok(())
}

/// Find a stele by name.
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
    let row: Option<i32> = match &conn {
        DatabaseConnection::Sqlite(ref pool) => {
            let mut connection = pool.acquire().await?;
            let row = sqlx::query(statement)
                .bind(name)
                .fetch_one(&mut *connection)
                .await
                .ok();
            row.map(|r| r.get(0))
        }
        DatabaseConnection::Postgres(pool) => {
            let mut connection = pool.acquire().await?;
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
