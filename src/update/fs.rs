use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tracing_batteries::prelude::*;

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

#[derive(Debug)]
struct DefaultFileSystem {}

#[async_trait::async_trait]
impl FileSystem for DefaultFileSystem {
    #[tracing::instrument(err, skip(path))]
    async fn delete_file(&self, path: &Path) -> Result<(), errors::Error> {
        let max_retries = 10;
        let mut retries = max_retries;

        while retries >= 0 {
            retries -= 1;

            match tokio::fs::remove_file(path).await {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

    #[tokio::test]
    async fn test_delete_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("test.txt");
        tokio::fs::write(&path, "test").await.unwrap();

        let fs = DefaultFileSystem {};
        fs.delete_file(&path).await.unwrap();

        assert!(!path.exists());
    }

    #[tokio::test]
    async fn test_copy_file() {
        let temp = tempfile::tempdir().unwrap();
        let from = temp.path().join("from.txt");
        let to = temp.path().join("to.txt");
        tokio::fs::write(&from, "test").await.unwrap();

        let fs = DefaultFileSystem {};
        fs.copy_file(&from, &to).await.unwrap();

        assert!(to.exists());
        let content = tokio::fs::read_to_string(&to).await.unwrap();
        assert_eq!(content, "test");
    }

    #[test]
    fn test_get_temp_app_path() {
        let release = Release {
            id: "1.0.0".to_string(),
            changelog: "".to_string(),
            version: Version {
                major: 1,
                minor: 0,
                patch: 0,
                pre: Default::default(),
                build: Default::default(),
            },
            prerelease: false,
            variants: vec![],
        };
        let fs = DefaultFileSystem {};
        let path = fs.get_temp_app_path(&release);

        #[cfg(target_os = "windows")]
        assert_eq!(path.file_name().unwrap(), "git-tool-update-1.0.0.exe");

        #[cfg(not(target_os = "windows"))]
        assert_eq!(path.file_name().unwrap(), "git-tool-update-1.0.0");
    }
}
