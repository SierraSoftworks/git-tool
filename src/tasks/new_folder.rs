use super::{core, Task, core::Target};

pub struct NewFolder {}

#[async_trait::async_trait]
impl Task for NewFolder {
    async fn apply_repo(&self, repo: &core::Repo) -> Result<(), core::Error> {
        let path = repo.get_path();

        std::fs::create_dir_all(path)?;

        Ok(())
    }

    async fn apply_scratchpad(&self, scratch: &core::Scratchpad) -> Result<(), core::Error> {
        let path = scratch.get_path();

        std::fs::create_dir_all(path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_repo_exists() {
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test1", 
            get_dev_dir().join("github.com").join("sierrasoftworks").join("test1"));

        let task = NewFolder{};
        
        assert_eq!(repo.get_path().exists(), true);

        task.apply_repo(&repo).await.unwrap();
        assert_eq!(repo.get_path().exists(), true);
    }

    #[tokio::test]
    async fn test_repo_new() {
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test3", 
            get_dev_dir().join("github.com").join("sierrasoftworks").join("test3"));

        let task = NewFolder{};
        
        assert_eq!(repo.get_path().exists(), false);

        task.apply_repo(&repo).await.unwrap();
        assert_eq!(repo.get_path().exists(), true);

        std::fs::remove_dir(repo.get_path()).unwrap();
    }

    #[tokio::test]
    async fn test_scratch_exists() {
        let scratch = core::Scratchpad::new(
            "2019w15", 
            get_dev_dir().join("scratch").join("2019w15"));

        let task = NewFolder{};
        
        assert_eq!(scratch.get_path().exists(), true);

        task.apply_scratchpad(&scratch).await.unwrap();
        assert_eq!(scratch.get_path().exists(), true);
    }

    #[tokio::test]
    async fn test_scratch_new() {
        let scratch = core::Scratchpad::new(
            "2019w19", 
            get_dev_dir().join("scratch").join("2019w19"));

        let task = NewFolder{};
        
        assert_eq!(scratch.get_path().exists(), false);

        task.apply_scratchpad(&scratch).await.unwrap();
        assert_eq!(scratch.get_path().exists(), true);

        std::fs::remove_dir(scratch.get_path()).unwrap();
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