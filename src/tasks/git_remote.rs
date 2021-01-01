use super::*;
use crate::{
    core::{features, Target},
    errors, git,
};

pub struct GitRemote<'a> {
    pub name: &'a str,
}

impl Default for GitRemote<'static> {
    fn default() -> Self {
        Self { name: "origin" }
    }
}

#[async_trait::async_trait]
impl<'a, C: Core> Task<C> for GitRemote<'a> {
    async fn apply_repo(&self, core: &C, repo: &core::Repo) -> Result<(), core::Error> {
        let service = core.config().get_service(&repo.get_domain()).ok_or(
            errors::user(
                &format!("Could not find a service entry in your config file for {}", repo.get_domain()), 
                &format!("Ensure that your git-tool configuration has a service entry for this service, or add it with `git-tool config add service/{}`", repo.get_domain()))
        )?;

        let url = if core.config().get_features().has(features::HTTP_TRANSPORT) {
            service.get_http_url(repo)?
        } else {
            service.get_git_url(repo)?
        };

        if git::git_remote_list(&repo.get_path())
            .await?
            .iter()
            .any(|r| r == self.name)
        {
            git::git_remote_set_url(&repo.get_path(), &self.name, &url).await
        } else {
            git::git_remote_add(&repo.get_path(), &self.name, &url).await
        }
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
    use crate::core::*;
    use crate::tasks::GitInit;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_repo() {
        let temp = tempdir().unwrap();
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-remote",
            temp.path().join("repo").into(),
        );

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
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
        let scratch = core::Scratchpad::new("2019w15", temp.path().join("scratch").into());

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        let task = GitRemote { name: "origin" };

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().join(".git").exists(), false);
        assert_eq!(scratch.exists(), false);
    }
}
