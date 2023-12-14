use anyhow::Result;
use git2::{Commit, Error, Oid};
use std::ops::Deref;
use std::path::{Path, PathBuf};
pub use stelae::stelae::types::repositories::{Custom, Repositories, Repository};

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
    Other(String),
}

/// Information about a data repository.
///
/// This struct is used to initialize a data repository in the test suite.
pub struct TestDataRepositoryContext<'repo> {
    /// The name of the data repository.
    pub name: &'repo str,
    /// The paths of the data repository.
    pub paths: Vec<&'repo str>,
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

impl<'repo> TestDataRepositoryContext<'repo> {
    pub fn new(
        name: &'repo str,
        paths: Vec<&'repo str>,
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
        let is_fallback = if let None = is_fallback {
            false
        } else {
            is_fallback.unwrap()
        };
        Ok(Self {
            name,
            paths,
            kind,
            serve_prefix,
            route_glob_patterns,
            is_fallback,
        })
    }
}

pub fn get_default_static_filename(file_extension: &str) -> &str {
    match file_extension {
        "html" => "index.html",
        "rdf" => "index.rdf",
        "xml" => "index.xml",
        "pdf" => "example.pdf",
        "json" => "example.json",
        "js" => "example.js",
        _ => "index.html",
    }
}

pub fn copy_file(from: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(&to.parent().unwrap()).unwrap();
    std::fs::copy(from, to).unwrap();
    Ok(())
}

pub fn get_basic_test_data_repositories() -> Result<Vec<TestDataRepositoryContext<'static>>> {
    Ok(vec![
        TestDataRepositoryContext::new(
            "law-html",
            vec![
                "./index.html",
                "./a/index.html",
                "./a/b/index.html",
                "./a/d/index.html",
                "./a/b/c.html",
                "./a/b/c/index.html",
            ],
            TestDataRepositoryType::Html,
            None,
            Some(vec![".*"]),
            None,
        )?,
        TestDataRepositoryContext::new(
            "law-rdf",
            vec![
                "./index.rdf",
                "./a/index.rdf",
                "./a/b/index.rdf",
                "./a/d/index.rdf",
                "./a/b/c.rdf",
                "./a/b/c/index.rdf",
            ],
            TestDataRepositoryType::Rdf,
            Some("_rdf"),
            None,
            None,
        )?,
        TestDataRepositoryContext::new(
            "law-xml",
            vec![
                "./index.xml",
                "./a/index.xml",
                "./a/b/index.xml",
                "./a/b/c.xml",
                "./a/b/c/index.xml",
                "./a/d/index.xml",
            ],
            TestDataRepositoryType::Xml,
            Some("_xml"),
            None,
            None,
        )?,
        TestDataRepositoryContext::new(
            "law-xml-codified",
            vec![
                "./index.xml",
                "./e/index.xml",
                "./e/f/index.xml",
                "./e/g/index.xml",
            ],
            TestDataRepositoryType::Xml,
            Some("_xml_codified"),
            None,
            None,
        )?,
        TestDataRepositoryContext::new(
            "law-pdf",
            vec!["./example.pdf", "./a/example.pdf", "./a/b/example.pdf"],
            TestDataRepositoryType::Pdf,
            None,
            Some(vec![".*\\.pdf"]),
            None,
        )?,
        TestDataRepositoryContext::new(
            "law-other",
            vec![
                "./index.html",
                "./example.json",
                "./a/index.html",
                "./a/b/index.html",
                "./a/b/c.html",
                "./a/d/index.html",
                "./_prefix/index.html",
                "./_prefix/a/index.html",
                "./a/_doc/e/index.html",
                "./a/e/_doc/f/index.html",
            ],
            TestDataRepositoryType::Other("example.json".to_string()),
            None,
            Some(vec![".*_doc/.*", "_prefix/.*"]),
            Some(true),
        )?,
    ])
}

impl From<&TestDataRepositoryContext<'_>> for Repository {
    fn from(context: &TestDataRepositoryContext) -> Self {
        let mut custom = Custom::default();
        custom.repository_type = Some(match context.kind {
            TestDataRepositoryType::Html => "html".to_string(),
            TestDataRepositoryType::Rdf => "rdf".to_string(),
            TestDataRepositoryType::Xml => "xml".to_string(),
            TestDataRepositoryType::Pdf => "pdf".to_string(),
            TestDataRepositoryType::Other(_) => "other".to_string(),
        });
        custom.serve = "latest".to_string();
        custom.scope = context.serve_prefix.map(|s| s.to_string());
        custom.routes = context
            .route_glob_patterns
            .as_ref()
            .map(|r| r.iter().map(|s| s.to_string()).collect());
        custom.is_fallback = Some(context.is_fallback);
        Self {
            name: context.name.to_string(),
            custom,
        }
    }
}

pub struct GitRepository {
    pub repo: git2::Repository,
    pub path: PathBuf,
}

impl GitRepository {
    pub fn init(path: &Path) -> Result<Self> {
        let repo = git2::Repository::init(path)?;
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

impl Into<git2::Repository> for GitRepository {
    fn into(self) -> git2::Repository {
        self.repo
    }
}

impl Deref for GitRepository {
    type Target = git2::Repository;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}
