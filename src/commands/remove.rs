use super::*;
use crate::engine::Target;
use clap::Arg;
use tracing_batteries::prelude::*;

pub struct RemoveCommand;
crate::command!(RemoveCommand);

#[async_trait]
impl CommandRunnable for RemoveCommand {
    fn name(&self) -> String {
        String::from("remove")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .visible_aliases(["rm"])
            .about("removes a repository from your local machine")
            .long_about("This command will remove the specified repository from your local machine. It requires that the repository name be provided in fully-qualified form.")
            .arg(Arg::new("repo")
                    .help("The name of the repository to open.")
                    .index(1)
                .required(true))
    }

    #[tracing::instrument(name = "gt remove", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, human_errors::Error> {
        let repo_name = matches
            .get_one::<String>("repo")
            .ok_or_else(|| {
                human_errors::user(
                    "No repository name was provided.",
                    &["Provide the name of the repository you wish to remove."],
                )
            })?
            .parse()?;

        let repo = core.resolver().get_best_repo(&repo_name)?;

        if repo.exists()
            && let Err(err) = std::fs::remove_dir_all(repo.get_path())
        {
            return Err(human_errors::wrap_user(
                err,
                "Could not remove the repository directory due to an error.",
                &["Make sure you have the correct permissions to remove the directory."],
            ));
        }

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt remove",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer_aliases(core);
        completer.offer("--create");
        completer.offer("--no-create-remote");
        completer.offer_apps(core);
        completer.offer_repos(core);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::*;
    use mockall::predicate::eq;

    #[tokio::test]
    async fn run() {
        let cmd = RemoveCommand {};

        let args = cmd.app().get_matches_from(vec!["remove", "repo"]);

        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("repo")).expect("the test repo should be created");

        let temp_path = temp.path().to_owned();
        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_mock_resolver(|mock| {
                let temp_path = temp_path.clone();
                let identifier: Identifier = "repo".parse().unwrap();
                mock.expect_get_best_repo()
                    .with(eq(identifier))
                    .times(1)
                    .returning(move |_| {
                        Ok(Repo::new("gh:git-fixtures/basic", temp_path.join("repo")))
                    });
            })
            .build();
        cmd.assert_run_successful(&core, &args).await;

        assert!(
            !temp.path().join("repo").exists(),
            "the repo should be removed"
        );
    }
}
