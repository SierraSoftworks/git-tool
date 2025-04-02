use super::*;
use crate::{engine::Target, git};
use tracing_batteries::prelude::*;

pub struct GitCheckout<'a> {
    pub branch: &'a str,
}

#[async_trait::async_trait]
impl Task for GitCheckout<'_> {
    #[tracing::instrument(name = "task:git_checkout(repo)", err, skip(self, _core))]
    async fn apply_repo(&self, _core: &Core, repo: &engine::Repo) -> Result<(), engine::Error> {
        git::git_checkout(&repo.get_path(), self.branch).await
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
            "gh:sierrasoftworks/test-git-checkout",
            temp.path().join("repo"),
        );

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        sequence![GitInit {}, GitCheckout { branch: "test" }]
            .apply_repo(&core, &repo)
            .await
            .unwrap();
        assert!(repo.valid());

        assert_eq!(
            git::git_current_branch(&repo.get_path()).await.unwrap(),
            "test"
        );
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = tempdir().unwrap();
        let scratch = Scratchpad::new("2019w15", temp.path().join("scratch"));

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        let task = GitCheckout { branch: "test" };

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert!(!scratch.exists());
    }
}
