use super::core::Target;
use super::*;
use clap::Arg;
use tracing_batteries::prelude::*;

pub struct InfoCommand;
crate::command!(InfoCommand);

#[async_trait]
impl CommandRunnable for InfoCommand {
    fn name(&self) -> String {
        String::from("info")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("gets the details of a specific repository")
            .visible_alias("i")
            .long_about("Gets the details of a specific repository, either the currently open one or one provided by its name or alias.")
            .arg(Arg::new("repo")
                    .help("The name of the repository to get information about.")
                    .index(1))
    }

    #[tracing::instrument(name = "gt info", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let mut output = core.output();
        let repo = match matches.get_one::<String>("repo") {
            Some(name) => core.resolver().get_best_repo(&name.parse()?)?,
            None => core.resolver().get_current_repo()?,
        };

        writeln!(output, "Name:      {}", repo.get_name())?;
        writeln!(output, "Namespace: {}", &repo.namespace)?;
        writeln!(output, "Service:   {}", &repo.service)?;
        writeln!(output, "Path:      {}", repo.path.display())?;

        if let Ok(svc) = core.config().get_service(&repo.service) {
            writeln!(output)?;
            writeln!(output, "URLs:")?;
            writeln!(output, " - Website:  {}", svc.get_website(&repo)?)?;
            writeln!(output, " - Git:  {}", svc.get_git_url(&repo)?)?;
        }

        Ok(0)
    }

    #[tracing::instrument(name = "gt complete -- gt info", skip(self, core, completer, _matches))]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer_aliases(core);
        completer.offer_repos(core);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::eq;

    use super::*;
    use crate::{console::MockConsoleProvider, core::*};

    #[tokio::test]
    async fn run() {
        let cmd = InfoCommand {};

        let args = cmd.app().get_matches_from(vec!["info", "repo"]);

        let cfg = Config::from_str("directory: /dev").unwrap();

        let console = Arc::new(MockConsoleProvider::new());
        let core = Core::builder()
            .with_config(cfg)
            .with_console(console.clone())
            .with_mock_resolver(|mock| {
                let identifier: Identifier = "repo".parse().unwrap();
                mock.expect_get_best_repo()
                    .with(eq(identifier))
                    .returning(|_| {
                        Ok(Repo::new(
                            "gh:sierrasoftworks/git-tool",
                            std::path::PathBuf::from("/test"),
                        ))
                    });
            })
            .build();
        cmd.assert_run_successful(&core, &args).await;
    }
}
