use super::*;
use crate::{core::Target, git};

pub struct GitCheckout {
    pub branch: String,
}

#[async_trait::async_trait]
impl<K: KeyChain, L: Launcher, R: Resolver, O: Output> Task<K, L, R, O> for GitCheckout {
    async fn apply_repo(&self, _core: &core::Core<K, L, R, O>, repo: &core::Repo) -> Result<(), core::Error> {
        git::git_checkout(&repo.get_path(), &self.branch).await
    }

    async fn apply_scratchpad(&self, _core: &core::Core<K, L, R, O>, _scratch: &core::Scratchpad) -> Result<(), core::Error> {
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
        let temp = TempDir::new("gt-tasks-checkout").unwrap();
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-checkout", 
            temp.path().join("repo").into());

        let core = get_core(temp.path());
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

        let core = get_core(temp.path());
        let task = GitCheckout{
            branch: "test".into(),
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