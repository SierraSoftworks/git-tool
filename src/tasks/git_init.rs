use super::*;
use crate::{core::Target, git};

pub struct GitInit { }

#[async_trait::async_trait]
impl<K: KeyChain, L: Launcher, R: Resolver, O: Output> Task<K, L, R, O> for GitInit {
    async fn apply_repo(&self, _core: &core::Core<K, L, R, O>, repo: &core::Repo) -> Result<(), core::Error> {
        git::git_init(&repo.get_path()).await
    }

    async fn apply_scratchpad(&self, _core: &core::Core<K, L, R, O>, _scratch: &core::Scratchpad) -> Result<(), core::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[tokio::test]
    async fn test_repo() {
        let temp = TempDir::new("gt-tasks-init").unwrap();
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-init", 
            temp.path().join("repo").into());

        let core = get_core(temp.path());
        let task = GitInit{};

        task.apply_repo(&core, &repo).await.unwrap();
        assert!(repo.valid());
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = TempDir::new("gt-tasks-init").unwrap();
        let scratch = core::Scratchpad::new(
            "2019w15", 
            temp.path().join("scratch").into());

        let core = get_core(temp.path());
        let task = GitInit{};

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