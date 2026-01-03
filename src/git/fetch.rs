use super::git_cmd;
use crate::git::cmd::validate_repo_path_exists;
use std::path;
use tokio::process::Command;
use tracing_batteries::prelude::*;

#[allow(dead_code)]
pub async fn git_fetch(repo: &path::Path, origin: &str) -> Result<(), human_errors::Error> {
    info!("Running `git fetch $ORIGIN`");
    validate_repo_path_exists(repo)?;
    git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("fetch")
            .arg(origin),
    )
    .await?;

    Ok(())
}
