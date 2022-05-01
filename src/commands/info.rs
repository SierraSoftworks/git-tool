use super::super::errors;
use super::core::Target;
use super::*;
use clap::{Arg, ArgMatches};

pub struct InfoCommand {}

impl Command for InfoCommand {
    fn name(&self) -> String {
        String::from("info")
    }

    fn app<'a>(&self) -> clap::Command<'a> {
        clap::Command::new(self.name().as_str())
            .version("1.0")
            .about("gets the details of a specific repository")
            .visible_alias("i")
            .long_about("Gets the details of a specific repository, either the currently open one or one provided by its name or alias.")
            .arg(Arg::new("repo")
                    .help("The name of the repository to get information about.")
                    .index(1))
    }
}

#[async_trait]
impl CommandRunnable for InfoCommand {
    #[tracing::instrument(name = "gt info", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let mut output = core.output();
        let repo = match matches.value_of("repo") {
            Some(name) => core.resolver().get_best_repo(name)?,
            None => core.resolver().get_current_repo()?,
        };

        writeln!(output, "Name:      {}", repo.get_name())?;
        writeln!(output, "Namespace: {}", &repo.namespace)?;
        writeln!(output, "Service:   {}", &repo.service)?;
        writeln!(output, "Path:      {}", repo.path.display())?;

        match core.config().get_service(&repo.service) {
            Some(svc) => {
                writeln!(output)?;
                writeln!(output, "URLs:")?;
                writeln!(output, " - Website:  {}", svc.get_website(&repo)?)?;
                writeln!(output, " - Git:  {}", svc.get_git_url(&repo)?)?;
            }
            None => {}
        }

        Ok(0)
    }

    #[tracing::instrument(name = "gt complete -- gt info", skip(self, core, completer, _matches))]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
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
        let cmd = InfoCommand {};

        let args = cmd.app().get_matches_from(vec!["info", "repo"]);

        let cfg = Config::from_str("directory: /dev").unwrap();

        Resolver::get_best_repo.mock_safe(move |_, name| {
            assert_eq!(
                name, "repo",
                "it should be called with the name of the repo to be cloned"
            );

            MockResult::Return(Ok(Repo::new(
                "gh:sierrasoftworks/git-tool",
                std::path::PathBuf::from("/test"),
            )))
        });

        let core = Core::builder().with_config(&cfg).build();

        crate::console::output::mock();

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }
    }
}
