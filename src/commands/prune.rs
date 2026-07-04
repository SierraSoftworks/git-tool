use super::*;
use crate::engine::Target;
use crate::errors::HumanErrorResultExt;
use crate::git;
use crate::tasks::Task;
use clap::Arg;
use human_errors::ResultExt;
use itertools::Itertools;
use tracing_batteries::prelude::*;

pub struct PruneCommand;
crate::command!(PruneCommand);

#[async_trait]
impl CommandRunnable for PruneCommand {
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
                if you use feature branches as part of your workflow and want to get rid of old ones.

                It will also remove any Git worktrees for the repository which do not contain
                uncommitted changes, leaving any worktree with pending work untouched.",
            )
            .arg(
                Arg::new("yes")
                    .long("yes")
                    .short('y')
                    .help("Do not prompt for confirmation before deleting branches")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("branches")
                    .long("branches")
                    .short('b')
                    .help("Prune merged branches (enabled by default unless --worktrees is set)")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("worktrees")
                    .long("worktrees")
                    .short('w')
                    .help("Prune clean worktrees (enabled by default unless --branches is set)")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(Arg::new("pattern")
                .help("Only prune branches and worktrees whose branch name contains one of these patterns")
                .action(clap::ArgAction::Append))
    }

    #[tracing::instrument(name = "gt prune", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, human_errors::Error> {
        let repo: Repo = core.resolve(())?;

        // Determine which subsets we should prune. When neither flag is provided we
        // prune both branches and worktrees; specifying one flag restricts the
        // operation to just that subset.
        let (prune_branches, prune_worktrees) =
            match (matches.get_flag("branches"), matches.get_flag("worktrees")) {
                (true, false) => (true, false),
                (false, true) => (false, true),
                _ => (true, true),
            };

        // The patterns are matched against branch names; an empty pattern list
        // matches everything.
        let patterns: Vec<String> = matches
            .get_many::<String>("pattern")
            .map(|values| values.cloned().collect())
            .unwrap_or_default();

        let branches_to_remove: Vec<String> = if prune_branches {
            self.get_prune_branches(&repo, &patterns).await?
        } else {
            Vec::new()
        };

        let worktrees_to_remove = if prune_worktrees {
            self.get_prune_worktrees(&repo, &patterns).await?
        } else {
            Vec::new()
        };

        if branches_to_remove.is_empty() && worktrees_to_remove.is_empty() {
            writeln!(core.output(), "No branches or worktrees to remove").to_human_error()?;
            return Ok(0);
        }

        if !matches.get_flag("yes") {
            if !branches_to_remove.is_empty() {
                writeln!(core.output(), "The following branches will be removed:")
                    .to_human_error()?;
                for branch in branches_to_remove.iter() {
                    writeln!(core.output(), "  {branch}").to_human_error()?;
                }
                writeln!(core.output()).to_human_error()?;
            }

            if !worktrees_to_remove.is_empty() {
                writeln!(core.output(), "The following worktrees will be removed:")
                    .to_human_error()?;
                for worktree in worktrees_to_remove.iter() {
                    writeln!(core.output(), "  {}", worktree.path.display()).to_human_error()?;
                }
                writeln!(core.output()).to_human_error()?;
            }

            let remove_branches = core
                .prompter()
                .prompt_bool(
                    "Are you sure you want to remove these branches and worktrees? [y/N]: ",
                    Some(false),
                )?
                .unwrap_or_default();

            if !remove_branches {
                writeln!(core.output(), "Okay, we'll keep them as-is.").to_human_error()?;
                return Ok(0);
            }
        }

        // Remove worktrees before deleting branches: a branch which is checked out
        // in a worktree cannot be deleted until that worktree has been removed. We
        // chain the individual operations together using a Sequence task so that
        // they are applied in order.
        let mut cleanup: Vec<std::sync::Arc<dyn crate::tasks::Task + Send + Sync>> = Vec::new();

        for worktree in worktrees_to_remove {
            cleanup.push(std::sync::Arc::new(tasks::GitWorktreeRemove {
                path: worktree.path,
            }));
        }

        for branch in branches_to_remove {
            cleanup.push(std::sync::Arc::new(tasks::GitBranchDelete { branch }));
        }

        tasks::Sequence::new(cleanup)
            .apply_repo(core, &repo)
            .await?;

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt prune",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        let repo: Result<Repo, _> = core.resolve(());
        if let Ok(repo) = repo {
            completer.offer("--yes");
            completer.offer("--branches");
            completer.offer("--worktrees");
            if let Ok(branches) = git::git_merged_branches(&repo.get_path()).await {
                completer.offer_many(branches.iter().unique().sorted());
            }
        }
    }
}

