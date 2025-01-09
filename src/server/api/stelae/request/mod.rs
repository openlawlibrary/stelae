use serde::Deserialize;
/// Structure for passing query parameters for stelae endpoint
#[derive(Debug, Deserialize)]
pub struct StelaeQueryData {
    /// commit (or reference) to the repo. Can pass in `HEAD`, a branch ref (e.g. main), or a commit SHA.
    pub commitish: String,
    /// path of the file (e.g. \us\ca\cities\san-mateo\index.html). If nothing is passed by default it will look for index.html
    pub remainder: Option<String>,
}
