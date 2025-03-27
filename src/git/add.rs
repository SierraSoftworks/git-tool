use super::git_cmd;
use crate::errors;
use crate::git::cmd::validate_repo_path_exists;
use std::path;
use tokio::process::Command;
use tracing_batteries::prelude::*;

pub async fn git_add(repo: &path::Path, paths: &Vec<&str>) -> Result<(), errors::Error> {
    info!("Running `git add` to add files to the index");
    validate_repo_path_exists(repo)?;
    git_cmd(Command::new("git").current_dir(repo).arg("add").args(paths)).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::*;
    use tempfile::tempdir;

    #[tokio::test]
    pub async fn test_add() {
        let temp = tempdir().unwrap();

        git_init(temp.path()).await.unwrap();

        std::fs::write(temp.path().join("test.txt"), "testing").unwrap();
        assert!(
            temp.path().join("test.txt").exists(),
            "the test file should exist"
        );

        git_add(temp.path(), &vec!["test.txt"]).await.unwrap();

        let files = git_cmd(
            Command::new("git")
                .current_dir(temp.path())
                .arg("diff")
                .arg("--name-only")
                .arg("--cached"),
        )
        .await
        .unwrap();
        assert!(files.contains("test.txt"))
    }
}
