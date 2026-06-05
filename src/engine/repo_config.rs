use std::collections::BTreeMap;

use human_errors::ResultExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{App, Repo, Target};

/// The name of the per-repository configuration file which Git-Tool looks for
/// in the root of a repository.
pub const REPO_CONFIG_FILE: &str = "git-tool.yml";

/// The per-repository configuration which may be present at the root of a
/// repository (in a `git-tool.yml` file). It allows a repository to define a
/// series of named tasks which can be executed using the `gt task` command, as
/// well as automation which should be applied when a worktree is created.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoConfig {
    #[serde(default)]
    tasks: BTreeMap<String, RepoTask>,

    #[serde(default)]
    worktree: Option<WorktreeConfig>,
}

/// A named task which can be executed within the context of a repository (or a
/// worktree created from it). Tasks mirror the structure of an [`App`] and are
/// executed through the same launcher, giving them templating and signal
/// forwarding for free.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoTask {
    command: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    args: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    environment: Vec<String>,
}

/// Configuration which controls the automation applied when a worktree is
/// created for a repository.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorktreeConfig {
    /// A list of paths (relative to the repository root) which should be
    /// symlinked from the worktree back to the original repository. This is
    /// intended for directories such as `node_modules` or Rust's `target`
    /// which are expensive to recreate and safe to share.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    symlinks: Vec<String>,

    /// A list of task names which should be executed (within the context of the
    /// worktree) once it has been created.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tasks: Vec<String>,
}

impl RepoConfig {
    /// Attempts to load the per-repository configuration for the provided
    /// repository. Returns `Ok(None)` if the repository does not define a
    /// `git-tool.yml` file.
    pub fn for_repo(repo: &Repo) -> Result<Option<RepoConfig>, human_errors::Error> {
        let path = repo.get_path().join(REPO_CONFIG_FILE);
        if !path.exists() {
            return Ok(None);
        }

        let bytes = std::fs::read(&path).wrap_user_err(
            format!(
                "Could not read the repository configuration file '{}' due to an OS-level error.",
                path.display()
            ),
            &["Make sure that Git-Tool has permission to read the file and then try again."],
        )?;

        Ok(Some(RepoConfig::from_bytes(&bytes)?))
    }

    /// Parses a repository configuration from its raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, human_errors::Error> {
        serde_yaml::from_slice(bytes).map_err(|e| {
            human_errors::wrap_user(
                e,
                "We couldn't parse your repository configuration file due to a YAML parser error.",
                &["Check that the YAML in your 'git-tool.yml' file is correctly formatted."],
            )
        })
    }

    /// Serializes this configuration to its canonical YAML form. This is the
    /// exact content that is hashed for trust verification, and is what we show
    /// to the user when prompting them to trust the repository. Because it is
    /// derived from the parsed configuration, cosmetic differences in the
    /// original file (such as comments, whitespace, or key ordering) are
    /// normalized away.
    pub fn to_yaml(&self) -> Result<String, human_errors::Error> {
        serde_yaml::to_string(self).map_err(|e| {
            human_errors::wrap_system(
                e,
                "We couldn't serialize your repository configuration to YAML.",
                &["Please report this issue on GitHub so that we can try and resolve it."],
            )
        })
    }

    /// Computes the SHA-256 hash (hex encoded) of the canonical serialization of
    /// this configuration. Cosmetic changes such as comments, whitespace, or key
    /// ordering do not affect the hash - only meaningful changes to the tasks or
    /// worktree automation do. Tasks are stored in a `BTreeMap` so that their
    /// ordering is deterministic across serializations.
    pub fn hash(&self) -> Result<String, human_errors::Error> {
        Ok(hash_bytes(self.to_yaml()?.as_bytes()))
    }

    /// Retrieves a task by name, if it is defined.
    pub fn get_task(&self, name: &str) -> Option<&RepoTask> {
        self.tasks.get(name)
    }

    /// Returns an iterator over the names of all defined tasks.
    pub fn task_names(&self) -> impl Iterator<Item = &String> {
        self.tasks.keys()
    }

    /// Returns the worktree automation configuration, if it is defined.
    pub fn worktree(&self) -> Option<&WorktreeConfig> {
        self.worktree.as_ref()
    }
}

