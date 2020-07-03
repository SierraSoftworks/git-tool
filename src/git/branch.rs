use super::git_cmd;
use crate::errors;
use std::path;
use tokio::process::Command;

pub async fn git_current_branch(repo: &path::Path) -> Result<String, errors::Error> {
    info!("Running `git symbolic-ref --short -q HEAD` to get the current branch name");
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
    info!("Running `git for-each-ref --format=%(refname:lstrip=2) refs/heads/` to get the list of branches");
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
