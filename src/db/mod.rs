//! Database related module.
#![allow(clippy::unreachable)]
use async_trait::async_trait;
use sqlx::Transaction;
use std::str::FromStr;

use sqlx::any::{self, AnyPoolOptions};
use sqlx::AnyPool;
use sqlx::ConnectOptions;
use tracing::instrument;

/// Database initialization.
pub mod init;
/// Models for the database.
pub mod models;

#[async_trait]
/// Generic Database
pub trait Db {
    /// Connects to a database.
    ///
    /// # Errors
    /// Errors if connection to database fails.
    async fn connect(url: &str) -> anyhow::Result<DatabaseConnection>;
}

#[async_trait]
/// Generic transaction
pub trait Tx {
    /// Begin a transaction.
    async fn begin(pool: AnyPool) -> anyhow::Result<DatabaseTransaction>;
    /// Commit a transaction.
    async fn commit(self) -> anyhow::Result<()>;
    /// Rollback a transaction.
    async fn rollback(self) -> anyhow::Result<()>;
}

/// Type of database connection.
#[derive(Debug, Clone)]
pub enum DatabaseKind {
    /// Sqlite database.
    Sqlite,
}

/// Database connection.
#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    /// Database connection pool.
    pub pool: AnyPool,
    /// Type of database connection.
    pub kind: DatabaseKind,
}

/// Database transaction.
pub struct DatabaseTransaction {
    /// Database transaction.
    pub tx: Transaction<'static, sqlx::Any>,
}

#[async_trait]
impl Db for DatabaseConnection {
    /// Connects to a database.
    ///
    /// # Errors
    /// Errors if connection to database fails.
    #[instrument(level = "trace")]
    async fn connect(db_url: &str) -> anyhow::Result<Self> {
        any::install_default_drivers();
        let options = any::AnyConnectOptions::from_str(db_url)?.disable_statement_logging();
        let pool = AnyPoolOptions::new()
            .max_connections(50)
            .connect_with(options)
            .await?;
        let connection = match db_url {
            url if url.starts_with("sqlite:///") => Self {
                pool,
                kind: DatabaseKind::Sqlite,
            },
            _ => anyhow::bail!("Unsupported database URL: {}", db_url),
        };

        Ok(connection)
    }
}

#[async_trait]
impl Tx for DatabaseTransaction {
    /// Begin a transaction.
    async fn begin(pool: AnyPool) -> anyhow::Result<Self> {
        let tx = pool.begin().await?;
        Ok(Self { tx })
    }
    /// Commit a transaction.
    async fn commit(self) -> anyhow::Result<()> {
        self.tx.commit().await?;
        Ok(())
    }

    /// Rollback a transaction.
    async fn rollback(self) -> anyhow::Result<()> {
        self.tx.rollback().await?;
        Ok(())
    }
}
