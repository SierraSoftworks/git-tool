use super::*;
use crate::engine::Repo;
use tracing_batteries::prelude::*;

pub struct ForkRepository {
    pub from_repo: Repo,
    pub no_create_remote: bool,
}

#[async_trait::async_trait]
impl Task for ForkRepository {
    #[cfg(feature = "auth")]
    #[tracing::instrument(name = "task:fork_repository(repo)", err, skip(self, core))]
    async fn apply_repo(&self, core: &Core, repo: &Repo) -> Result<(), engine::Error> {
        let service = core.config().get_service(&repo.service)?;
        let from_service = core.config().get_service(&self.from_repo.service)?;
        let url = service.get_git_url(repo)?;
        let from_url = from_service.get_git_url(&self.from_repo)?;
        let mut supported_service = false;

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
                supported_service = true;
                online_service
                    .fork_repo(core, service, &self.from_repo, repo)
                    .await?;
            }
        }

        let clone_path = if supported_service {
            None
        } else {
            Some(repo.path.to_str().unwrap().to_string())
        };

        let tasks = sequence![
            GitClone::with_path(clone_path),
            GitAddRemote {
                name: "origin".into(),
                url,
            },
            GitAddRemote {
                name: "upstream".into(),
                url: from_url,
            },
            CreateRemote {
                enabled: !self.no_create_remote
            }
        ];

        let task_target_repo = if supported_service {
            repo
        } else {
            &self.from_repo
        };
        tasks.apply_repo(core, task_target_repo).await?;

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
    #[tokio::test]
    #[cfg(feature = "auth")]
    #[case("gh:git-fixtures/basic", "git-fixtures/basic", "gh:cedi/basic")]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn fork_repo(#[case] source_repo: &str, #[case] source: &str, #[case] target_repo: &str) {
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
                let source = source_repo.to_owned();
                let source_identifier: Identifier = source.parse().unwrap();
                mock.expect_get_best_repo()
                    .with(eq(source_identifier))
                    .times(1)
                    .returning(move |_| {
                        Ok(Repo::new(
                            "gh:git-fixtures/basic",
                            source_temp_path.join(&source),
                        ))
                    });

                let target_temp_path = temp_path.clone();
                let target = target_repo.to_owned();
                let target_identifier: Identifier = target.parse().unwrap();
                mock.expect_get_best_repo()
                    .with(eq(target_identifier))
                    .times(1)
                    .returning(move |_| {
                        Ok(Repo::new(
                            "gh:git-fixtures/empty",
                            target_temp_path.join(&target),
                        ))
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

        ForkRepository {
            from_repo,
            no_create_remote: true,
        }
        .apply_repo(&core, &target_repo)
        .await
        .unwrap();
    }
}
