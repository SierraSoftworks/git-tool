use std::path;
use crate::errors;
use super::git_cmd;
use tokio::process::Command;

pub async fn git_remote_add(repo: &path::Path, name: &str, url: &str) -> Result<(), errors::Error> {
    git_cmd(Command::new("git")
        .current_dir(repo)
        .arg("remote")
        .arg("add")
        .arg(name)
        .arg(url)).await?;
        
    Ok(())
}