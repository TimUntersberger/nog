use repository::Repository;
use semantic_version::SemanticVersion;

pub mod repository;
pub mod semantic_version;
#[derive(Clone, Debug)]
pub struct UpdateChannel {
    pub name: String,
    pub repo: Repository,
    pub branch: String,
    pub version: SemanticVersion,
}

impl Default for UpdateChannel {
    fn default() -> Self {
        Self {
            name: "".into(),
            branch: "master".into(),
            repo: Repository::default(),
            version: SemanticVersion::default(),
        }
    }
}
