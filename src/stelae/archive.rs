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
    /// Will raise error if unable to determine the current
    /// root Stele.
    pub fn get_root(&mut self) -> anyhow::Result<Stele> {
        let conf = self.get_config()?;
        let root = Stele {
            archive_path: self.path.clone(),
            name: conf.root.name.clone(),
            path: self.path.clone().join(conf.root.name),
        };
        self.stelae.insert(root.clone().name, root.clone()).or(None);
        Ok(root)
    }

    /// Parse an Archive.
    /// # Errors
    /// Will raise error if unable to determine the current root stele or if unable to traverse the child steles.
    pub fn parse_archive(path: PathBuf) -> anyhow::Result<Self> {
        let mut archive = Self {
            path: path.clone(),
            stelae: HashMap::new(),
        };
        let root = archive.get_root()?;
        archive.traverse_children(&root, path)?;
        Ok(archive)
    }

    /// Traverse the child Steles of the current Stele.
    /// # Errors
    /// Will raise error if unable to traverse the child steles.
    pub fn traverse_children(
        &mut self,
        current_stele: &Stele,
        current_path: PathBuf,
    ) -> anyhow::Result<()> {
        if let Some(dependencies) = current_stele.get_dependencies()? {
            for (name, _) in dependencies.dependencies {
                let parent_dir = current_path.parent().unwrap_or(default); //TODO: handle 
                let stele = Stele {
                    archive_path: self.path.clone(),
                    name: name.clone(),
                    path: current_path.join(name),
                };
                self.stelae
                    .insert(stele.clone().name, stele.clone())
                    .or(None);
                self.traverse_children(&stele, stele.path.clone())?;
            }
        }
        Ok(())
    }

    // Determines whether the given path contains a root or a child stele and
    // returns the given stele.
    // pub fn determine_stele(&mut self, path: &Path) -> anyhow::Result<Stele> {
    //     let abs_path = path.canonicalize()?;
    //     let root_stele = self.get_root()?;
    //     let root_stele_path = Path::new(&root_stele.archive_path);
    //     let root_stele_path = root_stele_path.join(&root_stele.name);
    //     let root_stele_path = root_stele_path
    //         .parent()
    //         .expect("Path to current stele must be set");
    //     for working_path in abs_path.ancestors() {
    //         dbg!(working_path);
    //         // if working_path.join(".stelae").exists() {
    //         //     // return Ok(working_path.to_owned());
    //         // }
    //     }
    //     if root_stele_path.starts_with(abs_path) {

    //     }
    //     for working_path in root_stele_path.ancestors() {
    //         dbg!(working_path);
    //         // if working_path.join(".stelae").exists() {
    //         //     // return Ok(working_path.to_owned());
    //         // }
    //     }
    //     Ok(Stele::default())
    // }

    // pub fn get_stele(&self, path: &Path) -> anyhow::Result<Stele> {

    // }
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
) -> anyhow::Result<Box<Archive>> {
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
    let archive = Archive {
        path,
        stelae: HashMap::new(),
    };
    if root_url.is_some() {}
    Ok(Box::new(archive))
}
