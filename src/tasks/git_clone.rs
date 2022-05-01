use super::*;
use crate::{core::Target, errors, git};

pub struct GitClone {}

#[async_trait::async_trait]
impl Task for GitClone {
    #[tracing::instrument(name = "task:git_clone(repo)", err, skip(self, core))]
    async fn apply_repo(&self, core: &Core, repo: &core::Repo) -> Result<(), core::Error> {
        if repo.exists() {
            return Ok(());
        }

        let service = core.config().get_service(&repo.service).ok_or_else(|| errors::user(
                &format!("Could not find a service entry in your config file for {}", repo.service), 
                &format!("Ensure that your git-tool configuration has a service entry for this service, or add it with `git-tool config add service/{}`", repo.service)))?;

        let url = service.get_git_url(repo)?;

        git::git_clone(&repo.get_path(), &url).await
    }

    #[tracing::instrument(name = "task:git_clone(scratchpad)", err, skip(self, _core))]
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
    async fn test_repo_basic() {
        let temp = tempdir().unwrap();
        let repo = core::Repo::new("gh:git-fixtures/basic", temp.path().join("repo"));

        let core = core::Core::builder()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        GitClone {}.apply_repo(&core, &repo).await.unwrap();
        assert!(repo.valid());
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = tempdir().unwrap();
        let scratch = core::Scratchpad::new("2019w15", temp.path().join("scratch"));

        let core = core::Core::builder()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        let task = GitClone {};

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert!(!scratch.exists());
    }
}
