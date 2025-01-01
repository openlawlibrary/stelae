use serde::Deserialize;
/// Structure for
#[derive(Debug, Deserialize)]
pub struct StelaeQueryData {
    /// commit of the repo
    pub commitish: Option<String>,
    /// path of the file
    pub remainder: Option<String>,
}
