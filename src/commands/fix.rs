use super::*;
use crate::{search, tasks::*};
use clap::Arg;
use tracing_batteries::prelude::*;

pub struct FixCommand;
crate::command!(FixCommand);

#[async_trait]
impl CommandRunnable for FixCommand {
    fn name(&self) -> String {
        String::from("fix")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("fixes the remote configuration for a repository")
            .long_about("Updates the remote configuration for a repository to match its directory location.")
            .arg(Arg::new("repo")
                    .help("The name of the repository to fix.")
                    .index(1))
            .arg(Arg::new("all")
                .long("all")
                .short('a')
                .help("apply fixes to all matched repositories")
                .action(clap::ArgAction::SetTrue))
            .arg(Arg::new("no-create-remote")
                .long("no-create-remote")
                .short('R')
                .help("prevent the creation of a remote repository (on supported services)")
                .action(clap::ArgAction::SetTrue))
    }

    #[tracing::instrument(name = "gt fix", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let tasks = sequence![
            GitRemote { name: "origin" },
            CreateRemote {
                enabled: !matches.get_flag("no-create-remote")
            }
        ];

        match matches.get_flag("all") {
            true => {
                let filter = matches
                    .get_one::<String>("repo")
                    .map(|s| s.as_str())
                    .unwrap_or("");

                let repos = core.resolver().get_repos()?;
                for repo in search::best_matches_by(filter, repos.iter(), |r| {
                    format!("{}:{}", &r.service, r.get_full_name())
                }) {
                    writeln!(
                        core.output(),
                        "Fixing {}/{}",
                        &repo.service,
                        repo.get_full_name()
                    )?;
                    tasks.apply_repo(core, repo).await?;
                }
            }
            false => {
                let repo = match matches.get_one::<String>("repo") {
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
        completer.offer_aliases(core);
        completer.offer("--all");
        completer.offer("--no-create-remote");
        completer.offer_repos(core);
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use super::*;
    use crate::core::*;

    #[tokio::test]
    async fn run() {
        let cmd = FixCommand {};

        let args = cmd.app().get_matches_from(vec!["fix", "repo"]);

        let temp = tempfile::tempdir().unwrap();

        let temp_path = temp.path().to_path_buf();
        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_null_console()
            .with_mock_http_client(online::service::github::mocks::repo_created("exampleB"))
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(eq("gh"))
                    .returning(|_| Ok("test_token".into()));
            })
            .with_mock_resolver(|mock| {
                let temp_path = temp_path.clone();
                mock.expect_get_best_repo()
                    .with(eq("repo"))
                    .returning(move |_| Ok(Repo::new("gh:exampleB/test", temp_path.clone())));
            })
            .build();

        // Prep the repo
        sequence![GitInit {}, GitRemote { name: "origin" }]
            .apply_repo(
                &core,
                &Repo::new("gh:exampleA/test", core.config().get_dev_directory().into()),
            )
            .await
            .unwrap();

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }
    }
}
