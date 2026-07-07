//! Self-update support, built on the [`update-rs`](https://docs.rs/update-rs)
//! crate (which was extracted from this project).
//!
//! This module configures the updater for Git-Tool's GitHub releases and its
//! relaunch convention, then re-exports the handful of types the rest of the
//! application needs; all of the three-phase download/replace/relaunch machinery
//! lives in the crate.

pub use update_rs::Release;

use crate::engine::Core;
use std::ffi::OsString;
use update_rs::{GitHubSource, Launcher, UpdateManager, naming};

/// The GitHub repository Git-Tool's releases are published to.
const REPO: &str = "SierraSoftworks/git-tool";

/// Relaunches Git-Tool between update phases via its `update --state <json>`
/// sub-command — the convention Git-Tool has used since its Go implementation.
///
/// Keeping it means an update started by an older installed release (which
/// relaunches the new binary with `update --state <json>`) hands off cleanly to
/// this one. The active trace context is carried inside the state itself (via
/// update-rs's `opentelemetry` feature), so no extra arguments are needed.
struct GitToolLauncher {
    session_id: String,
}

impl Launcher for GitToolLauncher {
    fn resume_args(&self, state_json: &str) -> Vec<OsString> {
        vec!["update".into(), "--state".into(), state_json.into()]
    }

    fn extra_envs(&self) -> Vec<(OsString, OsString)> {
        vec![("GITTOOL_SESSION_ID".into(), self.session_id.clone().into())]
    }
}

/// Build an [`UpdateManager`] configured for Git-Tool's releases.
///
/// It downloads the Go-style `git-tool-<os>-<arch>[.exe]` asset for the current
/// platform (matching the names produced by `.github/workflows/release.yml`)
/// from the project's GitHub releases, whose tags are `vX.Y.Z`, reusing
/// Git-Tool's shared HTTP client (from `core`), and relaunches through the
/// `update --state` sub-command.
pub fn manager(core: &Core) -> UpdateManager<GitHubSource> {
    UpdateManager::new(
        GitHubSource::new(REPO, naming::go("git-tool"))
            .with_release_tag_prefix("v")
            .with_client(core.http_client().reqwest_client()),
    )
    .with_launcher(Box::new(GitToolLauncher {
        session_id: core.analytics().session_id().to_string(),
    }))
}
