//! Central place for database queries

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::db::models::publication::{self, Publication};
use crate::db::models::publication_version::PublicationVersion;
use crate::db::models::stele::Stele;
use crate::db::models::version::Version;
use crate::db::models::{document_change, library_change};
use crate::db::DatabaseConnection;
use std::collections::HashSet;

use crate::db::DatabaseKind;

/// Find a stele by `name`.
///
/// # Errors
/// Errors if can't establish a connection to the database.
pub async fn find_stele_by_name(conn: &DatabaseConnection, name: &str) -> anyhow::Result<Stele> {
    let statement = "
        SELECT *
        FROM stele
        WHERE name = $1
    ";
    let row = match conn.kind {
        DatabaseKind::Postgres | DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, Stele>(statement)
                .bind(name)
                .fetch_one(&mut *connection)
                .await?
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
    stele: &str,
) -> anyhow::Result<Option<Publication>> {
    let statement = "
        SELECT *
        FROM publication
        WHERE revoked = 0 AND stele = $1
        ORDER BY date DESC
        LIMIT 1
    ";
    let row = match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, Publication>(statement)
                .bind(stele)
                .fetch_one(&mut *connection)
                .await
                .ok()
        }
        DatabaseKind::Postgres => {
            unimplemented!()
        }
    };
    Ok(row)
}

/// Find a publication by `name` and `date` and `stele_id`.
///
/// # Errors
/// Errors if can't establish a connection to the database.
pub async fn find_publication_by_name_and_stele(
    conn: &DatabaseConnection,
    name: &str,
    stele: &str,
) -> anyhow::Result<Publication> {
    let statement = "
        SELECT *
        FROM publication
        WHERE name = $1 AND stele = $2
    ";
    let row = match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, Publication>(statement)
                .bind(name)
                .bind(stele)
                .fetch_one(&mut *connection)
                .await?
        }
        DatabaseKind::Postgres => {
            unimplemented!()
        }
    };
    Ok(row)
}

/// Find a publication version by `publication_id` and `version`.
///
/// # Errors
/// Errors if can't establish a connection to the database.
pub async fn find_all_publication_versions_by_publication_name_and_stele(
    conn: &DatabaseConnection,
    publication: &str,
    stele: &str,
) -> anyhow::Result<Vec<PublicationVersion>> {
    let statement = "
        SELECT *
        FROM publication_version
        WHERE publication = $1 AND stele = $2
    ";
    let rows = match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, PublicationVersion>(statement)
                .bind(publication)
                .bind(stele)
                .fetch_all(&mut *connection)
                .await?
        }
        DatabaseKind::Postgres => {
            unimplemented!()
        }
    };
    Ok(rows)
}

/// Find all publication versions in `publications`.
async fn find_all_publication_versions_in_publication_has_publication_versions(
    conn: &DatabaseConnection,
    publications: Vec<String>,
    stele: &str,
) -> anyhow::Result<Vec<PublicationVersion>> {
    let parameters = publications
        .iter()
        .map(|_| "?")
        .collect::<Vec<&str>>()
        .join(", ");
    let rows = match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;

            let statement = format!("
                SELECT DISTINCT pv.publication, pv.version
                FROM publication_version pv
                LEFT JOIN publication_has_publication_versions phpv ON pv.publication = phpv.referenced_publication AND pv.version = phpv.referenced_version
                WHERE phpv.publication IN ({parameters} AND pv.stele = ?)
            ");

            let mut query = sqlx::query_as::<_, PublicationVersion>(&statement);
            for publication in publications {
                query = query.bind(publication);
            }
            query = query.bind(stele);

            query.fetch_all(&mut *connection).await?
        }
        DatabaseKind::Postgres => {
            unimplemented!()
        }
    };
    Ok(rows)
}

/// Recursively find all publication versions starting from a given publication ID.

