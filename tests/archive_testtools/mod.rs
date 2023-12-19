use anyhow::Result;
use git2::{Commit, Error, Oid};
use std::borrow::Cow;
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
    pub paths: Vec<Cow<'static, str>>,
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
        paths: Vec<Cow<'static, str>>,
        kind: TestDataRepositoryType,
        serve_prefix: Option<&'repo str>,
        route_glob_patterns: Option<Vec<&'repo str>>,
        is_fallback: bool,
    ) -> Result<Self> {
        if let None = serve_prefix {
            if let None = route_glob_patterns {
                return Err(anyhow::anyhow!(
                    "A test data repository must have either a serve prefix or route glob patterns."
                ));
            }
        }
        Ok(Self {
            name,
            paths,
            kind,
            serve_prefix,
            route_glob_patterns,
            is_fallback,
        })
    }

    pub fn default_html_paths() -> Vec<Cow<'static, str>> {
        vec![
            "./index.html".into(),
            "./a/index.html".into(),
            "./a/b/index.html".into(),
            "./a/d/index.html".into(),
            "./a/b/c.html".into(),
            "./a/b/c/index.html".into(),
        ]
    }

    pub fn default_rdf_paths() -> Vec<Cow<'static, str>> {
        vec![
            "./index.rdf".into(),
            "./a/index.rdf".into(),
            "./a/b/index.rdf".into(),
            "./a/d/index.rdf".into(),
            "./a/b/c.rdf".into(),
            "./a/b/c/index.rdf".into(),
        ]
    }

    pub fn default_xml_paths() -> Vec<Cow<'static, str>> {
        vec![
            "./index.xml".into(),
            "./a/index.xml".into(),
            "./a/b/index.xml".into(),
            "./a/d/index.xml".into(),
            "./a/b/c.xml".into(),
            "./a/b/c/index.xml".into(),
        ]
    }

    pub fn default_pdf_paths() -> Vec<Cow<'static, str>> {
        vec![
            "./example.pdf".into(),
            "./a/example.pdf".into(),
            "./a/b/example.pdf".into(),
        ]
    }

    pub fn default_json_paths() -> Vec<Cow<'static, str>> {
        vec![
            "./example.json".into(),
            "./a/example.json".into(),
            "./a/b/example.json".into(),
        ]
    }

    pub fn default_other_paths() -> Vec<Cow<'static, str>> {
        vec![
            "./index.html".into(),
            "./example.json".into(),
            "./a/index.html".into(),
            "./a/b/index.html".into(),
            "./a/b/c.html".into(),
            "./a/d/index.html".into(),
            "./_prefix/index.html".into(),
            "./_prefix/a/index.html".into(),
            "./a/_doc/e/index.html".into(),
            "./a/e/_doc/f/index.html".into(),
        ]
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
            TestDataRepositoryContext::default_html_paths(),
            TestDataRepositoryType::Html,
            None,
            Some(vec![".*"]),
            false,
        )?,
        TestDataRepositoryContext::new(
            "law-rdf",
            TestDataRepositoryContext::default_rdf_paths(),
            TestDataRepositoryType::Rdf,
            Some("_rdf"),
            None,
            false,
        )?,
        TestDataRepositoryContext::new(
            "law-xml",
            TestDataRepositoryContext::default_xml_paths(),
            TestDataRepositoryType::Xml,
            Some("_xml"),
            None,
            false,
        )?,
        TestDataRepositoryContext::new(
            "law-xml-codified",
            vec![
                "./index.xml".into(),
                "./e/index.xml".into(),
                "./e/f/index.xml".into(),
                "./e/g/index.xml".into(),
            ],
            TestDataRepositoryType::Xml,
            Some("_xml_codified"),
            None,
            false,
        )?,
        TestDataRepositoryContext::new(
            "law-pdf",
            TestDataRepositoryContext::default_pdf_paths(),
            TestDataRepositoryType::Pdf,
            None,
            Some(vec![".*\\.pdf"]),
            false,
        )?,
        TestDataRepositoryContext::new(
            "law-other",
            TestDataRepositoryContext::default_other_paths(),
            TestDataRepositoryType::Other("example.json".to_string()),
            None,
            Some(vec![".*_doc/.*", "_prefix/.*"]),
            true,
        )?,
    ])
}

pub fn get_dependent_data_repositories_with_scopes(
    scopes: &Vec<Cow<'static, str>>,
) -> Result<Vec<TestDataRepositoryContext<'static>>> {
    let mut result = Vec::new();
    for kind in [
        TestDataRepositoryType::Html,
        TestDataRepositoryType::Rdf,
        TestDataRepositoryType::Xml,
        TestDataRepositoryType::Pdf,
        TestDataRepositoryType::Other("example.json".to_string()),
    ]
    .into_iter()
    {
        let mut paths = Vec::new();
        let name;
        let mut serve_prefix = None;
        let mut route_glob_patterns = None;
        let mut is_fallback = false;
        let mut default_paths;

        match kind {
            TestDataRepositoryType::Html => {
                name = "law-html";
                route_glob_patterns = Some(vec![".*"]);
                default_paths = TestDataRepositoryContext::default_html_paths();

                default_paths.extend(vec![
                    "./does-not-resolve.html".into(),
                    "./a/does-not-resolve.html".into(),
                    "./a/b/does-not-resolve.html".into(),
                ]);
            }
            TestDataRepositoryType::Rdf => {
                name = "law-rdf";
                serve_prefix = Some("_rdf");
                default_paths = TestDataRepositoryContext::default_rdf_paths();
            }
            TestDataRepositoryType::Xml => {
                name = "law-xml";
                serve_prefix = Some("_xml");
                default_paths = TestDataRepositoryContext::default_xml_paths()
            }
            TestDataRepositoryType::Pdf => {
                name = "law-pdf";
                route_glob_patterns = Some(vec![".*\\.pdf"]);
                default_paths = TestDataRepositoryContext::default_pdf_paths();
            }
            TestDataRepositoryType::Other(_) => {
                name = "law-other";
                route_glob_patterns = Some(vec![".*_doc/.*", "_prefix/.*"]);
                is_fallback = true;
                default_paths = TestDataRepositoryContext::default_other_paths();

                default_paths.extend(vec![
                    "./does-not-resolve.json".into(),
                    "./a/does-not-resolve.json".into(),
                    "./a/b/does-not-resolve.json".into(),
                ]);
            }
        }
        for scope in scopes {
            let additional_paths: Vec<String> = default_paths
                .iter()
                .map(|path| format!("{}/{}", scope, path))
                .collect();
            paths.extend(additional_paths.into_iter().map(|path| path.into()));
        }

        result.push(TestDataRepositoryContext::new(
            name,
            paths,
            kind,
            serve_prefix,
            route_glob_patterns,
            is_fallback,
        )?);
    }
    Ok(result)
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

    pub fn open(path: &Path) -> Result<Self> {
        let repo = git2::Repository::open(path)?;
        Ok(Self {
            repo,
            path: path.to_path_buf(),
        })
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
