//! Module for inserting changes into the database
#![allow(clippy::shadow_reuse)]
use crate::db::models::changed_library_document::ChangedLibraryDocument;
use crate::db::models::document_change::DocumentChange;
use crate::db::models::library::Library;
use crate::db::models::library_change::LibraryChange;
use crate::db::models::publication::Publication;
use crate::db::models::publication_has_publication_versions::PublicationHasPublicationVersions;
use crate::db::statements::inserts::{
    create_document, create_publication, create_publication_version, create_stele, create_version,
    insert_changed_library_document_bulk, insert_document_changes_bulk, insert_library_bulk,
    insert_library_changes_bulk, insert_publication_has_publication_versions_bulk,
    update_publication_by_name_and_stele_set_revoked_true,
};
use crate::db::statements::queries::{
    find_all_publications_by_date_and_stele_order_by_name_desc, find_last_inserted_publication,
    find_last_inserted_publication_version_by_publication_and_stele,
    find_publication_by_name_and_stele, find_publication_versions_for_publication,
};
use crate::history::rdf::graph::StelaeGraph;
use crate::history::rdf::namespaces::{dcterms, oll};
use crate::utils::archive::get_name_parts;
use crate::utils::git::Repo;
use crate::{
    db::{self, DatabaseConnection},
    stelae::archive::Archive,
};
use anyhow::Context;
use sophia::api::ns::rdfs;
use sophia::api::{prelude::*, term::SimpleTerm};
use sophia::xml::parser;
use sqlx::types::chrono::NaiveDate;
use std::path::{Path, PathBuf};

use super::rdf::graph::Bag;

/// Inserts changes from the archive into the database
///
/// # Errors
/// Errors if the changes cannot be inserted into the archive
#[actix_web::main]
pub async fn insert(
    raw_archive_path: &str,
    archive_path: PathBuf,
    stele: Option<String>,
) -> std::io::Result<()> {
    let conn = match db::init::connect(&archive_path).await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!(
                "error: could not connect to database. Confirm that DATABASE_URL env var is set correctly."
            );
            tracing::error!("Error: {:?}", err);
            std::process::exit(1);
        }
    };
    if let Some(stele) = stele {
        insert_changes_single_stele()?;
    } else {
        insert_changes_archive(&conn, raw_archive_path, &archive_path)
            .await
            .unwrap_or_else(|err| {
                tracing::error!("Failed to insert changes into archive");
                tracing::error!("{:?}", err);
            });
    }
    Ok(())
}

fn insert_changes_single_stele() -> std::io::Result<()> {
    todo!()
}

/// Insert changes from the archive into the database
async fn insert_changes_archive(
    conn: &DatabaseConnection,
    raw_archive_path: &str,
    archive_path: &Path,
) -> anyhow::Result<()> {
    let archive = Archive::parse(
        archive_path.to_path_buf(),
        &PathBuf::from(raw_archive_path),
        false,
    )?;

    for (name, mut stele) in archive.stelae {
        if let Some(repositories) = stele.get_repositories()? {
            let Some(rdf_data) = repositories.get_rdf_repository() else {
                continue;
            };
            let rdf_repo_path = archive_path.to_path_buf().join(&rdf_data.name);
            if !rdf_repo_path.exists() {
                anyhow::bail!(
                    "RDF repository should exist on disk but not found: {}",
                    rdf_repo_path.display()
                );
            }
            let (rdf_org, rdf_name) = get_name_parts(&rdf_data.name)?;
            let rdf_repo = Repo::new(archive_path, &rdf_org, &rdf_name)?;
            insert_changes_from_rdf_repository(conn, rdf_repo, &name).await?;
        }
    }
    Ok(())
}

