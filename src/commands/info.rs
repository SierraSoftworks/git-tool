use super::super::errors;
use super::core::Target;
use super::*;
use clap::{App, Arg, ArgMatches, SubCommand};

pub struct InfoCommand {}

impl Command for InfoCommand {
    fn name(&self) -> String {
        String::from("info")
    }

    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name().as_str())
            .version("1.0")
            .about("gets the details of a specific repository")
            .alias("i")
            .after_help("Gets the details of a specific repository, either the currently open one or one provided by its name or alias.")
            .arg(Arg::with_name("repo")
                    .help("The name of the repository to get information about.")
                    .index(1))
    }
}

#[async_trait]
impl<C: Core> CommandRunnable<C> for InfoCommand {
    async fn run<'a>(&self, core: &C, matches: &ArgMatches<'a>) -> Result<i32, errors::Error> {
        let mut output = core.output().writer();
        let repo = match matches.value_of("repo") {
            Some(name) => core.resolver().get_best_repo(name)?,
            None => core.resolver().get_current_repo()?,
        };

        writeln!(output, "Name:      {}", repo.get_name())?;
        writeln!(output, "Namespace: {}", repo.get_namespace())?;
        writeln!(output, "Service:   {}", repo.get_domain())?;
        writeln!(output, "Path:      {}", repo.get_path().display())?;

        match core.config().get_service(repo.get_domain().as_str()) {
            Some(svc) => {
                writeln!(output, "")?;
                writeln!(output, "URLs:")?;
                writeln!(output, " - Website:  {}", svc.get_website(&repo)?)?;
                writeln!(output, " - Git SSH:  {}", svc.get_git_url(&repo)?)?;
                writeln!(output, " - Git HTTP: {}", svc.get_http_url(&repo)?)?;
            }
            None => {}
        }

        Ok(0)
    }

    async fn complete<'a>(&self, core: &C, completer: &Completer, _matches: &ArgMatches<'a>) {
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
        let cmd = InfoCommand {};

        let args = cmd.app().get_matches_from(vec!["info", "repo"]);

        let cfg = Config::from_str("directory: /dev").unwrap();

        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .with_mock_resolver(|r| {
                r.set_repo(Repo::new(
                    "github.com/sierrasoftworks/git-tool",
                    std::path::PathBuf::from("/test"),
                ));
            })
            .build();

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
        }
    }
}
