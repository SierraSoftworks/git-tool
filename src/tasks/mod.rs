use super::core;
use async_trait::async_trait;

#[cfg(test)]
use tokio::sync::Mutex;

mod sequence;
mod new_folder;
mod git_init;

pub use sequence::Sequence;
pub use new_folder::NewFolder;
pub use git_init::GitInit;

#[async_trait]
pub trait Task {
    async fn apply_repo(&self, repo: &core::Repo) -> Result<(), core::Error>;
    async fn apply_scratchpad(&self, scratch: &core::Scratchpad) -> Result<(), core::Error>;
}

#[cfg(test)]
pub struct TestTask {
    ran_repo: Mutex<Option<core::Repo>>,
    ran_scratchpad: Mutex<Option<core::Scratchpad>>,
    error: Mutex<Option<core::Error>>
}


#[cfg(test)]
impl Default for TestTask {
    fn default() -> Self {
        Self {
            ran_repo: Mutex::new(None),
            ran_scratchpad: Mutex::new(None),
            error: Mutex::new(None)
        }
    }
}

#[cfg(test)]
#[async_trait]
impl Task for TestTask {
    async fn apply_repo(&self, repo: &core::Repo) -> Result<(), core::Error> {
        let mut r = self.ran_repo.lock().await;

        *r = Some(repo.clone());

        let e = self.error.lock().await;

        match e.clone() {
            Some(err) => Err(err),
            None => Ok(())
        }
    }

    async fn apply_scratchpad(&self, scratch: &core::Scratchpad) -> Result<(), core::Error> {
        let mut s = self.ran_scratchpad.lock().await;

        *s = Some(scratch.clone());

        let e = self.error.lock().await;

        match e.clone() {
            Some(err) => Err(err),
            None => Ok(())
        }
    }
}
