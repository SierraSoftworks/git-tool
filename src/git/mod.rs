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
mod worktree;

#[cfg(test)]
mod config;
#[cfg(test)]
mod refs;

pub use add::git_add;
#[allow(unused_imports)]
pub use branch::{
    git_branch_delete, git_branches, git_current_branch, git_default_branch, git_merged_branches,
};
pub use checkout::git_checkout;
pub use clone::git_clone;
pub use cmd::git_cmd;
pub use commit::git_commit;
#[allow(unused_imports)]
pub use fetch::git_fetch;
pub use init::git_init;
#[allow(unused_imports)]
pub use remote::{git_remote_add, git_remote_list, git_remote_rename, git_remote_set_url};
pub use switch::git_switch;
#[allow(unused_imports)]
pub use worktree::{
    Worktree, git_worktree_add, git_worktree_is_clean, git_worktree_list, git_worktree_remove,
};

// Only exposed to the fuzzing harness (cargo-afl sets `cfg(fuzzing)`), allowing
// the `git worktree list --porcelain` parser to be fuzzed without invoking git.
#[cfg(fuzzing)]
pub use worktree::parse_worktree_list_fuzz;

#[cfg(test)]
pub use config::git_config_set;
#[cfg(test)]
pub use refs::{git_rev_parse, git_update_ref};
