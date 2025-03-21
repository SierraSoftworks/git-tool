use super::*;
use crate::{core::Target, errors, git};
use tracing_batteries::prelude::*;

pub struct GitRemote<'a> {
    pub name: &'a str,
    pub host: String,
    pub namespace: String,
    pub is_ssh: bool,
    pub has_dot_git: bool,
}

impl Default for GitRemote<'static> {
    fn default() -> Self {
        Self { name: "origin", host: "".to_string(), namespace: "".to_string(), is_ssh: false, has_dot_git: false }
    }
}

impl GitRemote<'static> {
    pub fn parse(url: &str) -> Option<Self> {
        if url.starts_with("https://") {
            // Example: https://github.com/org/repo or /org/repo.git
            let url = url.strip_prefix("https://")?;
            let mut parts = url.splitn(2, '/');
            let host = parts.next()?.to_string();
            let path = parts.next()?;

            let path_parts = path.split('/').collect::<Vec<_>>();
            if path_parts.len() != 2 {
                return None;
            }

            let namespace = path_parts[0].to_string();
            let repo = path_parts[1].to_string();

            let has_dot_git = repo.ends_with(".git");

            Some(Self {
                name: "origin",
                host,
                namespace,
                is_ssh: false,
                has_dot_git,
            })
        } else if url.starts_with("git@") {
            // Example: git@github.com:org/repo or org/repo.git
            let url = url.strip_prefix("git@")?;
            let mut parts = url.splitn(2, ':');
            let host = parts.next()?.to_string();
            let path = parts.next()?;

            let path_parts = path.split('/').collect::<Vec<_>>();
            if path_parts.len() != 2 {
                return None;
            }

            let namespace = path_parts[0].to_string();
            let repo = path_parts[1].to_string();

            let has_dot_git = repo.ends_with(".git");

            Some(Self {
                name: "origin",
                host,
                namespace,
                is_ssh: true,
                has_dot_git,
            })
        } else {
            None
        }
    }

    pub fn with_repo_name(&self, new_name: &str) -> String {
        let repo = if self.has_dot_git {
            format!("{}.git", new_name)
        } else {
            new_name.to_string()
        };

        if self.is_ssh {
            format!("git@{}:{}/{}", self.host, self.namespace, repo)
        } else {
            format!("https://{}/{}/{}", self.host, self.namespace, repo)
        }
    }
}


#[async_trait::async_trait]
impl<'a> Task for GitRemote<'a> {
    #[tracing::instrument(name = "task:git_remote(repo)", err, skip(self, core))]
    async fn apply_repo(&self, core: &Core, repo: &core::Repo) -> Result<(), core::Error> {
        let service = core.config().get_service(&repo.service).ok_or_else(|| errors::user(
                &format!("Could not find a service entry in your config file for {}", &repo.service), 
                &format!("Ensure that your git-tool configuration has a service entry for this service, or add it with `git-tool config add service/{}`", repo.service)))?;

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

    #[tracing::instrument(name = "task:git_remote(scratchpad)", err, skip(self, _core))]
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
