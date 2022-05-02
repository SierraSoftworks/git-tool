use super::super::errors;
use super::*;
use crate::{search, tasks::*};
use clap::{Arg, ArgMatches};

pub struct FixCommand {}

impl Command for FixCommand {
    fn name(&self) -> String {
        String::from("fix")
    }

    fn app<'a>(&self) -> clap::Command<'a> {
        clap::Command::new(self.name().as_str())
            .version("1.0")
            .about("fixes the remote configuration for a repository")
            .visible_alias("i")
            .long_about("Updates the remote configuration for a repository to match its directory location.")
            .arg(Arg::new("repo")
                    .help("The name of the repository to fix.")
                    .index(1))
            .arg(Arg::new("all")
                .long("all")
                .short('a')
                .help("apply fixes to all matched repositories"))
            .arg(Arg::new("no-create-remote")
                .long("no-create-remote")
                .short('R')
                .help("prevent the creation of a remote repository (on supported services)"))
    }
}

#[async_trait]
impl CommandRunnable for FixCommand {
    #[tracing::instrument(name = "gt fix", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let tasks = sequence![
            GitRemote { name: "origin" },
            CreateRemote {
                enabled: !matches.is_present("no-create-remote")
            }
        ];

        match matches.is_present("all") {
            true => {
                let mut output = core.output();
                let filter = matches.value_of("repo").unwrap_or("");

                let repos = core.resolver().get_repos()?;
                for repo in search::best_matches_by(filter, repos.iter(), |r| {
                    format!("{}:{}", &r.service, r.get_full_name())
                }) {
                    writeln!(output, "Fixing {}/{}", &repo.service, repo.get_full_name())?;
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

    #[tracing::instrument(name = "gt complete -- gt fix", skip(self, core, completer, _matches))]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer("--all");
        completer.offer("--no-create-remote");
        completer.offer_many(core.config().get_aliases().map(|(a, _)| a));

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
    use mocktopus::mocking::*;

    #[tokio::test]
    async fn run() {
        let cmd = FixCommand {};

        let args = cmd.app().get_matches_from(vec!["fix", "repo"]);

        let temp = tempfile::tempdir().unwrap();
        let cfg = Config::for_dev_directory(temp.path());

        Resolver::get_best_repo.mock_safe(move |_, name| {
            assert_eq!(
                name, "repo",
                "it should be called with the name of the repo to be cloned"
            );

            MockResult::Return(Ok(Repo::new("gh:exampleB/test", temp.path().into())))
        });

        #[cfg(feature = "auth")]
        KeyChain::get_token.mock_safe(|_, token| {
            assert_eq!(token, "gh", "the correct token should be requested");
            MockResult::Return(Ok("test_token".into()))
        });

        crate::online::service::github::mocks::repo_created("exampleB");

        let core = Core::builder().with_config(&cfg).build();

        crate::console::output::mock();

        // Prep the repo
        sequence![GitInit {}, GitRemote { name: "origin" }]
            .apply_repo(
                &core,
                &Repo::new("gh:exampleA/test", cfg.get_dev_directory().into()),
            )
            .await
            .unwrap();

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }
    }
}
