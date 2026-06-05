use super::*;
use crate::engine::{Identifier, Repo, Target};
use crate::errors::HumanErrorResultExt;
use crate::git;
use crate::tasks::*;
use clap::Arg;
use human_errors::ResultExt;
use itertools::Itertools;
use tracing_batteries::prelude::*;

pub struct WorktreeCommand;
crate::command!(WorktreeCommand);

#[async_trait]
impl CommandRunnable for WorktreeCommand {
    fn name(&self) -> String {
        String::from("worktree")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .visible_aliases(["w", "wt"])
            .about("opens a git worktree for a branch using an application defined in your config")
            .long_about("This command prepares a git worktree for the specified branch and launches an application within it. It behaves like a combination of the `switch` and `open` commands: you can run it from anywhere by specifying a repository (`gt w <repo> <branch> [app]`), or from within a repository to operate on the current repo (`gt w <branch> [app]`).

If the base repository has not been cloned yet, it will be cloned automatically. Worktrees are created within the worktree directory configured in your config file (defaulting to `$DEV_DIRECTORY/worktrees`). When the requested branch does not exist yet, it will be created (use `--no-create` to disable this). The branch a new worktree is based on can be controlled with `--base`.")
            .arg(Arg::new("repo")
                    .help("The repository or branch to open a worktree for.")
                    .index(1))
            .arg(Arg::new("branch")
                    .help("The branch or application to use.")
                    .index(2))
            .arg(Arg::new("app")
                    .help("The application to launch.")
                    .index(3))
            .arg(Arg::new("no-create")
                    .short('N')
                    .long("no-create")
                    .help("don't create the branch if it doesn't exist.")
                    .action(clap::ArgAction::SetTrue))
            .arg(Arg::new("base")
                    .long("base")
                    .help("the branch that a newly created worktree branch should be based on.")
                    .action(clap::ArgAction::Set))
            .arg(Arg::new("rm")
                    .long("rm")
                    .help("remove the worktree once the launched application exits.")
                    .action(clap::ArgAction::SetTrue))
    }

    #[tracing::instrument(name = "gt worktree", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, human_errors::Error> {
        let first = matches.get_one::<String>("repo");
        let second = matches.get_one::<String>("branch");
        let third = matches.get_one::<String>("app");
        let no_create = matches.get_flag("no-create");
        let base = matches.get_one::<String>("base");
        let remove_after = matches.get_flag("rm");

        // Positional arguments are always filled in order, so collapsing them into
        // a contiguous slice lets us reason about how many were provided.
        let provided: Vec<&String> = [first, second, third].into_iter().flatten().collect();

        let current_repo = core.resolver().get_current_repo();

        // Resolve the base repository, the branch, and the requested application
        // following a context-first strategy.
        let (repo, branch, app_arg): (Repo, Option<String>, Option<String>) = if let Ok(current) =
            current_repo
        {
            match provided.as_slice() {
                // gt w <repo> <branch> <app>
                [repo, branch, app] => (
                    self.resolve_repo(core, repo)?,
                    Some((*branch).clone()),
                    Some((*app).clone()),
                ),
                // gt w <value> [second]
                [value, rest @ ..] => {
                    let maybe_second = rest.first().copied();
                    let second_is_app = maybe_second
                        .map(|v| core.config().get_app(v).is_some())
                        .unwrap_or(false);

                    if !second_is_app {
                        if let Ok(repo) = self.resolve_repo(core, value) {
                            // gt w <repo> [branch]
                            (repo, maybe_second.cloned(), None)
                        } else {
                            // gt w <branch> [app]
                            (current, Some((*value).clone()), maybe_second.cloned())
                        }
                    } else {
                        // gt w <branch> <app>
                        (current, Some((*value).clone()), maybe_second.cloned())
                    }
                }
                // gt w (no arguments) -> list worktrees for the current repo
                [] => (current, None, None),
            }
        } else {
            match provided.as_slice() {
                [repo, rest @ ..] => (
                    self.resolve_repo(core, repo)?,
                    rest.first().map(|v| (*v).clone()),
                    rest.get(1).map(|v| (*v).clone()),
                ),
                [] => {
                    return Err(human_errors::user(
                        "You did not specify the name of a repository to use.",
                        &[
                            "Remember to specify a repository name like this: 'git-tool worktree github.com/sierrasoftworks/git-tool feature/example'.",
                        ],
                    ));
                }
            }
        };

        // Without a branch we list the existing worktrees for the repository.
        let branch = match branch {
            Some(branch) => branch,
            None => {
                if !repo.exists() {
                    return Err(human_errors::user(
                        format!(
                            "The repository '{}' has not been cloned yet, so it has no worktrees.",
                            repo.get_full_name()
                        ),
                        &[
                            "Specify a branch name to create a worktree, like this: 'git-tool worktree feature/example'.",
                        ],
                    ));
                }

                let worktrees = git::git_worktree_list(&repo.get_path()).await?;
                let mut output = core.output();
                // `git worktree list` always reports the primary working tree
                // first, followed by each linked worktree, so we can rely on the
                // ordering to label the primary checkout.
                for (index, worktree) in worktrees.iter().enumerate() {
                    let label = match &worktree.branch {
                        Some(branch) => branch.clone(),
                        None => match &worktree.head {
                            Some(head) => {
                                format!("(detached HEAD {})", &head[..head.len().min(8)])
                            }
                            None => "(detached HEAD)".to_string(),
                        },
                    };

                    let suffix = if index == 0 { " [primary]" } else { "" };
                    writeln!(output, "{label}{suffix}").to_human_error()?;
                }

                return Ok(0);
            }
        };

        let app = match app_arg {
            Some(name) => core.config().get_app(&name).ok_or_else(|| {
                human_errors::user(
                    format!("Could not find application with name '{name}'."),
                    &["Check your configuration file for available applications, or install one with 'git-tool config add apps/bash'."],
                )
            })?,
            None => core.config().get_default_app().ok_or_else(|| {
                human_errors::user(
                    "No default application available.",
                    &["Make sure that you add an app to your config file using 'git-tool config add apps/bash' or similar."],
                )
            })?,
        };

        // Ensure the base repository has been cloned before creating a worktree.
        if !repo.exists() {
            sequence![GitClone::default()]
                .apply_repo(core, &repo)
                .await?;
        }

        let worktree_path = core
            .config()
            .get_worktree_directory()
            .join(Self::worktree_dir_name(&repo, &branch));

        GitWorktree {
            path: worktree_path.clone(),
            branch: branch.clone(),
            create_if_missing: !no_create,
            base: base.cloned(),
        }
        .apply_repo(core, &repo)
        .await?;

        let worktree_target = Repo::new(
            &format!("{}:{}", repo.service, repo.get_full_name()),
            worktree_path.clone(),
        );

        // Apply any worktree automation (symlinks and setup tasks) defined in the
        // repository's 'git-tool.yml' file before launching the application. The
        // configuration must be trusted before we run any of its tasks; if the
        // user declines we simply skip the automation and continue.
        if let Some(repo_config) = crate::engine::RepoConfig::for_repo(&repo)?
            && repo_config.worktree().is_some()
            && crate::commands::trust::ensure_trusted(core, &repo, &repo_config).await?
        {
            self.apply_worktree_automation(core, &repo, &worktree_target, &repo_config)
                .await?;
        }

        let result = core.launcher().run(app, &worktree_target).await;

        // When requested, remove the worktree now that the launched application has
        // exited. We always attempt cleanup (even if the application failed), but we
        // never force the removal: if there are uncommitted changes git will refuse
        // to delete the worktree and we surface that to the user so they don't lose
        // any work.
        if remove_after {
            let cleanup = git::git_worktree_remove(&repo.get_path(), &worktree_path).await;

            // Surface the application's failure first, since that's the user's
            // primary concern, before reporting any cleanup problems.
            let status = result?;

            cleanup.wrap_user_err(
                format!(
                    "Git-Tool could not remove the worktree at '{}' because it still contains changes which haven't been committed.",
                    worktree_path.display()
                ),
                &[
                    "Commit or discard your changes within the worktree and then remove it manually with 'git worktree remove <path>'.",
                ],
            )?;

            return Ok(status);
        }

        Ok(result?)
    }

