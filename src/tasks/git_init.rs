use super::*;
use crate::{core::Target, git};
use tracing_batteries::prelude::*;

pub struct GitInit {}

#[async_trait::async_trait]
impl Task for GitInit {
    #[tracing::instrument(name = "task:git_init(repo)", err, skip(self, _core))]
    async fn apply_repo(&self, _core: &Core, repo: &core::Repo) -> Result<(), core::Error> {
        git::git_init(&repo.get_path()).await?;

        #[cfg(test)]
        {
            git::git_config_set(&repo.get_path(), "user.name", "Example User").await?;
            git::git_config_set(&repo.get_path(), "user.email", "user@example.com").await?;
        }

        Ok(())
    }

    #[tracing::instrument(name = "task:git_init(scratchpad)", err, skip(self, _core))]
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
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_repo() {
        let temp = tempdir().unwrap();
        let repo = core::Repo::new("gh:sierrasoftworks/test-git-init", temp.path().join("repo"));

        let core = core::Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();
        let task = GitInit {};

        task.apply_repo(&core, &repo).await.unwrap();
        assert!(repo.valid());
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = tempdir().unwrap();
        let scratch = core::Scratchpad::new("2019w15", temp.path().join("scratch"));

        let core = core::Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();
        let task = GitInit {};

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert!(!scratch.exists());
    }
}
