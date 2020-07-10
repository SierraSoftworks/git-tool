use super::*;
use crate::{core::Target, git};

pub struct GitAdd<'a> {
    pub paths: Vec<&'a str>,
}

#[async_trait::async_trait]
impl<'a, C: Core> Task<C> for GitAdd<'a> {
    async fn apply_repo(&self, _core: &C, repo: &core::Repo) -> Result<(), core::Error> {
        git::git_add(&repo.get_path(), &self.paths).await
    }

    async fn apply_scratchpad(
        &self,
        _core: &C,
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
        let repo = core::Repo::new("github.com/git-fixtures/basic", temp.path().into());

        let core = core::CoreBuilder::default()
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

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        let task = GitAdd { paths: vec![] };

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().join(".git").exists(), false);
        assert_eq!(scratch.exists(), false);
    }
}
