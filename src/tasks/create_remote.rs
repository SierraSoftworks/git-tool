use super::*;

pub struct CreateRemote {
    pub enabled: bool,
}

impl Default for CreateRemote {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[async_trait::async_trait]
impl Task for CreateRemote {
    #[cfg(feature = "auth")]
    #[tracing::instrument(name = "task:create_remote(repo)", err, skip(self, core))]
    async fn apply_repo(&self, core: &Core, repo: &core::Repo) -> Result<(), core::Error> {
        if !self.enabled {
            return Ok(());
        }

        if !core
            .config()
            .get_features()
            .has(core::features::CREATE_REMOTE)
        {
            return Ok(());
        }

        let service = core.config().get_service(&repo.service).ok_or(
            crate::errors::user(
                &format!("Could not find a service entry in your config file for {}", &repo.service), 
                &format!("Ensure that your git-tool configuration has a service entry for this service, or add it with `git-tool config add service/{}`", &repo.service))
        )?;

        if let Some(online_service) = crate::online::services()
            .iter()
            .find(|s| s.handles(service))
        {
            online_service.ensure_created(core, service, repo).await?;
        }

        Ok(())
    }

    #[cfg(not(feature = "auth"))]
    #[tracing::instrument(name = "task:create_remote(repo)", err, skip(self, _core))]
    async fn apply_repo(&self, _core: &Core, _repo: &core::Repo) -> Result<(), core::Error> {
        Ok(())
    }

    #[tracing::instrument(name = "task:create_remote(scratchpad)", err, skip(self, _core))]
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
    use crate::core::{Config, KeyChain, Target};
    use mocktopus::mocking::*;
    use tempfile::tempdir;

    #[tokio::test]
    #[cfg(feature = "auth")]
    async fn test_repo() {
        let temp = tempdir().unwrap();
        let repo = core::Repo::new(
            "gh:sierrasoftworks/test-git-remote",
            temp.path().join("repo").into(),
        );

        KeyChain::get_token.mock_safe(|_, token| {
            assert_eq!(token, "gh", "the correct token should be requested");
            MockResult::Return(Ok("test_token".into()))
        });

        crate::online::service::github::mocks::repo_created("sierrasoftworks");

        let core = core::Core::builder()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();
        CreateRemote { enabled: true }
            .apply_repo(&core, &repo)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = tempdir().unwrap();
        let scratch = core::Scratchpad::new("2019w15", temp.path().join("scratch").into());

        let core = core::Core::builder()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        let task = CreateRemote { enabled: true };

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().join(".git").exists(), false);
        assert_eq!(scratch.exists(), false);
    }
}
