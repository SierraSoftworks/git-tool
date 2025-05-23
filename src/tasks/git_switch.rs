use super::*;
use crate::{engine::Target, git};
use tracing_batteries::prelude::*;

pub struct GitSwitch {
    pub branch: String,
    pub create_if_missing: bool,
}

#[async_trait::async_trait]
impl Task for GitSwitch {
    #[tracing::instrument(name = "task:git_switch(repo)", err, skip(self, _core))]
    async fn apply_repo(&self, _core: &Core, repo: &engine::Repo) -> Result<(), engine::Error> {
        let mut create = self.create_if_missing;

        if create
            && git::git_branches(&repo.get_path())
                .await?
                .iter()
                .any(|v| v == &self.branch || v == &format!("origin/{}", &self.branch))
        {
            create = false;
        }

        git::git_switch(&repo.get_path(), &self.branch, create).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_repo() {
        let temp = tempdir().unwrap();
        let repo = engine::Repo::new(
            "gh:sierrasoftworks/test-git-switch",
            temp.path().join("repo"),
        );

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_null_console()
            .build();

        sequence![
            GitInit {},
            GitCheckout { branch: "main" },
            GitSwitch {
                branch: "test".into(),
                create_if_missing: true,
            }
        ]
        .apply_repo(&core, &repo)
        .await
        .unwrap();
        assert!(repo.valid());

        assert_eq!(
            git::git_current_branch(&repo.get_path()).await.unwrap(),
            "test"
        );
    }

    #[tokio::test]
    async fn test_repo_no_create() {
        let temp = tempdir().unwrap();
        let repo = engine::Repo::new(
            "gh:sierrasoftworks/test-git-switch",
            temp.path().join("repo"),
        );

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_null_console()
            .build();

        sequence![
            GitInit {},
            GitCheckout { branch: "main" },
            GitSwitch {
                branch: "test".into(),
                create_if_missing: false,
            }
        ]
        .apply_repo(&core, &repo)
        .await
        .expect_err("this command should fail");
        assert!(repo.valid());

        assert_eq!(
            git::git_current_branch(&repo.get_path()).await.unwrap(),
            "main"
        );
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = tempdir().unwrap();
        let scratch = engine::Scratchpad::new("2019w15", temp.path().join("scratch"));

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_null_console()
            .build();

        let task = GitSwitch {
            branch: "test".into(),
            create_if_missing: true,
        };

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert!(!scratch.exists());
    }
}
