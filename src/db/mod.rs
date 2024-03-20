//! Database related module.
#![allow(clippy::unreachable)]
use async_trait::async_trait;
use std::str::FromStr;

use sqlx::any::{AnyPool, AnyPoolOptions};
use sqlx::ConnectOptions;
use tracing::instrument;

/// Database initialization.
pub mod init;
/// Statements for the database.
pub mod statements;

#[async_trait]
/// Generic Database
pub trait Db {
    /// Connects to a database.
    ///
    /// # Errors
    /// Errors if connection to database fails.
    async fn connect(url: &str) -> anyhow::Result<DatabaseConnection>;

    // async fn execute_statement(statement: &str, conn: &DatabaseConnection) -> anyhow::Result<()>;

    // async fn begin(&self) -> anyhow::Result<()>;

    // async fn close(&self) -> anyhow::Result<()>;
}

/// Type of database connection.
#[derive(Debug, Clone)]
pub enum DatabaseKind {
    /// Sqlite database.
    Sqlite,
    /// Postgres database.
    Postgres,
}

/// Database connection.
#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    /// Database connection pool.
    pub pool: AnyPool,
    /// Type of database connection.
    pub kind: DatabaseKind,
}

#[async_trait]
impl Db for DatabaseConnection {
    /// Connects to a database.
    ///
    /// # Errors
    /// Errors if connection to database fails.
    #[instrument(level = "trace")]
    async fn connect(db_url: &str) -> anyhow::Result<Self> {
        let options = sqlx::any::AnyConnectOptions::from_str(db_url)?
            .disable_statement_logging()
            .clone();
        let pool = AnyPoolOptions::new()
            .max_connections(50)
            .connect_with(options)
            .await?;
        let connection = match db_url {
            url if url.starts_with("sqlite://") => Self {
                pool,
                kind: DatabaseKind::Sqlite,
            },
            url if url.starts_with("postgres://") => Self {
                pool,
                kind: DatabaseKind::Postgres,
            },
            _ => anyhow::bail!("Unsupported database URL: {}", db_url),
        };

        Ok(connection)
    }
}
