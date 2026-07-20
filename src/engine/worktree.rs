use super::{Branch, Config, Repo, Target};
use gotmpl::Value;
use std::path::PathBuf;

/// A git worktree for a particular [`Branch`] of a [`Repo`], resolved to its
/// location within the configured worktree directory.
///
/// A `Worktree` is a launch [`Target`]: its path is the worktree directory, while
/// its template context mirrors the originating repository so that application
/// templates behave exactly as they would for a normal repository.
#[derive(Debug, Clone)]
pub struct Worktree {
    /// The launch target: it carries the originating repository's identity
    /// (service + full name) but points at the worktree directory.
    repo: Repo,
    branch: Branch,
}

impl Worktree {
    /// Maps a [`Repo`] and [`Branch`] to the worktree which should hold that
    /// branch's checkout, using the worktree directory configured in `config`.
    pub fn new(repo: &Repo, branch: &Branch, config: &Config) -> Self {
        let path = config
            .get_worktree_directory()
            .join(Self::dir_name(repo, branch));

        let target = Repo::new(&format!("{}:{}", repo.service, repo.get_full_name()), path);

        Self {
            repo: target,
            branch: branch.clone(),
        }
    }

    // Currently only exercised by tests, but a natural part of a worktree's API.
    #[allow(dead_code)]
    pub fn branch(&self) -> &Branch {
        &self.branch
    }

    pub fn path(&self) -> PathBuf {
        self.repo.get_path()
    }

    /// Builds the directory name used to store a worktree for the given repository
    /// and branch. The repository's short name and the (sanitized) branch keep the
    /// path human-readable, while an 8 character hash of the repository's full
    /// identity disambiguates repositories that share the same short name (for
    /// example `org-a/tools` and `org-b/tools`).
    pub(crate) fn dir_name(repo: &Repo, branch: &Branch) -> String {
        let sanitized_branch: String = branch
            .as_str()
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                    c
                } else {
                    '-'
                }
            })
            .collect();

        let identity = format!("{}:{}", repo.service, repo.get_full_name());

        format!(
            "{}-{}-{}",
            repo.name,
            sanitized_branch,
            Self::short_hash(&identity)
        )
    }

    /// Produces a stable 8 character hexadecimal hash of the provided string using
    /// the FNV-1a algorithm. FNV-1a is used (rather than [`std::hash::DefaultHasher`])
    /// because it produces identical output across platforms and toolchain
    /// versions, ensuring a repository always maps to the same worktree directory.
    pub(crate) fn short_hash(value: &str) -> String {
        const FNV_OFFSET_BASIS: u32 = 0x811c_9dc5;
        const FNV_PRIME: u32 = 0x0100_0193;

        let mut hash = FNV_OFFSET_BASIS;
        for byte in value.bytes() {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(FNV_PRIME);
        }

        format!("{hash:08x}")
    }
}

impl Target for Worktree {
    fn get_name(&self) -> String {
        self.repo.get_name()
    }

    fn get_path(&self) -> PathBuf {
        self.repo.get_path()
    }

    fn exists(&self) -> bool {
        self.repo.exists()
    }

    fn template_context(&self, config: &Config) -> Result<Value, human_errors::Error> {
        self.repo.template_context(config)
    }
}

impl std::fmt::Display for Worktree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", &self.repo, &self.branch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> Repo {
        Repo::new("gh:sierrasoftworks/git-tool", "/dev/git-tool".into())
    }

    #[test]
    fn dir_name_sanitizes_branch() {
        let repo = repo();

        assert!(
            Worktree::dir_name(&repo, &"feat/forgejo".parse().unwrap())
                .starts_with("git-tool-feat-forgejo-")
        );
        assert!(
            Worktree::dir_name(&repo, &"release/v1.2.3".parse().unwrap())
                .starts_with("git-tool-release-v1.2.3-")
        );
    }

    #[test]
    fn dir_name_disambiguates_repositories() {
        let repo_a = Repo::new("gh:org-a/tools", "/dev/a/tools".into());
        let repo_b = Repo::new("gh:org-b/tools", "/dev/b/tools".into());
        let branch: Branch = "main".parse().unwrap();

        assert_ne!(
            Worktree::dir_name(&repo_a, &branch),
            Worktree::dir_name(&repo_b, &branch)
        );

        assert_eq!(
            Worktree::dir_name(&repo_a, &branch),
            Worktree::dir_name(&repo_a, &branch)
        );
    }

    #[test]
    fn short_hash_is_eight_hex_characters() {
        let hash = Worktree::short_hash("gh:sierrasoftworks/git-tool");
        assert_eq!(hash.len(), 8);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn maps_branch_into_worktree_directory() {
        let repo = repo();
        let branch: Branch = "feature/test".parse().unwrap();
        let config = Config::for_dev_directory(std::path::Path::new("/dev"));

        let worktree = Worktree::new(&repo, &branch, &config);
        assert_eq!(
            worktree.path(),
            std::path::PathBuf::from("/dev/worktrees").join(Worktree::dir_name(&repo, &branch))
        );
        assert_eq!(worktree.branch().as_str(), "feature/test");
    }
}
