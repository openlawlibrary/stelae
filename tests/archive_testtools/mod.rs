use anyhow::Result;
use git2::{Commit, Error, Oid, Repository};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::{default, fmt};
use stelae::utils::paths::fix_unc_path;

pub enum ArchiveType {
    Basic(Jurisdiction),
    Multihost(Jurisdiction),
}

pub enum Jurisdiction {
    Single,
    Multi,
}

pub enum TestDataRepositoryType {
    Html,
    Rdf,
    Xml,
    Pdf,
    Other,
}

/// Information about a data repository.
///
/// This is used to initialize a data repository in a test archive.
pub struct TestDataRepositoryContext<'repo> {
    /// The name of the data repository.
    pub name: &'repo str,
    /// The subdirectories of the data repository.
    pub subdirectories: Vec<&'repo str>,
    /// The kind of data repository.
    pub kind: TestDataRepositoryType,
    /// The prefix to use when serving the data repository.
    ///
    /// If `None`, the data repository will be served at the root.
    /// If `Some("prefix")`, the data repository will be served from `/prefix/<data>`.
    pub serve_prefix: Option<&'repo str>,
    /// The route glob patterns to use when serving the data repository.
    pub route_glob_patterns: Option<Vec<&'repo str>>,
    /// Whether the data repository is a fallback.
    pub is_fallback: bool,
}

impl TestDataRepositoryContext<'_> {
    pub fn new<'repo>(
        name: &'repo str,
        subdirectories: Option<Vec<&'repo str>>,
        kind: TestDataRepositoryType,
        serve_prefix: Option<&'repo str>,
        route_glob_patterns: Option<Vec<&'repo str>>,
        is_fallback: Option<bool>,
    ) -> Result<Self> {
        if let None = serve_prefix {
            if let None = route_glob_patterns {
                return Err(anyhow::anyhow!(
                    "A data repository must have either a serve prefix or route glob patterns."
                ));
            }
        }
        let subdirectories = if let None = subdirectories {
            let default_subdirectories = vec!["./", "./a/", "./a/b/", "./a/b/c", "./a/d/"];
            default_subdirectories
        } else {
            subdirectories.unwrap()
        };
        let is_fallback = if let None = is_fallback {
            false
        } else {
            is_fallback.unwrap()
        };
        Ok(Self {
            name,
            subdirectories,
            kind,
            serve_prefix,
            route_glob_patterns,
            is_fallback,
        })
    }
}

pub fn get_basic_test_data_repositories() -> Result<Vec<TestDataRepositoryContext<'static>>> {
    Ok(vec![
        TestDataRepositoryContext::new(
            "law-html",
            None,
            TestDataRepositoryType::Html,
            None,
            Some(vec![".*"]),
            None,
        )?,
        TestDataRepositoryContext::new(
            "law-rdf",
            None,
            TestDataRepositoryType::Rdf,
            Some("_rdf"),
            None,
            None,
        )?,
        TestDataRepositoryContext::new(
            "law-xml",
            None,
            TestDataRepositoryType::Xml,
            Some("_xml"),
            None,
            None,
        )?,
        TestDataRepositoryContext::new(
            "law-xml-codified",
            None,
            TestDataRepositoryType::Xml,
            Some("_xml_codified"),
            None,
            None,
        )?,
        TestDataRepositoryContext::new(
            "law-pdf",
            None,
            TestDataRepositoryType::Pdf,
            None,
            Some(vec![".*\\.pdf"]),
            None,
        )?,
        TestDataRepositoryContext::new(
            "law-other",
            Some(vec![
                "./",
                "./a/",
                "./a/b/",
                "./a/b/c",
                "./a/d/",
                "./_prefix/",
                "./_prefix/a/",
                "./_prefix/a/b/",
                "./a/_doc/e/",
                "./a/e/_doc/f/",
            ]),
            TestDataRepositoryType::Other,
            None,
            Some(vec![".*_doc/.*", "_prefix/.*"]),
            Some(true),
        )?,
    ])
}

pub struct GitRepository {
    pub repo: Repository,
    pub path: PathBuf,
}

impl GitRepository {
    pub fn init(path: &Path) -> Result<Self> {
        let repo = Repository::init(path)?;
        Ok(Self {
            repo,
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
