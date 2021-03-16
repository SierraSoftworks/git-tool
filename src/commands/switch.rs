use super::super::errors;
use super::*;
use crate::core::Target;
use crate::git;
use crate::tasks::*;
use clap::{App, Arg};
use itertools::Itertools;

pub struct SwitchCommand {}

impl Command for SwitchCommand {
    fn name(&self) -> String {
        String::from("switch")
    }

    fn app<'a>(&self) -> App<'a> {
        App::new(self.name().as_str())
            .version("1.0")
            .about("switches to the specified branch.")
            .visible_aliases(&vec!["sw", "branch", "b", "br"])
            .long_about(
                "This command switches to the specified branch within the current repository.",
            )
            .arg(
                Arg::new("branch")
                    .about("The name of the branch to switch to.")
                    .index(1),
            )
            .arg(
                Arg::new("no-create")
                    .short('N')
                    .long("no-create")
                    .about("don't create the branch if it doesn't exist."),
            )
    }
}

#[async_trait]
impl CommandRunnable for SwitchCommand {
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let repo = core.resolver().get_current_repo()?;

        match matches.value_of("branch") {
            Some(branch) => {
                let task = tasks::GitSwitch {
                    branch: branch.to_string(),
                    create_if_missing: !matches.is_present("no-create"),
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
                    println!("{}", branch);
                }
            }
        };

        Ok(0)
    }

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
        if branch.starts_with("origin/") {
            branch["origin/".len()..].to_owned()
        } else {
            branch.to_owned()
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::core::*;
    use complete::helpers::test_completions_with_core;
    use mocktopus::mocking::*;
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
    async fn setup_test_repo_with_remote(core: &Core, temp: &tempfile::TempDir) -> Repo {
        let repo: Repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-switch-command",
            temp.path().join("repo").into(),
        );

        let repo_path = repo.get_path();
        Resolver::get_current_repo.mock_safe(move |_| {
            MockResult::Return(Ok(core::Repo::new(
                "github.com/sierrasoftworks/test-git-switch-command",
                repo_path.clone(),
            )))
        });

        let origin_repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-switch-command2",
            temp.path().join("repo2").into(),
        );

        sequence!(
            // Run a `git init` to setup the repo
            tasks::GitInit {},
            tasks::GitRemote { name: "origin" },
            // Create the branch we want to switch to
            tasks::GitCheckout {
                branch: "feature/test".into(),
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
            tasks::GitCheckout {
                branch: "main".into(),
            }
        )
        .apply_repo(core, &origin_repo)
        .await
        .unwrap();

        sequence!(tasks::GitInit {}, tasks::GitRemote { name: "origin" })
            .apply_repo(core, &repo)
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
            tasks::GitCheckout {
                branch: "main".into(),
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
            }
        )
        .apply_repo(core, &repo)
        .await
        .unwrap();

        assert!(repo.valid(), "the repository should exist and be valid");
        repo
    }

    #[tokio::test]
    async fn switch_completions() {
        let temp = tempdir().unwrap();

        let core = core::Core::builder()
            .with_config(&core::Config::for_dev_directory(temp.path()))
            .build();

        let _repo: Repo = setup_test_repo_with_remote(&core, &temp).await;

        test_completions_with_core(&core, "gt switch", "", vec!["main", "feature/test"]).await;
    }

    #[tokio::test]
    async fn switch_branch_exists() {
        let cmd: SwitchCommand = SwitchCommand {};

        let temp = tempdir().unwrap();

        let core = core::Core::builder()
            .with_config(&core::Config::for_dev_directory(temp.path()))
            .build();

        let repo: Repo = setup_test_repo_with_remote(&core, &temp).await;

        sequence!(
            tasks::GitCheckout {
                branch: "feature/test".into()
            },
            tasks::GitCheckout {
                branch: "main".into()
            }
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

        let core = core::Core::builder()
            .with_config(&core::Config::for_dev_directory(temp.path()))
            .build();

        let repo: Repo = setup_test_repo_with_remote(&core, &temp).await;

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

        let core = core::Core::builder()
            .with_config(&core::Config::for_dev_directory(temp.path()))
            .build();

        let repo: Repo = setup_test_repo_with_remote(&core, &temp).await;

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
        let core = core::Core::builder()
            .with_config(&core::Config::for_dev_directory(temp.path()))
            .build();

        let repo: Repo = setup_test_repo_with_remote(&core, &temp).await;

        Resolver::get_current_repo.mock_safe(move |_| MockResult::Return(Ok(repo.clone())));

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

        let core = core::Core::builder()
            .with_config(&core::Config::for_dev_directory(temp.path()))
            .build();

        let repo: Repo = setup_test_repo_with_remote(&core, &temp).await;

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
            "github.com/sierrasoftworks/test-git-switch-command",
            temp.path().join("repo").into(),
        );

        let core = core::Core::builder()
            .with_config(&core::Config::for_dev_directory(&temp.path()))
            .build();

        Resolver::get_current_repo.mock_safe(move |_| {
            MockResult::Return(Ok(core::Repo::new(
                "github.com/sierrasoftworks/test-git-switch-command",
                temp.path().join("repo").into(),
            )))
        });

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
