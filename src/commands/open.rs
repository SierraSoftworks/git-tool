use super::Command;
use super::*;
use crate::core::Target;
use crate::tasks::*;
use clap::{App, Arg, ArgMatches};

pub struct OpenCommand {}

impl Command for OpenCommand {
    fn name(&self) -> String {
        String::from("open")
    }

    fn app<'a>(&self) -> App<'a> {
        App::new(self.name().as_str())
            .version("1.0")
            .visible_aliases(&vec!["o", "run"])
            .about("opens a repository using an application defined in your config")
            .long_about("This command launches an application defined in your configuration within the specified repository. You can specify any combination of alias, app and repo. Aliases take precedence over repos, which take precedence over apps. When specifying an app, it should appear before the repo/alias parameter. If you are already inside a repository, you can specify only an app and it will launch in the context of the current repo.
            
New applications can be configured either by making changes to your configuration, or by using the `git-tool config add` command to install them from the GitHub registry. For example, you can use `gt config add apps/bash` to configure `bash` as an available app.")
            .arg(Arg::new("app")
                    .about("The name of the application to launch.")
                    .index(1))
            .arg(Arg::new("repo")
                    .about("The name of the repository to open.")
                    .index(2))
            .arg(Arg::new("create")
                    .long("create")
                    .short('c')
                    .about("create the repository if it does not exist."))
            .arg(Arg::new("no-create-remote")
                    .long("no-create-remote")
                    .short('R')
                    .about("prevent the creation of a remote repository (on supported services)"))
    }
}

#[async_trait]
impl<C: Core> CommandRunnable<C> for OpenCommand {
    async fn run(&self, core: &C, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let (app, repo) = match helpers::get_launch_app(core, matches.value_of("app"), matches.value_of("repo")) {
            helpers::LaunchTarget::AppAndTarget(app, target) => {
                (app, core.resolver().get_best_repo(target)?)
            },
            helpers::LaunchTarget::App(app) => {
                (app, core.resolver().get_current_repo()?)
            },
            helpers::LaunchTarget::Target(target) => {
                let app = core.config().get_default_app().ok_or(errors::user(
                    "No default application available.",
                    "Make sure that you add an app to your config file using 'git-tool config add apps/bash' or similar."))?;

                (app, core.resolver().get_best_repo(target)?)
            },
            helpers::LaunchTarget::Err(err) => {
                return Err(err)
            },
            helpers::LaunchTarget::None => {
                return Err(errors::user(
                    "You did not specify the name of a repository to use.",
                    "Remember to specify a repository name like this: 'git-tool open github.com/sierrasoftworks/git-tool'."))
            }
        };

        if !repo.exists() {
            match sequence![GitClone {}].apply_repo(core, &repo).await {
                Ok(()) => {}
                Err(_) if matches.is_present("create") => {
                    sequence![
                        GitInit {},
                        GitRemote { name: "origin" },
                        GitCheckout { branch: "main" },
                        CreateRemote {
                            enabled: !matches.is_present("no-create-remote")
                        }
                    ]
                    .apply_repo(core, &repo)
                    .await?;
                }
                Err(e) => return Err(e),
            }
        }

        let status = core.launcher().run(app, &repo).await?;
        Ok(status)
    }

    async fn complete(&self, core: &C, completer: &Completer, _matches: &ArgMatches) {
        completer.offer("--create");
        completer.offer("--no-create-remote");
        completer.offer_many(core.config().get_aliases().map(|(a, _)| a));
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
    use mocktopus::mocking::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn run() {
        let cmd = OpenCommand {};

        let args = cmd.app().get_matches_from(vec!["open", "test-app", "repo"]);

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
        let core = CoreBuilder::default().with_config(&cfg).build();

        let temp_path = temp.path().to_owned();
        super::Resolver::get_best_repo.mock_safe(move |_, name| {
            assert_eq!(
                name, "repo",
                "it should be called with the name of the repo to be cloned"
            );

            MockResult::Return(Ok(Repo::new(
                "github.com/git-fixtures/basic",
                temp_path.join("repo").into(),
            )))
        });

        crate::core::Launcher::run.mock_safe(move |_, app, target| {
            assert_eq!(
                app.get_name(),
                "test-app",
                "it should launch the correct app"
            );

            assert_eq!(
                target.get_path(),
                temp.path().join("repo"),
                "the target should be launched in the correct directory"
            );

            MockResult::Return(Box::pin(async move { Ok(5) }))
        });

        match cmd.run(&core, &args).await {
            Ok(status) => {
                assert_eq!(
                    status, 5,
                    "the status code of the child app should be forwarded"
                );
            }
            Err(err) => panic!(err.message()),
        }
    }
}