    #[tracing::instrument(
        name = "gt complete -- gt worktree",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer("--no-create");
        completer.offer("--base");
        completer.offer("--rm");
        completer.offer_aliases(core);
        completer.offer_apps(core);
        completer.offer_repos(core);

        if let Ok(repo) = core.resolver().get_current_repo()
            && let Ok(branches) = git::git_branches(&repo.get_path()).await
        {
            completer.offer_many(branches.iter().unique().sorted());
        }
    }
}

impl WorktreeCommand {
    fn resolve_repo(&self, core: &Core, value: &str) -> Result<Repo, human_errors::Error> {
        let identifier: Identifier = value.parse()?;
        core.resolver().get_best_repo(&identifier)
    }

    /// Applies the worktree automation defined in a repository's 'git-tool.yml'
    /// configuration: it creates the requested symlinks from the worktree back to
    /// the original repository and then runs the configured setup tasks within the
    /// worktree. Symlink and task failures are surfaced as warnings rather than
    /// hard errors, since the worktree itself has already been created and the user
    /// can continue working in it.
    async fn apply_worktree_automation(
        &self,
        core: &Core,
        repo: &Repo,
        worktree_target: &Repo,
        config: &crate::engine::RepoConfig,
    ) -> Result<(), human_errors::Error> {
        let worktree = match config.worktree() {
            Some(worktree) => worktree,
            None => return Ok(()),
        };

        for entry in worktree.symlinks() {
            let original = repo.get_path().join(entry);
            let link = worktree_target.get_path().join(entry);

            if link.exists() {
                continue;
            }

            if !original.exists() {
                writeln!(
                    core.output(),
                    "Warning: skipping worktree symlink '{entry}' because it does not exist in the source repository."
                )
                .to_human_error()?;
                continue;
            }

            if let Err(err) = crate::fs::create_link(&original, &link) {
                writeln!(
                    core.output(),
                    "Warning: could not create worktree symlink '{entry}': {}",
                    err.message()
                )
                .to_human_error()?;
            }
        }

        for task_name in worktree.tasks() {
            match config.get_task(task_name) {
                Some(task) => {
                    let app = task.to_app(task_name);
                    if let Err(err) = core.launcher().run(&app, worktree_target).await {
                        writeln!(
                            core.output(),
                            "Warning: worktree task '{task_name}' failed: {}",
                            err.message()
                        )
                        .to_human_error()?;
                    }
                }
                None => {
                    writeln!(
                        core.output(),
                        "Warning: worktree task '{task_name}' is not defined in the repository configuration."
                    )
                    .to_human_error()?;
                }
            }
        }

        Ok(())
    }

