use super::git_cmd;
use crate::errors;
use std::path;
use tokio::process::Command;

pub async fn git_add(repo: &path::Path, paths: &Vec<&str>) -> Result<(), errors::Error> {
    info!("Running `git add` to add files to the index");
    git_cmd(Command::new("git").current_dir(repo).arg("add").args(paths)).await?;

    Ok(())
}