/// This is necessary publication versions can be the same across publications.
/// To make versions query simpler, we walk the publication hierarchy starting from
/// `publication_name` looking for related publications.
/// The function returns all the `publication_version` IDs, even in simple cases where a publication
/// has no hierarchy.
///
/// # Errors
/// Errors if can't establish a connection to the database.
pub async fn find_publication_versions_for_publication(
    conn: &DatabaseConnection,
    publication_name: String,
    stele: String,
) -> anyhow::Result<Vec<PublicationVersion>> {
    let mut versions: HashSet<PublicationVersion> =
        find_all_publication_versions_by_publication_name_and_stele(
            conn,
            &publication_name,
            &stele,
        )
        .await?
        .into_iter()
        .collect();

    let mut checked_publication_names = HashSet::new();
    checked_publication_names.insert(publication_name.clone());

    let mut publication_names_to_check = HashSet::new();
    publication_names_to_check.insert(publication_name);

    while !publication_names_to_check.is_empty() {
        let new_versions: HashSet<PublicationVersion> =
            find_all_publication_versions_in_publication_has_publication_versions(
                conn,
                publication_names_to_check.clone().into_iter().collect(),
                &stele,
            )
            .await?
            .into_iter()
            .collect();
        versions.extend(new_versions.clone());

        checked_publication_names.extend(publication_names_to_check.clone());

        publication_names_to_check = new_versions
            .clone()
            .into_iter()
            .filter(|pv| !checked_publication_names.contains(&pv.publication.clone()))
            .map(|pv| pv.publication)
            .collect();
    }
    Ok(versions.into_iter().collect())
}

/// Find all publication names by date and stele.
///
/// # Errors
/// Errors if can't establish a connection to the database.
pub async fn find_all_publications_by_date_and_stele_order_by_name_desc(
    conn: &DatabaseConnection,
    date: String,
    stele: String,
) -> anyhow::Result<Vec<Publication>> {
    let statement = "
        SELECT *
        FROM publication
        WHERE date = $1 AND stele = $2
        ORDER BY name DESC
    ";
    let rows = match conn.kind {
        DatabaseKind::Sqlite => {
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, Publication>(statement)
                .bind(date)
                .bind(stele)
                .fetch_all(&mut *connection)
                .await?
        }
        DatabaseKind::Postgres => {
            unimplemented!();
        }
    };
    Ok(rows)
}

/// Find last inserted publication version in DB.
/// Used when partially inserted new changes to the database.
///
/// # Errors
/// Errors if can't establish a connection to the database.
pub async fn find_last_inserted_publication_version_by_publication_and_stele(
    conn: &DatabaseConnection,
    publication: &str,
    stele: &str,
) -> anyhow::Result<Option<PublicationVersion>> {
    let statement = "
        SELECT *
        FROM publication_version
        WHERE publication = $1 AND stele = $2
        ORDER BY version DESC
        LIMIT 1
    ";
    let row = match conn.kind {
        DatabaseKind::Sqlite | DatabaseKind::Postgres => {
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, PublicationVersion>(statement)
                .bind(publication)
                .bind(stele)
                .fetch_one(&mut *connection)
                .await
                .ok()
        }
    };
    Ok(row)
}

#[async_trait]
impl document_change::Manager for DatabaseConnection {
    /// Find one document materialized path by url.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_doc_mpath_by_url(&self, url: &str) -> anyhow::Result<String> {
        let statement = "
            SELECT doc_mpath
            FROM document_change
            WHERE url = $1
            LIMIT 1
        ";
        let row = match self.kind {
            DatabaseKind::Postgres | DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, (String,)>(statement)
                    .bind(url)
                    .fetch_one(&mut *connection)
                    .await?
            }
        };
        Ok(row.0)
    }

    /// All dates on which given document changed.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_all_document_versions_by_mpath_and_publication(
        &self,
        mpath: &str,
        publication: &str,
    ) -> anyhow::Result<Vec<Version>> {
        let mut statement = "
            SELECT DISTINCT phpv.referenced_version as codified_date
            FROM document_change dc
            LEFT JOIN publication_has_publication_versions phpv
                ON dc.publication = phpv.referenced_publication
                AND dc.version = phpv.referenced_version
            WHERE dc.doc_mpath LIKE $1 AND phpv.publication = $2
        ";
        let mut rows = match self.kind {
            DatabaseKind::Postgres | DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, Version>(statement)
                    .bind(format!("{mpath}%"))
                    .bind(publication)
                    .fetch_all(&mut *connection)
                    .await?
            }
        };
        statement = "
            SELECT phpv.referenced_version as codified_date
            FROM document_change dc
            LEFT JOIN publication_has_publication_versions phpv
                ON dc.publication = phpv.referenced_publication
                AND dc.version = phpv.referenced_version
            WHERE dc.doc_mpath = $1
            AND phpv.publication = $2
            AND dc.status = 'Element added'
            LIMIT 1
        ";
        let element_added = match self.kind {
            DatabaseKind::Postgres | DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, Version>(statement)
                    .bind(mpath)
                    .bind(publication)
                    .fetch_one(&mut *connection)
                    .await
                    .ok()
            }
        };

        if element_added.is_none() {
            // When element doesn't have date added, it means we're looking
            // at an old publication and this element doesn't yet exist in it
            rows.sort_by(|v1, v2| v2.codified_date.cmp(&v1.codified_date));
            return Ok(rows);
        }

        statement = "
            SELECT phpv.referenced_version as codified_date
            FROM document_change dc
            LEFT JOIN publication_has_publication_versions phpv
                ON dc.publication = phpv.referenced_publication
                AND dc.version = phpv.referenced_version
            WHERE dc.doc_mpath = $1
            AND phpv.publication = $2
            AND dc.status = 'Element effective'
            LIMIT 1
        ";
        let mut doc = mpath.split('|').next().unwrap_or("").to_owned();
        doc.push('|');

        let document_effective = match self.kind {
            DatabaseKind::Postgres | DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, Version>(statement)
                    .bind(doc)
                    .bind(publication)
                    .fetch_one(&mut *connection)
                    .await
                    .ok()
            }
        };

        if let (Some(doc_effective), Some(el_added)) = (document_effective, element_added) {
            if !rows.contains(&doc_effective)
                && NaiveDate::parse_from_str(&doc_effective.codified_date, "%Y-%m-%d")
                    .unwrap_or_default()
                    > NaiveDate::parse_from_str(&el_added.codified_date, "%Y-%m-%d")
                        .unwrap_or_default()
            {
                rows.push(doc_effective);
            }
        }
        rows.sort_by(|v1, v2| v2.codified_date.cmp(&v1.codified_date));
        Ok(rows)
    }
}

