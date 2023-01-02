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
pub struct Archive<'archive> {
    /// Path to the Archive
    pub path: PathBuf,
    /// map of auth repo name to Stele object
    pub stelae: HashMap<String, Stele<'archive>>,
}

impl Archive<'_> {
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
    /// Will raise error if unable to determine the current
    /// root Stele.
    pub fn get_root(&mut self) -> anyhow::Result<&Stele> {
        let conf = self.get_config()?;
        let root = Stele {
            archive: self,
            name: conf.root.name,
        };
        self.stelae.insert(root.name, root);
        Ok(&root)
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
    root_hash: Option<String>,
    root_url: Option<String>,
    shallow: bool,
) -> anyhow::Result<Box<Archive<'static>>> {
    raise_error_if_in_existing_archive(&path)?;
    let stelae_dir = path.join(PathBuf::from("./.stelae"));
    create_dir_all(&stelae_dir)?;
    let config_path = stelae_dir.join(PathBuf::from("./config.toml"));
    let conf = Config {
        root: stele::Config {
            name: root_name,
            hash: root_hash,
        },
        shallow,
    };
    let conf_str = toml::to_string_pretty(&conf)?;
    write(config_path, conf_str)?;
    let archive = Archive { path, stelae: HashMap::new() };
    if root_url.is_some() {
        
    }
    Ok(Box::new(archive))
}
