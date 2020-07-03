use super::core;
use super::core::{KeyChain, Launcher, Resolver, Output};
use async_trait::async_trait;

#[cfg(test)]
use tokio::sync::Mutex;

mod sequence;

#[macro_export]
macro_rules! sequence {
    [$($task:expr),+] => {
        crate::tasks::Sequence::new(
            vec![
                $(std::sync::Arc::new($task)),+
            ]
        )
    };
}

mod new_folder;
mod git_checkout;
mod git_clone;
mod git_init;
mod git_remote;

pub use sequence::Sequence;
pub use new_folder::NewFolder;
pub use git_checkout::GitCheckout;
pub use git_clone::GitClone;
pub use git_init::GitInit;
pub use git_remote::GitRemote;

#[async_trait]
pub trait Task<K: KeyChain, L: Launcher, R: Resolver, O: Output> {
    async fn apply_repo(&self, core: &core::Core<K, L, R, O>, repo: &core::Repo) -> Result<(), core::Error>;
    async fn apply_scratchpad(&self, core: &core::Core<K, L, R, O>, scratch: &core::Scratchpad) -> Result<(), core::Error>;
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
impl<K: KeyChain, L: Launcher, R: Resolver, O: Output> Task<K, L, R, O> for TestTask {
    async fn apply_repo(&self, _core: &core::Core<K, L, R, O>, repo: &core::Repo) -> Result<(), core::Error> {
        let mut r = self.ran_repo.lock().await;

        *r = Some(repo.clone());

        let e = self.error.lock().await;

        match e.clone() {
            Some(err) => Err(err),
            None => Ok(())
        }
    }

    async fn apply_scratchpad(&self, _core: &core::Core<K, L, R, O>, scratch: &core::Scratchpad) -> Result<(), core::Error> {
        let mut s = self.ran_scratchpad.lock().await;

        *s = Some(scratch.clone());

        let e = self.error.lock().await;

        match e.clone() {
            Some(err) => Err(err),
            None => Ok(())
        }
    }
}