#[async_trait]
impl library_change::Manager for DatabaseConnection {
    /// Find one library materialized path by url.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_lib_mpath_by_url(&self, url: &str) -> anyhow::Result<String> {
        let statement = "
            SELECT library_mpath
            FROM library_change
            WHERE url = $1
            LIMIT 1
        ";
        let row = match self.kind {
            DatabaseKind::Postgres | DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, (String,)>(statement)
                    .bind(url)
                    .fetch_one(&mut *connection)
                    .await?
            }
        };
        Ok(row.0)
    }
    /// All dates on which documents from this collection changed.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_all_collection_versions_by_mpath_and_publication(
        &self,
        mpath: &str,
        publication: &str,
    ) -> anyhow::Result<Vec<Version>> {
        let mut statement = "
            SELECT DISTINCT phpv.referenced_version as codified_date
            FROM changed_library_document cld
            LEFT JOIN publication_has_publication_versions phpv
                ON cld.publication = phpv.referenced_publication
                AND cld.version = phpv.referenced_version
            WHERE cld.library_mpath LIKE $1 AND phpv.publication = $2
        ";
        let mut rows = match self.kind {
            DatabaseKind::Postgres | DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, Version>(statement)
                    .bind(format!("{mpath}%"))
                    .bind(publication)
                    .fetch_all(&mut *connection)
                    .await?
            }
        };
        statement = "
            SELECT DISTINCT phpv.referenced_version as codified_date
            FROM library_change lc
            LEFT JOIN publication_has_publication_versions phpv
                ON lc.publication = phpv.referenced_publication
                AND lc.version = phpv.referenced_version
            WHERE lc.library_mpath LIKE $1 AND lc.status = 'Element added' AND phpv.publication = $2
            LIMIT 1
            ";
        let element_added = match self.kind {
            DatabaseKind::Postgres | DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, Version>(statement)
                    .bind(format!("{mpath}%"))
                    .bind(publication)
                    .fetch_one(&mut *connection)
                    .await
                    .ok()
            }
        };

        if let Some(el_added) = element_added {
            if !rows.contains(&el_added) {
                rows.push(el_added);
            }
        }
        rows.sort_by(|v1, v2| v2.codified_date.cmp(&v1.codified_date));
        Ok(rows)
    }
}

#[async_trait]
impl publication::Manager for DatabaseConnection {
    /// Find all publications which are not revoked for a given stele.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_all_non_revoked_publications(
        &self,
        stele: &str,
    ) -> anyhow::Result<Vec<Publication>> {
        let statement = "
            SELECT *
            FROM publication
            WHERE revoked = 0 AND stele = $1
            ORDER BY name DESC
        ";
        let rows = match self.kind {
            DatabaseKind::Postgres | DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, Publication>(statement)
                    .bind(stele)
                    .fetch_all(&mut *connection)
                    .await?
            }
        };
        Ok(rows)
    }
}
