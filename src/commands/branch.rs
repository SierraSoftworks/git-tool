use super::super::errors;
use super::*;
use crate::core::Target;
use crate::git;
use crate::tasks::*;
use clap::{App, Arg};

pub struct BranchCommand {}

impl Command for BranchCommand {
    fn name(&self) -> String {
        String::from("branch")
    }

    fn app<'a>(&self) -> App<'a> {
        App::new(self.name().as_str())
            .version("1.0")
            .visible_aliases(&vec!["b", "br"])
            .about("checkout a specific branch from the current repository")
            .long_about(
                "This tool checks out a branch with the given name from the current repository.",
            )
            .arg(
                Arg::new("branch")
                    .index(1)
                    .about("The name of the branch you want to checkout"),
            )
    }
}

#[async_trait]
impl<C: Core> CommandRunnable<C> for BranchCommand {
    async fn run(&self, core: &C, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let repo = core.resolver().get_current_repo()?;

        match matches.value_of("branch") {
            Some(branch) => {
                let task = tasks::GitCheckout { branch };

                task.apply_repo(core, &repo).await?;
            }
            None => {
                let branches = git::git_branches(&repo.get_path()).await?;
                let current_branch = git::git_current_branch(&repo.get_path()).await?;

                for branch in branches {
                    let prefix = if branch == current_branch { "* " } else { "  " };
                    writeln!(core.output(), "{}{}", prefix, branch)?;
                }
            }
        };

        Ok(0)
    }

    async fn complete(&self, core: &C, completer: &Completer, _matches: &ArgMatches) {
        if let Ok(repo) = core.resolver().get_current_repo() {
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
    use mocktopus::mocking::*;

    #[tokio::test]
    async fn checkout_branch_inside_repo() {
        let cmd = BranchCommand {};

        let temp = tempfile::tempdir().unwrap();

        let repo: Repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-checkout-command",
            temp.path().join("repo").into(),
        );

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        crate::console::output::mock();

        Resolver::get_current_repo.mock_safe(move |_| {
            MockResult::Return(Ok(core::Repo::new(
                "github.com/sierrasoftworks/test-git-checkout-command",
                temp.path().join("repo").into(),
            )))
        });

        // Run a `git init` to setup the repo
        tasks::GitInit {}.apply_repo(&core, &repo).await.unwrap();

        assert!(repo.valid(), "the repository should exist and be valid");

        let args: ArgMatches = cmd.app().get_matches_from(vec!["branch", "feature/test"]);
        cmd.run(&core, &args).await.unwrap();

        assert!(repo.valid(), "the repository should still be valid");
        assert_eq!(
            git::git_current_branch(&repo.get_path()).await.unwrap(),
            "feature/test"
        );
    }

    #[tokio::test]
    async fn checkout_branch_outside_repo() {
        let cmd = BranchCommand {};

        let temp = tempfile::tempdir().unwrap();

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        crate::console::output::mock();

        Resolver::get_current_repo.mock_safe(move |_| {
            MockResult::Return(Ok(Repo::new(
                "example.com/test/cmd-branch",
                temp.path().to_path_buf(),
            )))
        });

        let args: ArgMatches = cmd.app().get_matches_from(vec!["branch", "feature/test"]);

        cmd.run(&core, &args)
            .await
            .expect_err("this command should not have succeeded");
    }
}
