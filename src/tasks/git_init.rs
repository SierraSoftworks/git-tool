use super::*;
use crate::{core::Target, git};

pub struct GitInit {}

#[async_trait::async_trait]
impl Task for GitInit {
    async fn apply_repo(&self, _core: &Core, repo: &core::Repo) -> Result<(), core::Error> {
        git::git_init(&repo.get_path()).await
    }

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
    use crate::core;
    use crate::core::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_repo() {
        let temp = tempdir().unwrap();
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-init",
            temp.path().join("repo").into(),
        );

        let core = core::Core::builder()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();
        let task = GitInit {};

        task.apply_repo(&core, &repo).await.unwrap();
        assert!(repo.valid());
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = tempdir().unwrap();
        let scratch = core::Scratchpad::new("2019w15", temp.path().join("scratch").into());

        let core = core::Core::builder()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();
        let task = GitInit {};

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().join(".git").exists(), false);
        assert_eq!(scratch.exists(), false);
    }
}
