use super::git_cmd;
use crate::errors;
use std::path;
use tokio::process::Command;

pub async fn git_checkout(repo: &path::Path, name: &str) -> Result<(), errors::Error> {
    info!("Running `git checkout -B $BRANCH_NAME` to switch branches");
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
