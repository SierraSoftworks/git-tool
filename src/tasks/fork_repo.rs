use super::*;
use crate::engine::Repo;
use tracing_batteries::prelude::*;

pub struct ForkRepository {
    pub from_repo: Repo,
}

#[async_trait::async_trait]
impl Task for ForkRepository {
    #[cfg(feature = "auth")]
    #[tracing::instrument(name = "task:fork_repository(repo)", err, skip(self, core))]
    async fn apply_repo(&self, core: &Core, repo: &Repo) -> Result<(), engine::Error> {
        let service = core.config().get_service(&repo.service)?;

        // Forking a repository can come in two forms:
        // 1. Using a supported Online Service Attempt to perform a fork/template instantiation using the API.
        // 2. Clone the original repository in the new directory and update the origin URL
        if core
            .config()
            .get_features()
            .has(engine::features::MOVE_REMOTE)
        {
            if let Some(online_service) = crate::online::services()
                .iter()
                .find(|s| s.handles(service))
            {
                online_service
                    .fork_repo(core, service, &self.from_repo, repo)
                    .await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::{Core, Identifier, Repo};
    use crate::tasks::{ForkRepository, Task};
    use mockall::predicate::eq;
    use rstest::rstest;
    use tempfile::tempdir;

    #[rstest]
    #[cfg(feature = "auth")]
    #[case(
        "gh:git-fixtures/basic",
        "git-fixtures/basic",
        "gh:cedi/basic",
        "cedi/basic"
    )]
    #[case(
        "gh:git-fixtures/basic",
        "git-fixtures/basic",
        "gh:SierraSoftworks/basic",
        "SierraSoftworks/basic"
    )]
    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn fork_repo(
        #[case] source_repo: &str,
        #[case] source: &str,
        #[case] target_repo: &str,
        #[case] target: &str,
    ) {
        let temp = tempdir().unwrap();
        let temp_path = temp.path().to_path_buf();

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_mock_http_client(crate::online::service::github::mocks::repo_fork(source))
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(eq("gh"))
                    .returning(|_| Ok("test_token".into()));
            })
            .with_mock_resolver(|mock| {
                let source_temp_path = temp_path.clone();
                let source = source.to_owned();
                let source_segments = source.split('/');
                let full_source_path = source_segments
                    .fold(source_temp_path.clone(), |path, segment| path.join(segment));
                let source_identifier: Identifier = source_repo.parse().unwrap();
                mock.expect_get_best_repo()
                    .with(eq(source_identifier))
                    .times(1)
                    .returning(move |_| {
                        Ok(Repo::new("gh:git-fixtures/basic", full_source_path.clone()))
                    });

                let target_temp_path = temp_path.clone();
                let target = target.to_owned();
                let target_segments = target.split('/');
                let full_target_path = target_segments
                    .fold(target_temp_path.clone(), |path, segment| path.join(segment));
                let target_identifier: Identifier = target_repo.parse().unwrap();
                mock.expect_get_best_repo()
                    .with(eq(target_identifier))
                    .times(1)
                    .returning(move |_| {
                        Ok(Repo::new("gh:git-fixtures/empty", full_target_path.clone()))
                    });
            })
            .build();

        let from_repo = core
            .resolver()
            .get_best_repo(&source_repo.parse().unwrap())
            .unwrap();
        let target_repo = core
            .resolver()
            .get_best_repo(&target_repo.parse().unwrap())
            .unwrap();

        ForkRepository { from_repo }
            .apply_repo(&core, &target_repo)
            .await
            .unwrap();
    }
}
