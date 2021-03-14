mod add;
mod branch;
mod checkout;
mod clone;
mod cmd;
mod commit;
mod fetch;
mod init;
mod remote;
mod switch;

#[cfg(test)]
mod refs;

pub use add::git_add;
pub use branch::{git_branches, git_current_branch};
pub use checkout::git_checkout;
pub use clone::git_clone;
pub use cmd::git_cmd;
pub use commit::git_commit;
pub use fetch::git_fetch;
pub use init::git_init;
pub use remote::{git_remote_add, git_remote_list, git_remote_set_url};
pub use switch::git_switch;

#[cfg(test)]
pub use refs::{git_rev_parse, git_update_ref};
