use super::*;
use crate::engine::{Branch, Repo, Resolver, Target, Worktree};
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
            .long_about("This command prepares a git worktree for the specified branch of the current repository and launches an application within it. It behaves like a combination of the `switch` and `open` commands: run it from within a repository to operate on the current repo (`gt w <branch> [app]`). The branch is given first, optionally followed by an application; running the command with no arguments lists the repository's existing worktrees.

You may also append any number of KEY=VALUE tokens to override environment variables for the launched application (for example `gt w <branch> shell FOO=bar`). These overrides apply only to the launched application and are applied verbatim, taking precedence over any environment configured for the app.

Worktrees are created within the worktree directory configured in your config file (defaulting to `$DEV_DIRECTORY/worktrees`). When the requested branch does not exist yet, it will be created (use `--no-create` to disable this). The branch a new worktree is based on can be controlled with `--base`.")
            .arg(Arg::new("args")
                    .help("The branch to open a worktree for, an optional app to launch, and any KEY=VALUE environment overrides (in any order).")
                    .action(clap::ArgAction::Append))
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
        let no_create = matches.get_flag("no-create");
        let base = matches.get_one::<String>("base");
        let remove_after = matches.get_flag("rm");

        let repo: Repo = core.resolve(())?;

        let args: Vec<&str> = matches
            .get_many::<String>("args")
            .map(|values| values.map(|s| s.as_str()).collect())
            .unwrap_or_default();

        // Without any arguments we list the existing worktrees for the repository.
        if args.is_empty() {
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

        // A worktree always operates on an explicit branch, so there is no
        // implied context; a lone app-named token is therefore treated as the
        // branch.
        let parsed = crate::completion::parse::<Branch>(core, None, matches)?;
        let branch = &parsed.target;

        // Overrides apply only to the launched application, never to the
        // repository's worktree automation tasks.
        let app = parsed.launch_app(core)?;

        // Resolve the branch to the worktree it maps to within this repository.
        let worktree: Worktree = core.resolve((&repo, branch))?;
        let worktree_path = worktree.path();

        GitWorktree {
            path: worktree_path.clone(),
            branch: branch.to_string(),
            create_if_missing: !no_create,
            base: base.cloned(),
        }
        .apply_repo(core, &repo)
        .await?;

        // Apply any worktree automation (symlinks and setup tasks) defined in the
        // repository's 'git-tool.yml' file before launching the application. The
        // configuration must be trusted before we run any of its tasks; if the
        // user declines we simply skip the automation and continue.
        if let Some(repo_config) = crate::engine::RepoConfig::for_repo(&repo)?
            && repo_config.worktree().is_some()
            && crate::commands::trust::ensure_trusted(core, &repo, &repo_config).await?
        {
            self.apply_worktree_automation(core, &repo, &worktree, &repo_config)
                .await?;
        }

        let result = core.launcher().run(&app, &worktree).await;

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
                    "Commit or discard your changes within the worktree and then remove it manually with 'git-tool prune'.",
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
        completer.offer_apps(core);

        let repo: Result<Repo, _> = core.resolve(());
        if let Ok(repo) = repo
            && let Ok(branches) = git::git_branches(&repo.get_path()).await
        {
            completer.offer_many(branches.iter().unique().sorted());
        }
    }
}

impl WorktreeCommand {
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
        worktree_target: &Worktree,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{builder::CoreBuilderWithConfig, *};
    use tempfile::tempdir;

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

        let config = Config::for_dev_directory(temp.path());
        let branch: Branch = "feature/automation".parse().unwrap();
        let worktree_target = Worktree::new(&repo, &branch, &config);
        std::fs::create_dir_all(worktree_target.path()).unwrap();

        let repo_config = RepoConfig::for_repo(&repo).unwrap().unwrap();

        let core = Core::builder()
            .with_config(config)
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

        let link = worktree_target.path().join("target");
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
            .join(Worktree::dir_name(&repo, &"feature/test".parse().unwrap()));
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
    async fn run_in_repo_applies_env_override() {
        let cmd = WorktreeCommand {};
        let temp = tempdir().unwrap();

        let cfg = Config::for_dev_directory(temp.path());
        let core = Core::builder().with_config(cfg);
        let (core, repo) = setup_current_repo(core, &temp).await;

        let expected_path = temp
            .path()
            .join("worktrees")
            .join(Worktree::dir_name(&repo, &"feature/env".parse().unwrap()));

        let core = core
            .with_mock_launcher(move |mock| {
                let expected_path = expected_path.clone();
                mock.expect_run()
                    .withf(move |app, target| {
                        app.get_name() == "shell"
                            && target.get_path() == expected_path
                            && app
                                .get_overrides()
                                .contains(&("FOO".to_string(), "bar".to_string()))
                    })
                    .returning(|_, _| Box::pin(async { Ok(0) }));
            })
            .build();

        sequence!(GitInit {}, GitCheckout { branch: "main" })
            .apply_repo(&core, &repo)
            .await
            .unwrap();
        commit_initial(&repo).await;

        // `shell` is the default app; the branch is given first, then the app,
        // then the environment override.
        let args = cmd
            .app()
            .get_matches_from(vec!["worktree", "feature/env", "shell", "FOO=bar"]);
        cmd.run(&core, &args).await.unwrap();
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
            .join(Worktree::dir_name(&repo, &"feature/test".parse().unwrap()));
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
            .join(Worktree::dir_name(&repo, &"feature/boom".parse().unwrap()));
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
            .join(Worktree::dir_name(&repo, &"feature/dirty".parse().unwrap()));
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