impl PruneCommand {
    async fn get_prune_branches(
        &self,
        repo: &engine::Repo,
        patterns: &[String],
    ) -> Result<Vec<String>, human_errors::Error> {
        let merged = match git::git_merged_branches(&repo.get_path()).await {
            Ok(merged) => merged,
            Err(e) => {
                return Err(human_errors::wrap_user(
                    e,
                    "Could not determine the branches that have been merged into the default branch.",
                    &[
                        "Make sure that you have a correctly configured `origin` and that you have run `git fetch` before running this command again.",
                    ],
                ));
            }
        };

        let default_branch = git::git_default_branch(&repo.get_path()).await.wrap_user_err(
            "Could not determine the default branch for your repository, this probably means that you do not have a synchronized `origin`.",
            &["Make sure that you have a correctly configured `origin` and that you have run `git fetch` before running this command again."],
        )?;

        let candidates: Vec<String> = merged
            .into_iter()
            .filter(|b| b != &default_branch)
            .unique()
            .collect();

        Ok(crate::search::matches_any(&patterns, candidates))
    }

    async fn get_prune_worktrees(
        &self,
        repo: &engine::Repo,
        patterns: &[String],
    ) -> Result<Vec<git::Worktree>, human_errors::Error> {
        let worktrees: Vec<git::Worktree> = git::git_worktree_list(&repo.get_path())
            .await?
            .into_iter()
            .skip(1)
            .collect();

        // Worktrees are matched against their branch name; a detached HEAD (no
        // branch) is never matched when patterns are provided.
        let matching_branches: std::collections::HashSet<String> = crate::search::matches_any(
            &patterns,
            worktrees.iter().filter_map(|w| w.branch.clone()),
        )
        .into_iter()
        .collect();

        let mut worktrees_to_remove = Vec::new();

        for worktree in worktrees {
            let matches = match &worktree.branch {
                Some(branch) => matching_branches.contains(branch),
                None => patterns.is_empty(),
            };

            if !matches {
                continue;
            }

            if git::git_worktree_is_clean(&worktree.path)
                .await
                .unwrap_or(false)
            {
                worktrees_to_remove.push(worktree);
            }
        }

        Ok(worktrees_to_remove)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::builder::CoreBuilderWithConfig;
    use crate::engine::*;
    use crate::tasks::Task;
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
    ///  - `feature/test2`
    async fn setup_test_repo_with_remote(
        core: CoreBuilderWithConfig,
        temp: &tempfile::TempDir,
    ) -> (Core, Repo) {
        let repo: Repo = Repo::new(
            "gh:sierrasoftworks/test-git-switch-command",
            temp.path().join("repo"),
        );

        let repo_path = repo.get_path();
        std::fs::create_dir_all(&repo_path).expect("create test repo dir");
        let core = core
            .with_mock_resolver(|mock| {
                let repo_path = repo_path.clone();
                mock.expect_get_current_repo().returning(move || {
                    Ok(Repo::new(
                        "gh:sierrasoftworks/test-git-switch-command",
                        repo_path.clone(),
                    ))
                });
            })
            .build();

        let origin_repo = Repo::new(
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

        let core = Core::builder().with_config_for_dev_directory(temp.path());

        let (core, repo) = setup_test_repo_with_remote(core, &temp).await;

        git::git_cmd(
            tokio::process::Command::new("git")
                .current_dir(repo.get_path())
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

        git::git_cmd(
            tokio::process::Command::new("git")
                .current_dir(repo.get_path())
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
        cmd.assert_run_successful(&core, &args).await;

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
    async fn prune_clean_worktree() {
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

        let worktree_path = temp.path().join("worktrees").join("clean");
        git::git_worktree_add(
            &repo.get_path(),
            &worktree_path,
            "feature/clean-worktree",
            true,
            None,
        )
        .await
        .unwrap();

        assert!(
            worktree_path.exists(),
            "the worktree should have been created"
        );

        let args: ArgMatches = cmd.app().get_matches_from(vec!["prune"]);
        cmd.assert_run_successful(&core, &args).await;

        assert!(
            console.to_string().contains("clean"),
            "the output should mention the worktree being removed"
        );

        assert!(
            !worktree_path.exists(),
            "the clean worktree should have been removed"
        );
        assert!(repo.valid(), "the repository should still be valid");
    }

    #[tokio::test]
    async fn prune_preserves_dirty_worktree() {
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

        let worktree_path = temp.path().join("worktrees").join("dirty");
        git::git_worktree_add(
            &repo.get_path(),
            &worktree_path,
            "feature/dirty-worktree",
            true,
            None,
        )
        .await
        .unwrap();

        // Leave an uncommitted change behind so the worktree must be preserved.
        std::fs::write(worktree_path.join("work-in-progress.txt"), "not done yet").unwrap();

        let args: ArgMatches = cmd.app().get_matches_from(vec!["prune"]);
        cmd.assert_run_successful(&core, &args).await;

        assert!(
            worktree_path.exists(),
            "a worktree with uncommitted changes must not be removed"
        );
        assert!(repo.valid(), "the repository should still be valid");
    }

    #[tokio::test]
    async fn prune_pattern_filters_branches() {
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

        git::git_cmd(
            tokio::process::Command::new("git")
                .current_dir(repo.get_path())
                .arg("merge")
                .arg("feature/test2"),
        )
        .await
        .unwrap();

        // A pattern which doesn't match the merged branch should leave everything intact.
        let args: ArgMatches = cmd.app().get_matches_from(vec!["prune", "does-not-match"]);
        cmd.assert_run_successful(&core, &args).await;

        let mut branches = git::git_branches(&repo.get_path()).await.unwrap();
        branches.sort();
        assert_eq!(
            branches,
            vec!["feature/test", "feature/test2", "main"],
            "no branches should be removed when the pattern doesn't match"
        );

        // A matching pattern should remove the merged branch.
        let args: ArgMatches = cmd.app().get_matches_from(vec!["prune", "test2"]);
        cmd.assert_run_successful(&core, &args).await;

        let mut branches = git::git_branches(&repo.get_path()).await.unwrap();
        branches.sort();
        assert_eq!(
            branches,
            vec!["feature/test", "main"],
            "only the branch matching the pattern should be removed"
        );
    }

    #[tokio::test]
    async fn prune_branches_only_skips_worktrees() {
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

        git::git_cmd(
            tokio::process::Command::new("git")
                .current_dir(repo.get_path())
                .arg("merge")
                .arg("feature/test2"),
        )
        .await
        .unwrap();

        let worktree_path = temp.path().join("worktrees").join("clean");
        git::git_worktree_add(
            &repo.get_path(),
            &worktree_path,
            "feature/clean-worktree",
            true,
            None,
        )
        .await
        .unwrap();

        let args: ArgMatches = cmd.app().get_matches_from(vec!["prune", "--branches"]);
        cmd.assert_run_successful(&core, &args).await;

        assert!(
            worktree_path.exists(),
            "worktrees should be left untouched when --branches is specified"
        );

        let mut branches = git::git_branches(&repo.get_path()).await.unwrap();
        branches.sort();
        assert!(
            !branches.contains(&"feature/test2".to_string()),
            "the merged branch should have been removed"
        );
    }

    #[tokio::test]
    async fn prune_worktrees_only_skips_branches() {
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

        git::git_cmd(
            tokio::process::Command::new("git")
                .current_dir(repo.get_path())
                .arg("merge")
                .arg("feature/test2"),
        )
        .await
        .unwrap();

        let worktree_path = temp.path().join("worktrees").join("clean");
        git::git_worktree_add(
            &repo.get_path(),
            &worktree_path,
            "feature/clean-worktree",
            true,
            None,
        )
        .await
        .unwrap();

        let args: ArgMatches = cmd.app().get_matches_from(vec!["prune", "--worktrees"]);
        cmd.assert_run_successful(&core, &args).await;

        assert!(
            !worktree_path.exists(),
            "the clean worktree should have been removed"
        );

        let mut branches = git::git_branches(&repo.get_path()).await.unwrap();
        branches.sort();
        assert!(
            branches.contains(&"feature/test2".to_string()),
            "merged branches should be left untouched when --worktrees is specified"
        );
    }

    #[tokio::test]
    async fn prune_pattern_filters_worktrees() {
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

        let keep_path = temp.path().join("worktrees").join("keep");
        git::git_worktree_add(&repo.get_path(), &keep_path, "feature/keep-me", true, None)
            .await
            .unwrap();

        let remove_path = temp.path().join("worktrees").join("remove");
        git::git_worktree_add(
            &repo.get_path(),
            &remove_path,
            "feature/remove-me",
            true,
            None,
        )
        .await
        .unwrap();

        let args: ArgMatches = cmd.app().get_matches_from(vec!["prune", "remove-me"]);
        cmd.assert_run_successful(&core, &args).await;

        assert!(
            keep_path.exists(),
            "worktrees that don't match the pattern should be preserved"
        );
        assert!(
            !remove_path.exists(),
            "worktrees matching the pattern should be removed"
        );
    }

    #[tokio::test]
    async fn prune_bare_repo() {
        let cmd: PruneCommand = PruneCommand {};

        let temp = tempdir().unwrap();
        let repo: Repo = Repo::new(
            "gh:sierrasoftworks/test-git-prune-command",
            temp.path().join("repo"),
        );

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_mock_resolver(|mock| {
                let temp_path = temp.path().to_path_buf();
                mock.expect_get_current_repo().returning(move || {
                    Ok(Repo::new(
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
        assert!(
            git::git_branches(&repo.get_path())
                .await
                .unwrap()
                .is_empty()
        );
    }
}
