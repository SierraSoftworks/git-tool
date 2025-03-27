use super::*;
use crate::{core::Target, git};
use tracing_batteries::prelude::*;

pub struct GitAdd<'a> {
    pub paths: Vec<&'a str>,
}

#[async_trait::async_trait]
impl Task for GitAdd<'_> {
    #[tracing::instrument(name = "task:git_add(repo)", err, skip(self, _core))]
    async fn apply_repo(&self, _core: &Core, repo: &core::Repo) -> Result<(), core::Error> {
        git::git_add(&repo.get_path(), &self.paths).await
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
        let repo = Repo::new("gh:git-fixtures/basic", temp.path().into());

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
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
        let scratch = Scratchpad::new("2019w15", temp.path().join("scratch"));

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        let task = GitAdd { paths: vec![] };

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert!(!scratch.exists());
    }
}
