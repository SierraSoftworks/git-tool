use std::path;

use tokio::process::Command;
use tracing_batteries::prelude::*;

use crate::git::cmd::validate_repo_path_exists;
use crate::git::git_cmd;

pub async fn git_config_set(
    repo: &path::Path,
    key: &str,
    value: &str,
) -> Result<(), human_errors::Error> {
    info!("Running `git config` to set a configuration value");
    validate_repo_path_exists(repo)?;
    git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("config")
            .arg(key)
            .arg(value),
    )
    .await?;

    Ok(())
}
