use super::super::errors;
use super::*;
use crate::core::Target;
use crate::git;
use crate::tasks::*;
use clap::Arg;
use itertools::Itertools;

pub struct SwitchCommand {}

impl Command for SwitchCommand {
    fn name(&self) -> String {
        String::from("switch")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("switches to the specified branch.")
            .visible_aliases(["sw", "branch", "b", "br"])
            .long_about(
                "This command switches to the specified branch within the current repository.",
            )
            .arg(
                Arg::new("branch")
                    .help("The name of the branch to switch to.")
                    .index(1),
            )
            .arg(
                Arg::new("no-create")
                    .short('N')
                    .long("no-create")
                    .help("don't create the branch if it doesn't exist.")
                    .action(clap::ArgAction::SetTrue),
            )
    }
}

#[async_trait]
impl CommandRunnable for SwitchCommand {
    #[tracing::instrument(name = "gt switch", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let repo = core.resolver().get_current_repo()?;

        match matches.get_one::<String>("branch") {
            Some(branch) => {
                let task = tasks::GitSwitch {
                    branch: branch.to_string(),
                    create_if_missing: !matches.get_flag("no-create"),
                };

                task.apply_repo(core, &repo).await?;
            }
            None => {
                let branches = git::git_branches(&repo.get_path()).await?;
                for branch in branches
                    .iter()
                    .map(|v| Self::to_local_branch_name(v))
                    .unique()
                    .sorted()
                {
                    writeln!(core.output(), "{branch}")?;
                }
            }
        };

        Ok(0)
    }
    #[tracing::instrument(
        name = "gt complete -- gt switch",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        if let Ok(repo) = core.resolver().get_current_repo() {
            completer.offer("--create");
            if let Ok(branches) = git::git_branches(&repo.get_path()).await {
                completer.offer_many(
                    branches
                        .iter()
                        .map(|v| Self::to_local_branch_name(v))
                        .unique()
                        .sorted(),
                );
            }
        }
    }
}

impl SwitchCommand {
    fn to_local_branch_name(branch: &str) -> String {
        if let Some(short_branch) = branch.strip_prefix("origin/") {
            short_branch.to_owned()
        } else {
            branch.to_owned()
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::core::{builder::CoreBuilderWithConfig, *};
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
            tasks::GitCheckout { branch: "main" },
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
            }
        )
        .apply_repo(&core, &repo)
        .await
        .unwrap();

        assert!(repo.valid(), "the repository should exist and be valid");
        (core, repo)
    }

    #[tokio::test]
    async fn switch_completions() {
        let temp = tempdir().unwrap();

        let core = core::Core::builder().with_config_for_dev_directory(temp.path());

        let (core, _repo) = setup_test_repo_with_remote(core, &temp).await;

        test_completions_with_core(&core, "gt switch", "", vec!["main", "feature/test"]).await;
    }

    #[tokio::test]
    async fn switch_branch_exists() {
        let cmd: SwitchCommand = SwitchCommand {};

        let temp = tempdir().unwrap();

        let core = core::Core::builder().with_config_for_dev_directory(temp.path());

        let (core, repo) = setup_test_repo_with_remote(core, &temp).await;

        sequence!(
            tasks::GitCheckout {
                branch: "feature/test"
            },
            tasks::GitCheckout { branch: "main" }
        )
        .apply_repo(&core, &repo)
        .await
        .unwrap();

        let args: ArgMatches = cmd.app().get_matches_from(vec!["switch", "feature/test"]);
        cmd.run(&core, &args).await.unwrap();

        assert!(repo.valid(), "the repository should still be valid");
        assert_eq!(
            git::git_current_branch(&repo.get_path()).await.unwrap(),
            "feature/test"
        );
    }

    #[tokio::test]
    async fn switch_branch_exists_remote_only() {
        let cmd: SwitchCommand = SwitchCommand {};

        let temp = tempdir().unwrap();

        let core = core::Core::builder().with_config_for_dev_directory(temp.path());

        let (core, repo) = setup_test_repo_with_remote(core, &temp).await;

        let args: ArgMatches = cmd.app().get_matches_from(vec!["switch", "feature/test"]);
        cmd.run(&core, &args).await.unwrap();

        assert!(repo.valid(), "the repository should still be valid");
        assert_eq!(
            git::git_current_branch(&repo.get_path()).await.unwrap(),
            "feature/test"
        );

        let target_ref = git::git_rev_parse(&repo.get_path(), "origin/feature/test")
            .await
            .unwrap();

        let true_ref = git::git_rev_parse(&repo.get_path(), "feature/test")
            .await
            .unwrap();

        assert_eq!(
            true_ref, target_ref,
            "the current branch should be pointing at the original origin ref"
        );
    }

    #[tokio::test]
    async fn switch_no_create_branch_exists() {
        let cmd: SwitchCommand = SwitchCommand {};

        let temp = tempdir().unwrap();

        let core = core::Core::builder().with_config_for_dev_directory(temp.path());

        let (core, repo) = setup_test_repo_with_remote(core, &temp).await;

        let args: ArgMatches = cmd
            .app()
            .get_matches_from(vec!["switch", "-N", "feature/test"]);
        cmd.run(&core, &args)
            .await
            .expect("this command should have succeeded");

        assert_eq!(
            git::git_current_branch(&repo.get_path()).await.unwrap(),
            "feature/test"
        );
    }

    #[tokio::test]
    async fn switch_no_create() {
        let cmd: SwitchCommand = SwitchCommand {};

        let temp = tempdir().unwrap();
        let core = core::Core::builder().with_config_for_dev_directory(temp.path());

        let (core, _repo) = setup_test_repo_with_remote(core, &temp).await;

        let args: ArgMatches = cmd
            .app()
            .get_matches_from(vec!["switch", "-N", "feature/test2"]);
        cmd.run(&core, &args)
            .await
            .expect_err("this command should not have succeeded");
    }

    #[tokio::test]
    async fn switch_no_matching_branch() {
        let cmd: SwitchCommand = SwitchCommand {};

        let temp = tempdir().unwrap();

        let core = core::Core::builder().with_config_for_dev_directory(temp.path());

        let (core, repo) = setup_test_repo_with_remote(core, &temp).await;

        let args: ArgMatches = cmd.app().get_matches_from(vec!["switch", "feature/test2"]);
        cmd.run(&core, &args)
            .await
            .expect("this command should have succeeded");

        assert_eq!(
            git::git_current_branch(&repo.get_path()).await.unwrap(),
            "feature/test2"
        );
    }

    #[tokio::test]
    async fn switch_branch_bare_repo() {
        let cmd: SwitchCommand = SwitchCommand {};

        let temp = tempdir().unwrap();
        let repo: Repo = core::Repo::new(
            "gh:sierrasoftworks/test-git-switch-command",
            temp.path().join("repo"),
        );

        let repo_path = repo.get_path();
        let core = core::Core::builder()
            .with_config_for_dev_directory(temp.path())
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

        // Run a `git init` to setup the repo
        tasks::GitInit {}.apply_repo(&core, &repo).await.unwrap();

        assert!(repo.valid(), "the repository should exist and be valid");

        let args: ArgMatches = cmd.app().get_matches_from(vec!["switch", "feature/test"]);
        cmd.run(&core, &args).await.unwrap();

        assert!(repo.valid(), "the repository should still be valid");
        assert_eq!(
            git::git_current_branch(&repo.get_path()).await.unwrap(),
            "feature/test"
        );
    }
}
