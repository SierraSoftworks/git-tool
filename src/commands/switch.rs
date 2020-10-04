use super::super::errors;
use super::*;
use crate::core::Target;
use crate::git;
use crate::tasks::*;
use clap::{App, Arg};

pub struct SwitchCommand {}

impl Command for SwitchCommand {
    fn name(&self) -> String {
        String::from("switch")
    }

    fn app<'a>(&self) -> App<'a> {
        App::new(self.name().as_str())
            .version("1.0")
            .alias("s")
            .about("switches to the specified branch.")
            .long_about(
                "This command switches to the specified branch within the current repository.",
            )
            .arg(
                Arg::new("branch")
                    .about("The name of the branch to switch to.")
                    .index(1),
            )
            .arg(
                Arg::new("create")
                    .short('c')
                    .long("create")
                    .about("creates a new branch before switching to it."),
            )
    }
}

#[async_trait]
impl<C: Core> CommandRunnable<C> for SwitchCommand {
    async fn run(&self, core: &C, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let repo = core.resolver().get_current_repo()?;

        match matches.value_of("branch") {
            Some(branch) => {
                let task = tasks::GitSwitch {
                    branch: branch.to_string(),
                    create_if_missing: matches.is_present("create"),
                };

                task.apply_repo(core, &repo).await?;
            }
            None => {
                let branches = git::git_branches(&repo.get_path()).await?;
                for branch in branches {
                    println!("{}", branch);
                }
            }
        };

        Ok(0)
    }

    async fn complete(&self, core: &C, completer: &Completer, _matches: &ArgMatches) {
        if let Ok(repo) = core.resolver().get_current_repo() {
            completer.offer("--create");
            if let Ok(branches) = git::git_branches(&repo.get_path()).await {
                completer.offer_many(branches);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::core::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn switch_branch_exists() {
        let cmd: SwitchCommand = SwitchCommand {};

        let temp = tempdir().unwrap();
        let repo: Repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-switch-command",
            temp.path().join("repo").into(),
        );

        let core = core::CoreBuilder::default()
            .with_config(&core::Config::for_dev_directory(temp.path()))
            .with_mock_resolver(|r| r.set_repo(repo.clone()))
            .build();

        sequence!(
            // Run a `git init` to setup the repo
            tasks::GitInit {},
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
        .apply_repo(&core, &repo)
        .await
        .unwrap();

        assert!(repo.valid(), "the repository should exist and be valid");

        let args: ArgMatches = cmd.app().get_matches_from(vec!["switch", "feature/test"]);
        cmd.run(&core, &args).await.unwrap();

        assert!(repo.valid(), "the repository should still be valid");
        assert_eq!(
            git::git_current_branch(&repo.get_path()).await.unwrap(),
            "feature/test"
        );
    }

    #[tokio::test]
    async fn switch_branch_not_exists() {
        let cmd: SwitchCommand = SwitchCommand {};

        let temp = tempdir().unwrap();
        let repo: Repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-switch-command",
            temp.path().join("repo").into(),
        );

        let core = core::CoreBuilder::default()
            .with_config(&core::Config::for_dev_directory(temp.path()))
            .with_mock_resolver(|r| r.set_repo(repo.clone()))
            .build();

        // Run a `git init` to setup the repo
        tasks::GitInit {}.apply_repo(&core, &repo).await.unwrap();

        assert!(repo.valid(), "the repository should exist and be valid");

        let args: ArgMatches = cmd.app().get_matches_from(vec!["switch", "feature/test"]);
        cmd.run(&core, &args)
            .await
            .expect_err("this command should not have succeeded");
    }

    #[tokio::test]
    async fn switch_create_branch() {
        let cmd: SwitchCommand = SwitchCommand {};

        let temp = tempdir().unwrap();
        let repo: Repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-switch-command",
            temp.path().join("repo").into(),
        );

        let core = core::CoreBuilder::default()
            .with_config(&core::Config::for_dev_directory(temp.path()))
            .with_mock_resolver(|r| r.set_repo(repo.clone()))
            .build();

        // Run a `git init` to setup the repo
        tasks::GitInit {}.apply_repo(&core, &repo).await.unwrap();

        assert!(repo.valid(), "the repository should exist and be valid");

        let args: ArgMatches = cmd
            .app()
            .get_matches_from(vec!["switch", "-c", "feature/test"]);
        cmd.run(&core, &args).await.unwrap();

        assert!(repo.valid(), "the repository should still be valid");
        assert_eq!(
            git::git_current_branch(&repo.get_path()).await.unwrap(),
            "feature/test"
        );
    }
}
