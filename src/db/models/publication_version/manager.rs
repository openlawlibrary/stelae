//! Manager for the `publication_version` model.
use super::PublicationVersion;
use crate::db::DatabaseTransaction;
use async_trait::async_trait;
use std::collections::HashSet;

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Upsert a new publication version into the database.
    ///
    /// # Errors
    /// Errors if the publication version cannot be inserted into the database.
    async fn create(
        &mut self,
        publication: &str,
        codified_date: &str,
        stele: &str,
    ) -> anyhow::Result<Option<i64>> {
        let statement = "
            INSERT OR IGNORE INTO publication_version ( publication, version, stele )
            VALUES ( $1, $2, $3 )
        ";
        let id = sqlx::query(statement)
            .bind(publication)
            .bind(codified_date)
            .bind(stele)
            .execute(&mut *self.tx)
            .await?
            .last_insert_id();
        Ok(id)
    }

    /// Find the last inserted publication version by a `stele` and `publication`.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_last_inserted_by_publication_and_stele(
        &mut self,
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
        let row = sqlx::query_as::<_, PublicationVersion>(statement)
            .bind(publication)
            .bind(stele)
            .fetch_one(&mut *self.tx)
            .await
            .ok();
        Ok(row)
    }

    /// Find a publication version by `publication_id` and `version`.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_all_by_publication_name_and_stele(
        &mut self,
        publication: &str,
        stele: &str,
    ) -> anyhow::Result<Vec<PublicationVersion>> {
        let statement = "
            SELECT *
            FROM publication_version
            WHERE publication = $1 AND stele = $2
        ";
        let rows = sqlx::query_as::<_, PublicationVersion>(statement)
            .bind(publication)
            .bind(stele)
            .fetch_all(&mut *self.tx)
            .await?;
        Ok(rows)
    }

    /// Find all publication versions in `publications`.
    async fn find_all_in_publication_has_publication_versions(
        &mut self,
        publications: Vec<String>,
        stele: &str,
    ) -> anyhow::Result<Vec<PublicationVersion>> {
        let parameters = publications
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ");
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
        let rows = query.fetch_all(&mut *self.tx).await?;

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
    async fn find_all_recursive_for_publication(
        &mut self,
        publication_name: String,
        stele: String,
    ) -> anyhow::Result<Vec<PublicationVersion>> {
        let mut versions: HashSet<PublicationVersion> = self
            .find_all_by_publication_name_and_stele(&publication_name, &stele)
            .await?
            .into_iter()
            .collect();

        let mut checked_publication_names = HashSet::new();
        checked_publication_names.insert(publication_name.clone());

        let mut publication_names_to_check = HashSet::new();
        publication_names_to_check.insert(publication_name);

        while !publication_names_to_check.is_empty() {
            let new_versions: HashSet<PublicationVersion> = self
                .find_all_in_publication_has_publication_versions(
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
}
