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

pub async fn insert_stele(connection: &DatabaseConnection, stele: &str) {
    match connection.kind {
        DatabaseKind::Sqlite => {
            sqlx::query("INSERT OR IGNORE INTO stele (name) VALUES (?)")
                .bind(stele)
                .execute(&connection.pool)
                .await
                .unwrap();
        }
    }
}

/// Insert a publication row directly.
///
/// Passing `None` for `html_data_repo_name` reproduces the shape of a row written
/// before that column existed, which is otherwise unreachable through the normal
/// insert path.
pub async fn insert_publication(
    connection: &DatabaseConnection,
    id: &str,
    name: &str,
    date: &str,
    stele: &str,
    revoked: bool,
    html_data_repo_name: Option<&str>,
) {
    match connection.kind {
        DatabaseKind::Sqlite => {
            sqlx::query(
                "INSERT OR IGNORE INTO publication ( id, name, date, stele, revoked, html_data_repo_name )
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(id)
            .bind(name)
            .bind(date)
            .bind(stele)
            .bind(revoked)
            .bind(html_data_repo_name)
            .execute(&connection.pool)
            .await
            .unwrap();
        }
    }
}
