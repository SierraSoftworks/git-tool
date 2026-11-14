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
                // Each branch comes from a `branch <ref>` line with a single
                // leading refs/heads/ stripped. The result may itself start with
                // refs/heads/, since git allows branches named that way
                // (e.g. `git branch refs/heads/foo` => refs/heads/refs/heads/foo).
                assert!(input.lines().any(|line| {
                    line.trim_end()
                        .strip_prefix("branch ")
                        .map(|r| r.strip_prefix("refs/heads/").unwrap_or(r) == branch)
                        .unwrap_or(false)
                }));
            }
        }
    });
}
