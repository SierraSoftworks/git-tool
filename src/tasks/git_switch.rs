use super::*;
use crate::{core::Target, git};
use tracing_batteries::prelude::*;

pub struct GitSwitch {
    pub branch: String,
    pub create_if_missing: bool,
}

#[async_trait::async_trait]
impl Task for GitSwitch {
    #[tracing::instrument(name = "task:git_switch(repo)", err, skip(self, _core))]
    async fn apply_repo(&self, _core: &Core, repo: &core::Repo) -> Result<(), core::Error> {
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

    #[tracing::instrument(name = "task:git_switch(scratchpad)", err, skip(self, _core))]
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
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_repo() {
        let temp = tempdir().unwrap();
        let repo = core::Repo::new(
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
        let repo = core::Repo::new(
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
        let scratch = core::Scratchpad::new("2019w15", temp.path().join("scratch"));

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