/// Insert changes from the RDF repository into the database
async fn insert_changes_from_rdf_repository(
    conn: &DatabaseConnection,
    rdf_repo: Repo,
    stele_id: &str,
) -> anyhow::Result<()> {
    tracing::info!("Inserting changes from RDF repository: {}", stele_id);
    tracing::info!("RDF repository path: {}", rdf_repo.path.display());
    let tx = conn.pool.begin().await?;
    match load_delta_for_stele(conn, &rdf_repo, stele_id).await {
        Ok(_) => {
            tx.commit().await?;
            Ok(())
        }
        Err(err) => {
            tx.rollback().await?;
            Err(err)
        }
    }
}

/// Load deltas from the publications
async fn load_delta_for_stele(
    conn: &DatabaseConnection,
    rdf_repo: &Repo,
    stele: &str,
) -> anyhow::Result<()> {
    create_stele(conn, stele).await?;
    match find_last_inserted_publication(conn, stele).await? {
        Some(publication) => {
            tracing::info!("Inserting changes from last inserted publication");
            load_delta_from_publications(conn, rdf_repo, stele, Some(publication)).await?;
        }
        None => {
            tracing::info!("Inserting changes from beginning for stele: {}", stele);
            load_delta_from_publications(conn, rdf_repo, stele, None).await?;
        }
    }
    Ok(())
}

/// Iterate and load delta from all publications in the `_publication` directory
///
/// # Errors
/// Errors if the delta cannot be loaded from the publications
async fn load_delta_from_publications(
    conn: &DatabaseConnection,
    rdf_repo: &Repo,
    stele: &str,
    last_inserted_publication: Option<Publication>,
) -> anyhow::Result<()> {
    let head_commit = rdf_repo.repo.head()?.peel_to_commit()?;
    let tree = head_commit.tree()?;
    let publications_dir_entry = tree.get_path(&PathBuf::from("_publication"))?;
    let publications_subtree = rdf_repo.repo.find_tree(publications_dir_entry.id())?;
    let mut last_inserted_date: Option<NaiveDate> = None;
    for publication_entry in publications_subtree.iter() {
        let mut pub_graph = StelaeGraph::new();
        let object = publication_entry.to_object(&rdf_repo.repo)?;
        let publication_tree = object
            .as_tree()
            .context("Expected a tree but got something else")?;
        let index_rdf = publication_tree.get_path(&PathBuf::from("index.rdf"))?;
        let blob = rdf_repo.repo.find_blob(index_rdf.id())?;
        let data = blob.content();
        let reader = std::io::BufReader::new(data);
        parser::parse_bufread(reader).add_to_graph(&mut pub_graph.g)?;
        let pub_label = pub_graph.literal_from_triple_matching(None, Some(rdfs::label), None)?;
        let pub_name = pub_label
            .strip_prefix("Publication ")
            .context("Could not strip prefix")?
            .to_string();
        let pub_date =
            pub_graph.literal_from_triple_matching(None, Some(dcterms::available), None)?;
        let pub_date = NaiveDate::parse_from_str(pub_date.as_str(), "%Y-%m-%d")?;
        if let Some(last_inserted_pub) = &last_inserted_publication {
            let last_inserted_pub_date =
                NaiveDate::parse_from_str(&last_inserted_pub.date, "%Y-%m-%d")?;
            // continue from last inserted publication, since that publication can contain
            // new changes (versions) that are not in db
            if pub_date < last_inserted_pub_date {
                // skip past publications since they are already in db
                continue;
            }
            last_inserted_date = find_last_inserted_publication_version_by_publication_and_stele(
                conn, &pub_name, &stele,
            )
            .await?
            .map(|p| {
                NaiveDate::parse_from_str(&p.version, "%Y-%m-%d").context("Could not parse date")
            })
            .and_then(|d| d.ok());
        }
        tracing::info!("[{stele}] | Publication: {pub_name}");
        publication_tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
            let path_name = entry.name().unwrap();
            if path_name.contains(".rdf") {
                let current_blob = rdf_repo.repo.find_blob(entry.id()).unwrap();
                let current_content = current_blob.content();
                parser::parse_bufread(std::io::BufReader::new(current_content))
                    .add_to_graph(&mut pub_graph.g)
                    .unwrap();
            }
            git2::TreeWalkResult::Ok
        })?;

        let (last_valid_pub_name, last_valid_codified_date) =
            referenced_publication_information(&pub_graph);
        create_publication(
            conn,
            &pub_name,
            &pub_date,
            stele,
            last_valid_pub_name,
            last_valid_codified_date,
        )
        .await?;
        let publication = find_publication_by_name_and_stele(conn, &pub_name, stele).await?;
        load_delta_for_publication(conn, publication, &pub_graph, last_inserted_date).await?;
    }
    Ok(())
}

