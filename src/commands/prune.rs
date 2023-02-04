use super::super::errors;
use super::*;
use crate::core::Target;
use crate::git;
use clap::Arg;
use itertools::Itertools;

pub struct PruneCommand {}

impl Command for PruneCommand {
    fn name(&self) -> String {
        String::from("prune")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("removes local branches that have been merged.")
            .long_about(
                "This command identifies branches in your local repository which have been merged
                into upstream branches and will proceed to delete them. This is particularly helpful
                if you use feature branches as part of your workflow and want to get rid of old ones.",
            )
            .arg(
                Arg::new("yes")
                    .long("yes")
                    .short('y')
                    .help("Do not prompt for confirmation before deleting branches")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(Arg::new("pattern")
                .help("The branch patterns which should be pruned")
                .action(clap::ArgAction::Append))
    }
}

#[async_trait]
impl CommandRunnable for PruneCommand {
    #[tracing::instrument(name = "gt prune", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let repo = core.resolver().get_current_repo()?;

        let default_branch = match git::git_default_branch(&repo.get_path()).await {
            Ok(default_branch) => default_branch,
            Err(e) => return Err(errors::user_with_cause(
                "Could not determine the default branch for your repository, this probably means that you do not have a synchronized `origin`.", 
                "Make sure that you have a correctly configured `origin` and that you have run `git fetch` before running this command again.",
                e)),
        };

        let merged = match git::git_merged_branches(&repo.get_path()).await {
            Ok(merged) => merged,
            Err(e) => return Err(errors::user_with_cause(
                "Could not determine the branches that have been merged into the default branch.",
                "Make sure that you have a correctly configured `origin` and that you have run `git fetch` before running this command again.",
                e)),
        };

        let to_remove: Vec<&String> = merged
            .iter()
            .filter(|&b| b != &default_branch)
            .unique()
            .collect();

        if to_remove.is_empty() {
            writeln!(core.output(), "No branches to remove")?;
            return Ok(0);
        }

        if !matches.get_flag("yes") {
            writeln!(core.output(), "The following branches will be removed:")?;
            for branch in to_remove.iter() {
                writeln!(core.output(), "  {branch}")?;
            }
            writeln!(core.output())?;

            let remove_branches = core
                .prompter()
                .prompt_bool(
                    "Are you sure you want to remove these branches? [y/N]: ",
                    Some(false),
                )?
                .unwrap_or_default();

            if !remove_branches {
                writeln!(core.output(), "Okay, we'll keep them as-is.")?;
                return Ok(0);
            }
        }

