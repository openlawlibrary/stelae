use anyhow::Result;
use std::path::Path;
use tempfile::TempDir;

/// Stele testing framework requires working with bare repositories.
/// One idea was to initialize the git2 repository as a bare repository, and add/commit files to the bare repo.
/// However, this approach does not work because index methods fail on bare repositories, see [1].
/// Instead, we initialized the git2 repository as a normal repository, and then convert the repository to a bare repository.
///
/// [1] - https://libgit2.org/libgit2/#HEAD/group/index/git_index_add_all
pub fn make_all_git_repos_bare_recursive(td: &TempDir) -> Result<()> {
    visit_dirs(td.path())
}

fn visit_dirs(dir: &Path) -> Result<()> {
    if dir.is_dir() {
        process_directory(dir)?;
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path)?;
            }
        }
    }
    Ok(())
}

fn process_directory(dir_path: &Path) -> Result<()> {
    let git_dir = dir_path.join(".git");
    if git_dir.exists() && git_dir.is_dir() {
        // Remove contents from the current directory, excluding the .git directory
        for entry in std::fs::read_dir(dir_path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path != git_dir {
                if entry_path.is_dir() {
                    std::fs::remove_dir_all(&entry_path)?;
                } else {
                    std::fs::remove_file(&entry_path)?;
                }
            }
        }
        // Move contents of .git subdirectory to the current directory
        for entry in std::fs::read_dir(&git_dir)? {
            let entry = entry?;
            let entry_path = entry.path();
            let new_path = dir_path.join(entry_path.file_name().unwrap());

            std::fs::rename(&entry_path, &new_path)?;
        }
        // Remove the .git directory
        std::fs::remove_dir_all(&git_dir)?;
    }
    Ok(())
}
