use std::path;

use tokio::process::Command;
use tracing_batteries::prelude::*;

use crate::{errors, git::git_cmd};

pub async fn git_config_set(
    repo: &path::Path,
    key: &str,
    value: &str,
) -> Result<(), errors::Error> {
    info!("Running `git config` to set a configuration value");
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
