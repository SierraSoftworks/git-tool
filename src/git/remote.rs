use super::git_cmd;
use crate::errors;
use crate::git::cmd::validate_repo_path_exists;
use std::path;
use tokio::process::Command;
use tracing_batteries::prelude::*;

pub async fn git_remote_list(repo: &path::Path) -> Result<Vec<String>, errors::Error> {
    info!("Running `git remote` to list configured remotes");
    validate_repo_path_exists(repo)?;
    let output = git_cmd(Command::new("git").current_dir(repo).arg("remote"))
        .await?
        .split_terminator('\n')
        .map(|s| s.trim().to_string())
        .collect();

    Ok(output)
}

pub async fn git_remote_add(repo: &path::Path, name: &str, url: &str) -> Result<(), errors::Error> {
    info!("Running `git remote add $NAME $URL` to add new remote");
    validate_repo_path_exists(repo)?;
    git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("remote")
            .arg("add")
            .arg(name)
            .arg(url),
    )
    .await?;

    Ok(())
}

pub async fn git_remote_set_url(
    repo: &path::Path,
    name: &str,
    url: &str,
) -> Result<(), errors::Error> {
    info!("Running `git remote set-url $NAME $URL` to add new remote");
    validate_repo_path_exists(repo)?;
    git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("remote")
            .arg("set-url")
            .arg(name)
            .arg(url),
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::git::*;

    #[tokio::test]
    async fn test_git_remote() {
        let temp_dir = tempfile::tempdir().expect("a temporary directory");

        git_init(temp_dir.path())
            .await
            .expect("git init to succeed");

        let remotes = git_remote_list(temp_dir.path())
            .await
            .expect("git remote list to succeed");
        assert_eq!(remotes.len(), 0, "git remote list should be empty");

        git_remote_add(temp_dir.path(), "origin", "https://example.com/test.git")
            .await
            .expect("git remote add origin https://example.com/test.git to succeed");
        let remotes = git_remote_list(temp_dir.path())
            .await
            .expect("git remote list to succeed");
        assert_eq!(
            remotes,
            vec!["origin"],
            "git remote list should have one remote: [origin]"
        );

        git_remote_set_url(temp_dir.path(), "origin", "https://example.com/test2.git")
            .await
            .expect("git remote set-url origin https://example.com/test2.git to succeed");
        let remotes = git_remote_list(temp_dir.path())
            .await
            .expect("git remote list to succeed");
        assert_eq!(
            remotes,
            vec!["origin"],
            "git remote list should have one remote: [origin]"
        );
    }
}
