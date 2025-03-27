use tokio::process::Command;

use super::git_cmd;
use crate::errors;
use crate::git::cmd::validate_repo_path_exists;
use std::path;
use tracing_batteries::prelude::*;

pub async fn git_switch(repo: &path::Path, name: &str, create: bool) -> Result<(), errors::Error> {
    info!("Running `git switch $BRANCH_NAME` to switch branches");
    validate_repo_path_exists(repo)?;
    if create {
        git_cmd(
            Command::new("git")
                .current_dir(repo)
                .arg("switch")
                .arg("--create")
                .arg(name),
        )
        .await?;
    } else {
        git_cmd(
            Command::new("git")
                .current_dir(repo)
                .arg("switch")
                .arg(name),
        )
        .await?;
    }

    Ok(())
}
