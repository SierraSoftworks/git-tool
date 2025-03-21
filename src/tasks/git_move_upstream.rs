use crate::core::{Core, Repo, Target};
use crate::errors;
use crate::git::{git_remote_get_url, git_remote_list, git_remote_set_url};
use crate::tasks::Task;
use tracing_batteries::prelude::tracing;

pub struct GitMoveUpstream {
    pub new_repo: Repo,
}

#[async_trait::async_trait]
impl Task for GitMoveUpstream {
    #[tracing::instrument(name = "task:git_fix_upstream(repo)", err, skip(self, core))]
    async fn apply_repo(
        &self,
        core: &Core,
        repo: &crate::core::Repo,
    ) -> Result<(), crate::core::Error> {
        if !repo.exists() {
            return Err(errors::user(
                    &format!("The repository '{}' does not exist on your machine and cannot be moved as a result.", repo.name),
                    &format!("Make sure the name is correct and that the repository exists by running `git-tool clone {}` first.", repo.name),
                ));
        }

        let config = core.config();
        let service = config.get_service(&repo.service).ok_or_else(|| {
                errors::user(
                    &format!("Could not find a service entry in your config file for '{}'", repo.service),
                    &format!("Ensure that your git-tool configuration has a service entry for this service, or add it with `git-tool config add service/{}`", repo.service),
                )
            })?;

        let expected_url = service.get_git_url(repo)?;
        let new_url = service.get_git_url(&self.new_repo)?;

        let remotes = git_remote_list(&repo.path).await?;
        for remote in &remotes {
            let urls = git_remote_get_url(&repo.path, remote).await?;
            if urls.iter().any(|url| url == &expected_url) {
                git_remote_set_url(&repo.get_path(), remote, &new_url).await?;
            }
        }

        Ok(())
    }

    #[tracing::instrument(name = "task:git_rename(scratchpad)", err, skip(self, _core))]
    async fn apply_scratchpad(
        &self,
        _core: &Core,
        _scratch: &crate::core::Scratchpad,
    ) -> Result<(), crate::core::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Core;
    use crate::git::{git_remote_add, git_remote_rename};
    use crate::tasks::GitClone;
    use tempfile::tempdir;

    #[tokio::test]
    async fn move_upstream_errors_if_repo_does_not_exist() {
        let temp = tempdir().unwrap();
        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_null_console()
            .build();

        let repo = core
            .resolver()
            .get_best_repo("gh:git-fixtures/basic")
            .unwrap();

        let new_repo = core
            .resolver()
            .get_best_repo("gh:git-fixtures/renamed")
            .unwrap();

        let task = GitMoveUpstream {
            new_repo: new_repo.clone(),
        };

        let result = task.apply_repo(&core, &repo).await;
        assert!(result.is_err());
        let err_msg = format!("{result:?}");
        assert!(
            err_msg.contains("does not exist"),
            "Unexpected error message: {err_msg}"
        );
    }

    #[tokio::test]
    async fn move_upstream_updates_singular_remote() {
        let temp = tempdir().unwrap();
        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_null_console()
            .build();

        let repo = core
            .resolver()
            .get_best_repo("gh:git-fixtures/basic")
            .unwrap();

        let new_repo = core
            .resolver()
            .get_best_repo("gh:git-fixtures/renamed")
            .unwrap();

        GitClone {}.apply_repo(&core, &repo).await.unwrap();

        let task = GitMoveUpstream {
            new_repo: new_repo.clone(),
        };

        let result = task.apply_repo(&core, &repo).await;

        assert!(result.is_ok());

        let remotes = crate::git::git_remote_list(&repo.path).await.unwrap();
        for remote in remotes {
            let urls = crate::git::git_remote_get_url(&repo.path, &remote)
                .await
                .unwrap();
            assert!(
                urls.iter().any(|u| u.contains("git-fixtures/renamed")),
                "Remote was not updated to point to new repo"
            );
        }
    }

    #[tokio::test]
    async fn move_upstream_updates_forked_remote() {
        let temp = tempdir().unwrap();
        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_null_console()
            .build();

        let repo = core
            .resolver()
            .get_best_repo("gh:git-fixtures/basic")
            .unwrap();

        let new_repo = core
            .resolver()
            .get_best_repo("gh:git-fixtures/renamed")
            .unwrap();

        GitClone {}.apply_repo(&core, &repo).await.unwrap();

        // Simulate having a (forked) repository where the origin is our fork, and the original remote
        // is in upstream
        assert!(git_remote_rename(&repo.path, "origin", "upstream")
            .await
            .is_ok());
        assert!(git_remote_add(
            &repo.path,
            "origin",
            "git@github.com:sierrasoftworks/basic-git-fixtures.git"
        )
        .await
        .is_ok());

        let task = GitMoveUpstream {
            new_repo: new_repo.clone(),
        };

        let result = task.apply_repo(&core, &repo).await;

        assert!(result.is_ok());

        let urls = crate::git::git_remote_get_url(&repo.path, "origin")
            .await
            .unwrap();
        assert!(
            urls.iter()
                .any(|u| u.contains("sierrasoftworks/basic-git-fixtures")),
            "origin shall point to our forked repository URL"
        );

        let urls = crate::git::git_remote_get_url(&repo.path, "upstream")
            .await
            .unwrap();
        assert!(
            urls.iter().any(|u| u.contains("git-fixtures/renamed")),
            "upstream shall point to our updated repository URL"
        );
    }
}