    /// Builds the directory name used to store a worktree for the given repository
    /// and branch. The repository's short name and the (sanitized) branch keep the
    /// path human-readable, while an 8 character hash of the repository's full
    /// identity disambiguates repositories that share the same short name (for
    /// example `org-a/tools` and `org-b/tools`).
    fn worktree_dir_name(repo: &Repo, branch: &str) -> String {
        let sanitized_branch: String = branch
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
    fn short_hash(value: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{builder::CoreBuilderWithConfig, *};
    use tempfile::tempdir;

    #[test]
    fn worktree_dir_name_sanitizes_branch() {
        let repo = Repo::new("gh:sierrasoftworks/git-tool", "/dev/git-tool".into());

        // The human-readable prefix is the repo's short name and the sanitized
        // branch, followed by an 8 character hash of the repo's full identity.
        assert!(
            WorktreeCommand::worktree_dir_name(&repo, "feat/forgejo")
                .starts_with("git-tool-feat-forgejo-")
        );
        assert!(
            WorktreeCommand::worktree_dir_name(&repo, "release/v1.2.3")
                .starts_with("git-tool-release-v1.2.3-")
        );
    }

    #[test]
    fn worktree_dir_name_disambiguates_repositories() {
        let repo_a = Repo::new("gh:org-a/tools", "/dev/a/tools".into());
        let repo_b = Repo::new("gh:org-b/tools", "/dev/b/tools".into());

        // Two repositories that share a short name must map to distinct worktree
        // directories thanks to the identity hash suffix.
        assert_ne!(
            WorktreeCommand::worktree_dir_name(&repo_a, "main"),
            WorktreeCommand::worktree_dir_name(&repo_b, "main")
        );

        // The hash must be stable across invocations for a given repository.
        assert_eq!(
            WorktreeCommand::worktree_dir_name(&repo_a, "main"),
            WorktreeCommand::worktree_dir_name(&repo_a, "main")
        );
    }

    #[test]
    fn short_hash_is_eight_hex_characters() {
        let hash = WorktreeCommand::short_hash("gh:sierrasoftworks/git-tool");
        assert_eq!(hash.len(), 8);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[tokio::test]
    async fn apply_worktree_automation_creates_symlinks_and_runs_tasks() {
        let cmd = WorktreeCommand {};
        let temp = tempdir().unwrap();

        // The source repository contains a 'target' directory (to be symlinked)
        // and a git-tool.yml describing the worktree automation.
        let repo = Repo::new(
            "gh:sierrasoftworks/test-worktree-automation",
            temp.path().join("repo"),
        );
        std::fs::create_dir_all(repo.get_path().join("target")).unwrap();
        std::fs::write(repo.get_path().join("target").join("marker.txt"), "built").unwrap();
        std::fs::write(
            repo.get_path().join("git-tool.yml"),
            "worktree:\n  symlinks:\n    - target\n  tasks:\n    - setup\ntasks:\n  setup:\n    command: echo\n    args:\n      - setup\n",
        )
        .unwrap();

        let worktree_target = Repo::new(
            "gh:sierrasoftworks/test-worktree-automation",
            temp.path().join("worktree"),
        );
        std::fs::create_dir_all(worktree_target.get_path()).unwrap();

        let repo_config = RepoConfig::for_repo(&repo).unwrap().unwrap();

        let core = Core::builder()
            .with_config(
                Config::for_dev_directory(temp.path())
                    .with_trusted_repo(repo.to_string(), repo_config.hash().unwrap()),
            )
            .with_null_console()
            .with_mock_launcher(|mock| {
                mock.expect_run()
                    .withf(|app, _| app.get_name() == "setup")
                    .times(1)
                    .returning(|_, _| Box::pin(async { Ok(0) }));
            })
            .build();

        cmd.apply_worktree_automation(&core, &repo, &worktree_target, &repo_config)
            .await
            .unwrap();

        let link = worktree_target.get_path().join("target");
        assert!(link.exists(), "the symlink should have been created");
        assert_eq!(
            std::fs::read_to_string(link.join("marker.txt")).unwrap(),
            "built"
        );
    }

    /// Initializes a real git repository (with an initial commit) and configures
    /// it to be returned as the current repository by the resolver.
    async fn setup_current_repo(
        core: CoreBuilderWithConfig,
        temp: &tempfile::TempDir,
    ) -> (CoreBuilderWithConfig, Repo) {
        let repo = Repo::new(
            "gh:sierrasoftworks/test-worktree-command",
            temp.path().join("repo"),
        );

        let repo_path = repo.get_path();
        let core = core.with_mock_resolver(move |mock| {
            let repo_path = repo_path.clone();
            mock.expect_get_current_repo().returning(move || {
                Ok(Repo::new(
                    "gh:sierrasoftworks/test-worktree-command",
                    repo_path.clone(),
                ))
            });

            // Branch names do not resolve to repositories, so the resolver
            // reports that no matching repository could be found.
            mock.expect_get_best_repo().returning(|identifier| {
                Err(human_errors::user(
                    format!("No repository matched '{identifier}'."),
                    &["Specify a valid repository name."],
                ))
            });
        });

        (core, repo)
    }

    async fn commit_initial(repo: &Repo) {
        let path = repo.get_path();
        git::git_config_set(&path, "user.name", "Example User")
            .await
            .unwrap();
        git::git_config_set(&path, "user.email", "user@example.com")
            .await
            .unwrap();
        std::fs::write(path.join("README.md"), "testing").unwrap();
        git::git_add(&path, &vec!["README.md"]).await.unwrap();
        git::git_commit(&path, "Initial commit", &vec!["README.md"])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn run_in_repo_creates_worktree() {
        let cmd = WorktreeCommand {};
        let temp = tempdir().unwrap();

        let cfg = Config::for_dev_directory(temp.path());
        let core = Core::builder().with_config(cfg);
        let (core, repo) = setup_current_repo(core, &temp).await;

        let expected_path = temp
            .path()
            .join("worktrees")
            .join(WorktreeCommand::worktree_dir_name(&repo, "feature/test"));
        let expected_for_assert = expected_path.clone();

        let core = core
            .with_mock_launcher(move |mock| {
                let expected_path = expected_path.clone();
                mock.expect_run()
                    .withf(move |app, target| {
                        app.get_name() == "shell" && target.get_path() == expected_path
                    })
                    .returning(|_, _| Box::pin(async { Ok(7) }));
            })
            .build();

        sequence!(GitInit {}, GitCheckout { branch: "main" })
            .apply_repo(&core, &repo)
            .await
            .unwrap();
        commit_initial(&repo).await;

        let args = cmd.app().get_matches_from(vec!["worktree", "feature/test"]);
        let status = cmd.run(&core, &args).await.unwrap();

        assert_eq!(status, 7, "the launcher status code should be forwarded");
        assert!(
            expected_for_assert.join(".git").exists(),
            "the worktree should have been created"
        );
        assert_eq!(
            git::git_current_branch(&expected_for_assert).await.unwrap(),
            "feature/test"
        );
    }

    #[tokio::test]
    async fn run_in_repo_removes_worktree_after_exit() {
        let cmd = WorktreeCommand {};
        let temp = tempdir().unwrap();

        let cfg = Config::for_dev_directory(temp.path());
        let core = Core::builder().with_config(cfg);
        let (core, repo) = setup_current_repo(core, &temp).await;

        let expected_path = temp
            .path()
            .join("worktrees")
            .join(WorktreeCommand::worktree_dir_name(&repo, "feature/test"));
        let expected_for_assert = expected_path.clone();

        let core = core
            .with_mock_launcher(move |mock| {
                let expected_path = expected_path.clone();
                mock.expect_run()
                    .withf(move |app, target| {
                        // The worktree must still exist while the application runs.
                        app.get_name() == "shell"
                            && target.get_path() == expected_path
                            && expected_path.join(".git").exists()
                    })
                    .returning(|_, _| Box::pin(async { Ok(0) }));
            })
            .build();

        sequence!(GitInit {}, GitCheckout { branch: "main" })
            .apply_repo(&core, &repo)
            .await
            .unwrap();
        commit_initial(&repo).await;

        let args = cmd
            .app()
            .get_matches_from(vec!["worktree", "feature/test", "--rm"]);
        let status = cmd.run(&core, &args).await.unwrap();

        assert_eq!(status, 0, "the launcher status code should be forwarded");
        assert!(
            !expected_for_assert.exists(),
            "the worktree should have been removed after the application exited"
        );
    }

    #[tokio::test]
    async fn run_with_rm_cleans_up_after_launcher_failure() {
        let cmd = WorktreeCommand {};
        let temp = tempdir().unwrap();

        let cfg = Config::for_dev_directory(temp.path());
        let core = Core::builder().with_config(cfg);
        let (core, repo) = setup_current_repo(core, &temp).await;

        let expected_path = temp
            .path()
            .join("worktrees")
            .join(WorktreeCommand::worktree_dir_name(&repo, "feature/boom"));
        let expected_for_assert = expected_path.clone();

        let core = core
            .with_mock_launcher(|mock| {
                mock.expect_run().returning(|_, _| {
                    Box::pin(async {
                        Err(human_errors::user(
                            "The application exited with an error.",
                            &["Try running the command again."],
                        ))
                    })
                });
            })
            .build();

        sequence!(GitInit {}, GitCheckout { branch: "main" })
            .apply_repo(&core, &repo)
            .await
            .unwrap();
        commit_initial(&repo).await;

        let args = cmd
            .app()
            .get_matches_from(vec!["worktree", "feature/boom", "--rm"]);
        cmd.run(&core, &args)
            .await
            .expect_err("the launcher error should be surfaced to the caller");

        assert!(
            !expected_for_assert.exists(),
            "the worktree should still be cleaned up even when the application fails"
        );
    }

    #[tokio::test]
    async fn run_with_rm_preserves_dirty_worktree() {
        let cmd = WorktreeCommand {};
        let temp = tempdir().unwrap();

        let cfg = Config::for_dev_directory(temp.path());
        let core = Core::builder().with_config(cfg);
        let (core, repo) = setup_current_repo(core, &temp).await;

        let expected_path = temp
            .path()
            .join("worktrees")
            .join(WorktreeCommand::worktree_dir_name(&repo, "feature/dirty"));
        let expected_for_assert = expected_path.clone();

        let core = core
            .with_mock_launcher(move |mock| {
                let expected_path = expected_path.clone();
                mock.expect_run().returning(move |_, _| {
                    // Leave an uncommitted file behind so that removal (without
                    // --force) must refuse to delete the worktree.
                    std::fs::write(expected_path.join("uncommitted.txt"), "work in progress")
                        .unwrap();
                    Box::pin(async { Ok(0) })
                });
            })
            .build();

        sequence!(GitInit {}, GitCheckout { branch: "main" })
            .apply_repo(&core, &repo)
            .await
            .unwrap();
        commit_initial(&repo).await;

        let args = cmd
            .app()
            .get_matches_from(vec!["worktree", "feature/dirty", "--rm"]);
        cmd.run(&core, &args)
            .await
            .expect_err("removal should fail when the worktree has uncommitted changes");

        assert!(
            expected_for_assert.exists(),
            "the worktree must be preserved so the user does not lose their uncommitted work"
        );
    }

    #[tokio::test]
    async fn warns_when_base_ignored_for_existing_branch() {
        let cmd = WorktreeCommand {};
        let temp = tempdir().unwrap();
        let console = crate::console::mock();

        let cfg = Config::for_dev_directory(temp.path());
        let core = Core::builder()
            .with_config(cfg)
            .with_console(console.clone());
        let (core, repo) = setup_current_repo(core, &temp).await;

        let core = core
            .with_mock_launcher(|mock| {
                mock.expect_run()
                    .returning(|_, _| Box::pin(async { Ok(0) }));
            })
            .build();

        sequence!(GitInit {}, GitCheckout { branch: "main" })
            .apply_repo(&core, &repo)
            .await
            .unwrap();
        commit_initial(&repo).await;

        // Create a second branch that is not currently checked out, so that a
        // worktree can be created for it.
        git::git_checkout(&repo.get_path(), "feature/existing")
            .await
            .unwrap();
        git::git_checkout(&repo.get_path(), "main").await.unwrap();

        // 'feature/existing' already exists, so the requested --base should be
        // ignored and the user warned about it.
        let args =
            cmd.app()
                .get_matches_from(vec!["worktree", "feature/existing", "--base", "develop"]);
        cmd.run(&core, &args).await.unwrap();

        assert!(
            console.to_string().contains("ignoring '--base'"),
            "a warning should be printed when --base is ignored for an existing branch"
        );
    }

    #[tokio::test]
    async fn run_in_repo_lists_worktrees() {
        let cmd = WorktreeCommand {};
        let temp = tempdir().unwrap();
        let console = crate::console::mock();

        let cfg = Config::for_dev_directory(temp.path());
        let core = Core::builder()
            .with_config(cfg)
            .with_console(console.clone());
        let (core, repo) = setup_current_repo(core, &temp).await;
        let core = core.build();

        sequence!(GitInit {}, GitCheckout { branch: "main" })
            .apply_repo(&core, &repo)
            .await
            .unwrap();
        commit_initial(&repo).await;

        let args = cmd.app().get_matches_from(vec!["worktree"]);
        let status = cmd.run(&core, &args).await.unwrap();
        assert_eq!(status, 0, "listing worktrees should succeed");
        assert!(
            console.to_string().contains("[primary]"),
            "the primary worktree should be labelled in the listing"
        );
    }
}
