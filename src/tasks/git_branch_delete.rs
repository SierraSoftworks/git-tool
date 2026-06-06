use super::*;
use crate::{engine::Target, git};
use tracing_batteries::prelude::*;

pub struct GitBranchDelete {
    pub branch: String,
}

#[async_trait::async_trait]
impl Task for GitBranchDelete {
    #[tracing::instrument(name = "task:git_branch_delete(repo)", err, skip(self, _core))]
    async fn apply_repo(&self, _core: &Core, repo: &engine::Repo) -> Result<(), engine::Error> {
        git::git_branch_delete(&repo.get_path(), &self.branch).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_repo() {
        let temp = tempdir().unwrap();
        let repo = Repo::new(
            "gh:sierrasoftworks/test-git-branch-delete",
            temp.path().join("repo"),
        );

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        sequence![GitInit {}, GitCheckout { branch: "main" }]
            .apply_repo(&core, &repo)
            .await
            .unwrap();

        std::fs::write(repo.get_path().join("README.md"), "Example").unwrap();
        git::git_add(&repo.get_path(), &vec!["README.md"])
            .await
            .unwrap();
        git::git_commit(&repo.get_path(), "Initial commit", &vec!["README.md"])
            .await
            .unwrap();

        GitCheckout { branch: "feature" }
            .apply_repo(&core, &repo)
            .await
            .unwrap();
        GitCheckout { branch: "main" }
            .apply_repo(&core, &repo)
            .await
            .unwrap();

        assert!(
            git::git_branches(&repo.get_path())
                .await
                .unwrap()
                .contains(&"feature".to_string())
        );

        GitBranchDelete {
            branch: "feature".into(),
        }
        .apply_repo(&core, &repo)
        .await
        .unwrap();

        assert!(
            !git::git_branches(&repo.get_path())
                .await
                .unwrap()
                .contains(&"feature".to_string())
        );
    }
}
