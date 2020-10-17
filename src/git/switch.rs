use tokio::process::Command;

use super::git_cmd;
use crate::errors;
use std::path;

pub async fn git_switch(repo: &path::Path, name: &str, create: bool) -> Result<(), errors::Error> {
    if create {
        git_cmd(
            Command::new("git")
                .current_dir(repo)
                .arg("switch")
                .arg("--create")
                .arg(name),
        )
        .await?;
    } else {
        git_cmd(
            Command::new("git")
                .current_dir(repo)
                .arg("switch")
                .arg(name),
        )
        .await?;
    }

    Ok(())
}
