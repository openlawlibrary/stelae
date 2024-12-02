use crate::db::{DatabaseConnection, DatabaseKind, Db};
use std::env;
use std::path::{Path, PathBuf};
/// Connects to a database and applies migrations.
/// We use `SQLite` by default, but we can override this by setting the `DATABASE_URL` environment variable.
///
/// # Errors
/// Errors if connection to database fails.
/// Connections can fail if the database is not running, or if the database URL is invalid.
pub async fn connect(archive_path: &Path) -> anyhow::Result<DatabaseConnection> {
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        let sqlite_db_path = &archive_path.join(PathBuf::from(".taf/db.sqlite3"));
        format!("sqlite:///{}?mode=rwc", sqlite_db_path.to_string_lossy())
    });
    let connection = DatabaseConnection::connect(&db_url).await?;
    tracing::info!("Connected to database");
    match connection.kind {
        DatabaseKind::Sqlite => {
            sqlx::migrate!("./migrations/sqlite")
                .run(&connection.pool)
                .await?;
        }
    }
    Ok(connection)
}
