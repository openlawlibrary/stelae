//! The Stele module contains the Stele object for interacting with
//! Stelae.

use std::path::{Path, PathBuf};

use super::types::repositories::Repository;
use crate::{
    stelae::types::{dependencies::Dependencies, repositories::Repositories},
    utils::git::Repo,
};
use anyhow::Context;
use git2::Repository as GitRepository;
use serde_derive::{Deserialize, Serialize};
use serde_json;

/// Stele
#[derive(Debug, Clone)]
pub struct Stele {
    /// Path to the containing Stelae archive.
    pub archive_path: PathBuf,
    /// Stele's repositories (as specified in repositories.json).
    pub repositories: Option<Repositories>,
    /// Indicates whether or not the Stele is the root Stele.
    /// TODO: this does not seem correct
    pub root: bool,
    /// Stele's authentication repo.
    pub auth_repo: Repo,
}

impl Stele {
    /// Create a new Stele object
    /// # Errors
    /// Will error if unable to find or parse repositories file at `targets/repositories.json`
    /// # Panics
    /// Will panic if unable to determine the current root Stele.
    #[allow(clippy::shadow_reuse)]
    pub fn new(
        archive_path: &Path,
        name: Option<String>,
        org: Option<String>,
        path: Option<PathBuf>,
        root: bool,
    ) -> anyhow::Result<Self> {
        let name = name.unwrap_or_else(|| "law".to_owned());
        let org = if let Some(org) = org {
            org
        } else {
            path.as_ref()
                .context("path is None")?
                .file_name()
                .context("file_name is None")?
                .to_str()
                .context("to_str failed")?
                .to_owned()
        };
        let path = path.unwrap_or_else(|| archive_path.join(&org));
        let mut stele = Self {
            archive_path: archive_path.to_path_buf(),
            repositories: None,
            root,
            auth_repo: Repo {
                archive_path: archive_path.to_string_lossy().to_string(),
                path: path.join(&name),
                org,
                name: name.clone(),
                repo: GitRepository::open(path.join(&name))?,
            },
        };
        stele.get_repositories()?;
        Ok(stele)
    }

    /// Get Stele's dependencies.
    /// # Errors
    /// Will error if unable to parse dependencies file from `targets/dependencies.json`
    pub fn get_dependencies(&self) -> anyhow::Result<Option<Dependencies>> {
        let blob = self
            .auth_repo
            .get_bytes_at_path("HEAD", "targets/dependencies.json");
        if let Ok(dependencies_blob) = blob {
            let dependencies_str = String::from_utf8(dependencies_blob)?;
            let dependencies = serde_json::from_str(&dependencies_str)?;
            return Ok(Some(dependencies));
        }
        Ok(None)
    }
    /// Get Stele's repositories.
    /// # Errors
    /// Will error if unable to find or parse repositories file at `targets/repositories.json`
    pub fn get_repositories(&mut self) -> anyhow::Result<Option<Repositories>> {
        let blob = self
            .auth_repo
            .get_bytes_at_path("HEAD", "targets/repositories.json");
        if let Ok(repositories_blob) = blob {
            let repositories_str = String::from_utf8(repositories_blob)?;
            let repositories: Repositories = serde_json::from_str(&repositories_str)?;
            self.repositories = Some(repositories.clone());
            return Ok(Some(repositories));
        }
        Ok(None)
    }

    /// Get Stele's qualified name.
    #[must_use]
    pub fn get_qualified_name(&self) -> String {
        format!("{}/{}", self.auth_repo.org, self.auth_repo.name)
    }

    /// Get Stele's fallback repo.
    /// A fallback repository is a data repository which contains `is_fallback` = true in its custom field.
    /// # Returns
    /// Returns the first fallback repository found, or None if no fallback repository is found.
    #[must_use]
    pub fn get_fallback_repo(&self) -> Option<Repository> {
        if let Some(repositories) = &self.repositories {
            for repository in repositories.repositories.values() {
                let custom = &repository.custom;
                if let Some(ref is_fallback) = custom.is_fallback {
                    if *is_fallback {
                        return Some(repository.clone());
                    }
                }
            }
        }
        None
    }

    /// See if Stele is a root Stele.
    #[must_use]
    pub const fn is_root(&self) -> bool {
        self.root
    }
}

///Config object for a Stele
#[derive(Deserialize, Serialize)]
pub struct Config {
    /// Name of the authentication repo (e.g. law).
    pub name: String,
    /// Name of the Stele's directory, also known as Stele's organization (e.g. openlawlibrary).
    pub org: String,
    /// The out-of-band authenticated hash of the Stele.
    pub hash: Option<String>,
}
