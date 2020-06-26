use std::path;
use crate::errors;
use super::git_cmd;
use tokio::process::Command;

pub async fn git_clone(repo: &path::Path, url: &str) -> Result<(), errors::Error> {
    git_cmd(Command::new("git")
        .arg("clone")
        .arg("--recurse-submodules")
        .arg(url)
        .arg(repo)).await?;
        
    Ok(())
}