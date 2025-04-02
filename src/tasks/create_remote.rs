use super::*;
use tracing_batteries::prelude::*;

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
    async fn apply_repo(&self, core: &Core, repo: &engine::Repo) -> Result<(), engine::Error> {
        if !self.enabled {
            return Ok(());
        }

        if !core
            .config()
            .get_features()
            .has(engine::features::CREATE_REMOTE)
        {
            return Ok(());
        }

        let service = core.config().get_service(&repo.service)?;

        if let Some(online_service) = crate::online::services()
            .iter()
            .find(|s| s.handles(service))
        {
            online_service.ensure_created(core, service, repo).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Target;
    use tempfile::tempdir;

    #[tokio::test]
    #[cfg(feature = "auth")]
    async fn test_repo() {
        let temp = tempdir().unwrap();
        let repo = engine::Repo::new(
            "gh:sierrasoftworks/test-git-remote",
            temp.path().join("repo"),
        );

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_mock_http_client(crate::online::service::github::mocks::repo_created(
                "sierrasoftworks",
            ))
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(mockall::predicate::eq("gh"))
                    .returning(|_| Ok("test_token".into()));
            })
            .build();
        CreateRemote { enabled: true }
            .apply_repo(&core, &repo)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = tempdir().unwrap();
        let scratch = engine::Scratchpad::new("2019w15", temp.path().join("scratch"));

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        let task = CreateRemote { enabled: true };

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert!(!scratch.exists());
    }
}
