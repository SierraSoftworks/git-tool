use super::{core, Task};
use crate::{core::Target, git};

pub struct GitInit { }

#[async_trait::async_trait]
impl Task for GitInit {
    async fn apply_repo(&self, _core: &core::Core, repo: &core::Repo) -> Result<(), core::Error> {
        git::git_init(&repo.get_path()).await
    }

    async fn apply_scratchpad(&self, _core: &core::Core, _scratch: &core::Scratchpad) -> Result<(), core::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_repo() {
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-init", 
            get_dev_dir().join("github.com").join("sierrasoftworks").join("test-git-init"));

        let core = get_core();
        let task = GitInit{};

        let result = task.apply_repo(&core, &repo).await;
        assert!(repo.valid());

        let git_exists = repo.get_path().join(".git").exists();
        std::fs::remove_dir_all(repo.get_path()).unwrap();

        result.unwrap();
        assert_eq!(git_exists, true);
    }

    #[tokio::test]
    async fn test_scratch() {
        let scratch = core::Scratchpad::new(
            "2019w15", 
            get_dev_dir().join("scratch").join("2019w15"));

        let core = get_core();
        let task = GitInit{};
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