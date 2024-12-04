//! Module for inserting changes into the database
#![allow(
    clippy::exit,
    clippy::shadow_reuse,
    clippy::future_not_send,
    clippy::string_add
)]
use super::rdf::graph::Bag;
use crate::db::models::changed_library_document::{self, ChangedLibraryDocument};
use crate::db::models::document_change::{self, DocumentChange};
use crate::db::models::document_element::DocumentElement;
use crate::db::models::library::{self, Library};
use crate::db::models::library_change::{self, LibraryChange};
use crate::db::models::publication::{self, Publication};
use crate::db::models::publication_has_publication_versions::{
    self, PublicationHasPublicationVersions,
};
use crate::db::models::publication_version;
use crate::db::models::status::Status;
use crate::db::models::{document, document_element};
use crate::db::models::{stele, version};
use crate::db::{DatabaseTransaction, Tx};
use crate::history::rdf::graph::StelaeGraph;
use crate::history::rdf::namespaces::{dcterms, oll};
use crate::server::errors::CliError;
use crate::stelae::stele::Stele;
use crate::utils::archive::get_name_parts;
use crate::utils::git::Repo;
use crate::utils::md5;
use crate::{
    db::{self, DatabaseConnection},
    stelae::archive::Archive,
};
use anyhow::Context;
use git2::{TreeWalkMode, TreeWalkResult};
use sophia::api::ns::rdfs;
use sophia::api::{prelude::*, term::SimpleTerm};
use sophia::xml::parser;
use sqlx::types::chrono::NaiveDate;
use std::{
    borrow::ToOwned,
    io::{self, BufReader},
    path::{Path, PathBuf},
    result::Result,
};

/// Inserts changes from the archive into the database
///
/// # Errors
/// Errors if the changes cannot be inserted into the archive
#[actix_web::main]
#[tracing::instrument(name = "Stelae update", skip(raw_archive_path, archive_path))]
pub async fn insert(raw_archive_path: &str, archive_path: PathBuf) -> Result<(), CliError> {
    let conn = match db::init::connect(&archive_path).await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!(
                "error: could not connect to database.
                Confirm that `db.sqlite3` exists in `.stelae` dir or that DATABASE_URL env var is set correctly."
            );
            tracing::error!("Error: {err:?}");
            return Err(CliError::DatabaseConnectionError);
        }
    };
    insert_changes_archive(&conn, raw_archive_path, &archive_path)
        .await
        .map_err(|err| {
            tracing::error!("Failed to insert changes into archive");
            tracing::error!("{err:?}");
            CliError::GenericError
        })
}

/// Insert changes from the archive into the database
async fn insert_changes_archive(
    conn: &DatabaseConnection,
    raw_archive_path: &str,
    archive_path: &Path,
) -> anyhow::Result<()> {
    tracing::debug!("Inserting history into archive");

    let archive = Archive::parse(
        archive_path.to_path_buf(),
        &PathBuf::from(raw_archive_path),
        false,
    )?;

    let mut errors = Vec::new();
    for (name, mut stele) in archive.get_stelae() {
        let mut tx = DatabaseTransaction {
            tx: conn.pool.begin().await?,
        };
        match process_stele(&mut tx, &name, &mut stele, archive_path).await {
            Ok(()) => {
                tracing::debug!("Applying transaction for stele: {name}");
                tx.commit().await?;
            }
            Err(err) => {
                tracing::error!("Rolling back transaction for stele: {name} due to error: {err:?}");
                tx.rollback().await?;
                errors.push(format!("{name}: {err}"));
            }
        }
    }
    if !errors.is_empty() {
        let error_msg = errors.into_iter().collect::<Vec<_>>().join("\n");
        return Err(anyhow::anyhow!(
            "Errors occurred while inserting changes:\n{error_msg}"
        ));
    }
    Ok(())
}

