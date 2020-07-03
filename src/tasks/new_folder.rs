use super::{*, core::Target};

pub struct NewFolder {}

#[async_trait::async_trait]
impl<K: KeyChain, L: Launcher, R: Resolver, O: Output> Task<K, L, R, O> for NewFolder {
    async fn apply_repo(&self, _core: &core::Core<K, L, R, O>, repo: &core::Repo) -> Result<(), core::Error> {
        let path = repo.get_path();

        std::fs::create_dir_all(path)?;

        Ok(())
    }

    async fn apply_scratchpad(&self, _core: &core::Core<K, L, R, O>, scratch: &core::Scratchpad) -> Result<(), core::Error> {
        let path = scratch.get_path();

        std::fs::create_dir_all(path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[tokio::test]
    async fn test_repo_exists() {
        let temp = TempDir::new("gt-tasks-new-folder").unwrap();
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test1", 
            temp.path().into());

        let core = get_core(temp.path());
        let task = NewFolder{};
        
        assert_eq!(repo.get_path().exists(), true);

        task.apply_repo(&core, &repo).await.unwrap();
        assert_eq!(repo.get_path().exists(), true);
    }

    #[tokio::test]
    async fn test_repo_new() {
        let temp = TempDir::new("gt-tasks-new-folder").unwrap();
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test3", 
            temp.path().join("repo").into());

        let core = get_core(temp.path());
        let task = NewFolder{};
        
        assert_eq!(repo.get_path().exists(), false);

        task.apply_repo(&core, &repo).await.unwrap();
        assert_eq!(repo.get_path().exists(), true);
    }

    #[tokio::test]
    async fn test_scratch_exists() {
        let temp = TempDir::new("gt-tasks-new-folder").unwrap();
        let scratch = core::Scratchpad::new(
            "2019w15", 
            temp.path().into());

        let core = get_core(temp.path());
        let task = NewFolder{};
        
        assert_eq!(scratch.get_path().exists(), true);

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().exists(), true);
    }

    #[tokio::test]
    async fn test_scratch_new() {
        let temp = TempDir::new("gt-tasks-new-folder").unwrap();
        let scratch = core::Scratchpad::new(
            "2019w19", 
            temp.path().join("scratch").into());

        let core = get_core(temp.path());
        let task = NewFolder{};
        
        assert_eq!(scratch.get_path().exists(), false);

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().exists(), true);
    }

    fn get_core(dir: &std::path::Path) -> core::Core {
        core::Core::builder()
            .with_config(&core::Config::for_dev_directory(dir))
            .build()
    }
}