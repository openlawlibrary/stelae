//! Manager for the data repo commit model.
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::QueryBuilder;

use crate::db::{models::BATCH_SIZE, DatabaseTransaction};

use super::DataRepoCommits;

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Find all authentication commits for a given stele.
    ///
    /// # Errors
    /// Errors if the commits cannot be found.
    async fn find_all_auth_commits_for_stele(
        &mut self,
        stele_id: &str,
    ) -> anyhow::Result<Vec<DataRepoCommits>> {
        let query = "
            SELECT dc.*
            FROM data_repo_commits dc
            LEFT JOIN publication p ON dc.publication_id = p.id
            LEFT JOIN stele s ON p.stele = s.name
            WHERE s.name = $1
        ";
        let data_repo_commits = sqlx::query_as::<_, DataRepoCommits>(query)
            .bind(stele_id)
            .fetch_all(&mut *self.tx)
            .await?;
        Ok(data_repo_commits)
    }
    /// Upsert a bulk of data repository commits into the database.
    ///
    /// # Errors
    /// Errors if the commits cannot be inserted.
    async fn insert_bulk(&mut self, data_repo_commits: Vec<DataRepoCommits>) -> anyhow::Result<()> {
        let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO data_repo_commits ( commit_hash, codified_date, build_date, repo_type, auth_commit_hash, auth_commit_timestamp, publication_id ) ");
        for chunk in data_repo_commits.chunks(BATCH_SIZE) {
            query_builder.push_values(chunk, |mut bindings, dc| {
                bindings
                    .push_bind(&dc.commit_hash)
                    .push_bind(&dc.codified_date)
                    .push_bind(&dc.build_date)
                    .push_bind(&dc.repo_type)
                    .push_bind(&dc.auth_commit_hash)
                    .push_bind(&dc.auth_commit_timestamp)
                    .push_bind(&dc.publication_id);
            });
            let query = query_builder.build();
            query.execute(&mut *self.tx).await?;
            query_builder.reset();
        }
        Ok(())
    }
    /// Finds the most recent authentication commit for a given publication
    /// that is valid for the specified version date.
    ///
    /// If `version_date` is a valid ISO date (`YYYY-MM-DD`), the commit returned
    /// is the latest one whose `codified_date` is **less than or equal to**
    /// the provided date.  
    /// If `version_date` is not a valid ISO date, the latest available commit
    /// for the publication is returned regardless of date.
    ///
    /// # Errors
    /// Returns an error if the commit cannot be retrieved from the database
    /// or if the query execution fails.
    async fn find_commit_by_pub_id_and_version_date(
        &mut self,
        publication_id: &str,
        version_date: &str,
    ) -> anyhow::Result<DataRepoCommits> {
        let is_iso = NaiveDate::parse_from_str(version_date, "%Y-%m-%d").is_ok();
        let query = if is_iso {
            "
            SELECT *
            FROM data_repo_commits
            WHERE publication_id = $1
            AND codified_date <= $2
            ORDER BY codified_date DESC
            LIMIT 1
            "
        } else {
            "
            SELECT *
            FROM data_repo_commits
            WHERE publication_id = $1
            ORDER BY codified_date DESC
            LIMIT 1
            "
        };

        let mut commit: Option<DataRepoCommits> = if is_iso {
            sqlx::query_as::<_, DataRepoCommits>(query)
                .bind(publication_id)
                .bind(version_date)
                .fetch_optional(&mut *self.tx)
                .await?
        } else {
            sqlx::query_as::<_, DataRepoCommits>(query)
                .bind(publication_id)
                .fetch_optional(&mut *self.tx)
                .await?
        };

        if commit.is_none() {
            let fallback_query = "
                SELECT *
                FROM data_repo_commits
                WHERE publication_id = $1
                ORDER BY build_date DESC
                LIMIT 1
            ";

            commit = sqlx::query_as::<_, DataRepoCommits>(fallback_query)
                .bind(publication_id)
                .fetch_optional(&mut *self.tx)
                .await?;
        }

        commit.ok_or_else(|| {
            anyhow!(
                "No commit found for publication_id={} and version_date={}",
                publication_id,
                version_date
            )
        })
    }
}
