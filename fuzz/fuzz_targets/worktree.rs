use afl::fuzz;
use git_tool::git::parse_worktree_list_fuzz;

mod common;

fn main() {
    fuzz!(|data: &[u8]| {
        let input = String::from_utf8_lossy(common::bounded_bytes(data)).into_owned();

        // Parsing arbitrary `git worktree list --porcelain` output must never
        // panic, must terminate, and must be deterministic.
        let worktrees = parse_worktree_list_fuzz(&input);
        assert_eq!(worktrees, parse_worktree_list_fuzz(&input));

        for worktree in &worktrees {
            if let Some(branch) = &worktree.branch {
                // The refs/heads/ prefix is always stripped from branch names.
                assert!(!branch.starts_with("refs/heads/"));
            }
        }
    });
}
