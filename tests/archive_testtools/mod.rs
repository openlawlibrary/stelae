use anyhow::Result;
use git2::{Commit, Error, Oid, Repository};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::{default, fmt};
use stelae::utils::paths::fix_unc_path;

pub enum Jurisdiction {
    Single,
    Multi,
}

pub enum ArchiveType {
    Basic(Jurisdiction),
    Multihost(Jurisdiction),
}

type Name = String;

pub enum RepositoryType {
    Data(DataRepositoryType),
    Auth(Name),
}

pub enum DataRepositoryType {
    Html(Name),
    Rdf(Name),
    Xml(Name),
    Pdf(Name),
    Other(Name),
}

impl DataRepositoryType {}

impl fmt::Display for DataRepositoryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DataRepositoryType::Html(name)
                | DataRepositoryType::Rdf(name)
                | DataRepositoryType::Xml(name)
                | DataRepositoryType::Pdf(name)
                | DataRepositoryType::Other(name) => name,
            }
        )
    }
}

pub struct GitRepository {
    pub repo: Repository,
    pub kind: DataRepositoryType,
    pub path: PathBuf,
}

impl GitRepository {
    pub fn init(path: &Path) -> Result<Self> {
        let repo = Repository::init(path)?;
        Ok(Self {
            repo,
            kind: DataRepositoryType::Html("html".into()),
            path: path.to_path_buf(),
        })
    }

    pub fn commit(&self, path_str: Option<&str>, commit_msg: &str) -> Result<Oid, Error> {
        let mut index = self.repo.index().unwrap();
        if let Some(path_str) = path_str {
            index.add_path(&PathBuf::from(path_str)).unwrap();
        } else {
            index
                .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
                .unwrap();
        }
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = self.repo.find_tree(tree_id).unwrap();
        let sig = self.repo.signature().unwrap();

        let binding = self
            .repo
            .head()
            .ok()
            .and_then(|head| head.target())
            .and_then(|target_id| self.repo.find_commit(target_id).ok())
            .map(|parent_commit| vec![parent_commit])
            .unwrap_or_default();
        let parent_commits: Vec<&Commit> = binding.iter().collect();

        dbg!(&parent_commits);
        self.repo
            .commit(Some("HEAD"), &sig, &sig, commit_msg, &tree, &parent_commits)
    }

    pub fn add_file(&self, path: &Path, file_name: &str, content: &str) -> Result<()> {
        std::fs::create_dir_all(&path)?;
        let path = path.join(file_name);
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Into<Repository> for GitRepository {
    fn into(self) -> Repository {
        self.repo
    }
}

impl Deref for GitRepository {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}
