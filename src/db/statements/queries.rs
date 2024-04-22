//! Central place for database queries

use crate::db::models::publication::Publication;
use crate::db::models::publication_version::PublicationVersion;
use crate::db::models::stele::Stele;
use crate::db::DatabaseConnection;
use std::collections::HashSet;

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
    stele: &str,
) -> anyhow::Result<Option<Publication>> {
    let row = match conn.kind {
        DatabaseKind::Sqlite => {
            let statement: &'static str = r#"
                SELECT *
                FROM publication
                WHERE revoked = 0 AND stele = $1
                ORDER BY date DESC
                LIMIT 1
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, Publication>(statement)
                .bind(stele)
                .fetch_one(&mut *connection)
                .await
                .ok()
        }
        DatabaseKind::Postgres => {
            let statement: &'static str = r#"
                SELECT *
                FROM publication
                WHERE revoked = FALSE AND stele = $1
                ORDER BY date DESC
                LIMIT 1
            "#;
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, Publication>(statement)
                .bind(stele)
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
pub async fn find_publication_by_name_and_stele(
    conn: &DatabaseConnection,
    name: &str,
    stele: &str,
) -> anyhow::Result<Publication> {
    let statement: &'static str = r#"
        SELECT *
        FROM publication
        WHERE name = $1 AND stele = $2
    "#;
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
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, Publication>(statement)
                .bind(name)
                .bind(stele)
                .fetch_one(&mut *connection)
                .await?
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
    let statement: &'static str = r#"
        SELECT *
        FROM publication_version
        WHERE publication = $1 AND stele = $2
    "#;
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
            let mut connection = conn.pool.acquire().await?;
            sqlx::query_as::<_, PublicationVersion>(statement)
                .bind(publication)
                .bind(stele)
                .fetch_all(&mut *connection)
                .await?
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
            todo!("Postgres not implemented")
        }
    };
    Ok(rows)
}

/// Find all publication versions for a publication given a name and stele.
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
    checked_publication_names.insert(publication_name.to_string());

    let mut publication_names_to_check = HashSet::new();
    publication_names_to_check.insert(publication_name);

    while publication_names_to_check.len() > 0 {
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
            .filter(|pv| !checked_publication_names.contains(&pv.publication.to_string()))
            .map(|pv| pv.publication.to_string())
            .collect();
    }
    Ok(versions.into_iter().collect())
}

/// Find all publication names by date and stele.
pub async fn find_all_publications_by_date_and_stele_order_by_name_desc(
    conn: &DatabaseConnection,
    date: String,
    stele: String,
) -> anyhow::Result<Vec<Publication>> {
    let statement = r#"
                SELECT *
                FROM publication
                WHERE date = $1 AND stele = $2
                ORDER BY name DESC
            "#;
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
            todo!("Postgres not implemented");
        }
    };
    Ok(rows)
}