pub mod config;
pub mod utils;

use anyhow::Result;
use git2::{Commit, Error, Oid};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use stelae::stelae::archive::{self, Headers};
use stelae::stelae::types::dependencies::{Dependencies, Dependency};
use stelae::stelae::types::repositories::{Repositories, Repository};
use tempfile::TempDir;

use self::config::{
    get_basic_test_data_repositories, get_dependent_data_repositories_with_scopes, ArchiveType,
    Jurisdiction, TestDataRepositoryContext,
};

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

pub struct GitRepository {
    pub repo: git2::Repository,
    pub path: PathBuf,
}

impl GitRepository {
    pub fn init(path: &Path) -> Result<Self> {
        let repo = git2::Repository::init(path)?;
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "name").unwrap();
        config.set_str("user.email", "email").unwrap();
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

pub fn initialize_archive_inner(archive_type: ArchiveType, td: &TempDir) -> Result<()> {
    match archive_type {
        ArchiveType::Basic(Jurisdiction::Single) => initialize_archive_basic(td),
        ArchiveType::Basic(Jurisdiction::Multi) => initialize_archive_multijurisdiction(td),
        ArchiveType::Multihost => initialize_archive_multihost(td),
    }
}

fn initialize_archive_basic(td: &TempDir) -> Result<()> {
    let org_name = "test_org";

    archive::init(
        td.path().to_owned(),
        "law".into(),
        org_name.into(),
        None,
        false,
        None,
    )
    .unwrap();
    initialize_stele(
        td.path().to_path_buf(),
        org_name,
        get_basic_test_data_repositories().unwrap().as_slice(),
        None,
    )
    .unwrap();
    // anyhow::bail!("Something went wrong!");
    Ok(())
}

fn initialize_archive_multijurisdiction(td: &TempDir) -> Result<()> {
    let root_org_name = "root_test_org";

    archive::init(
        td.path().to_owned(),
        "law".into(),
        root_org_name.into(),
        None,
        false,
        None,
    )
    .unwrap();

    initialize_stele(
        td.path().to_path_buf(),
        root_org_name,
        get_basic_test_data_repositories().unwrap().as_slice(),
        None,
    )
    .unwrap();

    let dependent_stele_1_org_name = "dependent_stele_1";
    let dependent_stele_1_scopes: Vec<String> = vec!["sub/scope/1".into(), "sub/scope/2".into()];

    initialize_stele(
        td.path().to_path_buf(),
        dependent_stele_1_org_name,
        get_dependent_data_repositories_with_scopes(&dependent_stele_1_scopes)
            .unwrap()
            .as_slice(),
        Some(&dependent_stele_1_scopes),
    )
    .unwrap();

    let dependent_stele_2_org_name = "dependent_stele_2";
    let dependent_stele_2_scopes: Vec<String> = vec!["sub/scope/3".into(), "sub/scope/4".into()];

    initialize_stele(
        td.path().to_path_buf(),
        dependent_stele_2_org_name,
        get_dependent_data_repositories_with_scopes(&dependent_stele_2_scopes)
            .unwrap()
            .as_slice(),
        Some(&dependent_stele_2_scopes),
    )
    .unwrap();

    add_dependencies(
        td.path(),
        root_org_name,
        vec![dependent_stele_1_org_name, dependent_stele_2_org_name],
    )?;

    // anyhow::bail!("Something went wrong!");
    Ok(())
}

fn initialize_archive_multihost(td: &TempDir) -> Result<()> {
    let root_org_name = "root_stele";

    archive::init(
        td.path().to_owned(),
        "law".into(),
        root_org_name.into(),
        None,
        false,
        Some(Headers {
            current_documents_guard: Some("X-Current-Documents-Guard".into()),
        }),
    )
    .unwrap();

    initialize_stele(
        td.path().to_path_buf(),
        root_org_name,
        get_basic_test_data_repositories().unwrap().as_slice(),
        None,
    )
    .unwrap();

    let stele_1_org_name = "stele_1";

    initialize_stele(
        td.path().to_path_buf(),
        stele_1_org_name,
        get_basic_test_data_repositories().unwrap().as_slice(),
        None,
    )
    .unwrap();

    let stele_2_org_name = "stele_2";

    initialize_stele(
        td.path().to_path_buf(),
        stele_2_org_name,
        get_basic_test_data_repositories().unwrap().as_slice(),
        None,
    )
    .unwrap();

    add_dependencies(
        td.path(),
        root_org_name,
        vec![stele_1_org_name, stele_2_org_name],
    )?;

    Ok(())
}

pub fn initialize_stele(
    path: PathBuf,
    org_name: &str,
    data_repositories: &[TestDataRepositoryContext],
    scopes: Option<&Vec<String>>,
) -> Result<()> {
    let path = path.join(org_name);
    init_data_repositories(&path, data_repositories)?;
    init_auth_repository(&path, org_name, data_repositories, scopes)?;
    Ok(())
}

pub fn init_auth_repository(
    path: &Path,
    org_name: &str,
    data_repositories: &[TestDataRepositoryContext],
    scopes: Option<&Vec<String>>,
) -> Result<GitRepository> {
    let mut path = path.to_path_buf();
    path.push("law");
    std::fs::create_dir_all(&path).unwrap();

    let repo = GitRepository::init(&path).unwrap();

    path.push("targets");

    let repositories: Repositories =
        data_repositories
            .iter()
            .fold(Repositories::default(), |mut repositories, data_repo| {
                let mut repository = Repository::from(data_repo);
                repository.name = format!("{}/{}", org_name, repository.name);
                repositories
                    .repositories
                    .entry(repository.name.clone())
                    .or_insert(repository);
                repositories.scopes =
                    scopes.map(|vec| vec.into_iter().map(|scope| scope.into()).collect());
                repositories
            });
    let content = serde_json::to_string_pretty(&repositories).unwrap();

    repo.add_file(&path, "repositories.json", &content).unwrap();
    repo.commit(Some("targets/repositories.json"), "Add repositories.json")
        .unwrap();
    Ok(repo)
}

pub fn init_data_repositories(
    path: &Path,
    data_repositories: &[TestDataRepositoryContext],
) -> Result<Vec<GitRepository>> {
    let mut data_git_repositories: Vec<GitRepository> = Vec::new();
    for data_repo in data_repositories {
        let mut path = path.to_path_buf();
        path.push(&data_repo.name);
        std::fs::create_dir_all(&path).unwrap();
        let git_repo = GitRepository::init(&path).unwrap();
        init_data_repository(&git_repo, data_repo)?;
        data_git_repositories.push(git_repo);
    }
    Ok(data_git_repositories)
}

fn init_data_repository(
    git_repo: &GitRepository,
    data_repo: &TestDataRepositoryContext,
) -> Result<()> {
    for path in data_repo.paths.iter() {
        add_fixture_file_to_git_repo(git_repo, path)?;
    }
    git_repo.commit(None, "Add initial data").unwrap();
    Ok(())
}

fn add_fixture_file_to_git_repo(git_repo: &GitRepository, path: &str) -> Result<()> {
    let path_buf = PathBuf::from(path);
    let filename = path_buf.file_name().unwrap().to_str().unwrap();
    let static_file_path = get_static_file_path(filename);
    copy_file(&static_file_path, &git_repo.path.join(path)).unwrap();
    Ok(())
}

/// Returns a static file path for the given filename.
/// If the file does not exist, returns the default static file path.
fn get_static_file_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures/static_files");
    let static_file_path = PathBuf::from(&path).join(filename);

