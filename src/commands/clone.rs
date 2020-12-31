use super::Command;
use super::*;
use crate::core::Target;
use crate::tasks::*;
use clap::{App, Arg, ArgMatches};

pub struct CloneCommand {}

impl Command for CloneCommand {
    fn name(&self) -> String {
        String::from("clone")
    }

    fn app<'a>(&self) -> App<'a> {
        App::new(self.name().as_str())
            .version("1.0")
            .about("clones a repository")
            .long_about("This command clones a repository if it does not already exist in your dev directory. It works similarly to the `gt open` command, however it will not launch an application in the repository upon completion.")
            .arg(Arg::new("repo")
                    .about("The name of the repository to open.")
                    .required(true)
                    .index(1))
    }
}

#[async_trait]
impl<C: Core> CommandRunnable<C> for CloneCommand {
    async fn run(&self, core: &C, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let repo_name = matches.value_of("repo").ok_or(errors::user(
            "You didn't specify the repository you wanted to clone.",
            "Remember to specify a repository name like this: 'git-tool clone github.com/sierrasoftworks/git-tool'."))?;

        let repo = core.resolver().get_best_repo(repo_name)?;

        if !repo.exists() {
            match sequence![GitClone {}].apply_repo(core, &repo).await {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
        }

        Ok(0)
    }

    async fn complete(&self, core: &C, completer: &Completer, _matches: &ArgMatches) {
        completer.offer_many(core.config().get_apps().map(|a| a.get_name()));

        let default_svc = core
            .config()
            .get_default_service()
            .map(|s| s.get_domain())
            .unwrap_or_default();

        match core.resolver().get_repos() {
            Ok(repos) => {
                completer.offer_many(
                    repos
                        .iter()
                        .filter(|r| r.get_domain() == default_svc)
                        .map(|r| r.get_full_name()),
                );
                completer.offer_many(
                    repos
                        .iter()
                        .map(|r| format!("{}/{}", r.get_domain(), r.get_full_name())),
                );
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::core::{Config, CoreBuilder, Repo};
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
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
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_launcher(|l| {})
            .with_mock_resolver(|r| {
                r.set_repo(Repo::new(
                    "github.com/git-fixtures/basic",
                    temp.path().join("repo").into(),
                ));
            })
            .build();

        match cmd.run(&core, &args).await {
            Ok(status) => {
                assert_eq!(status, 0);
                let launches = core.launcher().launches.lock().await;
                assert!(launches.len() == 0);
            }
            Err(err) => panic!(err.message()),
        }
    }
}
