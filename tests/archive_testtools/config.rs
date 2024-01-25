use anyhow::Result;
use std::path::PathBuf;
pub use stelae::stelae::types::repositories::{Custom, Repositories, Repository};

pub enum ArchiveType {
    Basic(Jurisdiction),
    Multihost,
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
pub struct TestDataRepositoryContext {
    /// The name of the data repository.
    pub name: String,
    /// The paths of the data repository.
    pub paths: Vec<String>,
    /// The kind of data repository.
    pub kind: TestDataRepositoryType,
    /// The prefix to use when serving the data repository.
    ///
    /// If `None`, the data repository will be served at the root.
    /// If `Some("prefix")`, the data repository will be served from `/prefix/<data>`.
    pub serve_prefix: Option<String>,
    /// The route glob patterns to use when serving the data repository.
    pub route_glob_patterns: Option<Vec<String>>,
    /// Whether the data repository is a fallback.
    pub is_fallback: bool,
}

impl TestDataRepositoryContext {
    pub fn new(
        name: String,
        paths: Vec<String>,
        kind: TestDataRepositoryType,
        serve_prefix: Option<String>,
        route_glob_patterns: Option<Vec<String>>,
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

    pub fn default_html_paths() -> Vec<String> {
        let paths = &[
            "./index.html",
            "./a/index.html",
            "./a/b/index.html",
            "./a/d/index.html",
            "./a/b/c.html",
            "./a/b/c/index.html",
        ];
        paths.iter().map(|&x| x.into()).collect()
    }

    pub fn default_rdf_paths() -> Vec<String> {
        let paths = &[
            "./index.rdf",
            "./a/index.rdf",
            "./a/b/index.rdf",
            "./a/d/index.rdf",
            "./a/b/c.rdf",
            "./a/b/c/index.rdf",
        ];
        paths.iter().map(|&x| x.into()).collect()
    }

    pub fn default_xml_paths() -> Vec<String> {
        let paths = &[
            "./index.xml",
            "./a/index.xml",
            "./a/b/index.xml",
            "./a/d/index.xml",
            "./a/b/c.xml",
            "./a/b/c/index.xml",
        ];
        paths.iter().map(|&x| x.into()).collect()
    }

    pub fn default_pdf_paths() -> Vec<String> {
        let paths = &["./example.pdf", "./a/example.pdf", "./a/b/example.pdf"];
        paths.iter().map(|&x| x.into()).collect()
    }

    pub fn default_json_paths() -> Vec<String> {
        let paths = &["./example.json", "./a/example.json", "./a/b/example.json"];
        paths.iter().map(|&x| x.into()).collect()
    }

    pub fn default_other_paths() -> Vec<String> {
        let paths = &[
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
        ];
        paths.iter().map(|&x| x.into()).collect()
    }
}

pub fn get_basic_test_data_repositories() -> Result<Vec<TestDataRepositoryContext>> {
    Ok(vec![
        TestDataRepositoryContext::new(
            "law-html".into(),
            TestDataRepositoryContext::default_html_paths(),
            TestDataRepositoryType::Html,
            None,
            Some(vec![".*".into()]),
            false,
        )?,
        TestDataRepositoryContext::new(
            "law-rdf".into(),
            TestDataRepositoryContext::default_rdf_paths(),
            TestDataRepositoryType::Rdf,
            Some("_rdf".into()),
            None,
            false,
        )?,
        TestDataRepositoryContext::new(
            "law-xml".into(),
            TestDataRepositoryContext::default_xml_paths(),
            TestDataRepositoryType::Xml,
            Some("_xml".into()),
            None,
            false,
        )?,
        TestDataRepositoryContext::new(
            "law-xml-codified".into(),
            vec![
                "./index.xml".into(),
                "./e/index.xml".into(),
                "./e/f/index.xml".into(),
                "./e/g/index.xml".into(),
            ],
            TestDataRepositoryType::Xml,
            Some("_xml_codified".into()),
            None,
            false,
        )?,
        TestDataRepositoryContext::new(
            "law-pdf".into(),
            TestDataRepositoryContext::default_pdf_paths(),
            TestDataRepositoryType::Pdf,
            None,
            Some(vec![".*\\.pdf".into()]),
            false,
        )?,
        TestDataRepositoryContext::new(
            "law-other".into(),
            TestDataRepositoryContext::default_other_paths(),
            TestDataRepositoryType::Other("example.json".to_string()),
            None,
            Some(vec![".*_doc/.*".into(), "_prefix/.*".into()]),
            true,
        )?,
    ])
}

pub fn get_dependent_data_repositories_with_scopes(
    scopes: &Vec<String>,
) -> Result<Vec<TestDataRepositoryContext>> {
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
                name = "law-html".into();
                route_glob_patterns = Some(vec![".*".into()]);
                default_paths = TestDataRepositoryContext::default_html_paths();

                default_paths.extend(vec![
                    "./does-not-resolve.html".into(),
                    "./a/does-not-resolve.html".into(),
                    "./a/b/does-not-resolve.html".into(),
                ]);
            }
            TestDataRepositoryType::Rdf => {
                name = "law-rdf".into();
                serve_prefix = Some("_rdf".into());
                default_paths = TestDataRepositoryContext::default_rdf_paths();
            }
            TestDataRepositoryType::Xml => {
                name = "law-xml".into();
                serve_prefix = Some("_xml".into());
                default_paths = TestDataRepositoryContext::default_xml_paths()
            }
            TestDataRepositoryType::Pdf => {
                name = "law-pdf".into();
                route_glob_patterns = Some(vec![".*\\.pdf".into()]);
                default_paths = TestDataRepositoryContext::default_pdf_paths();
            }
            TestDataRepositoryType::Other(_) => {
                name = "law-other".into();
                route_glob_patterns = Some(vec![".*_doc/.*".into(), "_prefix/.*".into()]);
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
                .map(|path| format!("{scope}/{path}"))
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

impl From<&TestDataRepositoryContext> for Repository {
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
        custom.scope = context.serve_prefix.clone().map(|s| s.to_string());
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
