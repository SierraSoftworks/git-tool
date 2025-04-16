use super::*;
use crate::{engine::Target, git};
use tracing_batteries::prelude::*;

#[derive(Default)]
pub struct GitClone {
    pub url: Option<String>,
}

#[async_trait::async_trait]
impl Task for GitClone {
    #[tracing::instrument(name = "task:git_clone(repo)", err, skip(self, core))]
    async fn apply_repo(&self, core: &Core, repo: &engine::Repo) -> Result<(), engine::Error> {
        if repo.exists() {
            return Ok(());
        }

        let service = core.config().get_service(&repo.service)?;

        let default_url = service.get_git_url(repo)?;
        let url = self.url.as_deref().unwrap_or(default_url.as_str());

        git::git_clone(&repo.get_path(), url).await?;

        #[cfg(test)]
        {
            git::git_config_set(&repo.get_path(), "user.name", "Example User").await?;
            git::git_config_set(&repo.get_path(), "user.email", "user@example.com").await?;
        }

        Ok(())
    }
}

impl GitClone {
    pub fn with_url<S: ToString>(url: S) -> Self {
        Self {
            url: Some(url.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::*;
    use tempfile::tempdir;

    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn test_repo_basic() {
        let temp = tempdir().unwrap();
        let repo = Repo::new("gh:git-fixtures/basic", temp.path().join("repo"));

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        GitClone::default().apply_repo(&core, &repo).await.unwrap();
        assert!(repo.valid());
    }

    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn test_repo_custom_url() {
        let temp = tempdir().unwrap();
        let repo = Repo::new("gh:git-fixtures/basic", temp.path().join("repo2"));

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        GitClone::with_url("https://github.com/git-fixtures/basic.git")
            .apply_repo(&core, &repo)
            .await
            .unwrap();
        assert!(repo.valid());
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = tempdir().unwrap();
        let scratch = Scratchpad::new("2019w15", temp.path().join("scratch"));

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        let task = GitClone::default();

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert!(!scratch.exists());
    }
}
