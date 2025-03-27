use super::git_cmd;
use crate::errors;
use crate::git::cmd::validate_repo_path_exists;
use std::path;
use tokio::process::Command;
use tracing_batteries::prelude::*;

pub async fn git_rev_parse(repo: &path::Path, ref_name: &str) -> Result<String, errors::Error> {
    info!("Running `git rev-parse --verify` to get the SHA of a specific reference");
    validate_repo_path_exists(repo)?;
    Ok(git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("rev-parse")
            .arg("--verify")
            .arg(ref_name),
    )
    .await?
    .trim()
    .to_string())
}

pub async fn git_update_ref(
    repo: &path::Path,
    ref_name: &str,
    sha: &str,
) -> Result<(), errors::Error> {
    info!("Running `git update-ref` to update a reference to point at a specific commit");
    validate_repo_path_exists(repo)?;
    git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("update-ref")
            .arg(ref_name)
            .arg(sha),
    )
    .await?;

    Ok(())
}
