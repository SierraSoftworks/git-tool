use super::Config;
use super::Error;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::prelude::*;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FileSource: Send + Sync {
    async fn read(&self, path: &std::path::Path) -> Result<String, Error>;
    async fn write<S: AsRef<str>>(&self, path: &std::path::Path, contents: S) -> Result<(), Error>;
}

pub fn file_source() -> Arc<dyn FileSource + Send + Sync> {
    Arc::new(LocalFileSource {})
}

struct LocalFileSource {}

impl FileSource for LocalFileSource {
    #[tracing::instrument(err, skip(self, path))]
    async fn read(&self, path: &std::path::Path) -> Result<String, Error> {
        let mut file = tokio::fs::File::open(path).await?;

        let mut buffer = vec![];
        file.read_to_end(&mut buffer).await?;

        Ok(String::from_utf8(buffer)?)
    }

    #[tracing::instrument(err, skip(self, path, content))]
    async fn write<S: AsRef<str>>(&self, path: &std::path::Path, content: S) -> Result<(), Error> {
        let mut file = tokio::fs::File::create(path).await?;

        file.write_all(content.as_ref().as_bytes()).await?;

        Ok(())
    }
}

#[cfg(test)]
pub fn mock_file_source(error: Option<Error>) -> Arc<dyn FileSource> {
    let mut mock = MockFileSource::new();
    let files = Arc::new(RwLock::new(HashMap::new()));

    mock.expect_read().returning(|path| {
        if let Some(err) = self.error.as_ref() {
            Err(err.clone())
        } else {
            match self.files.read() {
                Ok(f) => match f.get(path) {
                    Some(content) => Ok(content.clone()),
                    None => Err(user(
                        "File not found.",
                        "Check that the file path is correct and try again.",
                    )),
                },
                Err(err) => Err(system_with_internal(
                    "Unable to read files.",
                    "Please check the inner exception and try again.",
                    err,
                )),
            }
        }
    });

    mock.expect_write().returning(|path, contents| {
        if let Some(err) = self.error.as_ref() {
            Err(err.clone())
        } else {
            match self.files.write() {
                Ok(mut f) => {
                    f.insert(path.to_path_buf(), contents.as_ref().to_string());
                    Ok(())
                }
                Err(err) => Err(system_with_internal(
                    "Unable to write files.",
                    "Please check the inner exception and try again.",
                    err,
                )),
            }
        }
    });

    Arc::new(mock)
}
