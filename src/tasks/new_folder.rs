use crate::errors;

use super::{engine::Target, *};
use tracing_batteries::prelude::*;

pub struct NewFolder {}

#[async_trait::async_trait]
impl Task for NewFolder {
    #[tracing::instrument(name = "task:new_folder(repo)", err, skip(self, _core))]
    async fn apply_repo(&self, _core: &Core, repo: &engine::Repo) -> Result<(), engine::Error> {
        let path = repo.get_path();

        std::fs::create_dir_all(&path).map_err(|err| {
            human_errors::wrap_user(
                format!(
                    "Could not create the repository directory '{}' due to an OS-level error.",
                    path.display()
                ),
                "Check that Git-Tool has permission to create this directory and any missing parent directories.",
                err,
            )
        })?;

        Ok(())
    }

    #[tracing::instrument(name = "task:new_folder(scratchpad)", err, skip(self, _core))]
    async fn apply_scratchpad(
        &self,
        _core: &Core,
        scratch: &engine::Scratchpad,
    ) -> Result<(), engine::Error> {
        let path = scratch.get_path();

        std::fs::create_dir_all(&path).map_err(|err| {
            human_errors::wrap_user(
                format!(
                    "Could not create the scratchpad directory '{}' due to an OS-level error.",
                    path.display()
                ),
                "Check that Git-Tool has permission to create this directory and any missing parent directories.",
                err,
            )
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_repo_exists() {
        let temp = tempdir().unwrap();
        let repo = Repo::new("gh:sierrasoftworks/test1", temp.path().into());

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        let task = NewFolder {};

        assert!(repo.get_path().exists());

        task.apply_repo(&core, &repo).await.unwrap();
        assert!(repo.get_path().exists());
    }

    #[tokio::test]
    async fn test_repo_new() {
        let temp = tempdir().unwrap();
        let repo = Repo::new("gh:sierrasoftworks/test3", temp.path().join("repo"));

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        let task = NewFolder {};

        assert!(!repo.get_path().exists());

        task.apply_repo(&core, &repo).await.unwrap();
        assert!(repo.get_path().exists());
    }

    #[tokio::test]
    async fn test_scratch_exists() {
        let temp = tempdir().unwrap();
        let scratch = Scratchpad::new("2019w15", temp.path().into());

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        let task = NewFolder {};

        assert!(scratch.get_path().exists());

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert!(scratch.get_path().exists());
    }

    #[tokio::test]
    async fn test_scratch_new() {
        let temp = tempdir().unwrap();
        let scratch = Scratchpad::new("2019w19", temp.path().join("scratch"));

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        let task = NewFolder {};

        assert!(!scratch.get_path().exists());

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert!(scratch.get_path().exists());
    }
}
