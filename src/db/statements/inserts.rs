//! Central place for database queries in Stelae
use sqlx::types::chrono::NaiveDate;
use sqlx::QueryBuilder;

use crate::db::models::changed_library_document::ChangedLibraryDocument;
use crate::db::models::document_change::DocumentChange;
use crate::db::models::library::Library;
use crate::db::models::library_change::LibraryChange;
use crate::db::models::publication_has_publication_versions::PublicationHasPublicationVersions;
use crate::db::DatabaseConnection;
use crate::db::DatabaseKind;

/// Size of the batch for bulk inserts.
const BATCH_SIZE: usize = 1000;

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
            let statement = "
                INSERT OR IGNORE INTO document ( doc_id )
                VALUES ( $1 )
            ";
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(doc_id)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
        DatabaseKind::Postgres => {
            let statement = "
                INSERT INTO document ( doc_id )
                VALUES ( $1 )
                ON CONFLICT ( doc_id ) DO NOTHING;
            ";
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

/// Upsert a bulk of document changes into the database.
///
/// # Errors
/// Errors if the document changes cannot be inserted into the database.
pub async fn insert_document_changes_bulk(
    conn: &DatabaseConnection,
    document_changes: Vec<DocumentChange>,
) -> anyhow::Result<()> {
    match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO document_change (doc_mpath, status, url, change_reason, publication, version, stele, doc_id) ");
            for chunk in document_changes.chunks(BATCH_SIZE) {
                query_builder.push_values(chunk, |mut bindings, dc| {
                    bindings
                        .push_bind(&dc.doc_mpath)
                        .push_bind(&dc.status)
                        .push_bind(&dc.url)
                        .push_bind(&dc.change_reason)
                        .push_bind(&dc.publication)
                        .push_bind(&dc.version)
                        .push_bind(&dc.stele)
                        .push_bind(&dc.doc_id);
                });
                let query = query_builder.build();
                query.execute(&mut *connection).await?;
                query_builder.reset();
            }
        }
        DatabaseKind::Postgres => {
            anyhow::bail!("Not supported yet")
        }
    };

    Ok(())
}

/// Upsert a bulk of libraries into the database.
///
/// # Errors
/// Errors if the libraries cannot be inserted into the database.
pub async fn insert_library_bulk(
    conn: &DatabaseConnection,
    libraries: Vec<Library>,
) -> anyhow::Result<()> {
    match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO library (mpath) ");
            for chunk in libraries.chunks(BATCH_SIZE) {
                query_builder.push_values(chunk, |mut bindings, lb| {
                    bindings.push_bind(&lb.mpath);
                });
                let query = query_builder.build();
                query.execute(&mut *connection).await?;
            }
        }
        DatabaseKind::Postgres => {
            anyhow::bail!("Not supported yet")
        }
    }
    Ok(())
}

/// Upsert a bulk of library changes into the database.
///
/// # Errors
/// Errors if the library changes cannot be inserted into the database.
pub async fn insert_library_changes_bulk(
    conn: &DatabaseConnection,
    library_changes: Vec<LibraryChange>,
) -> anyhow::Result<()> {
    match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO library_change (library_mpath, publication, version, stele, status, url) ");
            for chunk in library_changes.chunks(BATCH_SIZE) {
                query_builder.push_values(chunk, |mut bindings, lc| {
                    bindings
                        .push_bind(&lc.library_mpath)
                        .push_bind(&lc.publication)
                        .push_bind(&lc.version)
                        .push_bind(&lc.stele)
                        .push_bind(&lc.status)
                        .push_bind(&lc.url);
                });
                let query = query_builder.build();
                query.execute(&mut *connection).await?;
            }
        }
        DatabaseKind::Postgres => {
            anyhow::bail!("Not supported yet")
        }
    }
    Ok(())
}

