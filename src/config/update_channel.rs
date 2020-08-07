use semantic_version::SemanticVersion;
use repository::Repository;

pub mod semantic_version;
pub mod repository;
#[derive(Clone, Debug)]
pub struct UpdateChannel{
    name: String,
    repo: Repository,
    branch: String,
    version: SemanticVersion
}