//! The redirects module contains tools for interacting with the redirects of the Stele.
// The redirects module contains logic for inserting redirect objects into the database.
use crate::db::models::redirects;
use crate::db::models::redirects::RedirectPair;
use crate::db::DatabaseTransaction;
use crate::server::api::state;
use crate::stelae::stele::Stele;

/// Inserts multiple redirect mappings for a repository within an existing transaction.
///
/// This function delegates to the transactional redirect manager and ensures
/// that all redirect pairs are inserted atomically as part of the provided
/// database transaction.
///
/// # Arguments
///
/// * `tx` - The active database transaction.
/// * `stele` - The stele (tenant / namespace) identifier.
/// * `repo_name` - The repository name.
/// * `pairs` - A collection of redirect mappings to insert.
///
/// # Errors
///
/// Returns an error if the bulk insert fails
pub async fn insert_redirect_pairs(
    tx: &mut DatabaseTransaction,
    stele: &str,
    repo_name: &str,
    pairs: Vec<RedirectPair>,
) -> anyhow::Result<()> {
    redirects::TxManager::insert_bulk(tx, stele, repo_name, pairs).await?;
    Ok(())
}

/// Inserts redirect pairs for all repositories of a stele within an existing transaction.
///
/// # Arguments
///
/// * `tx` - The active database transaction.
/// * `stele` - The stele to process redirects for.
///
/// # Errors
///
/// Returns an error if repositories cannot be read or if any bulk insert fails
pub async fn insert_redirects_for_stele(
    tx: &mut DatabaseTransaction,
    stele: &mut Stele,
) -> anyhow::Result<()> {
    let Some(repositories) = stele.get_repositories()? else {
        return Ok(());
    };
    let all_repositories = repositories.get_all();
    for repository in all_repositories {
        let repo_state = state::init_repo(repository, stele)?;
        let pairs: Vec<RedirectPair> = repo_state
            .get_redirects()
            .into_iter()
            .map(|(from, to)| RedirectPair {
                from_url: from,
                to_url: to,
            })
            .collect();
        insert_redirect_pairs(tx, &stele.get_qualified_name(), &repo_state.name, pairs).await?;
    }
    Ok(())
}
