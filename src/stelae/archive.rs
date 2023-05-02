//! The archive module contains the Archive object for interacting with
//! Stelae Archives, as well as several factory methods.

use crate::stelae::stele;
use crate::stelae::stele::Stele;
use crate::utils::archive::find_archive_path;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::{Path, PathBuf};

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
    /// Will error if unable to find or parse config file at `.stelae/config.toml`
    pub fn get_config(&self) -> anyhow::Result<Config> {
        let config_path = &self.path.join(PathBuf::from(".stelae/config.toml"));
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
            .find(|s| s.is_root())
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
            root = Stele::new(self.path.clone(), None, None, Some(individual_path), true)?;
        } else {
            tracing::info!("Serving an archive at path: {:?}", self.path);
            let conf = self.get_config()?;

            let org = conf.root.org;
            let name = conf.root.name;

            root = Stele::new(
                self.path.clone(),
                Some(name),
                Some(org.clone()),
                Some(self.path.clone().join(org)),
                true,
            )?;
        }
        self.stelae.insert(root.get_qualified_name(), root);
        Ok(())
    }

    /// Parse an Archive.
    /// # Errors
    /// Will raise error if unable to determine the current root stele or if unable to traverse the child steles.
    pub fn parse(
        archive_path: PathBuf,
        mut actual_path: PathBuf,
        individual: bool,
    ) -> anyhow::Result<Self> {
        let mut archive = Self {
            path: archive_path,
            stelae: HashMap::new(),
        };

        if individual {
            actual_path = actual_path.canonicalize()?;
            archive.set_root(Some(actual_path))?;
        } else {
            archive.set_root(None)?;
        };

        archive.traverse_children(&archive.get_root()?.clone())?;
        Ok(archive)
    }

    /// Traverse the child Steles of the current Stele.
    /// # Errors
    /// Will raise error if unable to traverse the child steles.
    /// # Panics
    /// If unable to unwrap the parent directory of the current path.
    pub fn traverse_children(&mut self, current: &Stele) -> anyhow::Result<()> {
        if let Some(dependencies) = current.get_dependencies()? {
            for (name, _) in dependencies.dependencies {
                dbg!(&name);
                let parent_dir = self.path.clone();
                let name_parts: Vec<&str> = name.split("/").collect();
                let org = name_parts.get(0).unwrap().to_string();
                let name = name_parts.get(1).unwrap().to_string();

                let child = Stele::new(
                    self.path.clone(),
                    Some(name),
                    Some(org.clone()),
                    Some(parent_dir.join(org)),
                    false,
                )?;
                self.stelae
                    .entry(format!("{}/{}", child.org, child.name))
                    .or_insert_with(|| child.clone());
                self.traverse_children(&child)?;
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
    root: stele::Config,
    /// Whether this is a shallow archive (all repos depth=1)
    shallow: bool,
}

/// Create a new Stelae Archive at path, and return the new archive.
/// # Errors
/// Will error if archive is created inside of an existing archive.
pub fn init(
    path: PathBuf,
    root_name: String,
    root_org: String,
    root_hash: Option<String>,
    root_url: Option<String>,
    shallow: bool,
) -> anyhow::Result<Box<Archive>> {
    raise_error_if_in_existing_archive(&path)?;
    let stelae_dir = path.join(PathBuf::from("./.stelae"));
    create_dir_all(&stelae_dir)?;
    let config_path = stelae_dir.join(PathBuf::from("./config.toml"));
    let conf = Config {
        root: stele::Config {
            name: root_name,
            org: root_org,
            hash: root_hash,
        },
        shallow,
    };
    let conf_str = toml::to_string_pretty(&conf)?;
    write(config_path, conf_str)?;
    let archive = Archive {
        path,
        stelae: HashMap::new(),
    };
    if root_url.is_some() {}
    Ok(Box::new(archive))
}