/// Process the stele and insert changes into the database
async fn process_stele(
    tx: &mut DatabaseTransaction,
    name: &str,
    stele: &mut Stele,
    archive_path: &Path,
) -> anyhow::Result<()> {
    let Some(repositories) = stele.get_repositories()? else {
        tracing::warn!("No repositories found for stele: {name}");
        return Ok(());
    };
    let Some(rdf_repo) = repositories.get_rdf_repository() else {
        tracing::warn!("No RDF repository found for stele: {name}");
        return Ok(());
    };
    let rdf_repo_path = archive_path.to_path_buf().join(&rdf_repo.name);
    if !rdf_repo_path.exists() {
        return Err(anyhow::anyhow!(
            "RDF repository should exist on disk but not found: {}",
            rdf_repo_path.display()
        ));
    }
    let (rdf_org, rdf_name) = get_name_parts(&rdf_repo.name)?;
    let rdf = Repo::new(archive_path, &rdf_org, &rdf_name)?;
    insert_changes_from_rdf_repository(tx, rdf, name).await?;
    // insert_commit_hashes_from_auth_repository(tx, &stele.auth_repo).await?;
    Ok(())
}

/// Insert changes from the RDF repository into the database
async fn insert_changes_from_rdf_repository(
    tx: &mut DatabaseTransaction,
    rdf_repo: Repo,
    stele_id: &str,
) -> anyhow::Result<()> {
    tracing::info!("Inserting changes from RDF repository: {}", stele_id);
    tracing::info!("RDF repository path: {}", rdf_repo.path.display());
    load_delta_for_stele(tx, &rdf_repo, stele_id).await?;
    Ok(())
}

/// Load deltas from the publications
async fn load_delta_for_stele(
    tx: &mut DatabaseTransaction,
    rdf_repo: &Repo,
    stele: &str,
) -> anyhow::Result<()> {
    stele::TxManager::create(tx, stele).await?;
    if let Some(publication) = publication::TxManager::find_last_inserted(tx, stele).await? {
        tracing::info!("Inserting changes from last inserted publication");
        load_delta_from_publications(tx, rdf_repo, stele, Some(publication)).await?;
    } else {
        tracing::info!("Inserting changes from beginning for stele: {}", stele);
        load_delta_from_publications(tx, rdf_repo, stele, None).await?;
    }
    Ok(())
}

