use super::git_cmd;
use crate::errors;
use crate::git::cmd::validate_repo_path_exists;
use std::path;
use tokio::process::Command;
use tracing_batteries::prelude::*;

pub async fn git_checkout(repo: &path::Path, name: &str) -> Result<(), errors::Error> {
    info!("Running `git checkout -B $BRANCH_NAME` to switch branches");
    validate_repo_path_exists(repo)?;
    git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("checkout")
            .arg("-B")
            .arg(name),
    )
    .await?;

    Ok(())
}
