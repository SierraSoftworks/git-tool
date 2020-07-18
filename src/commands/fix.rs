use super::super::errors;
use super::*;
use crate::{search, tasks::*};
use clap::{App, Arg, ArgMatches, SubCommand};

pub struct FixCommand {}

impl Command for FixCommand {
    fn name(&self) -> String {
        String::from("fix")
    }

    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name().as_str())
            .version("1.0")
            .about("fixes the remote configuration for a repository")
            .alias("i")
            .after_help("Updates the remote configuration for a repository to match its directory location.")
            .arg(Arg::with_name("repo")
                    .help("The name of the repository to fix.")
                    .index(1))
            .arg(Arg::with_name("all")
                .long("all")
                .short("a")
                .help("apply fixes to all matched repositories"))
            .arg(Arg::with_name("no-create-remote")
                .long("no-create-remote")
                .short("R")
                .help("prevent the creation of a remote repository (on supported services)"))
    }
}

#[async_trait]
impl<C: Core> CommandRunnable<C> for FixCommand {
    async fn run<'a>(&self, core: &C, matches: &ArgMatches<'a>) -> Result<i32, errors::Error> {
        let tasks = sequence![
            GitRemote { name: "origin" },
            CreateRemote {
                enabled: !matches.is_present("no-create-remote")
            }
        ];

        match matches.is_present("all") {
            true => {
                let mut output = core.output().writer();
                let filter = match matches.value_of("repo") {
                    Some(name) => name,
                    None => "",
                };

                let repos = core.resolver().get_repos()?;
                for repo in repos.iter().filter(|r| {
                    search::matches(&format!("{}/{}", r.get_domain(), r.get_full_name()), filter)
                }) {
                    writeln!(
                        output,
                        "Fixing {}/{}",
                        repo.get_domain(),
                        repo.get_full_name()
                    )?;
                    tasks.apply_repo(core, repo).await?;
                }
            }
            false => {
                let repo = match matches.value_of("repo") {
                    Some(name) => core.resolver().get_best_repo(name)?,
                    None => core.resolver().get_current_repo()?,
                };

                tasks.apply_repo(core, &repo).await?;
            }
        }

        Ok(0)
    }

    async fn complete<'a>(&self, core: &C, completer: &Completer, _matches: &ArgMatches<'a>) {
        completer.offer("--all");
        completer.offer("--no-create-remote");
        completer.offer_many(core.config().get_aliases().map(|(a, _)| a));

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

    #[tokio::test]
    async fn run() {
        let cmd = FixCommand {};

        let args = cmd.app().get_matches_from(vec!["fix", "repo"]);

        let temp = tempfile::tempdir().unwrap();
        let cfg = Config::for_dev_directory(temp.path());

        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .with_mock_keychain(|s| {
                s.set_token("github.com", "test_token").unwrap();
            })
            .with_http_connector(
                crate::online::service::github::mocks::NewRepoSuccessFlow::default(),
            )
            .with_mock_resolver(|r| {
                r.set_repo(Repo::new(
                    "github.com/exampleB/test",
                    temp.path().to_path_buf(),
                ));
            })
            .build();

        // Prep the repo
        sequence![GitInit {}, GitRemote { name: "origin" }]
            .apply_repo(
                &core,
                &Repo::new("github.com/exampleA/test", temp.path().to_path_buf()),
            )
            .await
            .unwrap();

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
        }
    }
}
