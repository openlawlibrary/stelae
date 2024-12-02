//! The archive module contains the Archive object for interacting with
//! Stelae Archives, as well as several factory methods.

use crate::stelae::stele;
use crate::stelae::stele::Stele;
use crate::utils::archive::{find_archive_path, get_name_parts};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, create_dir_all, read_to_string, write};
use std::path::{Path, PathBuf};
use toml_edit::ser;

/// The Archive struct is used for interacting with a Stelae Archive.
#[derive(Debug, Clone)]
pub struct Archive {
    /// Path to the Archive
    pub path: PathBuf,
    /// map of auth repo name to Stele object
    pub stelae: HashMap<String, Stele>,
}

impl Archive {
    /// Get an archive's config object.
    /// # Errors
    /// Will error if unable to find or parse config file at `.taf/config.toml`
    pub fn get_config(&self) -> anyhow::Result<Config> {
        let config_path = &self.path.join(PathBuf::from(".taf/config.toml"));
        let config_str = read_to_string(config_path)?;
        let conf: Config = toml::from_str(&config_str)?;
        Ok(conf)
    }

    /// Get the Archive's root Stele.
    /// # Errors
    /// Will raise error if unable to find the current root Stele
    pub fn get_root(&self) -> anyhow::Result<&Stele> {
        let root = self
            .stelae
            .values()
            .find(|stele| stele.is_root())
            .ok_or_else(|| anyhow::anyhow!("No root Stele found in archive"))?;
        Ok(root)
    }

    /// Set the Archive's root Stele.
    /// # Errors
    /// Will raise error if unable to determine the current
    /// root Stele.
    pub fn set_root(&mut self, path: Option<PathBuf>) -> anyhow::Result<()> {
        let root: Stele;
        if let Some(individual_path) = path {
            tracing::info!("Serving individual Stele at path: {:?}", individual_path);
            root = Stele::new(&self.path, None, None, Some(individual_path), true)?;
        } else {
            let conf = self.get_config()?;

            let org = conf.root.org;
            let name = conf.root.name;

            tracing::info!("Serving {}/{} at path: {:?}", &org, &name, self.path);

            root = Stele::new(
                &self.path,
                Some(name),
                Some(org.clone()),
                Some(self.path.clone().join(org)),
                true,
            )?;
        }
        self.stelae.insert(root.get_qualified_name(), root);
        Ok(())
    }

    /// Return sorted vector of all Stelae in the Archive.
    #[must_use]
    pub fn get_stelae(&self) -> Vec<(String, Stele)> {
        let mut stelae = self.stelae.clone();
        let mut stelae_vec: Vec<(String, Stele)> = stelae.drain().collect();
        stelae_vec.sort_by(|first_stele, second_stele| first_stele.0.cmp(&second_stele.0));
        stelae_vec
    }

    /// Parse an Archive.
    /// # Errors
    /// Will raise error if unable to determine the current root stele or if unable to traverse the child steles.
    pub fn parse(
        archive_path: PathBuf,
        actual_path: &Path,
        individual: bool,
    ) -> anyhow::Result<Self> {
        let mut archive = Self {
            path: archive_path,
            stelae: HashMap::new(),
        };

        let path = if individual {
            actual_path.canonicalize().ok()
        } else {
            None
        };
        archive.set_root(path)?;

        let root = archive.get_root()?;
        let mut visited = vec![root.get_qualified_name()];
        archive.traverse_children(&root.clone(), &mut visited)?;
        Ok(archive)
    }

    /// Traverse the child Steles of the current Stele.
    /// # Errors
    /// Will raise error if unable to traverse the child steles.
    /// # Panics
    /// If unable to unwrap the parent directory of the current path.
    pub fn traverse_children(
        &mut self,
        current: &Stele,
        visited: &mut Vec<String>,
    ) -> anyhow::Result<()> {
        if let Some(dependencies) = current.get_dependencies()? {
            for qualified_name in dependencies.sorted_dependencies_names() {
                if visited.contains(&qualified_name) {
                    continue;
                }
                let parent_dir = self.path.clone();
                let (org, name) = get_name_parts(&qualified_name)?;
                if fs::metadata(parent_dir.join(&org).join(&name)).is_err() {
                    // Stele does not exist on the filesystem, continue to traverse other Steles
                    continue;
                }
                let child = Stele::new(
                    &self.path,
                    Some(name),
                    Some(org.clone()),
                    Some(parent_dir.join(org)),
                    false,
                )?;
                self.stelae
                    .entry(format!(
                        "{org}/{name}",
                        org = child.auth_repo.org,
                        name = child.auth_repo.name
                    ))
                    .or_insert_with(|| child.clone());
                visited.push(child.get_qualified_name());
                self.traverse_children(&child, visited)?;
            }
        }
        Ok(())
    }
}

/// Check if the `path` is inside an existing archive
/// # Errors
/// Return an error if the path is inside an existing archive.
fn raise_error_if_in_existing_archive(path: &Path) -> anyhow::Result<bool> {
    let existing_archive_path = find_archive_path(path);
    match existing_archive_path {
        Ok(_) => anyhow::bail!("You cannot create a new archive inside of an existing archive."),
        Err(_) => Ok(false),
    }
}

/// Config object for an Archive
#[derive(Deserialize, Serialize)]
pub struct Config {
    /// The root Stele for this archive
    pub root: stele::Config,
    /// Whether this is a shallow archive (all repos depth=1)
    pub shallow: bool,
    /// Custom HTTP headers used to interact with the Stele
    pub headers: Option<Headers>,
}

/// Optional Header configuration for an Archive
#[derive(Default, Deserialize, Serialize)]
pub struct Headers {
    /// Specify a custom header guard to use when requesting a Stele's current documents.
    pub current_documents_guard: Option<String>,
}

/// Create a new Stelae Archive at path, and return the new archive.
/// # Errors
/// Will error if archive is created inside of an existing archive.
pub fn init(
    path: PathBuf,
    root_name: String,
    root_org: String,
    root_hash: Option<String>,
    shallow: bool,
    headers: Option<Headers>,
) -> anyhow::Result<Box<Archive>> {
    raise_error_if_in_existing_archive(&path)?;
    let stelae_dir = path.join(PathBuf::from("./.taf"));
    create_dir_all(&stelae_dir)?;
    let config_path = stelae_dir.join(PathBuf::from("./config.toml"));
    let conf = Config {
        root: stele::Config {
            name: root_name,
            org: root_org,
            hash: root_hash,
        },
        shallow,
        headers,
    };
    let conf_str = ser::to_string_pretty(&conf)?;
    write(config_path, conf_str)?;
    let archive = Archive {
        path,
        stelae: HashMap::new(),
    };
    Ok(Box::new(archive))
}
