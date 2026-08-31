//use crate::db::{DatabaseConnection, DatabaseKind, Db as _};
use stelae::db::{DatabaseConnection, DatabaseKind, DatabaseTransaction, Tx as _};
use stelae::redirects::insert_redirects_for_stele;
use stelae::stelae::stele::Stele;

pub async fn insert_redirects(
    connection: &DatabaseConnection,
    stele: &str,
    repo_name: &str,
    redirects: Vec<(&str, &str)>,
) {
    for (from, to) in redirects {
        match connection.kind {
            DatabaseKind::Sqlite => {
                sqlx::query(
                    "INSERT OR IGNORE INTO redirects (stele_name, repo_name, from_url, to_url) VALUES (?, ?, ?, ?)",
                )
                .bind(stele)
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

pub async fn load_redirects(connection: &DatabaseConnection, stele: &mut Stele) {
    let mut tx = DatabaseTransaction::begin(connection.pool.clone())
        .await
        .unwrap();
    insert_redirects_for_stele(&mut tx, stele).await.unwrap();
    tx.commit().await.unwrap();
}
