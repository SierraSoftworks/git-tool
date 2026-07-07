use super::*;
use crate::{engine::Target, git};
use std::path::PathBuf;
use tracing_batteries::prelude::*;

pub struct GitWorktreeRemove {
    pub path: PathBuf,
}

#[async_trait::async_trait]
impl Task for GitWorktreeRemove {
    fn name(&self) -> &'static str {
        "git-worktree-remove"
    }

    #[tracing::instrument(name = "task:git_worktree_remove(repo)", err, skip(self, _core))]
    async fn apply_repo(&self, _core: &Core, repo: &engine::Repo) -> Result<(), engine::Error> {
        git::git_worktree_remove(&repo.get_path(), &self.path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn prepare_commit(repo: &engine::Repo) {
        let path = repo.get_path();
        std::fs::write(path.join("test.txt"), "testing").unwrap();
        git::git_add(&path, &vec!["test.txt"]).await.unwrap();
        git::git_commit(&path, "Initial commit", &vec!["test.txt"])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_remove_worktree() {
        let temp = tempdir().unwrap();
        let repo = engine::Repo::new(
            "gh:sierrasoftworks/test-git-worktree-remove",
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

        let worktree_path = temp.path().join("worktrees").join("feature");
        GitWorktree {
            path: worktree_path.clone(),
            branch: "feature".into(),
            create_if_missing: true,
            base: None,
        }
        .apply_repo(&core, &repo)
        .await
        .unwrap();

        assert!(
            worktree_path.exists(),
            "the worktree should have been created"
        );

        GitWorktreeRemove {
            path: worktree_path.clone(),
        }
        .apply_repo(&core, &repo)
        .await
        .unwrap();

        assert!(
            !worktree_path.exists(),
            "the worktree should have been removed"
        );
    }
}
