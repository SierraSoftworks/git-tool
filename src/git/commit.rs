use super::git_cmd;
use crate::errors;
use std::path;
use tokio::process::Command;

pub async fn git_commit(
    repo: &path::Path,
    message: &str,
    paths: &Vec<&str>,
) -> Result<(), errors::Error> {
    info!("Running `git commit` to create a new commit for specific files");
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

        std::fs::write(temp.path().join("test.txt"), "testing").unwrap();
        assert!(
            temp.path().join("test.txt").exists(),
            "the test file should exist"
        );

        git_add(temp.path(), &vec!["test.txt"]).await.unwrap();

        assert!(git_cmd(
            Command::new("git")
                .current_dir(temp.path())
                .arg("rev-parse")
                .arg("--short")
                .arg("HEAD"),
        )
        .await
        .is_err());

        git_commit(temp.path(), "test commit", &vec!["test.txt"])
            .await
            .unwrap();

        assert!(git_cmd(
            Command::new("git")
                .current_dir(temp.path())
                .arg("rev-parse")
                .arg("--short")
                .arg("HEAD"),
        )
        .await
        .is_ok());
    }
}