/// Upsert a bulk of changed library documents into the database.
///
/// # Errors
/// Errors if the changed library documents cannot be inserted into the database.
pub async fn insert_changed_library_document_bulk(
    conn: &DatabaseConnection,
    changed_library_document: Vec<ChangedLibraryDocument>,
) -> anyhow::Result<()> {
    match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO changed_library_document (publication, version, stele, doc_mpath, status, library_mpath, url) ");
            for chunk in changed_library_document.chunks(BATCH_SIZE) {
                query_builder.push_values(chunk, |mut bindings, cl| {
                    bindings
                        .push_bind(&cl.publication)
                        .push_bind(&cl.version)
                        .push_bind(&cl.stele)
                        .push_bind(&cl.doc_mpath)
                        .push_bind(&cl.status)
                        .push_bind(&cl.library_mpath)
                        .push_bind(&cl.url);
                });
                let query = query_builder.build();
                query.execute(&mut *connection).await?;
            }
        }
        DatabaseKind::Postgres => {
            anyhow::bail!("Not supported yet")
        }
    }
    Ok(())
}

/// Upsert a bulk of `publication_has_publication_versions` into the database.
///
/// # Errors
/// Errors if the `publication_has_publication_versions` cannot be inserted into the database.
pub async fn insert_publication_has_publication_versions_bulk(
    conn: &DatabaseConnection,
    publication_has_publication_versions: Vec<PublicationHasPublicationVersions>,
) -> anyhow::Result<()> {
    match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO publication_has_publication_versions (publication, referenced_publication, referenced_version, stele) ");
            for chunk in publication_has_publication_versions.chunks(BATCH_SIZE) {
                query_builder.push_values(chunk, |mut bindings, pb| {
                    bindings
                        .push_bind(&pb.publication)
                        .push_bind(&pb.referenced_publication)
                        .push_bind(&pb.referenced_version)
                        .push_bind(&pb.stele);
                });
                let query = query_builder.build();
                query.execute(&mut *connection).await?;
            }
        }
        DatabaseKind::Postgres => {
            anyhow::bail!("Not supported yet")
        }
    }
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
            let statement = "
                INSERT OR IGNORE INTO publication ( name, date, stele, revoked, last_valid_publication_name, last_valid_version )
                VALUES ( $1, $2, $3, FALSE, $4, $5 )
            ";
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(name)
                .bind(date.to_string())
                .bind(stele)
                .bind(last_valid_publication_name)
                .bind(last_valid_version)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
        DatabaseKind::Postgres => {
            let statement = "
                INSERT INTO publication ( name, date, stele, revoked, last_valid_publication_name, last_valid_version )
                VALUES ( $1, $2, $3, FALSE, $4, $5 )
                ON CONFLICT ( name, stele ) DO NOTHING;
            ";
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(name)
                .bind(date.to_string())
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
            let statement = "
                INSERT OR IGNORE INTO stele ( name )
                VALUES ( $1 )
            ";
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(stele)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
        DatabaseKind::Postgres => {
            let statement = "
                INSERT INTO stele ( name )
                VALUES ( $1 )
                ON CONFLICT ( name ) DO NOTHING;
            ";
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
            let statement = "
                INSERT OR IGNORE INTO version ( codified_date )
                VALUES ( $1 )
            ";
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(codified_date)
                .execute(&mut *connection)
                .await?
                .last_insert_id()
        }
        DatabaseKind::Postgres => {
            let statement = "
                INSERT INTO version ( codified_date )
                VALUES ( $1 )
                ON CONFLICT ( codified_date ) DO NOTHING;
            ";
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
            let statement = "
                INSERT OR IGNORE INTO publication_version ( publication, version, stele )
                VALUES ( $1, $2, $3 )
            ";
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
            let statement = "
                INSERT INTO publication_version ( publication, version, stele )
                VALUES ( $1, $2, $3 )
                ON CONFLICT ( publication, version, stele ) DO NOTHING;
            ";
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

/// Update a publication by name and stele to be revoked.
///
/// # Errors
/// Errors if the publication cannot be updated.
pub async fn update_publication_by_name_and_stele_set_revoked_true(
    conn: &DatabaseConnection,
    name: &str,
    stele: &str,
) -> anyhow::Result<()> {
    let statement = "
        UPDATE publication
        SET revoked = TRUE
        WHERE name = $1 AND stele = $2
    ";
    match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            sqlx::query(statement)
                .bind(name)
                .bind(stele)
                .execute(&mut *connection)
                .await?;
        }
        DatabaseKind::Postgres => {
            unimplemented!()
        }
    }
    Ok(())
}
