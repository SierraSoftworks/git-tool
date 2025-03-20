use super::*;
use crate::{core::Target, errors, git};
use tracing_batteries::prelude::*;

pub struct GitRemote<'a> {
    pub name: &'a str,
}

impl Default for GitRemote<'static> {
    fn default() -> Self {
        Self { name: "origin" }
    }
}

#[async_trait::async_trait]
impl<'a> Task for GitRemote<'a> {
    #[tracing::instrument(name = "task:git_remote(repo)", err, skip(self, core))]
    async fn apply_repo(&self, core: &Core, repo: &core::Repo) -> Result<(), core::Error> {
        let service = core.config().get_service(&repo.service).ok_or_else(|| errors::user(
                &format!("Could not find a service entry in your config file for {}", &repo.service), 
                &format!("Ensure that your git-tool configuration has a service entry for this service, or add it with `git-tool config add service/{}`", repo.service)))?;

        let url = service.get_git_url(repo)?;

        if git::git_remote_list(&repo.get_path())
            .await?
            .iter()
            .any(|r| r == self.name)
        {
            git::git_remote_set_url(&repo.get_path(), self.name, &url).await
        } else {
            git::git_remote_add(&repo.get_path(), self.name, &url).await
        }
    }

    #[tracing::instrument(name = "task:git_remote(scratchpad)", err, skip(self, _core))]
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
        let repo = Repo::new(
            "gh:sierrasoftworks/test-git-remote",
            temp.path().join("repo"),
        );

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        sequence![GitInit {}, GitRemote { name: "origin" }]
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

        let task = GitRemote { name: "origin" };

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert!(!scratch.exists());
    }
}
