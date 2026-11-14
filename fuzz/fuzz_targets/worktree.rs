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

        // Every `branch <ref>` line in the input, with exactly one leading
        // `refs/heads/` stripped (mirroring the parser's line handling). Git
        // only ever strips the prefix once, so a ref such as
        // `refs/heads/refs/heads/x` legitimately yields the branch
        // `refs/heads/x`; asserting that no prefix remains would reject it.
        let expected_branches: Vec<&str> = input
            .lines()
            .filter_map(|line| line.trim_end().strip_prefix("branch "))
            .map(|raw| raw.strip_prefix("refs/heads/").unwrap_or(raw))
            .collect();

        for worktree in &worktrees {
            if let Some(branch) = &worktree.branch {
                // The refs/heads/ prefix is stripped exactly once from the raw
                // ref reported by git.
                assert!(
                    expected_branches.contains(&branch.as_str()),
                    "branch {branch:?} is not a reported ref with a single refs/heads/ prefix stripped"
                );
            }
        }
    });
}