/// Iterate and load delta from all publications in the `_publication` directory
///
/// # Errors
/// Errors if the delta cannot be loaded from the publications
#[allow(clippy::unwrap_used)]
async fn load_delta_from_publications(
    tx: &mut DatabaseTransaction,
    rdf_repo: &Repo,
    stele: &str,
    last_inserted_publication: Option<Publication>,
) -> anyhow::Result<()> {
    let head_commit = rdf_repo.repo.head()?.peel_to_commit()?;
    let tree = head_commit.tree()?;
    let publications_dir_entry = tree.get_path(&PathBuf::from("_publication"))?;
    let publications_subtree = rdf_repo.repo.find_tree(publications_dir_entry.id())?;
    let mut last_inserted_date: Option<NaiveDate> = None;
    let last_inserted_pub_date = if let Some(last_inserted_pub) = last_inserted_publication.as_ref()
    {
        last_inserted_date =
            publication_version::TxManager::find_last_inserted_date_by_publication_id(
                tx,
                &last_inserted_pub.id,
            )
            .await?
            .map(|pv| {
                NaiveDate::parse_from_str(&pv.version, "%Y-%m-%d").context("Could not parse date")
            })
            .and_then(Result::ok);
        Some(NaiveDate::parse_from_str(
            &last_inserted_pub.date,
            "%Y-%m-%d",
        )?)
    } else {
        None
    };
    for publication_entry in &publications_subtree {
        let mut pub_graph = StelaeGraph::new();
        let object = publication_entry.to_object(&rdf_repo.repo)?;
        let publication_tree = object
            .as_tree()
            .context("Expected a tree but got something else")?;
        let index_rdf = publication_tree.get_path(&PathBuf::from("index.rdf"))?;
        let blob = rdf_repo.repo.find_blob(index_rdf.id())?;
        let data = blob.content();
        let reader = io::BufReader::new(data);
        parser::parse_bufread(reader).add_to_graph(&mut pub_graph.fast_graph)?;
        let pub_label = pub_graph.literal_from_triple_matching(None, Some(rdfs::label), None)?;
        let pub_name = pub_label
            .strip_prefix("Publication ")
            .context("Could not strip prefix")?
            .to_owned();
        let pub_date =
            pub_graph.literal_from_triple_matching(None, Some(dcterms::available), None)?;
        let pub_date = NaiveDate::parse_from_str(pub_date.as_str(), "%Y-%m-%d")?;
        // continue from last inserted publication, since that publication can contain
        // new changes (versions) that are not in db
        if let Some(last_inserted_publication_date) = last_inserted_pub_date {
            if pub_date < last_inserted_publication_date {
                // skip past publications since they are already in db
                continue;
            }
        }
        tracing::info!("[{stele}] | Publication: {pub_name}");
        publication_tree.walk(TreeWalkMode::PreOrder, |_, entry| {
            let path_name = entry.name().unwrap_or_default();
            if path_name.contains(".rdf") {
                match rdf_repo.repo.find_blob(entry.id()) {
                    Ok(current_blob) => {
                        let current_content = current_blob.content();
                        if let Err(err) = parser::parse_bufread(BufReader::new(current_content))
                            .add_to_graph(&mut pub_graph.fast_graph)
                        {
                            tracing::error!(
                                "Error adding content to graph for entry {path_name}: {err:?}"
                            );
                        }
                    }
                    Err(err) => {
                        tracing::error!("Error finding blob for entry {path_name}: {err:?}");
                    }
                }
            }
            TreeWalkResult::Ok
        })?;
        let (last_valid_pub_name, last_valid_codified_date) =
            referenced_publication_information(&pub_graph);
        let publication_hash = md5::compute(pub_name.clone() + stele);
        let last_inserted_pub_id = if let Some(valid_pub_name) = last_valid_pub_name {
            let last_inserted_pub =
                publication::TxManager::find_by_name_and_stele(tx, &valid_pub_name, stele).await?;
            Some(last_inserted_pub.id)
        } else {
            None
        };
        publication::TxManager::create(
            tx,
            &publication_hash,
            &pub_name,
            &pub_date,
            stele,
            last_inserted_pub_id,
            last_valid_codified_date,
        )
        .await?;
        let publication =
            publication::TxManager::find_by_name_and_stele(tx, &pub_name, stele).await?;
        load_delta_for_publication(tx, publication, &pub_graph, last_inserted_date).await?;
        // reset last inserted date for next publication
        last_inserted_date = None;
    }
    Ok(())
}

/// Load all deltas for the publication given a stele
///
/// # Errors
/// Errors if database connection fails or if delta cannot be loaded for the publication
async fn load_delta_for_publication(
    tx: &mut DatabaseTransaction,
    publication: Publication,
    pub_graph: &StelaeGraph,
    last_inserted_date: Option<NaiveDate>,
) -> anyhow::Result<()> {
    let pub_document_versions =
        pub_graph.all_iris_from_triple_matching(None, None, Some(oll::DocumentVersion))?;
    let pub_collection_versions =
        pub_graph.all_iris_from_triple_matching(None, None, Some(oll::CollectionVersion))?;

    insert_document_changes(
        tx,
        &last_inserted_date,
        pub_document_versions,
        pub_graph,
        &publication,
    )
    .await?;

    insert_library_changes(
        tx,
        &last_inserted_date,
        pub_collection_versions,
        pub_graph,
        &publication,
    )
    .await?;

    insert_shared_publication_versions_for_publication(tx, &publication).await?;

    revoke_same_date_publications(tx, publication).await?;

    Ok(())
}

