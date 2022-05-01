use super::Command;
use super::*;
use crate::core::Target;
use clap::{Arg, ArgMatches};

pub struct RemoveCommand {}

impl Command for RemoveCommand {
    fn name(&self) -> String {
        String::from("remove")
    }

    fn app<'a>(&self) -> clap::Command<'a> {
        clap::Command::new(self.name().as_str())
            .version("1.0")
            .visible_aliases(&["rm"])
            .about("removes a repository from your local machine")
            .long_about("This command will remove the specified repository from your local machine. It requires that the repository name be provided in fully-qualified form.")
            .arg(Arg::new("repo")
                    .help("The name of the repository to open.")
                    .index(1)
                .required(true))
    }
}

#[async_trait]
impl CommandRunnable for RemoveCommand {
    #[tracing::instrument(name = "gt remove", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let repo_name = matches.value_of("repo").ok_or_else(|| {
            errors::user(
                "No repository name was provided.",
                "Provide the name of the repository you wish to remove.",
            )
        })?;

        let repo = core.resolver().get_best_repo(repo_name)?;

        if repo.exists() {
            if let Err(err) = std::fs::remove_dir_all(repo.get_path()) {
                return Err(errors::user_with_internal(
                    "Could not remove the repository directory due to an error.",
                    "Make sure you have the correct permissions to remove the directory.",
                    err,
                ));
            }
        }

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt remove",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer("--create");
        completer.offer("--no-create-remote");
        completer.offer_many(core.config().get_aliases().map(|(a, _)| a));
        completer.offer_many(core.config().get_apps().map(|a| a.get_name()));

        let default_svc = core
            .config()
            .get_default_service()
            .map(|s| s.name.clone())
            .unwrap_or_default();

        if let Ok(repos) = core.resolver().get_repos() {
            completer.offer_many(
                repos
                    .iter()
                    .filter(|r| r.service == default_svc)
                    .map(|r| r.get_full_name()),
            );
            completer.offer_many(
                repos
                    .iter()
                    .map(|r| format!("{}:{}", r.service, r.get_full_name())),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use mocktopus::mocking::*;

    #[tokio::test]
    async fn run() {
        let cmd = RemoveCommand {};

        let args = cmd.app().get_matches_from(vec!["remove", "repo"]);

        let temp = tempfile::tempdir().unwrap();
        let cfg = Config::for_dev_directory(temp.path());

        std::fs::create_dir_all(temp.path().join("repo")).expect("the test repo should be created");

        let core = Core::builder().with_config(&cfg).build();

        let temp_path = temp.path().to_owned();
        Resolver::get_best_repo.mock_safe(move |_, name| {
            assert_eq!(
                name, "repo",
                "it should be called with the name of the repo to be removed"
            );

            MockResult::Return(Ok(Repo::new(
                "gh:git-fixtures/basic",
                temp_path.join("repo").into(),
            )))
        });

        match cmd.run(&core, &args).await {
            Ok(status) => {
                assert_eq!(status, 0, "the command should exit successfully");
            }
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            !temp.path().join("repo").exists(),
            "the repo should be removed"
        );
    }
}
