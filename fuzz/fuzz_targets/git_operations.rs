use afl::fuzz;
use git_tool::git;

mod common;

fn main() {
    fuzz!(|data: &[u8]| {
        common::runtime().block_on(async {
            let temp = tempfile::tempdir().expect("a temporary repository directory");
            git::git_init(temp.path())
                .await
                .expect("git init to succeed");

            for (opcode, branch) in common::operations(data) {
                match opcode % 4 {
                    0 => {
                        let _ = git::git_checkout(temp.path(), &branch).await;
                    }
                    1 => {
                        let _ = git::git_switch(temp.path(), &branch, true).await;
                    }
                    2 => {
                        let _ = git::git_switch(temp.path(), &branch, false).await;
                    }
                    _ => {
                        let _ = git::git_branch_delete(temp.path(), &branch).await;
                    }
                }

                assert!(temp.path().join(".git").is_dir());
                git::git_branches(temp.path())
                    .await
                    .expect("the initialized repository to remain queryable");
            }
        });
    });
}
