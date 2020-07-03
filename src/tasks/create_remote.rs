use super::*;
use crate::errors;

pub struct CreateRemote {}

impl Default for CreateRemote {
    fn default() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl<C: Core> Task<C> for CreateRemote {
    async fn apply_repo(
        &self,
        core: &C,
        repo: &core::Repo,
    ) -> Result<(), core::Error> {
        if !core.config().get_features().create_remote() {
            return Ok(());
        }

        let service = core.config().get_service(&repo.get_domain()).ok_or(
            errors::user(
                &format!("Could not find a service entry in your config file for {}", repo.get_domain()), 
                &format!("Ensure that your git-tool configuration has a service entry for this service, or add it with `git-tool config add service/{}`", repo.get_domain()))
        )?;

        if let Some(online_service) = crate::online::services()
            .iter()
            .find(|s| s.handles(service))
        {
            online_service.ensure_created(core, repo).await?;
        }

        Ok(())
    }

    async fn apply_scratchpad(
        &self,
        _core: &C,
        _scratch: &core::Scratchpad,
    ) -> Result<(), core::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Target, KeyChain, Config};
    use tempdir::TempDir;

    #[tokio::test]
    async fn test_repo() {
        let temp = TempDir::new("gt-tasks-create-remote").unwrap();
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-remote",
            temp.path().join("repo").into(),
        );

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
            .with_mock_keychain(|s| {
                s.set_token("github.com", "test_token").unwrap();
            })
            .build();
        CreateRemote {}.apply_repo(&core, &repo).await.unwrap();
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = TempDir::new("gt-tasks-create-remote").unwrap();
        let scratch = core::Scratchpad::new("2019w15", temp.path().join("scratch").into());

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
            .with_mock_keychain(|s| {
                s.set_token("github.com", "test_token").unwrap();
            })
            .build();

        let task = CreateRemote {};

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().join(".git").exists(), false);
        assert_eq!(scratch.exists(), false);
    }
}
