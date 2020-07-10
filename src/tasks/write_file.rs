use super::{core::Target, *};
use std::path::PathBuf;

pub struct WriteFile<'a> {
    pub path: PathBuf,
    pub content: &'a str,
}

#[async_trait::async_trait]
impl<'a, C: Core> Task<C> for WriteFile<'a> {
    async fn apply_repo(&self, _core: &C, repo: &core::Repo) -> Result<(), core::Error> {
        let path = repo.get_path().join(&self.path);

        tokio::fs::write(path, self.content).await?;

        Ok(())
    }

    async fn apply_scratchpad(
        &self,
        _core: &C,
        scratch: &core::Scratchpad,
    ) -> Result<(), core::Error> {
        let path = scratch.get_path().join(&self.path);

        tokio::fs::write(path, self.content).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_repo_exists() {
        let temp = tempdir().unwrap();
        let repo = core::Repo::new("github.com/sierrasoftworks/test1", temp.path().into());

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
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
        let scratch = core::Scratchpad::new("2019w15", temp.path().into());

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
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
