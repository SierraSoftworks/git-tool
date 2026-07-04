use std::fmt::Display;
use std::str::FromStr;

/// A git branch name.
///
/// Wrapping the raw branch name in a dedicated type lets us attach the behaviour
/// which is specific to branches and keep it distinct from other string-like
/// identifiers. The mapping from a branch to its worktree lives on
/// [`super::Repo::worktree`], since a worktree belongs to a repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch(String);

impl Branch {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Branch {
    type Err = human_errors::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(human_errors::user(
                "A branch name cannot be empty.",
                &["Provide the name of the branch you want to open a worktree for."],
            ));
        }

        Ok(Branch(trimmed.to_string()))
    }
}

impl Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_branch_name() {
        let branch: Branch = "feature/test".parse().unwrap();
        assert_eq!(branch.as_str(), "feature/test");
        assert_eq!(branch.to_string(), "feature/test");
    }

    #[test]
    fn trims_and_rejects_empty() {
        assert_eq!("  main  ".parse::<Branch>().unwrap().as_str(), "main");
        assert!("   ".parse::<Branch>().is_err());
    }
}
