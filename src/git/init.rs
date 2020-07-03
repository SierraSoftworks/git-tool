use std::path;
use crate::errors;
use super::git_cmd;
use tokio::process::Command;

pub async fn git_init(path: &path::Path) -> Result<(), errors::Error> {
    info!("Running `git init` to prepare repository");
    git_cmd(Command::new("git")
        .arg("init")
        .arg(path)).await?;
        
    Ok(())
}