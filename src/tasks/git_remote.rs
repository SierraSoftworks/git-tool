use super::*;
use crate::{errors, core::Target, git};

pub struct GitRemote {
    pub name: String,
}

impl Default for GitRemote {
    fn default() -> Self {
        Self {
            name: "origin".into()
        }
    }
}

#[async_trait::async_trait]
impl<F: FileSource, L: Launcher, R: Resolver> Task<F, L, R> for GitRemote {
    async fn apply_repo(&self, core: &core::Core<F, L, R>, repo: &core::Repo) -> Result<(), core::Error> {
        let service = core.config.get_service(&repo.get_domain()).ok_or(
            errors::user(
                &format!("Could not find a service entry in your config file for {}", repo.get_domain()), 
                &format!("Ensure that your git-tool configuration has a service entry for this service, or add it with `git-tool config add service/{}`", repo.get_domain()))
        )?;

        // TODO: This should support a feature flag for HTTP/Git URL usage
        let url = service.get_git_url(repo)?;

        git::git_remote_add(&repo.get_path(), &self.name, &url).await
    }

    async fn apply_scratchpad(&self, _core: &core::Core<F, L, R>, _scratch: &core::Scratchpad) -> Result<(), core::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::GitInit;
    use tempdir::TempDir;

    #[tokio::test]
    async fn test_repo() {
        let temp = TempDir::new("gt-tasks-remote").unwrap();
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-remote", 
            temp.path().join("repo").into());

        let core = get_core(temp.path());
        let result = sequence![
            GitInit{},
            GitRemote{
                name: "origin".into()
            }
        ].apply_repo(&core, &repo).await;
        assert!(repo.valid());

        std::fs::remove_dir_all(repo.get_path()).unwrap();
        result.unwrap();
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = TempDir::new("gt-tasks-remote").unwrap();
        let scratch = core::Scratchpad::new(
            "2019w15", 
            temp.path().join("scratch").into());

        let core = get_core(temp.path());
        let task = GitRemote{
            name: "origin".into(),
        };

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().join(".git").exists(), false);
        assert_eq!(scratch.exists(), false);
    }

    fn get_core(dir: &std::path::Path) -> core::Core {
        core::Core::builder()
            .with_config(&core::Config::for_dev_directory(dir))
            .build()
    }
}