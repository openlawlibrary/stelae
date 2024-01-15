//! Database related module.
#![allow(clippy::unreachable)]
use tracing::instrument;

/// Database initialization.
pub mod init;

/// Generic Database
pub struct Database;

/// Database connection.
pub enum DatabaseConnection {
    /// SQLite connection.
    Sqlite(sqlx::SqlitePool),
    /// Postgres connection.
    Postgres(sqlx::PgPool),
}

impl Database {
    /// Connects to a database.
    ///
    /// # Errors
    /// Errors if connection to database fails.
    #[instrument(level = "trace")]
    pub async fn connect(db_url: &str) -> anyhow::Result<DatabaseConnection> {
        let connection = match db_url {
            url if url.starts_with("sqlite://") => {
                let pool = sqlx::SqlitePool::connect(url).await?;
                DatabaseConnection::Sqlite(pool)
            }
            url if url.starts_with("postgres://") => {
                let pool = sqlx::PgPool::connect(url).await?;
                DatabaseConnection::Postgres(pool)
            }
            _ => anyhow::bail!("Unsupported database URL: {}", db_url),
        };

        Ok(connection)
    }
}
