use git2::{Blob, Error, Object, Oid, Repository, Tree};
use std::fmt;

pub struct Repo {
    lib_path: String,
    namespace: String,
    name: String,
    repo: Repository,
}

impl fmt::Debug for Repo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Repo for {}/{} in the library at {}",
            self.namespace, self.name, self.lib_path
        )
    }
}

impl Repo {
    pub fn new(lib_path: &str, namespace: &str, name: &str) -> Result<Repo, Error> {
        let repo_path = format!("{lib_path}/{namespace}/{name}");
        Ok(Repo {
            lib_path: String::from(lib_path),
            namespace: String::from(namespace),
            name: String::from(name),
            repo: Repository::open(repo_path)?,
        })
    }
    fn get_commit_tree(&self, commitish: &str) -> Result<Tree, Error> {
        let oid = Oid::from_str(commitish)?;
        let commit = self.repo.find_commit(oid)?;
        let tree = commit.tree()?;
        Ok(tree)
    }

    fn get_child_blob(&self, path_part: &str, tree: &Tree) -> anyhow::Result<Blob> {
        let tree_entry = match tree.get_name(path_part) {
            Some(entry) => entry,
            None => return Err(anyhow::anyhow!("No entry")),
        };
        let obj = tree_entry.to_object(&self.repo)?;
        let blob = match obj.into_blob() {
            Ok(blob) => blob,
            Err(_) => return Err(anyhow::anyhow!("No blob")),
        };
        Ok(blob)
    }

    fn get_child_object(&self, path_part: &str, tree: &Tree) -> anyhow::Result<Object> {
        let tree_entry = match tree.get_name(path_part) {
            Some(entry) => entry,
            None => return Err(anyhow::anyhow!("No entry")),
        };
        let obj = tree_entry.to_object(&self.repo)?;
        Ok(obj)
    }
    fn get_child_tree(&self, path_part: &str, tree: &Tree) -> anyhow::Result<Tree> {
        let obj = self.get_child_object(path_part, tree)?;
        let new_tree = match obj.into_tree() {
            Ok(tree) => tree,
            Err(_) => return Err(anyhow::anyhow!("No tree")),
        };
        Ok(new_tree)
    }

    fn get_tree(&self, path: &[&str], tree: &Tree) -> anyhow::Result<Tree> {
        let path_part = path[0];
        let new_path = &path[1..];
        let new_tree = self.get_child_tree(path_part, tree)?;
        match new_path.len() {
            0 => Ok(new_tree),
            _ => self.get_tree(new_path, &new_tree),
        }
    }

