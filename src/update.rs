//! Self-update support, built on the [`update-rs`](https://docs.rs/update-rs)
//! crate (which was extracted from this project).
//!
//! This module just configures the updater for Git-Tool's GitHub releases and
//! re-exports the handful of types the rest of the application needs; all of the
//! three-phase download/replace/relaunch machinery lives in the crate.

pub use update_rs::{RESUME_FLAG, Release};

use update_rs::{GitHubSource, UpdateManager, naming};

/// The GitHub repository Git-Tool's releases are published to.
const REPO: &str = "SierraSoftworks/git-tool";

/// Build an [`UpdateManager`] configured for Git-Tool's releases.
///
/// It downloads the Go-style `git-tool-<os>-<arch>[.exe]` asset for the current
/// platform (matching the names produced by `.github/workflows/release.yml`)
/// from the project's GitHub releases, whose tags are `vX.Y.Z`.
pub fn manager() -> UpdateManager<GitHubSource> {
    UpdateManager::new(GitHubSource::new(REPO, naming::go("git-tool")).with_release_tag_prefix("v"))
}
