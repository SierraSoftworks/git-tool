use super::git_cmd;
use crate::errors;
use std::path;
use tokio::process::Command;

pub async fn git_clone(repo: &path::Path, url: &str) -> Result<(), errors::Error> {
    info!("Running `git clone --recurse-submodules $URL` to prepare repository");
    git_cmd(
        Command::new("git")
            .arg("clone")
            .arg("--recurse-submodules")
            .arg(url)
            .arg(repo),
    )
    .await?;

    Ok(())
}