async fn load_delta_for_publication(
    conn: &DatabaseConnection,
    publication: Publication,
    pub_graph: &StelaeGraph,
    last_inserted_date: Option<NaiveDate>,
) -> anyhow::Result<()> {
    let pub_document_versions = get_document_publication_versions(&pub_graph);
    let pub_collection_versions = get_collection_publication_versions(&pub_graph);

    insert_document_changes(
        conn,
        &last_inserted_date,
        pub_document_versions,
        pub_graph,
        &publication,
    )
    .await?;

    insert_library_changes(
        conn,
        &last_inserted_date,
        pub_collection_versions,
        pub_graph,
        &publication,
    )
    .await?;
    insert_shared_publication_versions_for_publication(conn, &publication).await?;

    revoke_same_date_publications(conn, publication).await?;
    Ok(())
}

async fn insert_document_changes(
    conn: &DatabaseConnection,
    last_inserted_date: &Option<NaiveDate>,
    pub_document_versions: Vec<&SimpleTerm<'_>>,
    pub_graph: &StelaeGraph,
    publication: &Publication,
) -> anyhow::Result<()> {
    let mut document_changes_bulk: Vec<DocumentChange> = vec![];
    for version in pub_document_versions {
        let codified_date =
            pub_graph.literal_from_triple_matching(Some(version), Some(oll::codifiedDate), None)?;
        if let Some(last_inserted_date) = last_inserted_date {
            let codified_date = NaiveDate::parse_from_str(codified_date.as_str(), "%Y-%m-%d")?;
            if &codified_date <= last_inserted_date {
                // Date already inserted
                continue;
            }
        }
        create_version(conn, &codified_date).await?;
        create_publication_version(conn, &publication.name, &codified_date, &publication.stele)
            .await?;
        let doc_id =
            pub_graph.literal_from_triple_matching(Some(version), Some(oll::docId), None)?;
        create_document(conn, &doc_id).await?;

        let changes_uri =
            pub_graph.iri_from_triple_matching(Some(version), Some(oll::hasChanges), None)?;
        let changes = Bag::new(&pub_graph, changes_uri);
        for change in changes.items()? {
            let doc_mpath = pub_graph.literal_from_triple_matching(
                Some(&change),
                Some(oll::documentMaterializedPath),
                None,
            )?;
            let url =
                pub_graph.literal_from_triple_matching(Some(&change), Some(oll::url), None)?;
            let reason = pub_graph
                .literal_from_triple_matching(Some(&change), Some(oll::reason), None)
                .ok();
            let statuses = pub_graph.all_literals_from_triple_matching(
                Some(&change),
                Some(oll::status),
                None,
            )?;
            for status in statuses {
                document_changes_bulk.push(DocumentChange {
                    doc_mpath: doc_mpath.to_string(),
                    status: status.to_string(),
                    url: url.to_string(),
                    change_reason: reason.clone(),
                    publication: publication.name.clone(),
                    version: codified_date.to_string(),
                    stele: publication.stele.clone(),
                    doc_id: doc_id.to_string(),
                });
            }
        }
    }
    insert_document_changes_bulk(conn, &document_changes_bulk).await?;
    Ok(())
}

