use std::{
    path::{Path, PathBuf},
    time::Duration,
};

#[cfg(test)]
use mockall::automock;

use crate::errors;

use super::Release;

pub(super) fn default() -> Box<dyn FileSystem + Send + Sync> {
    Box::new(DefaultFileSystem {})
}

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait FileSystem {
    async fn delete_file(&self, path: &Path) -> Result<(), errors::Error>;
    async fn copy_file(&self, from: &Path, to: &Path) -> Result<(), errors::Error>;
    fn get_temp_app_path(&self, release: &Release) -> PathBuf;
}

struct DefaultFileSystem {}

impl std::fmt::Debug for DefaultFileSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultFileSystem").finish()
    }
}

#[async_trait::async_trait]
impl FileSystem for DefaultFileSystem {
    #[tracing::instrument(err, skip(path))]
    async fn delete_file(&self, path: &Path) -> Result<(), errors::Error> {
        let max_retries = 10;
        let mut retries = max_retries;

        while retries >= 0 {
            retries -= 1;

            match tokio::fs::remove_file(path).await {
                Err(e) if retries < 0 => return Err(errors::user_with_internal(
                    &format!("Could not remove the old application file '{}' after {} retries.", path.display(), max_retries),
                    "This probably means that Git-Tool is still running in another terminal. Please exit any running Git-Tool processes (including shells launched by it) before trying again.",
                    e
                )),
                Ok(_) => return Ok(()),
                _ => tokio::time::sleep(Duration::from_millis(500)).await,
            }
        }

        Ok(())
    }

    #[tracing::instrument(err, skip(from, to))]
    async fn copy_file(&self, from: &Path, to: &Path) -> Result<(), errors::Error> {
        let max_retries = 10;
        let mut retries = max_retries;

        while retries > 0 {
            retries -= 1;

            match tokio::fs::copy(from, to).await {
                Err(e) if retries < 0 => return Err(errors::user_with_internal(
                    &format!("Could not copy the new application file '{}' to overwrite the old application file '{}' after {} retries.", from.display(), to.display(), max_retries),
                    "This probably means that Git-Tool is still running in another terminal. Please exit any running Git-Tool processes (including shells launched by it) before trying again.",
                    e
                )),
                Ok(_) => return Ok(()),
                _ => tokio::time::sleep(Duration::from_millis(500)).await,
            }
        }

        Ok(())
    }

    fn get_temp_app_path(&self, release: &Release) -> PathBuf {
        let file_name = format!(
            "git-tool-update-{}{}",
            release.id,
            if cfg!(windows) { ".exe" } else { "" }
        );
        std::env::temp_dir().join(file_name)
    }
}
