use super::*;
use crate::{core::Target, git};

pub struct GitCommit<'a> {
    pub message: &'a str,
    pub paths: Vec<&'a str>,
}

#[async_trait::async_trait]
impl<'a> Task for GitCommit<'a> {
    async fn apply_repo(&self, _core: &Core, repo: &core::Repo) -> Result<(), core::Error> {
        git::git_commit(&repo.get_path(), self.message, &self.paths).await
    }

    async fn apply_scratchpad(
        &self,
        _core: &Core,
        _scratch: &core::Scratchpad,
    ) -> Result<(), core::Error> {
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
    async fn test_repo_basic() {
        let temp = tempdir().unwrap();
        let repo = core::Repo::new("gh:git-fixtures/basic", temp.path().into());

        let core = core::Core::builder()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        sequence![
            GitInit {},
            WriteFile {
                path: PathBuf::from("README.md"),
                content: "This is a test"
            },
            GitAdd {
                paths: vec!["README.md"]
            },
            GitCommit {
                message: "Test Commit",
                paths: vec!["README.md"]
            }
        ]
        .apply_repo(&core, &repo)
        .await
        .unwrap();
        assert!(repo.valid());
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = tempdir().unwrap();
        let scratch = core::Scratchpad::new("2019w15", temp.path().join("scratch").into());

        let core = core::Core::builder()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        let task = GitCommit {
            message: "Doesn't Matter",
            paths: vec![],
        };

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().join(".git").exists(), false);
        assert_eq!(scratch.exists(), false);
    }
}
