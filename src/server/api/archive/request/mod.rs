use serde::Deserialize;
/// Structure for passing query parameters for archive endpoint
#[derive(Debug, Deserialize)]
pub struct ArchiveQueryData {
    /// commit (or reference) to the repo. Can pass in `HEAD`, a branch ref (e.g. main), or a commit SHA. If nothing is passed by default it will look for HEAD
    pub commitish: Option<String>,
    /// path of the file (e.g. \us\ca\cities\san-mateo\index.html). If nothing is passed by default it will look for index.html
    pub path: Option<String>,
}
