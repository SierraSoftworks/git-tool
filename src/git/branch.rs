use super::git_cmd;
use crate::errors;
use std::path;
use tokio::process::Command;

pub async fn git_current_branch(repo: &path::Path) -> Result<String, errors::Error> {
    Ok(git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("symbolic-ref")
            .arg("--short")
            .arg("-q")
            .arg("HEAD"),
    )
    .await?
    .trim()
    .to_string())
}

pub async fn git_branches(repo: &path::Path) -> Result<Vec<String>, errors::Error> {
    let output = git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("for-each-ref")
            .arg("--format=%(refname:lstrip=2)")
            .arg("refs/heads/"),
    )
    .await?;

    Ok(output
        .split_terminator('\n')
        .map(|s| s.trim().to_string())
        .collect())
}