    pub fn get_bytes_at_path(&self, commitish: &str, path: &[&str]) -> anyhow::Result<Vec<u8>> {
        let root_tree = self.get_commit_tree(commitish)?;
        let (path_part, parent_tree) = match path.len() {
            0 => ("index.html", root_tree),
            1 => match path[0] {
                "" => ("index.html", root_tree),
                _ => (path[0], root_tree),
            },
            _ => {
                let last = path.len() - 1;
                let path_part = path[last];
                let tree_path = &path[0..last];
                let parent_tree = self.get_tree(tree_path, &root_tree)?;
                (path_part, parent_tree)
            }
        };

        // exact match
        if let Ok(blob) = self.get_child_blob(path_part, &parent_tree) {
            return Ok(blob.content().to_owned());
        }

        // append `/index.html`
        if let Ok(tree) = self.get_child_tree(path_part, &parent_tree) {
            let blob = self.get_child_blob("index.html", &tree)?;
            return Ok(blob.content().to_owned());
        }

        // append `.html`
        match self.get_child_blob(&format!("{path_part}.html"), &parent_tree) {
            Ok(blob) => Ok(blob.content().to_owned()),
            Err(_) => Err(anyhow::anyhow!("Not found")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::git::Repo;
    use std::env::current_exe;
    use std::fs::create_dir_all;
    use std::path::PathBuf;
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub fn initialize() {
        INIT.call_once(|| {
            let repo_path = get_test_library_path().join(PathBuf::from("test/law-html"));
            let heads_path = repo_path.join(PathBuf::from("refs/heads"));
            create_dir_all(heads_path).unwrap();
            let tags_path = repo_path.join(PathBuf::from("refs/tags"));
            create_dir_all(tags_path).unwrap();
        });
    }

    fn get_test_library_path() -> PathBuf {
        let mut library_path = current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        library_path.push("test");
        library_path.push("library");
        library_path
    }

    #[test]
    fn test_get_bytes_at_path_when_empty_path_expect_index_html() {
        initialize();
        let test_library_path = get_test_library_path();
        let repo = Repo::new(test_library_path.to_str().unwrap(), "test", "law-html").unwrap();
        let actual = repo
            .get_bytes_at_path("ed782e08d119a580baa3067e2ea5df06f3d1cd05", &[""])
            .unwrap();
        let expected = "<!DOCTYPE html>";
        assert_eq!(
            &std::str::from_utf8(actual.as_slice()).unwrap()[..15],
            expected
        );
    }

    #[test]
    fn test_get_bytes_at_path_when_full_path_expect_data() {
        initialize();
        let test_library_path = get_test_library_path();
        let repo = Repo::new(test_library_path.to_str().unwrap(), "test", "law-html").unwrap();
        let actual = repo
            .get_bytes_at_path(
                "ed782e08d119a580baa3067e2ea5df06f3d1cd05",
                &["a", "b", "c.html"],
            )
            .unwrap();
        let expected = "<!DOCTYPE html>";
        assert_eq!(
            &std::str::from_utf8(actual.as_slice()).unwrap()[..15],
            expected
        );
    }

    #[test]
    fn test_get_bytes_at_path_when_omit_html_expect_data() {
        initialize();
        let test_library_path = get_test_library_path();
        let repo = Repo::new(test_library_path.to_str().unwrap(), "test", "law-html").unwrap();
        let actual = repo
            .get_bytes_at_path("ed782e08d119a580baa3067e2ea5df06f3d1cd05", &["a", "b", "c"])
            .unwrap();
        let expected = "<!DOCTYPE html>";
        assert_eq!(
            &std::str::from_utf8(actual.as_slice()).unwrap()[..15],
            expected
        );
    }

    #[test]
    fn test_get_bytes_at_path_when_omit_index_expect_data() {
        initialize();
        let test_library_path = get_test_library_path();
        let repo = Repo::new(test_library_path.to_str().unwrap(), "test", "law-html").unwrap();
        let actual = repo
            .get_bytes_at_path("ed782e08d119a580baa3067e2ea5df06f3d1cd05", &["a", "b", "d"])
            .unwrap();
        let expected = "<!DOCTYPE html>";
        assert_eq!(
            &std::str::from_utf8(actual.as_slice()).unwrap()[..15],
            expected
        );
    }

    #[test]
    fn test_get_bytes_at_path_when_invalid_repo_namespace_expect_error() {
        initialize();
        let test_library_path = get_test_library_path();
        let actual = Repo::new(test_library_path.to_str().unwrap(), "xxx", "law-html").unwrap_err();
        let expected = "failed to resolve path";
        assert_eq!(&format!("{}", actual)[..22], expected);
    }

    #[test]
    fn test_get_bytes_at_path_when_invalid_repo_name_expect_error() {
        initialize();
        let test_library_path = get_test_library_path();
        let actual = Repo::new(test_library_path.to_str().unwrap(), "test", "xxx").unwrap_err();
        let expected = "failed to resolve path";
        assert_eq!(&format!("{}", actual)[..22], expected);
    }

    #[test]
    fn test_get_bytes_at_path_when_invalid_path_expect_error() {
        initialize();
        let test_library_path = get_test_library_path();
        let repo = Repo::new(test_library_path.to_str().unwrap(), "test", "law-html").unwrap();
        let actual = repo
            .get_bytes_at_path("ed782e08d119a580baa3067e2ea5df06f3d1cd05", &["a", "b", "x"])
            .unwrap_err();
        let expected = "Not found";
        assert_eq!(format!("{}", actual), expected);
    }
}
