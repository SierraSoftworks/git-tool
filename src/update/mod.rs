mod api;
mod release;
mod manager;
mod github;

pub use release::{Release, ReleaseVariant};
pub use api::*;
pub use github::GitHubSource;
pub use manager::UpdateManager;