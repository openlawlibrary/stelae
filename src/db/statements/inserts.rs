//! Central place for database queries in Stelae
use sqlx::types::chrono::NaiveDate;

use crate::db::DatabaseConnection;
use crate::db::DatabaseKind;

/// Upsert a new document into the database.
///
/// # Errors
/// Errors if the document cannot be inserted into the database.
pub async fn create_document(
    conn: &DatabaseConnection,
    doc_id: &str,
) -> anyhow::Result<Option<i64>> {
    let id = match conn.kind {
        DatabaseKind::Sqlite => {
            let statement: &'static str = r#"
                INSERT OR IGNORE INTO document ( doc_id )
                VALUES ( $1 )
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(doc_id)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
        DatabaseKind::Postgres => {
            let statement = r#"
                INSERT INTO document ( doc_id )
                VALUES ( $1 )
                ON CONFLICT ( doc_id ) DO NOTHING;
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(doc_id)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
    };
    Ok(id)
}

/// Upsert a new publication into the database.
/// # Errors
/// Errors if the publication cannot be inserted into the database.
pub async fn create_publication(
    conn: &DatabaseConnection,
    name: &str,
    date: &NaiveDate,
    stele_id: i32,
) -> anyhow::Result<Option<i64>> {
    let id = match conn.kind {
        DatabaseKind::Sqlite => {
            let statement: &'static str = r#"
                INSERT OR IGNORE INTO publication ( name, date, stele_id, revoked )
                VALUES ( $1, $2, $3, FALSE )
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(name)
                .bind(date)
                .bind(stele_id)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
        DatabaseKind::Postgres => {
            let statement: &'static str = r#"
                INSERT INTO publication ( name, date, stele_id, revoked )
                VALUES ( $1, $2, $3, FALSE )
                ON CONFLICT ( name, date, stele_id ) DO NOTHING;
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(name)
                .bind(date)
                .bind(stele_id)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
    };
    Ok(id)
}

/// Upsert a new stele into the database.
///
/// # Errors
/// Errors if the stele cannot be inserted into the database.
pub async fn create_stele(
    conn: &DatabaseConnection,
    stele_id: &str,
) -> anyhow::Result<Option<i64>> {
    let id = match conn.kind {
        DatabaseKind::Sqlite => {
            let statement: &'static str = r#"
                INSERT OR IGNORE INTO stele ( name )
                VALUES ( $1 )
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(stele_id)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
        DatabaseKind::Postgres => {
            let statement: &'static str = r#"
                INSERT INTO stele ( name )
                VALUES ( $1 )
                ON CONFLICT ( name ) DO NOTHING;
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(stele_id)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
    };
    Ok(id)
}

/// Upsert a new version into the database.
///
/// # Errors
/// Errors if the version cannot be inserted into the database.
pub async fn create_version(
    conn: &DatabaseConnection,
    codified_date: &str,
) -> anyhow::Result<Option<i64>> {
    let id = match conn.kind {
        DatabaseKind::Sqlite => {
            let statement: &'static str = r#"
                INSERT OR IGNORE INTO version ( codified_date )
                VALUES ( $1 )
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(codified_date)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
        DatabaseKind::Postgres => {
            let statement: &'static str = r#"
                INSERT INTO version ( codified_date )
                VALUES ( $1 )
                ON CONFLICT ( codified_date ) DO NOTHING;
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(codified_date)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
    };
    Ok(id)
}

/// Upsert a new publication version into the database.
///
/// # Errors
/// Errors if the publication version cannot be inserted into the database.
pub async fn create_publication_version(
    conn: &DatabaseConnection,
    publication_id: i32,
    codified_date: &str,
) -> anyhow::Result<Option<i64>> {
    let id = match conn.kind {
        DatabaseKind::Sqlite => {
            let statement = r#"
                INSERT OR IGNORE INTO publication_version ( publication_id, version )
                VALUES ( $1, $2 )
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(publication_id)
                .bind(codified_date)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
        DatabaseKind::Postgres => {
            let statement = r#"
                INSERT INTO publication_version ( publication_id, version )
                VALUES ( $1, $2 )
                ON CONFLICT ( publication_id, version ) DO NOTHING;
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(publication_id)
                .bind(codified_date)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
    };
    Ok(id)
}
