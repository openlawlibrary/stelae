//! Central place for database queries in Stelae
use sqlx::types::chrono::NaiveDate;
use sqlx::Any;
use sqlx::QueryBuilder;
use sqlx::Sqlite;

use crate::db::models::document_change::DocumentChange;
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

pub async fn insert_document_changes_bulk(
    conn: &DatabaseConnection,
    document_changes: Vec<DocumentChange>,
) -> anyhow::Result<()> {
    match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            let mut query_builder: QueryBuilder<'_, Any> = QueryBuilder::new("INSERT OR IGNORE INTO document_change (doc_mpath, status, url, change_reason, publication, version, stele, doc_id) ");
            query_builder.push_values(document_changes, |mut b, dc| {
                b.push_bind(dc.doc_mpath)
                    .push_bind(dc.status)
                    .push_bind(dc.url)
                    .push_bind(dc.change_reason)
                    .push_bind(dc.publication)
                    .push_bind(dc.version)
                    .push_bind(dc.stele)
                    .push_bind(dc.doc_id);
            });
            let query = query_builder.build().persistent(false);
            query.fetch(&mut *connection);
        },
        DatabaseKind::Postgres => {
            anyhow::bail!("Not supported yet")
        }
    };

    Ok(())
}

/// Upsert a new publication into the database.
/// # Errors
/// Errors if the publication cannot be inserted into the database.
pub async fn create_publication(
    conn: &DatabaseConnection,
    name: &str,
    date: &NaiveDate,
    stele: &str,
    last_valid_publication_name: Option<String>,
    last_valid_version: Option<String>,
) -> anyhow::Result<Option<i64>> {
    let id = match conn.kind {
        DatabaseKind::Sqlite => {
            let statement: &'static str = r#"
                INSERT OR IGNORE INTO publication ( name, date, stele, revoked, last_valid_publication_name, last_valid_version )
                VALUES ( $1, $2, $3, FALSE, $4, $5 )
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(name)
                .bind(date)
                .bind(stele)
                .bind(last_valid_publication_name)
                .bind(last_valid_version)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
        DatabaseKind::Postgres => {
            let statement: &'static str = r#"
                INSERT INTO publication ( name, date, stele, revoked, last_valid_publication_name, last_valid_version )
                VALUES ( $1, $2, $3, FALSE, $4, $5 )
                ON CONFLICT ( name, stele ) DO NOTHING;
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(name)
                .bind(date)
                .bind(stele)
                .bind(last_valid_publication_name)
                .bind(last_valid_version)
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
pub async fn create_stele(conn: &DatabaseConnection, stele: &str) -> anyhow::Result<Option<i64>> {
    let id = match conn.kind {
        DatabaseKind::Sqlite => {
            let statement: &'static str = r#"
                INSERT OR IGNORE INTO stele ( name )
                VALUES ( $1 )
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(stele)
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
                .bind(stele)
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
    publication: &str,
    codified_date: &str,
    stele: &str,
) -> anyhow::Result<Option<i64>> {
    let id = match conn.kind {
        DatabaseKind::Sqlite => {
            let statement = r#"
                INSERT OR IGNORE INTO publication_version ( publication, version, stele )
                VALUES ( $1, $2, $3 )
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(publication)
                .bind(codified_date)
                .bind(stele)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
        DatabaseKind::Postgres => {
            let statement = r#"
                INSERT INTO publication_version ( publication, version, stele )
                VALUES ( $1, $2, $3 )
                ON CONFLICT ( publication, version, stele ) DO NOTHING;
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(publication)
                .bind(codified_date)
                .bind(stele)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
    };
    Ok(id)
}
