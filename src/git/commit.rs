use super::git_cmd;
use crate::errors;
use crate::git::cmd::validate_repo_path_exists;
use std::path;
use tokio::process::Command;
use tracing_batteries::prelude::*;

#[allow(dead_code)]
pub async fn git_commit(
    repo: &path::Path,
    message: &str,
    paths: &Vec<&str>,
) -> Result<(), errors::Error> {
    info!("Running `git commit` to create a new commit for specific files");
    validate_repo_path_exists(repo)?;
    git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("commit")
            .arg("-m")
            .arg(message)
            .args(paths),
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::*;
    use tempfile::tempdir;

    #[tokio::test]
    pub async fn test_commit() {
        let temp = tempdir().unwrap();

        git_init(temp.path()).await.unwrap();

        // Configure Git's user to avoid requiring this to be set in the environment
        git_config_set(temp.path(), "user.name", "Test User")
            .await
            .unwrap();
        git_config_set(temp.path(), "user.email", "user@example.com")
            .await
            .unwrap();

        std::fs::write(temp.path().join("test.txt"), "testing").unwrap();
        assert!(
            temp.path().join("test.txt").exists(),
            "the test file should exist"
        );

        git_add(temp.path(), &vec!["test.txt"]).await.unwrap();

        assert!(
            git_cmd(
                Command::new("git")
                    .current_dir(temp.path())
                    .arg("rev-parse")
                    .arg("--short")
                    .arg("HEAD"),
            )
            .await
            .is_err()
        );

        git_commit(temp.path(), "test commit", &vec!["test.txt"])
            .await
            .unwrap();

        assert!(
            git_cmd(
                Command::new("git")
                    .current_dir(temp.path())
                    .arg("rev-parse")
                    .arg("--short")
                    .arg("HEAD"),
            )
            .await
            .is_ok()
        );
    }
}
