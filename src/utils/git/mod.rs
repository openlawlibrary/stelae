use git2::{Blob, Error, Object, Oid, Repository, Tree};


pub struct Repo {
    repo: Repository,
}

impl Repo {
    pub fn new(lib_path: &str, namespace: &str, name: &str) -> Result<Repo, Error> {
        let repo_path = format!("{lib_path}/{namespace}/{name}");
        Ok(Repo {
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
