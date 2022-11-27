// use std::str::Bytes;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use git2::{Blob, Error, Object, Oid, Repository, Tree};
use std::env;
// use std::path::Path;
// use std::ffi::OsStr;

// fn get_extension_from_filename(filename: &str) -> Option<&str> {
//     Path::new(filename)
//         .extension()
//         .and_then(OsStr::to_str)}

struct Repo {
    repo: Repository,
}

impl Repo {
    fn new(namespace: &String, name: &String) -> Result<Repo, Error> {
        let lib_path = match env::var("OLL_LIBRARY_ROOT") {
            Ok(env_var) => env_var,
            Err(_) => String::from("."),
        };
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

    fn get_bytes_at_path(&self, commitish: &str, path: &[&str]) -> anyhow::Result<Vec<u8>> {
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

fn clean_path(path: &str) -> &str {
    let start = usize::from(path.starts_with('/'));
    let end = if path.len() > 1 && path.ends_with('/') {
        path.len() - 1
    } else {
        path.len()
    };
    &path[start..end]
}

#[get("/{namespace}/{name}/{commitish}{remainder:(/[^{}]*)?}")]
async fn get_blob(path: web::Path<(String, String, String, String)>) -> impl Responder {
    let (namespace, name, commitish, remainder) = path.into_inner();
    let repo = match Repo::new(&namespace, &name) {
        Ok(repo) => repo,
        Err(_e) => {
            return HttpResponse::NotFound().body(format!("repo {namespace}/{name} does not exist"))
        }
    };
    let blob_path: Vec<&str> = clean_path(&remainder).split('/').collect();

    match repo.get_bytes_at_path(&commitish, &blob_path) {
        Ok(content) => HttpResponse::Ok().body(content),
        Err(_e) => HttpResponse::NotFound().body(format!(
            "content at {remainder} for {commitish} in repo {namespace}/{name} does not exist"
        )),
    }
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(get_blob))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
