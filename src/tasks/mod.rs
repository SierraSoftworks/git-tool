use super::core;
use super::core::Core;
use async_trait::async_trait;

#[cfg(test)]
use tokio::sync::Mutex;

mod sequence;

#[macro_export]
macro_rules! sequence {
    [$($task:expr),+] => {
        $crate::tasks::Sequence::new(
            vec![
                $(std::sync::Arc::new($task)),+
            ]
        )
    };
}

mod create_remote;
mod ensure_no_remote;
mod git_add;
mod git_checkout;
mod git_clone;
mod git_commit;
mod git_init;
mod git_remote;
mod git_switch;
mod move_directory;
mod move_remote;
mod new_folder;
mod write_file;

pub use create_remote::CreateRemote;
pub use ensure_no_remote::EnsureNoRemote;
#[allow(unused_imports)]
pub use git_add::GitAdd;
pub use git_checkout::GitCheckout;
pub use git_clone::GitClone;
#[allow(unused_imports)]
pub use git_commit::GitCommit;
pub use git_init::GitInit;
pub use git_remote::GitRemote;
pub use git_switch::GitSwitch;
pub use move_directory::MoveDirectory;
pub use move_remote::MoveRemote;
pub use new_folder::NewFolder;
pub use sequence::Sequence;
#[allow(unused_imports)]
pub use write_file::WriteFile;

#[async_trait]
pub trait Task {
    async fn apply_repo(&self, _core: &Core, _repo: &core::Repo) -> Result<(), core::Error> {
        Ok(())
    }
    async fn apply_scratchpad(
        &self,
        _core: &Core,
        _scratch: &core::Scratchpad,
    ) -> Result<(), core::Error> {
        Ok(())
    }
}

#[cfg(test)]
pub struct TestTask {
    ran_repo: Mutex<Option<core::Repo>>,
    ran_scratchpad: Mutex<Option<core::Scratchpad>>,
    error: bool,
}

#[cfg(test)]
impl Default for TestTask {
    fn default() -> Self {
        Self {
            ran_repo: Mutex::new(None),
            ran_scratchpad: Mutex::new(None),
            error: false,
        }
    }
}

#[cfg(test)]
#[async_trait]
impl Task for TestTask {
    async fn apply_repo(&self, _core: &Core, repo: &core::Repo) -> Result<(), core::Error> {
        let mut r = self.ran_repo.lock().await;

        *r = Some(repo.clone());

        match self.error {
            true => Err(human_errors::Error::SystemError(
                "Mock Error".to_string(),
                "Configure the mock to not throw an error".to_string(),
                None,
                None,
            )
            .into()),
            false => Ok(()),
        }
    }

    async fn apply_scratchpad(
        &self,
        _core: &Core,
        scratch: &core::Scratchpad,
    ) -> Result<(), core::Error> {
        let mut s = self.ran_scratchpad.lock().await;

        *s = Some(scratch.clone());

        match self.error {
            true => Err(human_errors::Error::SystemError(
                "Mock Error".to_string(),
                "Configure the mock to not throw an error".to_string(),
                None,
                None,
            )
            .into()),
            false => Ok(()),
        }
    }
}
