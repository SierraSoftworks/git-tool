use super::*;
use crate::{core::Target, git};

pub struct GitCheckout<'a> {
    pub branch: &'a str,
}

#[async_trait::async_trait]
impl<'a> Task for GitCheckout<'a> {
    #[tracing::instrument(name = "task:git_checkout(repo)", err, skip(self, _core))]
    async fn apply_repo(&self, _core: &Core, repo: &core::Repo) -> Result<(), core::Error> {
        git::git_checkout(&repo.get_path(), self.branch).await
    }

    #[tracing::instrument(name = "task:git_checkout(scratchpad)", err, skip(self, _core))]
    async fn apply_scratchpad(
        &self,
        _core: &Core,
        _scratch: &core::Scratchpad,
    ) -> Result<(), core::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use crate::tasks::GitInit;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_repo() {
        let temp = tempdir().unwrap();
        let repo = core::Repo::new(
            "gh:sierrasoftworks/test-git-checkout",
            temp.path().join("repo"),
        );

        let core = core::Core::builder()
            .with_config(&Config::for_dev_directory(temp.path()))
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
        let scratch = core::Scratchpad::new("2019w15", temp.path().join("scratch"));

        let core = core::Core::builder()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        let task = GitCheckout { branch: "test" };

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert!(!scratch.exists());
    }
}