/// Insert document changes into the database
async fn insert_document_changes(
    tx: &mut DatabaseTransaction,
    last_inserted_date: &Option<NaiveDate>,
    pub_document_versions: Vec<&SimpleTerm<'_>>,
    pub_graph: &StelaeGraph,
    publication: &Publication,
) -> anyhow::Result<()> {
    let mut document_elements_bulk: Vec<DocumentElement> = vec![];
    let mut document_changes_bulk: Vec<DocumentChange> = vec![];
    for version in pub_document_versions {
        let codified_date =
            pub_graph.literal_from_triple_matching(Some(version), Some(oll::codifiedDate), None)?;
        if let Some(last_inserted_date) = last_inserted_date.as_ref() {
            let codified_date = NaiveDate::parse_from_str(codified_date.as_str(), "%Y-%m-%d")?;
            if &codified_date <= last_inserted_date {
                // Date already inserted
                continue;
            }
        }
        version::TxManager::create(tx, &codified_date).await?;
        let pub_version_hash =
            md5::compute(publication.name.clone() + &codified_date + &publication.stele);
        publication_version::TxManager::create(
            tx,
            &pub_version_hash,
            &publication.id,
            &codified_date,
        )
        .await?;
        let doc_id =
            pub_graph.literal_from_triple_matching(Some(version), Some(oll::docId), None)?;
        document::TxManager::create(tx, &doc_id).await?;
        let Ok(changes_uri) =
            pub_graph.iri_from_triple_matching(Some(version), Some(oll::hasChanges), None)
        else {
            continue;
        };
        let changes = Bag::new(pub_graph, changes_uri);
        for change in changes.items()? {
            let doc_mpath = pub_graph.literal_from_triple_matching(
                Some(&change),
                Some(oll::documentMaterializedPath),
                None,
            )?;
            let url =
                pub_graph.literal_from_triple_matching(Some(&change), Some(oll::url), None)?;
            document_elements_bulk.push(DocumentElement::new(
                doc_mpath.clone(),
                url.clone(),
                doc_id.clone(),
                publication.stele.clone(),
            ));
            let reason = pub_graph
                .literal_from_triple_matching(Some(&change), Some(oll::reason), None)
                .ok();
            let statuses = pub_graph.all_literals_from_triple_matching(
                Some(&change),
                Some(oll::status),
                None,
            )?;
            for el_status in statuses {
                let status = Status::from_string(&el_status)?;
                let document_change_hash = md5::compute(
                    pub_version_hash.clone() + &doc_mpath.clone() + &status.to_int().to_string(),
                );
                document_changes_bulk.push(DocumentChange::new(
                    document_change_hash,
                    status.to_int(),
                    reason.clone(),
                    pub_version_hash.clone(),
                    doc_mpath.clone(),
                ));
            }
        }
    }
    document_element::TxManager::insert_bulk(tx, document_elements_bulk).await?;
    document_change::TxManager::insert_bulk(tx, document_changes_bulk).await?;
    Ok(())
}

/// Insert library changes into the database
async fn insert_library_changes(
    tx: &mut DatabaseTransaction,
    last_inserted_date: &Option<NaiveDate>,
    pub_collection_versions: Vec<&SimpleTerm<'_>>,
    pub_graph: &StelaeGraph,
    publication: &Publication,
) -> anyhow::Result<()> {
    let mut library_changes_bulk: Vec<LibraryChange> = vec![];
    let mut changed_library_document_bulk: Vec<ChangedLibraryDocument> = vec![];
    let mut library_bulk: Vec<Library> = vec![];
    for version in pub_collection_versions {
        let codified_date =
            pub_graph.literal_from_triple_matching(Some(version), Some(oll::codifiedDate), None)?;
        if let Some(last_inserted_date) = last_inserted_date.as_ref() {
            let codified_date = NaiveDate::parse_from_str(codified_date.as_str(), "%Y-%m-%d")?;
            if &codified_date <= last_inserted_date {
                // Date already inserted
                continue;
            }
        }
        let library_mpath = pub_graph.literal_from_triple_matching(
            Some(version),
            Some(oll::libraryMaterializedPath),
            None,
        )?;
        let url = pub_graph.literal_from_triple_matching(Some(version), Some(oll::url), None)?;
        let el_status =
            pub_graph.literal_from_triple_matching(Some(version), Some(oll::status), None)?;
        let library_status = Status::from_string(&el_status)?;
        library_bulk.push(Library::new(
            library_mpath.clone(),
            url.clone(),
            publication.stele.clone(),
        ));
        let pub_version_hash =
            md5::compute(publication.name.clone() + &codified_date + &publication.stele);
        library_changes_bulk.push(LibraryChange::new(
            pub_version_hash.clone(),
            library_status.to_int(),
            library_mpath.clone(),
        ));
        let changes_uri =
            pub_graph.iri_from_triple_matching(Some(version), Some(oll::hasChanges), None)?;
        let changes = Bag::new(pub_graph, changes_uri);
        for change in changes.items()? {
            let Ok(found_status) =
                pub_graph.literal_from_triple_matching(Some(&change), Some(oll::status), None)
            else {
                continue;
            };
            let Ok(doc_mpath) = pub_graph.literal_from_triple_matching(
                Some(&change),
                Some(oll::documentMaterializedPath),
                None,
            ) else {
                continue;
            };
            let status = Status::from_string(&found_status)?;
            let document_change_hash = md5::compute(
                pub_version_hash.clone() + &doc_mpath.clone() + &status.to_int().to_string(),
            );
            changed_library_document_bulk.push(ChangedLibraryDocument::new(
                document_change_hash,
                library_mpath.clone(),
            ));
        }
    }
    library::TxManager::insert_bulk(tx, library_bulk).await?;
    library_change::TxManager::insert_bulk(tx, library_changes_bulk).await?;
    changed_library_document::TxManager::insert_bulk(tx, changed_library_document_bulk).await?;
    Ok(())
}

