use super::git_cmd;
use crate::git::cmd::validate_repo_path_exists;
use std::path;
use tokio::process::Command;
use tracing_batteries::prelude::*;

/// A worktree associated with a repository, as reported by `git worktree list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Worktree {
    pub path: path::PathBuf,
    pub branch: Option<String>,
    /// The commit currently checked out in the worktree. This is primarily useful
    /// for worktrees with a detached HEAD, where no branch is associated.
    pub head: Option<String>,
}

pub async fn git_worktree_add(
    repo: &path::Path,
    worktree: &path::Path,
    branch: &str,
    create: bool,
    base: Option<&str>,
) -> Result<(), human_errors::Error> {
    info!("Running `git worktree add` to create a new worktree");
    validate_repo_path_exists(repo)?;

    let mut cmd = Command::new("git");
    cmd.current_dir(repo).arg("worktree").arg("add");

    if create {
        cmd.arg("-b").arg(branch).arg(worktree);
        if let Some(base) = base {
            cmd.arg(base);
        }
    } else {
        cmd.arg(worktree).arg(branch);
    }

    git_cmd(&mut cmd).await?;

    Ok(())
}

pub async fn git_worktree_remove(
    repo: &path::Path,
    worktree: &path::Path,
) -> Result<(), human_errors::Error> {
    info!("Running `git worktree remove` to remove a worktree");
    validate_repo_path_exists(repo)?;

    let mut cmd = Command::new("git");
    cmd.current_dir(repo)
        .arg("worktree")
        .arg("remove")
        .arg(worktree);

    git_cmd(&mut cmd).await?;

    Ok(())
}

/// Determines whether the worktree at the given path is free of uncommitted
/// changes (including staged changes and untracked files). Returns `true` when
/// the worktree is clean and can be safely removed.
pub async fn git_worktree_is_clean(worktree: &path::Path) -> Result<bool, human_errors::Error> {
    info!("Running `git status --porcelain` to check for uncommitted changes");
    validate_repo_path_exists(worktree)?;

    let output = git_cmd(
        Command::new("git")
            .current_dir(worktree)
            .arg("status")
            .arg("--porcelain"),
    )
    .await?;

    Ok(output.trim().is_empty())
}

pub async fn git_worktree_list(repo: &path::Path) -> Result<Vec<Worktree>, human_errors::Error> {
    info!("Running `git worktree list --porcelain` to list worktrees");
    validate_repo_path_exists(repo)?;

    let output = git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("worktree")
            .arg("list")
            .arg("--porcelain"),
    )
    .await?;

    let mut worktrees = Vec::new();
    let mut current_path: Option<path::PathBuf> = None;
    let mut current_branch: Option<String> = None;
    let mut current_head: Option<String> = None;

    for line in output.lines() {
        let line = line.trim_end();
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = Some(path::PathBuf::from(path));
        } else if let Some(head) = line.strip_prefix("HEAD ") {
            current_head = Some(head.to_string());
        } else if let Some(branch) = line.strip_prefix("branch ") {
            current_branch = Some(
                branch
                    .strip_prefix("refs/heads/")
                    .unwrap_or(branch)
                    .to_string(),
            );
        } else if line.is_empty() {
            if let Some(path) = current_path.take() {
                worktrees.push(Worktree {
                    path: canonicalize_worktree_path(path),
                    branch: current_branch.take(),
                    head: current_head.take(),
                });
            }
            current_branch = None;
            current_head = None;
        }
    }

    if let Some(path) = current_path.take() {
        worktrees.push(Worktree {
            path: canonicalize_worktree_path(path),
            branch: current_branch.take(),
            head: current_head.take(),
        });
    }

    Ok(worktrees)
}

/// Normalizes a worktree path to its canonical form so that callers can compare
/// it against other paths (such as a repository's location) without having to
/// account for symlinks or platform-specific path representations. If the path
/// cannot be canonicalized (for example, because it no longer exists on disk) the
/// original path is returned unchanged.
fn canonicalize_worktree_path(path: path::PathBuf) -> path::PathBuf {
    std::fs::canonicalize(&path).unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::{git_add, git_checkout, git_commit, git_config_set, git_init};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_worktree_add_and_list() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");

        git_init(&repo).await.unwrap();
        git_checkout(&repo, "main").await.unwrap();

        // We need at least one commit before a worktree can be created.
        git_config_set(&repo, "user.name", "Example User")
            .await
            .unwrap();
        git_config_set(&repo, "user.email", "user@example.com")
            .await
            .unwrap();
        std::fs::write(repo.join("test.txt"), "testing").unwrap();
        git_add(&repo, &vec!["test.txt"]).await.unwrap();
        git_commit(&repo, "Initial commit", &vec!["test.txt"])
            .await
            .unwrap();

        let worktree_path = temp.path().join("worktrees").join("feature");
        git_worktree_add(&repo, &worktree_path, "feature", true, None)
            .await
            .unwrap();

        assert!(worktree_path.join(".git").exists());

        let worktrees = git_worktree_list(&repo).await.unwrap();
        assert!(
            worktrees
                .iter()
                .any(|w| w.branch.as_deref() == Some("feature"))
        );

        // Every entry should report the commit it has checked out.
        assert!(worktrees.iter().all(|w| w.head.is_some()));

        // The worktree can be removed again, provided it has no pending changes.
        git_worktree_remove(&repo, &worktree_path).await.unwrap();
        assert!(!worktree_path.exists());

        let worktrees = git_worktree_list(&repo).await.unwrap();
        assert!(
            !worktrees
                .iter()
                .any(|w| w.branch.as_deref() == Some("feature"))
        );
    }

    #[tokio::test]
    async fn test_worktree_detached_head() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");

        git_init(&repo).await.unwrap();
        git_checkout(&repo, "main").await.unwrap();

        git_config_set(&repo, "user.name", "Example User")
            .await
            .unwrap();
        git_config_set(&repo, "user.email", "user@example.com")
            .await
            .unwrap();
        std::fs::write(repo.join("test.txt"), "testing").unwrap();
        git_add(&repo, &vec!["test.txt"]).await.unwrap();
        git_commit(&repo, "Initial commit", &vec!["test.txt"])
            .await
            .unwrap();

        // Create a worktree with a detached HEAD by checking out the current commit.
        let worktree_path = temp.path().join("worktrees").join("detached");
        git_worktree_add(&repo, &worktree_path, "HEAD", false, None)
            .await
            .unwrap();

        let worktrees = git_worktree_list(&repo).await.unwrap();
        // The detached worktree is the one without an associated branch.
        let detached = worktrees
            .iter()
            .find(|w| w.branch.is_none())
            .expect("the detached worktree should be listed");
        assert!(
            detached.head.is_some(),
            "a detached worktree should still report its HEAD commit"
        );
    }
}
