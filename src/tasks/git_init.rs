use super::*;
use crate::{engine::Target, git};
use tracing_batteries::prelude::*;

pub struct GitInit {}

#[async_trait::async_trait]
impl Task for GitInit {
    #[tracing::instrument(name = "task:git_init(repo)", err, skip(self, _core))]
    async fn apply_repo(&self, _core: &Core, repo: &engine::Repo) -> Result<(), engine::Error> {
        git::git_init(&repo.get_path()).await?;

        #[cfg(test)]
        {
            git::git_config_set(&repo.get_path(), "user.name", "Example User").await?;
            git::git_config_set(&repo.get_path(), "user.email", "user@example.com").await?;
        }

        Ok(())
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
        let repo = Repo::new("gh:sierrasoftworks/test-git-init", temp.path().join("repo"));

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();
        let task = GitInit {};

        task.apply_repo(&core, &repo).await.unwrap();
        assert!(repo.valid());
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = tempdir().unwrap();
        let scratch = Scratchpad::new("2019w15", temp.path().join("scratch"));

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();
        let task = GitInit {};

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert!(!scratch.exists());
    }
}
