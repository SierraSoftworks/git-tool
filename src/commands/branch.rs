use super::super::errors;
use super::*;
use crate::core::Target;
use crate::tasks::*;
use crate::{core, git};
use clap::{App, Arg, SubCommand};

pub struct BranchCommand {}

impl Command for BranchCommand {
    fn name(&self) -> String {
        String::from("branch")
    }

    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name().as_str())
            .version("1.0")
            .alias("b")
            .alias("br")
            .about("checkout a specific branch from the current repository")
            .help_message(
                "This tool checks out a branch with the given name from the current repository.",
            )
            .arg(
                Arg::with_name("branch")
                    .index(1)
                    .help("The name of the branch you want to checkout"),
            )
    }
}

#[async_trait]
impl<K: KeyChain, L: Launcher, R: Resolver> CommandRunnable<K, L, R> for BranchCommand {
    async fn run<'a>(
        &self,
        core: &core::Core<K, L, R>,
        matches: &ArgMatches<'a>,
    ) -> Result<i32, errors::Error> {
        let repo = core.resolver.get_current_repo()?;

        match matches.value_of("branch") {
            Some(branch) => {
                let task = tasks::GitCheckout {
                    branch: branch.to_string(),
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

    async fn complete<'a>(
        &self,
        core: &Core<K, L, R>,
        completer: &Completer,
        _matches: &ArgMatches<'a>,
    ) {
        if let Ok(repo) = core.resolver.get_current_repo() {
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
    use tempdir::TempDir;

    #[tokio::test]
    async fn checkout_branch_inside_repo() {
        let cmd = BranchCommand {};

        let temp = TempDir::new("gt-commands-branch").unwrap();

        let repo: Repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-checkout-command",
            temp.path().join("repo").into(),
        );

        let core = core::Core::builder()
            .with_config(&core::Config::for_dev_directory(temp.path()))
            .with_mock_resolver(|r| r.set_repo(repo.clone()))
            .build();

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

        let temp = TempDir::new("gt-commands-branch").unwrap();

        let core = core::Core::builder()
            .with_config(&core::Config::for_dev_directory(temp.path()))
            .with_mock_resolver(|r| {
                r.set_repo(Repo::new(
                    "example.com/test/cmd-branch",
                    temp.path().to_path_buf(),
                ))
            })
            .build();

        let args: ArgMatches = cmd.app().get_matches_from(vec!["branch", "feature/test"]);

        match cmd.run(&core, &args).await {
            Ok(_) => panic!("This command should not have succeeded"),
            _ => {}
        }
    }
}
