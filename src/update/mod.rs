mod api;
mod github;
mod manager;
mod release;

pub use api::*;
pub use github::GitHubSource;
pub use manager::UpdateManager;
pub use release::{Release, ReleaseVariant};
