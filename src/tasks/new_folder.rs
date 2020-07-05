use super::{core::Target, *};

pub struct NewFolder {}

#[async_trait::async_trait]
impl<C: Core> Task<C> for NewFolder {
    async fn apply_repo(&self, _core: &C, repo: &core::Repo) -> Result<(), core::Error> {
        let path = repo.get_path();

        std::fs::create_dir_all(path)?;

        Ok(())
    }

    async fn apply_scratchpad(
        &self,
        _core: &C,
        scratch: &core::Scratchpad,
    ) -> Result<(), core::Error> {
        let path = scratch.get_path();

        std::fs::create_dir_all(path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_repo_exists() {
        let temp = tempdir().unwrap();
        let repo = core::Repo::new("github.com/sierrasoftworks/test1", temp.path().into());

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        let task = NewFolder {};

        assert_eq!(repo.get_path().exists(), true);

        task.apply_repo(&core, &repo).await.unwrap();
        assert_eq!(repo.get_path().exists(), true);
    }

    #[tokio::test]
    async fn test_repo_new() {
        let temp = tempdir().unwrap();
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test3",
            temp.path().join("repo").into(),
        );

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        let task = NewFolder {};

        assert_eq!(repo.get_path().exists(), false);

        task.apply_repo(&core, &repo).await.unwrap();
        assert_eq!(repo.get_path().exists(), true);
    }

    #[tokio::test]
    async fn test_scratch_exists() {
        let temp = tempdir().unwrap();
        let scratch = core::Scratchpad::new("2019w15", temp.path().into());

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        let task = NewFolder {};

        assert_eq!(scratch.get_path().exists(), true);

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().exists(), true);
    }

    #[tokio::test]
    async fn test_scratch_new() {
        let temp = tempdir().unwrap();
        let scratch = core::Scratchpad::new("2019w19", temp.path().join("scratch").into());

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        let task = NewFolder {};

        assert_eq!(scratch.get_path().exists(), false);

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().exists(), true);
    }
}
