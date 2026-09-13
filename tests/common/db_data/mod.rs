//use crate::db::{DatabaseConnection, DatabaseKind, Db as _};
use taf_server::db::{DatabaseConnection, DatabaseKind, DatabaseTransaction, Tx as _};
use taf_server::fonds::fonds::Fonds;
use taf_server::redirects::insert_redirects_for_fonds;

pub async fn insert_redirects(
    connection: &DatabaseConnection,
    fonds: &str,
    repo_name: &str,
    redirects: Vec<(&str, &str)>,
) {
    for (from, to) in redirects {
        match connection.kind {
            DatabaseKind::Sqlite => {
                sqlx::query(
                    "INSERT OR IGNORE INTO redirects (fonds_name, repo_name, from_url, to_url) VALUES (?, ?, ?, ?)",
                )
                .bind(fonds)
                .bind(repo_name)
                .bind(from)
                .bind(to)
                .execute(&connection.pool)
                .await
                .unwrap();
            }
        }
    }
}

pub async fn load_redirects(connection: &DatabaseConnection, fonds: &mut Fonds) {
    let mut tx = DatabaseTransaction::begin(connection.pool.clone())
        .await
        .unwrap();
    insert_redirects_for_fonds(&mut tx, fonds).await.unwrap();
    tx.commit().await.unwrap();
}
