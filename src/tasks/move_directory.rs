use crate::core::{Core, Target};
use crate::errors;
use crate::tasks::Task;
use std::{fs, path};
use tracing_batteries::prelude::tracing;

pub struct MoveDirectory {
    pub new_path: path::PathBuf,
}

#[async_trait::async_trait]
impl Task for MoveDirectory {
    #[tracing::instrument(name = "task:move_directory(repo)", err, skip(self, _core))]
    async fn apply_repo(
        &self,
        _core: &Core,
        repo: &crate::core::Repo,
    ) -> Result<(), crate::core::Error> {
        if !repo.exists() {
            return Err(errors::user(
                format!("The repository {} does not exist on your machine and cannot be moved as a result.", repo.name).as_str(),
                format!("Make sure the name is correct and that the repository exists by running `git-tool clone {}` first.", repo.name).as_str()
            ));
        }

        if let Some(parent) = self.new_path.parent() {
            // Ensure that the parent directory exists for the target folder
            fs::create_dir_all(parent)?;
        }

        fs::rename(repo.path.clone(), self.new_path.clone()).map_err(|err| {
            errors::user_with_internal(
                &format!(
                    "Could not rename the repository directory '{}' to '{}' due to an OS-level error.",
                    repo.path.display(),
                    self.new_path.display()
                ),
                "Check that Git-Tool has permission to create this directory and any missing parent directories.",
                err,
            )
        })?;

        Ok(())
    }

    #[tracing::instrument(name = "task:git_rename(scratchpad)", err, skip(self, _core))]
    async fn apply_scratchpad(
        &self,
        _core: &Core,
        scratch: &crate::core::Scratchpad,
    ) -> Result<(), crate::core::Error> {
        if !scratch.exists() {
            return Err(errors::user(
                format!("The scratchpad {} does not exist on your machine and cannot be moved as a result.", scratch.get_name()).as_str(),
                "Make sure the name is correct and that the scratchpad exists first."
            ));
        }

        fs::rename(scratch.get_path(), self.new_path.clone()).map_err(|err| {
            errors::user_with_internal(
                &format!(
                    "Could not rename the scratchpad directory '{}' to '{}' due to an OS-level error.",
                    scratch.get_path().display(),
                    self.new_path.display()
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
    use tempfile::tempdir;

    #[tokio::test]
    async fn move_repository() {
        let temp = tempdir().unwrap();

        let original = temp.path().join("original");
        let moved = temp.path().join("moved");
        let repo = crate::core::Repo::new("gh:sierrasoftworks/git-tool", original.clone());

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_null_console()
            .build();

        assert!(
            fs::create_dir(repo.path.clone()).is_ok(),
            "original directory should exist"
        );

        assert!(repo.path.exists());

        MoveDirectory {
            new_path: moved.clone(),
        }
        .apply_repo(&core, &repo)
        .await
        .unwrap();

        assert!(!original.exists());
        assert!(moved.exists());
    }

    #[tokio::test]
    async fn move_directory_no_repository_to_rename() {
        let temp = tempdir().unwrap();

        let repo = crate::core::Repo::new(
            "gh:sierrasoftworks/git-tool",
            temp.path().join("non_existent_repo"),
        );

        let new_repo =
            crate::core::Repo::new("gh:sierrasoftworks/gt", temp.path().join("new_repo"));

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_null_console()
            .build();

        let result = MoveDirectory {
            new_path: new_repo.path.clone(),
        }
        .apply_repo(&core, &repo)
        .await;

        assert!(result.is_err());

        let err = result.unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("does not exist on your machine and cannot be moved as a result."),
            "Unexpected error message: {msg}"
        );
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = tempdir().unwrap();
        let scratch = crate::core::Scratchpad::new("2019w15", temp.path().join("scratch"));
        let new_scratch = temp.path().join("new_scratch");

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .build();

        assert!(
            fs::create_dir(scratch.get_path()).is_ok(),
            "scratchpad directory should be created"
        );

        assert!(scratch.exists());
        assert!(!new_scratch.clone().exists());

        MoveDirectory {
            new_path: new_scratch.clone(),
        }
        .apply_scratchpad(&core, &scratch)
        .await
        .unwrap();

        assert!(
            !scratch.get_path().exists(),
            "old scratchpad should no longer exist."
        );
        assert!(
            new_scratch.exists(),
            "scratchpad should be moved to new directory"
        );
    }
}
