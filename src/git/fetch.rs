use super::git_cmd;
use crate::errors;
use std::path;
use tokio::process::Command;
use tracing_batteries::prelude::*;

#[allow(dead_code)]
pub async fn git_fetch(repo: &path::Path, origin: &str) -> Result<(), errors::Error> {
    info!("Running `git fetch $ORIGIN`");
    git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("fetch")
            .arg(origin),
    )
    .await?;

    Ok(())
}
