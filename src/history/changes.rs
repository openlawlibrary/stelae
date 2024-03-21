//! Module for inserting changes into the database
#![allow(clippy::shadow_reuse)]
use crate::db::models::publication::Publication;
use crate::db::statements::queries::{find_last_inserted_publication, find_publication_by_name_and_date_and_stele_id, find_stele_by_name};
use crate::db::statements::inserts::{create_document, create_publication, create_stele};
use crate::utils::archive::get_name_parts;
use crate::utils::git::Repo;
use crate::{
    db::{self, DatabaseConnection},
    stelae::archive::Archive,
};
use anyhow::Context;
use sophia::api::{prelude::*, term::SimpleTerm, MownStr};
use sophia::xml::parser;
use sophia::{api::ns::rdfs, inmem::graph::FastGraph};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use sqlx::types::chrono::NaiveDate;
use walkdir::WalkDir;
use crate::history::rdf::namespaces::{oll, dcterms};

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
    let run_documents = false;
    if run_documents {
        let mut graph = FastGraph::new();
        let head_commit = rdf_repo.repo.head()?.peel_to_commit()?;
        let tree = head_commit.tree()?;
        tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
            let path_name = entry.name().unwrap();
            if path_name.contains(".rdf") {
                let blob = rdf_repo.repo.find_blob(entry.id()).unwrap();
                let data = blob.content();
                let reader = std::io::BufReader::new(data);
                parser::parse_bufread(reader)
                    .add_to_graph(&mut graph)
                    .unwrap();
            }
            git2::TreeWalkResult::Ok
        })?;
        // for entry in WalkDir::new(&rdf_repo.path) {
        //     match entry {
        //         Ok(entry) if is_rdf(&entry) => {
        //             tracing::debug!("Parsing file: {:?}", entry.path());
        //             let file = std::fs::File::open(entry.path())?;
        //             let reader = std::io::BufReader::new(file);
        //             parser::parse_bufread(reader).add_to_graph(&mut graph)?;
        //         }
        //         Ok(entry) => {
        //             tracing::debug!("Skipping non-RDF file: {:?}", entry.path());
        //             continue;
        //         }
        //         Err(err) => {
        //             tracing::error!("Error reading file: {:?}", err);
        //         }
        //     }
        // }
        let documents = graph.triples_matching(Any, Any, [oll::DocumentVersion]);
        let mut doc_to_versions: HashMap<String, Vec<String>> = HashMap::new();
        for triple in documents {
            let triple = triple.unwrap();
            let document = triple.s();
            let mut doc_id_triples = graph.triples_matching([document], [oll::docId], Any);
            if let Some(doc_id_triple) = doc_id_triples.next() {
                let object = doc_id_triple.unwrap().o();
                let document_iri = document.iri().unwrap().to_string();
                if let SimpleTerm::LiteralDatatype(doc_id, _) = object {
                    doc_to_versions
                        .entry(doc_id.to_string())
                        .or_insert_with(Vec::new)
                        .push(document_iri);
                }
            }
        }
        for versions in doc_to_versions.values() {
            // Find the version with the maximum docId
            let doc_version = versions
                .iter()
                .max_by_key(|&v| {
                    let mut doc_id_triples = graph.triples_matching([v.as_str()], [oll::docId], Any);
                    doc_id_triples
                        .next()
                        .map_or_else(String::new, |doc_id_triple| {
                            let object = doc_id_triple.unwrap().o();
                            if let SimpleTerm::LiteralDatatype(doc_id, _) = object {
                                doc_id.to_string()
                            } else {
                                String::new()
                            }
                        })
                })
                .unwrap();
            // Get the docId for this version
            // dbg!(&doc_version);
            let doc_version_iri_ref = IriRef::new_unchecked(MownStr::from_str(doc_version.as_str()));
            let mut doc_id_triples =
                graph.triples_matching([SimpleTerm::Iri(doc_version_iri_ref)], [oll::docId], Any);
            if let Some(doc_id_triple) = doc_id_triples.next() {
                let object = doc_id_triple.unwrap().o();
                if let SimpleTerm::LiteralDatatype(doc_id, _) = object {
                    create_document(conn, doc_id).await?;
                }
            }
        }
    }
    let tx = conn.pool.begin().await?;
    match load_delta_from_publications(conn, &rdf_repo, stele_id).await {
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

    let oll_document_version: NsTerm = oll.get("DocumentVersion").unwrap();
    let oll_doc_id = oll.get("docId").unwrap();

/// Load deltas from the publications
async fn load_delta_from_publications(
    conn: &DatabaseConnection,
    rdf_repo: &Repo,
    stele_name: &str,
) -> anyhow::Result<()> {
    create_stele(conn, stele_name).await?;
    let stele = find_stele_by_name(conn, stele_name).await?.unwrap();
    match find_last_inserted_publication(conn, stele.id).await? {
        Some(publication) => {
            tracing::info!("Inserting changes from last inserted publication");
            load_delta_from_publications_from_last_inserted_publication().await?;
        },
        None => {
            tracing::info!(
                "Inserting changes from beginning for stele: {}",
                stele_name
            );
            load_delta_from_publications_from_beginning(conn, rdf_repo, stele.id).await?;
        }
    }
    for versions in doc_to_versions.values() {
        // Find the version with the maximum docId
        let doc_version = versions
            .iter()
            .max_by_key(|&v| {
                let mut doc_id_triples = graph.triples_matching([v.as_str()], [oll_doc_id], Any);
                doc_id_triples
                    .next()
                    .map_or_else(String::new, |doc_id_triple| {
                        let object = doc_id_triple.unwrap().o();
                        if let SimpleTerm::LiteralDatatype(doc_id, _) = object {
                            doc_id.to_string()
                        } else {
                            String::new()
                        }
                    })
            })
            .unwrap();
        // Get the docId for this version
        // dbg!(&doc_version);
        let doc_version_iri_ref = IriRef::new_unchecked(MownStr::from_str(doc_version.as_str()));
        let mut doc_id_triples =
            graph.triples_matching([SimpleTerm::Iri(doc_version_iri_ref)], [oll_doc_id], Any);
        if let Some(doc_id_triple) = doc_id_triples.next() {
            let object = doc_id_triple.unwrap().o();
            if let SimpleTerm::LiteralDatatype(doc_id, _) = object {
                insert_new_document(conn, doc_id).await?;
            }
        }
    }

    load_delta_from_publications(&mut graph, conn, rdf_repo_path.join("_publication"), name)
        .await?;
    Ok(())
}

/// Check if the entry is an RDF file
fn is_rdf(entry: &walkdir::DirEntry) -> bool {
    entry.path().extension() == Some("rdf".as_ref())
}

/// Load deltas from the publications
async fn load_delta_from_publications(
    graph: &mut FastGraph,
    conn: &DatabaseConnection,
    publication_path: PathBuf,
    name: &str,
) -> anyhow::Result<()> {
    insert_new_stele(conn, name).await?;
    let id = find_stele_by_name(conn, name).await?;
    tracing::info!("Inserting changes from publications for stele: {}", name);
    dbg!(&id);
    dbg!(&publication_path);
    Ok(())
}