    if static_file_path.exists() {
        return static_file_path;
    }

    let ext = Path::new(filename)
        .extension()
        .map_or("html", |ext| ext.to_str().map_or("", |ext_str| ext_str));
    let filename = get_default_static_filename(ext);

    PathBuf::from(path).join(filename)
}

pub fn add_dependencies(
    path: &Path,
    root_org_name: &str,
    dependent_stele_org_names: Vec<&str>,
) -> Result<()> {
    let root_repo = get_repository(path, &format!("{root_org_name}/law"));

    let dependencies = Dependencies {
        dependencies: {
            let mut dependencies = HashMap::new();
            for dependent_stele_org_name in dependent_stele_org_names {
                dependencies.insert(
                    format!(
                        "{dependent_stele_org_name}/law",
                        dependent_stele_org_name = dependent_stele_org_name
                    ),
                    Dependency {
                        out_of_band_authentication: "sha256".into(),
                        branch: "main".into(),
                    },
                );
            }
            dependencies
        }
        .into_iter()
        .collect(),
    };
    let content = serde_json::to_string_pretty(&dependencies).unwrap();

    root_repo
        .add_file(
            &path
                .to_path_buf()
                .join(format!("{root_org_name}/law/targets")),
            "dependencies.json",
            &content,
        )
        .unwrap();
    root_repo
        .commit(Some("targets/dependencies.json"), "Add dependencies.json")
        .unwrap();
    Ok(())
}
pub fn get_repository(path: &Path, name: &str) -> GitRepository {
    let mut path = path.to_path_buf();
    path.push(name);
    GitRepository::open(&path).unwrap()
}
