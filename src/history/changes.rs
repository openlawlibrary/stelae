//! Module for inserting changes into the database
#![allow(clippy::shadow_reuse)]
use crate::db::models::document_change::DocumentChange;
use crate::db::models::publication::Publication;
use crate::db::statements::queries::{find_last_inserted_publication, find_publication_by_name_and_date_and_stele_id, find_publication_version_by_publication_id_and_version, find_stele_by_name};
use crate::db::statements::inserts::{create_document, create_publication, create_publication_version, create_stele, create_version};
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
    Ok(())
}

/// Iterate and load delta from all publications in the `_publication` directory
///
/// # Errors
/// Errors if the delta cannot be loaded from the publications
async fn load_delta_from_publications_from_beginning(
    conn: &DatabaseConnection,
    rdf_repo: &Repo,
    stele_id: i32,
) -> anyhow::Result<()> {
    let head_commit = rdf_repo.repo.head()?.peel_to_commit()?;
    let tree = head_commit.tree()?;
    let publications_dir_entry = tree.get_path(&PathBuf::from("_publication"))?;
    let publications_subtree = rdf_repo.repo.find_tree(publications_dir_entry.id())?;
    for publication_entry in publications_subtree.iter() {
        let name = publication_entry.name().unwrap();
        dbg!(&name);
        let mut pub_graph = FastGraph::new();
        let object = publication_entry.to_object(&rdf_repo.repo)?;
        let Some(publication_tree) = object.as_tree() else {
            anyhow::bail!("Expected a tree but got something else");
        };
        let index_rdf = publication_tree.get_path(&PathBuf::from("index.rdf"))?;
        let blob = rdf_repo.repo.find_blob(index_rdf.id())?;
        let data = blob.content();
        let reader = std::io::BufReader::new(data);
        parser::parse_bufread(reader).add_to_graph(&mut pub_graph)?;
        let Some(pub_label_obj) = pub_graph.triples_matching(Any, [rdfs::label], Any).next() else {
            anyhow::bail!("Could not find pub_label in a publication");
        };
        let pub_label = {
            let SimpleTerm::LiteralLanguage(pub_label, _) = pub_label_obj?.o() else 
            {
                anyhow::bail!("Found pub_label in a publication, but it was not a literal");
            };
            pub_label.to_string()
        };
        tracing::info!("Publication: {pub_label}");
        let pub_name = pub_label.strip_prefix("Publication ").context("Could not strip prefix")?.to_string();
        let Some(pub_date_obj) = pub_graph.triples_matching(Any, [dcterms::available], Any).next() else {
            anyhow::bail!("Could not find pub_date in a publication");
        };
        let pub_date = {
            let SimpleTerm::LiteralDatatype(pub_date, _) = pub_date_obj?.o() else 
            {
                anyhow::bail!("Expected pub_date as a literal");
            };
            pub_date.to_string()
        };
        publication_tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
            let path_name = entry.name().unwrap();
            if path_name.contains(".rdf") {
                let current_blob = rdf_repo.repo.find_blob(entry.id()).unwrap();
                let current_content = current_blob.content();
                parser::parse_bufread(std::io::BufReader::new(current_content))
                    .add_to_graph(&mut pub_graph)
                    .unwrap();
            }
            git2::TreeWalkResult::Ok
        })?;
        
        let pub_document_versions = get_document_publication_versions(&pub_graph);
        let pub_collection_versions = get_collection_publication_versions(&pub_graph);
        load_delta_for_publication(conn, pub_document_versions, pub_collection_versions, pub_name, pub_date, &pub_graph, stele_id, None).await?;
    }
    Ok(())
}

async fn load_delta_from_publications_from_last_inserted_publication() -> anyhow::Result<()> {
    todo!()
}

/// 
async fn load_delta_for_publication(
    conn: &DatabaseConnection,
    pub_document_versions: Vec<&SimpleTerm<'_>>,
    pub_collection_versions: Vec<&SimpleTerm<'_>>,
    pub_name: String,
    pub_date: String,
    pub_graph: &FastGraph,
    stele_id: i32,
    last_inserted_date: Option<String>,
) -> anyhow::Result<()> {
    let pub_date = NaiveDate::parse_from_str(pub_date.as_str(), "%Y-%m-%d")?;
    create_publication(conn, &pub_name, &pub_date, stele_id).await?;
    let publication = find_publication_by_name_and_date_and_stele_id(conn, &pub_name, &pub_date, stele_id).await?.unwrap();

    insert_document_changes(conn, &last_inserted_date, pub_document_versions, pub_graph, &publication).await?;

    insert_library_changes(conn, &last_inserted_date, pub_collection_versions, pub_graph, &publication).await?;

    revoke_same_date_publications(conn, publication,  stele_id).await?;
    Ok(())
}

async fn insert_document_changes(conn: &DatabaseConnection,  last_inserted_date: &Option<String>, pub_document_versions: Vec<&SimpleTerm<'_>>, pub_graph: &FastGraph, publication: &Publication) -> anyhow::Result<()> {
    for version in pub_document_versions {
        let Some(codified_date_obj) = pub_graph.triples_matching([version], [oll::codifiedDate], Any).next() else {
                anyhow::bail!("Could not find codifiedDate for a document version");
        };
        let codified_date = {
            let SimpleTerm::LiteralDatatype(codified_date, _) = codified_date_obj?.o() else {
                anyhow::bail!("Expected codifiedDate as a literal");
            };
            codified_date.to_string()
        };
        if let Some(last_inserted_date) = last_inserted_date {
            let codified_date = NaiveDate::parse_from_str(codified_date.as_str(), "%Y-%m-%d")?;
            let last_inserted_date = NaiveDate::parse_from_str(last_inserted_date.as_str(), "%Y-%m-%d")?;
            if codified_date <= last_inserted_date {
                // Date already inserted
                continue;
            }
        }
        create_version(conn, &codified_date).await?;
        create_publication_version(conn, publication.id, &codified_date).await?;
        let publication_version = find_publication_version_by_publication_id_and_version(conn, publication.id, &codified_date).await?.context("Could not find publication version")?;
        }
        Ok(())
}

async fn insert_library_changes(conn: &DatabaseConnection, last_inserted_date: &Option<String>, pub_collection_versions: Vec<&SimpleTerm<'_>>, pub_graph: &FastGraph, publication: &Publication) -> anyhow::Result<()> {
    todo!()
}

async fn revoke_same_date_publications(conn: &DatabaseConnection, publication: Publication, stele_id: i32) -> anyhow::Result<()> {
    todo!()
}

/// Get the document publication version IRIs from the graph
fn get_document_publication_versions(graph: &FastGraph) -> Vec<&SimpleTerm> {
    let triples = graph.triples_matching(Any, Any, [oll::DocumentVersion]);
    triples
        .filter_map(|t| {
            let t = t.ok()?;
            let subject = t.s();
            Some(subject)
        })
        .collect()
}

/// Get the collection publication version IRIs from the graph
fn get_collection_publication_versions(graph: &FastGraph) -> Vec<&SimpleTerm> {
    let triples = graph.triples_matching(Any, Any, [oll::CollectionVersion]);
    triples
        .filter_map(|t| {
            let t = t.ok()?;
            let subject = t.s();
            Some(subject)
        })
        .collect()
}