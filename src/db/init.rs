//! Use this module to initialize database connection and run migrations.
use entity::sea_orm::{Database, DatabaseConnection};
use migration::{Migrator, MigratorTrait};
use std::env;
use std::path::{Path, PathBuf};

/// Connects to a database.
/// We use `SQLite` by default, but we can override this by setting the `DATABASE_URL` environment variable.
///
/// # Errors
/// Errors if connection to database fails.
/// Connections can fail if the database is not running, or if the database URL is invalid.
pub async fn connect(archive_path: &Path) -> anyhow::Result<DatabaseConnection> {
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        // Connect to SQLite database which is on file system in .stelae directory
        let sqlite_db_path = &archive_path.join(PathBuf::from(".stelae/db.sqlite3"));
        format!("sqlite://{}", sqlite_db_path.to_string_lossy())
    });

    let connection = Database::connect(&db_url).await?;
    // Run migrations
    match db_url {
        url if url.starts_with("sqlite://") => {
            Migrator::up(&connection, None).await?;
        }
        _ => {
            tracing::warn!("Migrations are not supported for this database. Skipping migrations.");
        }
    }
    Ok(connection)
}
