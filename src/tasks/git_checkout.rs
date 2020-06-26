use super::*;
use crate::{core::Target, git};

pub struct GitCheckout {
    pub branch: String,
}

#[async_trait::async_trait]
impl<F: FileSource, L: Launcher, R: Resolver> Task<F, L, R> for GitCheckout {
    async fn apply_repo(&self, _core: &core::Core<F, L, R>, repo: &core::Repo) -> Result<(), core::Error> {
        git::git_checkout(&repo.get_path(), &self.branch).await
    }

    async fn apply_scratchpad(&self, _core: &core::Core<F, L, R>, _scratch: &core::Scratchpad) -> Result<(), core::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tasks::GitInit, test::get_dev_dir};
    use tempdir::TempDir;

    #[tokio::test]
    async fn test_repo() {
        let temp = TempDir::new("gt-tasks-checkout").unwrap();
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-checkout", 
            temp.path().join("repo").into());

        let core = get_core();
        sequence![
            GitInit{},
            GitCheckout{
                branch: "test".into()
            }
        ].apply_repo(&core, &repo).await.unwrap();
        assert!(repo.valid());

        assert_eq!(git::git_current_branch(&repo.get_path()).await.unwrap(), "test");
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = TempDir::new("gt-tasks-checkout").unwrap();
        let scratch = core::Scratchpad::new(
            "2019w15", 
            temp.path().join("scratch").into());

        let core = get_core();
        let task = GitCheckout{
            branch: "test".into(),
        };

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().join(".git").exists(), false);
        assert_eq!(scratch.exists(), false);
    }

    fn get_core() -> core::Core {
        core::Core::builder()
            .with_config(&core::Config::for_dev_directory(get_dev_dir().as_path()))
            .build()
    }
}