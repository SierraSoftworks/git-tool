#![no_main]

use git_tool::{
    engine::{Config, Core, Repo},
    git,
    tasks::{GitBranchDelete, GitCheckout, GitInit, GitSwitch, Task},
};
use libfuzzer_sys::fuzz_target;

mod common;

fuzz_target!(|data: &[u8]| {
    common::runtime().block_on(async {
        let temp = tempfile::tempdir().expect("a temporary repository directory");
        let repo = Repo::new("local:fuzz/repository", temp.path().to_path_buf());
        let config = Config::default().with_dev_directory(temp.path());
        let core = Core::builder().with_config(config).build();

        GitInit {}
            .apply_repo(&core, &repo)
            .await
            .expect("the git-init task to succeed");

        for (opcode, branch) in common::operations(data) {
            match opcode % 4 {
                0 => {
                    let _ = GitCheckout { branch: &branch }
                        .apply_repo(&core, &repo)
                        .await;
                }
                1 => {
                    let _ = GitSwitch {
                        branch,
                        create_if_missing: true,
                    }
                    .apply_repo(&core, &repo)
                    .await;
                }
                2 => {
                    let _ = GitSwitch {
                        branch,
                        create_if_missing: false,
                    }
                    .apply_repo(&core, &repo)
                    .await;
                }
                _ => {
                    let _ = GitBranchDelete { branch }.apply_repo(&core, &repo).await;
                }
            }

            assert!(temp.path().join(".git").is_dir());
            git::git_branches(temp.path())
                .await
                .expect("the task-managed repository to remain queryable");
        }
    });
});
