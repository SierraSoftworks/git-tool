use super::git_cmd;
use crate::{errors};
use std::path;
use tokio::process::Command;
use tracing_batteries::prelude::*;
use crate::tasks::GitRemote;

pub async fn git_remote_list(repo: &path::Path) -> Result<Vec<String>, errors::Error> {
    info!("Running `git remote` to list configured remotes");
    let output = git_cmd(Command::new("git").current_dir(repo).arg("remote"))
        .await?
        .split_terminator('\n')
        .map(|s| s.trim().to_string())
        .collect();

    Ok(output)
}

pub async fn git_remote_get_url(repo: &path::Path, name: &str) -> Result<Vec<String>, errors::Error> {
    info!("Running `git remote get-url` to list configured remote url");
    let output = git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("remote")
            .arg("get-url")
            .arg(name))
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

pub async fn git_remote_update_repo_name(
    repo: &path::Path,
    new_name: &str,
) -> Result<(), errors::Error> {
    info!("Updating git remote URL to match the new name");

    // git remote get-url origin

    let remote = git_remote_get_url(repo, "origin").await?
        .into_iter()
        .next();

    if let Some(remote_url) = remote.and_then(|url| GitRemote::parse(&url)) {
        let new_url = remote_url.with_repo_name(new_name);
        git_remote_set_url(repo, "origin", new_url.as_str()).await?;
    }

    Ok(())
}
