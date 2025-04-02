use super::*;
use crate::core::Repo;
use tracing_batteries::prelude::*;

pub struct MoveRemote {
    pub enabled: bool,
    pub target: Repo,
}

#[async_trait::async_trait]
impl Task for MoveRemote {
    #[cfg(feature = "auth")]
    #[tracing::instrument(name = "task:move_remote(repo)", err, skip(self, core))]
    async fn apply_repo(&self, core: &Core, repo: &core::Repo) -> Result<(), core::Error> {
        if !self.enabled {
            return Ok(());
        }

        if !core
            .config()
            .get_features()
            .has(core::features::MOVE_REMOTE)
        {
            return Ok(());
        }

        if repo.service != self.target.service {
            info!("Skipping move of remote repository as the target service '{}' does not match the source '{}'", self.target.service, repo.service);
            return Ok(());
        }

        let service = core.config().get_service(&repo.service)?;

        if let Some(online_service) = crate::online::services()
            .iter()
            .find(|s| s.handles(service))
        {
            online_service
                .move_repo(core, service, repo, &self.target)
                .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::move_remote::MoveRemote;
    use crate::tasks::Task;
    use tempfile::tempdir;

    #[tokio::test]
    #[cfg(feature = "auth")]
    async fn test_repo() {
        let temp = tempdir().unwrap();
        let src_repo = core::Repo::new("gh:test/old-name", temp.path().join("repo"));
        let dest_repo = core::Repo::new("gh:test/new-name", temp.path().join("repo"));

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_mock_http_client(crate::online::service::github::mocks::repo_update_name(
                "test/old-name",
            ))
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(mockall::predicate::eq("gh"))
                    .returning(|_| Ok("test_token".into()));
            })
            .build();
        MoveRemote {
            enabled: true,
            target: dest_repo,
        }
        .apply_repo(&core, &src_repo)
        .await
        .unwrap();
    }
}
