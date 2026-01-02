use crate::errors;

use super::{engine::Target, *};
use std::path::{Path, PathBuf};
use tracing_batteries::prelude::*;

#[allow(dead_code)]
pub struct WriteFile<'a> {
    pub path: PathBuf,
    pub content: &'a str,
}

#[async_trait::async_trait]
impl Task for WriteFile<'_> {
    #[tracing::instrument(name = "task:write_file(repo)", err, skip(self, _core))]
    async fn apply_repo(&self, _core: &Core, repo: &engine::Repo) -> Result<(), engine::Error> {
        let path = repo.get_path().join(&self.path);

        self.write_file(&path).await?;

        Ok(())
    }

    #[tracing::instrument(name = "task:write_file(scratchpad)", err, skip(self, _core))]
    async fn apply_scratchpad(
        &self,
        _core: &Core,
        scratch: &engine::Scratchpad,
    ) -> Result<(), engine::Error> {
        let path = scratch.get_path().join(&self.path);

        self.write_file(&path).await?;

        Ok(())
    }
}

#[allow(dead_code)]
impl WriteFile<'_> {
    async fn write_file(&self, path: &Path) -> Result<(), errors::Error> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?
        };

        tokio::fs::write(&path, self.content).await.map_err(|err| {
            errors::user_with_internal(
                format!(
                    "Could not write data to the file '{}' due to an OS-level error.",
                    path.display()
                ),
                "Check that Git-Tool has permission to create and write to this file and that the parent directory exists.",
                err,
            )
        })
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

        let task = WriteFile {
            path: PathBuf::from("README.md"),
            content: "This is an example",
        };

        assert!(repo.exists(), "the repo should exist initially");

        task.apply_repo(&core, &repo).await.unwrap();
        assert!(repo.exists(), "the repo should still exist");
        assert!(
            repo.get_path().join("README.md").exists(),
            "the file should exist"
        );

        let content = tokio::fs::read_to_string(repo.get_path().join("README.md"))
            .await
            .expect("the file should be readable");
        assert_eq!(
            content, "This is an example",
            "the file should have the correct content"
        );
    }

    #[tokio::test]
    async fn test_scratch_exists() {
        let temp = tempdir().unwrap();
        let scratch = Scratchpad::new("2019w15", temp.path().into());

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        let task = WriteFile {
            path: PathBuf::from("README.md"),
            content: "This is an example",
        };

        assert!(
            scratch.get_path().exists(),
            "the scratchpad should exist initially"
        );

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert!(
            scratch.get_path().exists(),
            "the scratchpad should still exist"
        );
        assert!(
            scratch.get_path().join("README.md").exists(),
            "the file should exist"
        );

        let content = tokio::fs::read_to_string(scratch.get_path().join("README.md"))
            .await
            .expect("the file should be readable");
        assert_eq!(
            content, "This is an example",
            "the file should have the correct content"
        );
    }
}
