use super::Error;
use tokio::prelude::*;
use async_trait::async_trait;
use super::Config;
use std::sync::Arc;

#[cfg(test)]
use mocktopus::macros::*;

#[cfg_attr(test, mockable)]
pub struct FileSource {}

impl From<Arc<Config>> for FileSource {
    fn from(_: Arc<Config>) -> Self {
        Self{}
    }
}

impl FileSource {
    #[tracing::instrument(err, skip(self, path))]
    async fn read(&self, path: &std::path::Path) -> Result<String, Error> {
        let mut file = tokio::fs::File::open(path).await?;

        let mut buffer = vec![];
        file.read_to_end(&mut buffer).await?;

        Ok(String::from_utf8(buffer)?)
    }

    #[tracing::instrument(err, skip(self, path, content))]
    async fn write(&self, path: &std::path::Path, content: String) -> Result<(), Error> {
        let mut file = tokio::fs::File::create(path).await?;

        file.write_all(content.as_bytes()).await?;

        Ok(())
    }
}

#[cfg(test)]
pub mod mocks {
    use super::*;
    use std::sync::RwLock;
    use std::collections::HashMap;
    use crate::errors::{user, system_with_internal};

    #[derive(Clone)]
    pub struct TestFileSource {
        files: Arc<RwLock<HashMap<std::path::PathBuf, String>>>,
        error: Option<Error>,
    }

    impl From<Arc<Config>> for TestFileSource {
        fn from(_: Arc<Config>) -> Self {
            Self{
                files: Arc::new(RwLock::new(HashMap::new())),
                error: None
            }
        }
    }

    #[async_trait]
    impl FileSource for TestFileSource {
        async fn read(&self, path: &std::path::Path) -> Result<String, Error> {
            if let Some(err) = self.error.as_ref() {
                Err(err.clone())
            } else {
                match self.files.read() {
                    Ok(f) => {
                        match f.get(path) {
                            Some(content) => Ok(content.clone()),
                            None => Err(user("File not found.", "Check that the file path is correct and try again."))
                        }
                    },
                    Err(err) => {
                        Err(system_with_internal("Unable to read files.", "Please check the inner exception and try again.", err))
                    }
                }
            }
        }

        async fn write(&self, path: &std::path::Path, content: String) -> Result<(), Error> {
            if let Some(err) = self.error.as_ref() {
                Err(err.clone())
            } else {
                match self.files.write() {
                    Ok(mut f) => {
                        f.insert(path.to_path_buf(), content.clone());
                        Ok(())
                    },
                    Err(err) => {
                        Err(system_with_internal("Unable to read files.", "Please check the inner exception and try again.", err))
                    }
                }
            }
        }
    }
}
