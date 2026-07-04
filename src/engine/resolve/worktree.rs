use super::Resolver;
use crate::engine::{Branch, Core, Repo, Worktree};

/// Resolves the worktree a branch maps to within a repository, using the
/// configured worktree directory.
impl Resolver<(&Repo, &Branch), Worktree> for Core {
    fn resolve(&self, (repo, branch): (&Repo, &Branch)) -> Result<Worktree, human_errors::Error> {
        Ok(repo.worktree(branch, self.config()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_a_worktree_from_repo_and_branch() {
        let core = Core::builder().with_default_config().build();
        let repo = Repo::new("gh:sierrasoftworks/git-tool", "/dev/git-tool".into());
        let branch: Branch = "feature/test".parse().unwrap();

        let worktree: Worktree = core.resolve((&repo, &branch)).unwrap();
        assert_eq!(worktree.branch().as_str(), "feature/test");
    }
}
