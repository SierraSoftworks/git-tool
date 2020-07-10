use super::git_cmd;
use crate::errors;
use std::path;
use tokio::process::Command;

pub async fn git_commit(
    repo: &path::Path,
    message: &str,
    paths: &Vec<&str>,
) -> Result<(), errors::Error> {
    info!("Running `git commit` to create a new commit for specific files");
    git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("commit")
            .arg("-m")
            .arg(message)
            .args(paths),
    )
    .await?;

    Ok(())
}
