use anyhow::Result;
use git2::{Commit, Error, Oid, Repository};
use lazy_static::lazy_static;
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

#[derive(Default)]
pub enum DataRepositoryType {
    #[default]
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
}

impl GitRepository {
    pub fn init(path: &Path) -> Result<Self> {
        let repo = Repository::init(path)?;
        Ok(Self {
            repo,
            kind: DataRepositoryType::Html("html".into()),
        })
    }

    pub fn commit(&self, path_str: &str, commit_msg: &str) -> Result<Oid, Error> {
        let mut index = self.repo.index().unwrap();
        index.add_path(&PathBuf::from(path_str)).unwrap();
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

    pub fn write_file(&self, path: &Path, file_name: &str, content: &str) -> Result<()> {
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

lazy_static! {
    static ref SCRIPT_PATH: PathBuf = {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/fixtures/scripts");
        path
    };
}

pub fn execute_script(script_name: &str, mut script_result_directory: PathBuf) -> Result<()> {
    let script_absolute_path = fix_unc_path(&SCRIPT_PATH.join(script_name).canonicalize()?);
    let env_path = std::env::current_dir()?.join(script_name);
    let mut cmd = std::process::Command::new(&script_absolute_path);
    let output = match configure_command(&mut cmd, &script_result_directory).output() {
        Ok(out) => out,
        Err(err)
            if err.kind() == std::io::ErrorKind::PermissionDenied || err.raw_os_error() == Some(193) /* windows */ =>
        {
            cmd = std::process::Command::new("bash");
            let output = configure_command(cmd.arg(&script_absolute_path), &script_result_directory).output().unwrap();
            output
        }
        Err(err) => return Err(err.into()),
    };
    Ok(())
}

fn configure_command<'a>(
    cmd: &'a mut std::process::Command,
    script_result_directory: &Path,
) -> &'a mut std::process::Command {
    let never_path = if cfg!(windows) { "-" } else { ":" };
    dbg!(&cmd);
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir(script_result_directory)
        .env_remove("GIT_DIR")
        .env_remove("GIT_ASKPASS")
        .env_remove("SSH_ASKPASS")
        .env("GIT_CONFIG_SYSTEM", never_path)
        .env("GIT_CONFIG_GLOBAL", never_path)
        .env("GIT_TERMINAL_PROMPT", "false")
        .env("GIT_AUTHOR_DATE", "2000-01-01 00:00:00 +0000")
        .env("GIT_AUTHOR_EMAIL", "author@openlawlib.org")
        .env("GIT_AUTHOR_NAME", "author")
        .env("GIT_COMMITTER_DATE", "2000-01-02 00:00:00 +0000")
        .env("GIT_COMMITTER_EMAIL", "committer@openlawlib.org")
        .env("GIT_COMMITTER_NAME", "committer")
}
