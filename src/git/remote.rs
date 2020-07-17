use super::git_cmd;
use crate::errors;
use std::path;
use tokio::process::Command;

pub async fn git_remote_list(repo: &path::Path) -> Result<Vec<String>, errors::Error> {
    info!("Running `git remote` to list configured remotes");
    let output = git_cmd(Command::new("git").current_dir(repo).arg("remote"))
        .await?
        .split_terminator('\n')
        .map(|s| s.trim().to_string())
        .collect();

    Ok(output)
}

pub async fn git_remote_add(repo: &path::Path, name: &str, url: &str) -> Result<(), errors::Error> {
    info!("Running `git remote add $NAME $URL` to add new remote");
    git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("remote")
            .arg("add")
            .arg(name)
            .arg(url),
    )
    .await?;

    Ok(())
}

pub async fn git_remote_set_url(
    repo: &path::Path,
    name: &str,
    url: &str,
) -> Result<(), errors::Error> {
    info!("Running `git remote set-url $NAME $URL` to add new remote");
    git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("remote")
            .arg("set-url")
            .arg(name)
            .arg(url),
    )
    .await?;

    Ok(())
}
