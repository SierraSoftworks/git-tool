use super::*;
use tracing_batteries::prelude::*;

pub struct EnsureNoRemote {
    pub enabled: bool,
}

impl Default for EnsureNoRemote {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[async_trait::async_trait]
impl Task for EnsureNoRemote {
    #[cfg(feature = "auth")]
    #[tracing::instrument(name = "task:ensure_no_remote(repo)", err, skip(self, core))]
    async fn apply_repo(&self, core: &Core, repo: &engine::Repo) -> Result<(), engine::Error> {
        if !self.enabled {
            return Ok(());
        }

        if !core
            .config()
            .get_features()
            .has(engine::features::CHECK_EXISTS)
        {
            return Ok(());
        }

        let service = core.config().get_service(&repo.service)?;

        if let Some(online_service) = crate::online::services()
            .iter()
            .find(|s| s.handles(service))
        {
            if online_service.is_created(core, service, repo).await? {
                return Err(human_errors::user(
                    format!(
                        "The remote repository {} already exists. If you want to open this repository, you can clone it locally with `git-tool open {}:{}`",
                        repo.get_full_name(),
                        &repo.service,
                        repo.get_full_name()
                    ),
                    &["Check if you meant to clone an existing repository instead of creating a new one."],
                ));
            }
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
    async fn test_repo_exists() {
        let temp = tempdir().unwrap();
        let repo = engine::Repo::new(
            "gh:sierrasoftworks/test-git-remote",
            temp.path().join("repo"),
        );

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_mock_http_client(crate::online::service::github::mocks::get_repo_exists(
                "sierrasoftworks/test-git-remote",
            ))
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(mockall::predicate::eq("gh"))
                    .times(1)
                    .returning(|_| Ok("test_token".into()));
            })
            .build();
        EnsureNoRemote { enabled: true }
            .apply_repo(&core, &repo)
            .await
            .expect_err("Expected an error to be returned");
    }

    #[tokio::test]
    #[cfg(feature = "auth")]
    async fn test_repo_not_exists() {
        let temp = tempdir().unwrap();
        let repo = engine::Repo::new(
            "gh:sierrasoftworks/test-git-remote",
            temp.path().join("repo"),
        );

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_mock_http_client(crate::online::service::github::mocks::get_repo_not_exists(
                "sierrasoftworks/test-git-remote",
            ))
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(mockall::predicate::eq("gh"))
                    .times(1)
                    .returning(|_| Ok("test_token".into()));
            })
            .build();
        EnsureNoRemote { enabled: true }
            .apply_repo(&core, &repo)
            .await
            .expect("Expected no error to be returned");
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = tempdir().unwrap();
        let scratch = engine::Scratchpad::new("2019w15", temp.path().join("scratch"));

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        let task = EnsureNoRemote { enabled: true };

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert!(!scratch.exists());
    }
}