async fn insert_library_changes(
    conn: &DatabaseConnection,
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
        if let Some(last_inserted_date) = last_inserted_date {
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
        let status =
            pub_graph.literal_from_triple_matching(Some(version), Some(oll::status), None)?;
        library_bulk.push(Library {
            mpath: library_mpath.to_string(),
        });
        library_changes_bulk.push(LibraryChange {
            library_mpath: library_mpath.to_string(),
            publication: publication.name.to_string(),
            version: codified_date.to_string(),
            stele: publication.stele.to_string(),
            status: status.to_string(),
            url: url.to_string(),
        });
        let changes_uri =
            pub_graph.iri_from_triple_matching(Some(version), Some(oll::hasChanges), None)?;
        let changes = Bag::new(&pub_graph, changes_uri);
        for change in changes.items()? {
            let Ok(status) =
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
            changed_library_document_bulk.push(ChangedLibraryDocument {
                publication: publication.name.to_string(),
                version: codified_date.to_string(),
                stele: publication.stele.to_string(),
                doc_mpath: doc_mpath.to_string(),
                status: status.to_string(),
                library_mpath: library_mpath.to_string(),
                url: url.to_string(),
            });
        }
    }
    insert_library_bulk(conn, &library_bulk).await?;
    insert_library_changes_bulk(conn, &library_changes_bulk).await?;
    insert_changed_library_document_bulk(conn, &changed_library_document_bulk).await?;
    Ok(())
}

async fn insert_shared_publication_versions_for_publication(
    conn: &DatabaseConnection,
    publication: &Publication,
) -> anyhow::Result<()> {
    let mut publication_has_publication_versions_bulk: Vec<PublicationHasPublicationVersions> =
        vec![];
    let mut publication_versions = find_publication_versions_for_publication(
        conn,
        publication.name.to_string(),
        publication.stele.to_string(),
    )
    .await?;
    if let (Some(last_valid_pub_name), Some(_)) = (
        &publication.last_valid_publication_name,
        &publication.last_valid_version,
    ) {
        let publication_versions_last_valid = find_publication_versions_for_publication(
            conn,
            last_valid_pub_name.to_string(),
            publication.stele.to_string(),
        )
        .await?;
        publication_versions.extend(publication_versions_last_valid);
    }
    publication_has_publication_versions_bulk.extend(publication_versions.iter().map(|pv| {
        PublicationHasPublicationVersions {
            publication: publication.name.to_string(),
            referenced_publication: pv.publication.to_string(),
            referenced_version: pv.version.to_string(),
            stele: publication.stele.to_string(),
        }
    }));
    insert_publication_has_publication_versions_bulk(
        conn,
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
        .and_then(|p: String| {
            p.strip_prefix("Publication ")
                .and_then(|s| Some(s.to_string()))
        });
    let last_valid_version = pub_graph
        .literal_from_triple_matching(None, Some(oll::lastValidCodifiedDate), None)
        .ok();
    return (last_valid_pub, last_valid_version);
}

async fn revoke_same_date_publications(
    conn: &DatabaseConnection,
    publication: Publication,
) -> anyhow::Result<()> {
    let publications = find_all_publications_by_date_and_stele_order_by_name_desc(
        conn,
        publication.date.to_string(),
        publication.stele.to_string(),
    )
    .await?;
    for publication in &publications[1..] {
        update_publication_by_name_and_stele_set_revoked_true(
            conn,
            &publication.name,
            &publication.stele,
        )
        .await?;
    }
    Ok(())
}

/// Get the document publication version IRIs from the graph
fn get_document_publication_versions(graph: &StelaeGraph) -> Vec<&SimpleTerm> {
    let triples = graph.g.triples_matching(Any, Any, [oll::DocumentVersion]);
    triples
        .filter_map(|t| {
            let t = t.ok()?;
            let subject = t.s();
            Some(subject)
        })
        .collect()
}

/// Get the collection publication version IRIs from the graph
fn get_collection_publication_versions(graph: &StelaeGraph) -> Vec<&SimpleTerm> {
    let triples = graph.g.triples_matching(Any, Any, [oll::CollectionVersion]);
    triples
        .filter_map(|t| {
            let t = t.ok()?;
            let subject = t.s();
            Some(subject)
        })
        .collect()
}