impl RepoTask {
    /// Builds an [`App`] which can be launched to execute this task. The
    /// provided name is used purely for diagnostic output.
    pub fn to_app(&self, name: &str) -> App {
        let mut builder = App::builder();
        builder.with_name(name).with_command(&self.command);

        if !self.args.is_empty() {
            builder.with_args(self.args.iter().map(|s| s.as_str()).collect());
        }

        if !self.environment.is_empty() {
            builder.with_environment(self.environment.iter().map(|s| s.as_str()).collect());
        }

        App::from(&mut builder)
    }
}

impl WorktreeConfig {
    /// The paths (relative to the repository root) which should be symlinked
    /// from the worktree back to the original repository.
    pub fn symlinks(&self) -> &[String] {
        &self.symlinks
    }

    /// The names of the tasks which should be run once the worktree is created.
    pub fn tasks(&self) -> &[String] {
        &self.tasks
    }
}

/// Computes the hex-encoded SHA-256 hash of the provided bytes.
fn hash_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest.iter() {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = r#"
tasks:
  build:
    command: cargo
    args:
      - build
  test:
    command: cargo
    args:
      - test
    environment:
      - RUST_LOG=debug
worktree:
  symlinks:
    - node_modules
    - target
  tasks:
    - build
"#;

    #[test]
    fn parse_example() {
        let config = RepoConfig::from_bytes(EXAMPLE.as_bytes()).unwrap();

        let build = config.get_task("build").expect("build task should exist");
        assert_eq!(build.command, "cargo");
        assert_eq!(build.args, vec!["build"]);

        let test = config.get_task("test").expect("test task should exist");
        assert_eq!(test.environment, vec!["RUST_LOG=debug"]);

        let worktree = config.worktree().expect("worktree config should exist");
        assert_eq!(worktree.symlinks(), &["node_modules", "target"]);
        assert_eq!(worktree.tasks(), &["build"]);
    }

    #[test]
    fn missing_sections_default_empty() {
        let config = RepoConfig::from_bytes(b"tasks: {}").unwrap();
        assert!(config.get_task("build").is_none());
        assert!(config.worktree().is_none());
        assert_eq!(config.task_names().count(), 0);
    }

    #[test]
    fn hash_is_stable_and_content_dependent() {
        let a = RepoConfig::from_bytes(EXAMPLE.as_bytes()).unwrap();
        let b = RepoConfig::from_bytes(EXAMPLE.as_bytes()).unwrap();
        assert_eq!(a.hash().unwrap(), b.hash().unwrap());
        assert_eq!(a.hash().unwrap().len(), 64);

        let c = RepoConfig::from_bytes(b"tasks: {}").unwrap();
        assert_ne!(a.hash().unwrap(), c.hash().unwrap());
    }

    #[test]
    fn hash_ignores_comments_and_formatting() {
        // The same logical configuration expressed with comments, different
        // whitespace, and a different key ordering must produce the same hash.
        let a = RepoConfig::from_bytes(EXAMPLE.as_bytes()).unwrap();

        let reformatted = r#"
# This repository builds and tests with cargo.
worktree:
  tasks: [build]
  symlinks: [node_modules, target]
tasks:
  test:
    command: cargo
    environment: [RUST_LOG=debug]   # enable debug logging
    args: [test]
  build:
    command: cargo
    args: [build]
"#;
        let b = RepoConfig::from_bytes(reformatted.as_bytes()).unwrap();

        assert_eq!(a.hash().unwrap(), b.hash().unwrap());
    }

    #[test]
    fn task_to_app() {
        let config = RepoConfig::from_bytes(EXAMPLE.as_bytes()).unwrap();
        let app = config.get_task("test").unwrap().to_app("test");
        assert_eq!(app.get_name(), "test");
        assert_eq!(app.get_command(), "cargo");
        assert_eq!(app.get_args(), vec!["test"]);
        assert_eq!(app.get_environment(), vec!["RUST_LOG=debug"]);
    }
}