/// Insert shared publication versions for the publication
/// Support for lightweight publications.
/// Populate the many-to-many mapping between change objects and publications
async fn insert_shared_publication_versions_for_publication(
    tx: &mut DatabaseTransaction,
    publication: &Publication,
) -> anyhow::Result<()> {
    let mut publication_has_publication_versions_bulk: Vec<PublicationHasPublicationVersions> =
        vec![];
    let mut publication_version_ids =
        publication_version::TxManager::find_all_recursive_for_publication(
            tx,
            publication.id.clone(),
        )
        .await?;
    if let (Some(last_valid_pub_id), Some(_)) = (
        publication.last_valid_publication_id.as_ref(),
        publication.last_valid_version.as_ref(),
    ) {
        let publication_version_ids_last_valid =
            publication_version::TxManager::find_all_recursive_for_publication(
                tx,
                last_valid_pub_id.clone(),
            )
            .await?;
        publication_version_ids.extend(publication_version_ids_last_valid);
    }
    publication_has_publication_versions_bulk.extend(publication_version_ids.iter().map(|pv| {
        PublicationHasPublicationVersions {
            publication_id: publication.id.clone(),
            publication_version_id: pv.id.clone(),
        }
    }));
    publication_has_publication_versions::TxManager::insert_bulk(
        tx,
        publication_has_publication_versions_bulk,
    )
    .await?;

    Ok(())
}

/// Get the last valid publication name and codified date from the graph
fn referenced_publication_information(pub_graph: &StelaeGraph) -> (Option<String>, Option<String>) {
    let last_valid_pub = pub_graph
        .literal_from_triple_matching(None, Some(oll::lastValidPublication), None)
        .ok()
        .and_then(|pub_name: String| pub_name.strip_prefix("Publication ").map(ToOwned::to_owned));
    let last_valid_version = pub_graph
        .literal_from_triple_matching(None, Some(oll::lastValidCodifiedDate), None)
        .ok();
    (last_valid_pub, last_valid_version)
}

/// Revoke publications that have the same date as the current publication
///
/// # Errors
/// Errors if db operations fail
async fn revoke_same_date_publications(
    tx: &mut DatabaseTransaction,
    publication: Publication,
) -> anyhow::Result<()> {
    let duplicate_publications =
        publication::TxManager::find_all_by_date_and_stele_order_by_name_desc(
            tx,
            publication.date,
            publication.stele,
        )
        .await?;
    if let Some(duplicate_publications_slice) = duplicate_publications.get(1..) {
        for duplicate_pub in duplicate_publications_slice {
            publication::TxManager::update_by_name_and_stele_set_revoked_true(
                tx,
                &duplicate_pub.name,
                &duplicate_pub.stele,
            )
            .await?;
        }
    }
    Ok(())
}

// /// Walk the auth repository and insert commit hashes into the database
// async fn insert_commit_hashes_from_auth_repository(
//     tx: &mut DatabaseTransaction,
//     auth_repo: &Repo,
// ) -> anyhow::Result<()> {
//     Ok(())
// }