        for branch in to_remove.iter() {
            git::git_branch_delete(&repo.get_path(), branch).await?;
        }

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt prune",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        if let Ok(repo) = core.resolver().get_current_repo() {
            completer.offer("--yes");
            if let Ok(branches) = git::git_merged_branches(&repo.get_path()).await {
                completer.offer_many(branches.iter().unique().sorted());
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::core::builder::CoreBuilderWithConfig;
    use crate::core::*;
    use crate::tasks::{self, Task};
    use complete::helpers::test_completions_with_core;
    use tempfile::tempdir;

    /// Sets up a test repo in your provided temp directory.
    ///
    /// Creates a pair of repos in your provided temp directory,
    /// initializing the first with a `feature/test` branch and `main` branch.
    /// The second is cloned from the first and the `main` branch is then updated
    /// with a new commit.
    /// The second repo is then returned to the caller and configured to be the
    /// mock result for the current repo.
    ///
    /// ## Branches
    ///  - `origin/feature/test`
    ///  - `origin/main`
    ///  - `main`
    ///  ` `feature/test2`
    async fn setup_test_repo_with_remote(
        core: CoreBuilderWithConfig,
        temp: &tempfile::TempDir,
    ) -> (Core, Repo) {
        let repo: Repo = core::Repo::new(
            "gh:sierrasoftworks/test-git-switch-command",
            temp.path().join("repo"),
        );

        let repo_path = repo.get_path();
        let core = core
            .with_mock_resolver(|mock| {
                let repo_path = repo_path.clone();
                mock.expect_get_current_repo().returning(move || {
                    Ok(core::Repo::new(
                        "gh:sierrasoftworks/test-git-switch-command",
                        repo_path.clone(),
                    ))
                });
            })
            .build();

        let origin_repo = core::Repo::new(
            "gh:sierrasoftworks/test-git-switch-command2",
            temp.path().join("repo2"),
        );

        sequence!(
            // Run a `git init` to setup the repo
            tasks::GitInit {},
            tasks::GitRemote { name: "origin" },
            // Create the branch we want to switch to
            tasks::GitCheckout {
                branch: "feature/test",
            },
            tasks::WriteFile {
                path: "README.md".into(),
                content: "This is an example README file.",
            },
            tasks::GitAdd {
                paths: vec!["README.md"],
            },
            tasks::GitCommit {
                message: "Add README.md",
                paths: vec!["README.md"],
            },
            tasks::GitCheckout { branch: "main" }
        )
        .apply_repo(&core, &origin_repo)
        .await
        .unwrap();

        sequence!(tasks::GitInit {}, tasks::GitRemote { name: "origin" })
            .apply_repo(&core, &repo)
            .await
            .unwrap();

        git::git_remote_set_url(
            &repo.get_path(),
            "origin",
            origin_repo.get_path().to_str().unwrap(),
        )
        .await
        .unwrap();

        git::git_fetch(&repo.get_path(), "origin").await.unwrap();

        sequence!(
            tasks::GitSwitch {
                branch: "main".into(),
                create_if_missing: false,
            },
            tasks::GitSwitch {
                branch: "feature/test2".into(),
                create_if_missing: true,
            },
            tasks::WriteFile {
                path: "README.md".into(),
                content: "This is an example README file with some changes.",
            },
            tasks::GitAdd {
                paths: vec!["README.md"],
            },
            tasks::GitCommit {
                message: "Update README.md",
                paths: vec!["README.md"],
            },
            tasks::GitSwitch {
                branch: "main".into(),
                create_if_missing: false,
            },
            tasks::WriteFile {
                path: ".git/refs/remotes/origin/HEAD".into(),
                content: "ref: refs/remotes/origin/main",
            }
        )
        .apply_repo(&core, &repo)
        .await
        .unwrap();

        assert!(repo.valid(), "the repository should exist and be valid");
        (core, repo)
    }

    #[tokio::test]
    async fn prune_completions() {
        let temp = tempdir().unwrap();

        let core = core::Core::builder().with_config_for_dev_directory(temp.path());

        let (core, repo) = setup_test_repo_with_remote(core, &temp).await;

        crate::git::git_cmd(
            tokio::process::Command::new("git")
                .current_dir(&repo.get_path())
                .arg("merge")
                .arg("feature/test2"),
        )
        .await
        .unwrap();

        test_completions_with_core(&core, "gt prune", "", vec!["--yes", "feature/test2"]).await;
    }

    #[tokio::test]
    async fn prune_local_only_branch() {
        let cmd: PruneCommand = PruneCommand {};

        let temp = tempdir().unwrap();

        let console = crate::console::mock_with_input("y\n");
        let (core, repo) = setup_test_repo_with_remote(
            Core::builder()
                .with_config_for_dev_directory(temp.path())
                .with_console(console.clone()),
            &temp,
        )
        .await;

        crate::git::git_cmd(
            tokio::process::Command::new("git")
                .current_dir(&repo.get_path())
                .arg("merge")
                .arg("feature/test2"),
        )
        .await
        .unwrap();

        let mut branches = git::git_branches(&repo.get_path()).await.unwrap();
        branches.sort();
        assert_eq!(branches, vec!["feature/test", "feature/test2", "main"]);

        assert_eq!(
            git::git_merged_branches(&repo.get_path()).await.unwrap(),
            vec!["feature/test2"]
        );

        let args: ArgMatches = cmd.app().get_matches_from(vec!["prune"]);
        cmd.run(&core, &args).await.unwrap();

        assert!(
            console.to_string().contains("feature/test2"),
            "the output should contain the branch name being cleaned up"
        );

        assert!(repo.valid(), "the repository should still be valid");
        let mut branches = git::git_branches(&repo.get_path()).await.unwrap();
        branches.sort();
        assert_eq!(branches, vec!["feature/test", "main"]);
    }

    #[tokio::test]
    async fn prune_bare_repo() {
        let cmd: PruneCommand = PruneCommand {};

        let temp = tempdir().unwrap();
        let repo: Repo = core::Repo::new(
            "gh:sierrasoftworks/test-git-prune-command",
            temp.path().join("repo"),
        );

        let core = core::Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_mock_resolver(|mock| {
                let temp_path = temp.path().to_path_buf();
                mock.expect_get_current_repo().returning(move || {
                    Ok(core::Repo::new(
                        "gh:sierrasoftworks/test-git-prune-command",
                        temp_path.join("repo"),
                    ))
                });
            })
            .build();

        // Run a `git init` to setup the repo
        tasks::GitInit {}.apply_repo(&core, &repo).await.unwrap();

        assert!(repo.valid(), "the repository should exist and be valid");

        let args: ArgMatches = cmd.app().get_matches_from(vec!["prune"]);
        cmd.run(&core, &args)
            .await
            .expect_err("the command should fail");

        assert!(repo.valid(), "the repository should still be valid");
        assert!(git::git_branches(&repo.get_path())
            .await
            .unwrap()
            .is_empty());
    }
}
