use std::path;
use crate::errors;
use super::git_cmd;
use tokio::process::Command;

pub async fn git_checkout(repo: &path::Path, name: &str) -> Result<(), errors::Error> {
    git_cmd(Command::new("git")
        .current_dir(repo)
        .arg("checkout")
        .arg("-B")
        .arg(name)).await?;
        
    Ok(())
}