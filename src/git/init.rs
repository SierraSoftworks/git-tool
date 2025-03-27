use super::git_cmd;
use crate::errors;
use crate::git::cmd::validate_repo_path_exists;
use std::path;
use tokio::process::Command;
use tracing_batteries::prelude::*;

pub async fn git_init(path: &path::Path) -> Result<(), errors::Error> {
    info!("Running `git init` to prepare repository");
    git_cmd(Command::new("git").arg("init").arg(path)).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_git_init() {
        let temp_dir = tempfile::tempdir().expect("a temporary directory");

        git_init(temp_dir.path())
            .await
            .expect("git init to succeed");

        assert!(
            temp_dir.path().join(".git").exists(),
            "git init to create .git directory"
        );
    }
}
