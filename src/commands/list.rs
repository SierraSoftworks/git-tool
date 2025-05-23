use super::*;
use crate::engine::Target;
use crate::search;
use clap::Arg;
use tracing_batteries::prelude::*;

pub struct ListCommand;
crate::command!(ListCommand);

#[async_trait]
impl CommandRunnable for ListCommand {
    fn name(&self) -> String {
        String::from("list")
    }
    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .visible_aliases(["ls", "ll"])
            .about("list your repositories")
            .after_help("Gets the list of repositories managed by Git-Tool. These repositories can be opened using the `git-tool open` command.")
            .arg(Arg::new("filter")
                .help("A filter which limits the repositories that are returned.")
                .index(1))
            .arg(Arg::new("quiet")
                .long("quiet")
                .short('q')
                .help("Prints only the name of the repository.")
                .action(clap::ArgAction::SetTrue))
            .arg(Arg::new("full")
                .long("full")
                .help("Prints detailed information about each repository.")
                .conflicts_with("quiet")
                .action(clap::ArgAction::SetTrue))
    }

    #[tracing::instrument(name = "gt list", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, engine::Error>
where {
        let mut output = core.output();

        let filter = matches
            .get_one::<String>("filter")
            .map(|s| s.as_str())
            .unwrap_or("");

        let quiet = matches.get_flag("quiet");
        let full = matches.get_flag("full");

        let repos = core.resolver().get_repos()?;

        let mut first = true;
        for repo in search::best_matches_by(filter, repos.iter(), |r| {
            format!("{}:{}", &r.service, r.get_full_name())
        }) {
            if quiet {
                writeln!(output, "{}:{}", &repo.service, repo.get_full_name())?;
            } else if full {
                if !first {
                    writeln!(output, "---")?;
                }

                writeln!(
                    output,
                    "
Name:           {name}
Namespace:      {namespace}
Service:        {domain}
Path:           {path}",
                    name = repo.get_name(),
                    namespace = &repo.namespace,
                    domain = &repo.service,
                    path = repo.path.display()
                )?;

                if let Ok(svc) = core.config().get_service(&repo.service) {
                    writeln!(
                        output,
                        "
URLs:
  - Website:    {website}
  - Git:    {git}",
                        website = svc.get_website(repo)?,
                        git = svc.get_git_url(repo)?,
                    )?
                };
            } else {
                match core.config().get_service(&repo.service) {
                    Ok(svc) => writeln!(
                        output,
                        "{}:{} ({})",
                        &repo.service,
                        repo.get_full_name(),
                        svc.get_website(repo)?
                    )?,
                    Err(_) => writeln!(output, "{}:{}", &repo.service, repo.get_full_name())?,
                };
            }

            first = false;
        }

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt list",
        skip(self, _core, completer, _matches)
    )]
    async fn complete(&self, _core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer_many(vec!["--quiet", "-q", "--full", "-f"]);
    }
}

#[cfg(test)]
mod tests {
    use super::engine::*;
    use super::*;
    use crate::console::MockConsoleProvider;
    use crate::test::get_dev_dir;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    async fn run_normal() {
        let console = Arc::new(MockConsoleProvider::new());
        let core = Core::builder()
            .with_config_for_dev_directory(get_dev_dir())
            .with_console(console.clone())
            .build();

        let cmd = ListCommand {};

        let args = cmd.app().get_matches_from(vec!["list"]);

        cmd.assert_run_successful(&core, &args).await;

        assert!(
            console
                .to_string()
                .contains("gh:sierrasoftworks/test1 (https://github.com/sierrasoftworks/test1)\n"),
            "the output should contain the repo: {}\ngot: {}",
            "gh:sierrasoftworks/test1 (https://github.com/sierrasoftworks/test1)",
            &console.to_string()
        );
    }

    #[tokio::test]
    async fn run_search_full() {
        let core = Core::builder()
            .with_default_config()
            .with_null_console()
            .with_mock_resolver(|mock| {
                mock.expect_get_repos().returning(|| {
                    Ok(vec![
                        Repo::new("example.com:ns1/a", PathBuf::from("/dev/example.com/ns1/a")),
                        Repo::new("example.com:ns1/b", PathBuf::from("/dev/example.com/ns1/b")),
                        Repo::new("example.com:ns2/c", PathBuf::from("/dev/example.com/ns2/c")),
                    ])
                });
            })
            .build();

        let cmd = ListCommand {};
        let args = cmd.app().get_matches_from(vec!["list", "ns2", "--full"]);

        cmd.assert_run_successful(&core, &args).await;
    }

    #[tokio::test]
    async fn run_search_quiet() {
        let console = Arc::new(MockConsoleProvider::new());
        let core = Core::builder()
            .with_default_config()
            .with_console(console.clone())
            .with_mock_resolver(|mock| {
                mock.expect_get_repos().returning(|| {
                    Ok(vec![
                        Repo::new("example.com:ns1/a", PathBuf::from("/dev/example.com/ns1/a")),
                        Repo::new("example.com:ns1/b", PathBuf::from("/dev/example.com/ns1/b")),
                        Repo::new("example.com:ns2/c", PathBuf::from("/dev/example.com/ns2/c")),
                    ])
                });
            })
            .build();

        let cmd = ListCommand {};
        let args = cmd.app().get_matches_from(vec!["list", "ns1", "--quiet"]);

        cmd.assert_run_successful(&core, &args).await;

        assert!(
            console.to_string().contains("example.com:ns1/a\n"),
            "the output should contain the first match"
        );
        assert!(
            console.to_string().contains("example.com:ns1/b\n"),
            "the output should contain the second match"
        );
    }
}
