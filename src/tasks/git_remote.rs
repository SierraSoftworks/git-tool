use super::{core, Task};
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
impl Task for GitRemote {
    async fn apply_repo(&self, core: &core::Core, repo: &core::Repo) -> Result<(), core::Error> {
        let service = core.config.get_service(&repo.get_domain()).ok_or(
            errors::user(
                &format!("Could not find a service entry in your config file for {}", repo.get_domain()), 
                &format!("Ensure that your git-tool configuration has a service entry for this service, or add it with git-tool config add service/{}", repo.get_domain()))
        )?;

        // TODO: This should support a feature flag for HTTP/Git URL usage
        let url = service.get_git_url(repo)?;

        git::git_remote_add(&repo.get_path(), &self.name, &url).await
    }

    async fn apply_scratchpad(&self, _core: &core::Core, _scratch: &core::Scratchpad) -> Result<(), core::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::GitInit;

    #[tokio::test]
    async fn test_repo() {
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-remote", 
            get_dev_dir().join("github.com").join("sierrasoftworks").join("test-git-remote"));

        let core = get_core();
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
        let scratch = core::Scratchpad::new(
            "2019w15", 
            get_dev_dir().join("scratch").join("2019w15"));

        let core = get_core();
        let task = GitRemote{
            name: "origin".into(),
        };
        assert_eq!(scratch.get_path().exists(), true);

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().join(".git").exists(), false);
    }

    fn get_core() -> core::Core {
        core::Core::builder()
            .with_config(&core::Config::for_dev_directory(get_dev_dir().as_path()))
            .build()
    }

    fn get_dev_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(file!())
            .parent()
            .and_then(|f| f.parent())
            .and_then(|f| f.parent())
            .and_then(|f| Some(f.join("test")))
            .and_then(|f| Some(f.join("devdir")))
            .unwrap()
    }
}