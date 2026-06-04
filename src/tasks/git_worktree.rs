use super::*;
use crate::{engine::Target, git};
use std::io::Write;
use std::path::PathBuf;
use tracing_batteries::prelude::*;

pub struct GitWorktree {
    pub path: PathBuf,
    pub branch: String,
    pub create_if_missing: bool,
    pub base: Option<String>,
}

#[async_trait::async_trait]
impl Task for GitWorktree {
    #[tracing::instrument(name = "task:git_worktree(repo)", err, skip(self, core))]
    async fn apply_repo(&self, core: &Core, repo: &engine::Repo) -> Result<(), engine::Error> {
        if self.path.exists() {
            return Ok(());
        }

        let mut create = self.create_if_missing;

        if create
            && git::git_branches(&repo.get_path())
                .await?
                .iter()
                .any(|v| v == &self.branch || v == &format!("origin/{}", &self.branch))
        {
            create = false;

            // The branch already exists, so any requested base branch will be
            // ignored. Let the user know rather than silently doing nothing.
            if self.base.is_some() {
                writeln!(
                    core.output(),
                    "Warning: ignoring '--base' because the branch '{}' already exists.",
                    self.branch
                )
                .map_err(|e| {
                    human_errors::wrap_system(
                        e,
                        "Git-Tool was unable to write to the output console.",
                        &["Please report this issue to us on GitHub so that we can investigate further."],
                    )
                })?;
            }
        }

        git::git_worktree_add(
            &repo.get_path(),
            &self.path,
            &self.branch,
            create,
            self.base.as_deref(),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn prepare_commit(repo: &engine::Repo) {
        let path = repo.get_path();
        git::git_config_set(&path, "user.name", "Example User")
            .await
            .unwrap();
        git::git_config_set(&path, "user.email", "user@example.com")
            .await
            .unwrap();
        std::fs::write(path.join("test.txt"), "testing").unwrap();
        git::git_add(&path, &vec!["test.txt"]).await.unwrap();
        git::git_commit(&path, "Initial commit", &vec!["test.txt"])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_worktree() {
        let temp = tempdir().unwrap();
        let repo = engine::Repo::new(
            "gh:sierrasoftworks/test-git-worktree",
            temp.path().join("repo"),
        );

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_null_console()
            .build();

        sequence![GitInit {}, GitCheckout { branch: "main" }]
            .apply_repo(&core, &repo)
            .await
            .unwrap();
        prepare_commit(&repo).await;

        let worktree_path = temp.path().join("worktrees").join("test-feature");
        GitWorktree {
            path: worktree_path.clone(),
            branch: "feature".into(),
            create_if_missing: true,
            base: None,
        }
        .apply_repo(&core, &repo)
        .await
        .unwrap();

        assert!(worktree_path.join(".git").exists());
        assert_eq!(
            git::git_current_branch(&worktree_path).await.unwrap(),
            "feature"
        );
    }

    #[tokio::test]
    async fn test_no_create_missing_branch() {
        let temp = tempdir().unwrap();
        let repo = engine::Repo::new(
            "gh:sierrasoftworks/test-git-worktree",
            temp.path().join("repo"),
        );

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_null_console()
            .build();

        sequence![GitInit {}, GitCheckout { branch: "main" }]
            .apply_repo(&core, &repo)
            .await
            .unwrap();
        prepare_commit(&repo).await;

        let worktree_path = temp.path().join("worktrees").join("test-missing");
        GitWorktree {
            path: worktree_path.clone(),
            branch: "missing".into(),
            create_if_missing: false,
            base: None,
        }
        .apply_repo(&core, &repo)
        .await
        .expect_err("this command should fail");
    }
}
