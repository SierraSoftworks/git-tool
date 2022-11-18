pub(self) mod cmd;
pub(self) mod fs;
mod github;
mod manager;
mod release;
mod state;

pub use github::GitHubSource;
pub use manager::UpdateManager;
pub use release::{Release, ReleaseVariant};
pub use state::*;
