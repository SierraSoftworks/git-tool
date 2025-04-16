use super::*;
use crate::{engine::Target, git};
use tracing_batteries::prelude::*;

pub struct GitRemote<'a> {
    pub name: &'a str,
}

impl Default for GitRemote<'static> {
    fn default() -> Self {
        Self { name: "origin" }
    }
}

#[async_trait::async_trait]
impl Task for GitRemote<'_> {
    #[tracing::instrument(name = "task:git_remote(repo)", err, skip(self, core))]
    async fn apply_repo(&self, core: &Core, repo: &engine::Repo) -> Result<(), engine::Error> {
        let service = core.config().get_service(&repo.service)?;

        let url = service.get_git_url(repo)?;

        if git::git_remote_list(&repo.get_path())
            .await?
            .iter()
            .any(|r| r == self.name)
        {
            git::git_remote_set_url(&repo.get_path(), self.name, &url).await
        } else {
            git::git_remote_add(&repo.get_path(), self.name, &url).await
        }
    }
}

pub struct GitAddRemote {
    pub name: String,
    pub url: String,
}

#[async_trait::async_trait]
impl Task for GitAddRemote {
    #[tracing::instrument(name = "task:git_remote(repo)", err, skip(self, _core))]
    async fn apply_repo(&self, _core: &Core, repo: &engine::Repo) -> Result<(), engine::Error> {
        if git::git_remote_list(&repo.get_path())
            .await?
            .iter()
            .any(|r| r == self.name.as_str())
        {
            git::git_remote_set_url(&repo.get_path(), self.name.as_str(), &self.url).await
        } else {
            git::git_remote_add(&repo.get_path(), self.name.as_str(), &self.url).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_repo() {
        let temp = tempdir().unwrap();
        let repo = Repo::new(
            "gh:sierrasoftworks/test-git-remote",
            temp.path().join("repo"),
        );

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        sequence![GitInit {}, GitRemote { name: "origin" }]
            .apply_repo(&core, &repo)
            .await
            .unwrap();
        assert!(repo.valid());

        let remotes = git::git_remote_list(&repo.get_path()).await.unwrap();
        assert!(remotes.iter().any(|r| r == "origin"))
    }

    #[tokio::test]
    async fn test_add_remote() {
        let temp = tempdir().unwrap();
        let repo = Repo::new(
            "gh:sierrasoftworks/test-git-remote",
            temp.path().join("repo"),
        );

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        sequence![
            GitInit {},
            GitRemote { name: "origin" },
            GitAddRemote {
                name: "upstream".to_string(),
                url: "https://github.com/git-fixtures/basic.git".to_string()
            }
        ]
        .apply_repo(&core, &repo)
        .await
        .unwrap();
        assert!(repo.valid());

        let remotes = git::git_remote_list(&repo.get_path()).await.unwrap();
        assert!(remotes.iter().any(|r| r == "origin"));
        assert!(remotes.iter().any(|r| r == "upstream"));
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = tempdir().unwrap();
        let scratch = Scratchpad::new("2019w15", temp.path().join("scratch"));

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        let task = GitRemote { name: "origin" };

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert!(!scratch.exists());
    }
}
