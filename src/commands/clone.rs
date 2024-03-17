use super::*;
use crate::core::Target;
use crate::tasks::*;
use clap::Arg;

pub struct CloneCommand;

crate::command!(CloneCommand);

#[async_trait]
impl CommandRunnable for CloneCommand {
    fn name(&self) -> String {
        String::from("clone")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("clones a repository")
            .long_about("This command clones a repository if it does not already exist in your dev directory. It works similarly to the `gt open` command, however it will not launch an application in the repository upon completion.")
            .arg(Arg::new("repo")
                    .help("The name of the repository to open.")
                    .required(true)
                    .index(1))
    }

    #[tracing::instrument(name = "gt clone", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let repo_name = matches.get_one::<String>("repo").ok_or_else(|| errors::user(
            "You didn't specify the repository you wanted to clone.",
            "Remember to specify a repository name like this: 'git-tool clone gh:sierrasoftworks/git-tool'."))?;

        let repo = core.resolver().get_best_repo(repo_name)?;

        if !repo.exists() {
            match sequence![GitClone {}].apply_repo(core, &repo).await {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
        }

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt clone",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
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
                    .map(|r| format!("{}:{}", &r.service, r.get_full_name())),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use tempfile::tempdir;

    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn run() {
        let cmd = CloneCommand {};

        let args = cmd.app().get_matches_from(vec!["clone", "repo"]);

        let cfg = Config::from_str(
            "
directory: /dev

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'

features:
  http_transport: true
",
        )
        .unwrap();

        let temp = tempdir().unwrap();

        let temp_path = temp.path().to_path_buf();
        let core = Core::builder()
            .with_config(cfg)
            .with_mock_launcher(|mock| {
                mock.expect_run().never();
            })
            .with_mock_resolver(|mock| {
                let temp_path = temp_path.clone();
                mock.expect_get_best_repo()
                    .once()
                    .with(mockall::predicate::eq("repo"))
                    .returning(move |_| {
                        Ok(Repo::new("gh:git-fixtures/basic", temp_path.join("repo")))
                    });
            })
            .build();

        match cmd.run(&core, &args).await {
            Ok(status) => {
                assert_eq!(status, 0);
            }
            Err(err) => panic!("{}", err.message()),
        }
    }
}
