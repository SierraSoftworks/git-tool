use super::async_trait;
use super::online::gitignore;
use super::*;
use clap::{value_parser, Arg};
use tracing_batteries::prelude::*;

pub struct IgnoreCommand;
crate::command!(IgnoreCommand);

#[async_trait]
impl CommandRunnable for IgnoreCommand {
    fn name(&self) -> String {
        String::from("ignore")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .visible_alias("gitignore")
            .about("generates a .gitignore file for the provided languages")
            .long_about("This will manage your .gitignore file using the gitignore.io API to add and update languages.")
            .arg(Arg::new("path")
                    .long("path")
                    .help("The path to the .gitignore file you wish to update.")
                    .default_value(".gitignore")
                    .value_name("GITIGNORE")
                    .action(clap::ArgAction::Set)
                    .value_parser(value_parser!(std::path::PathBuf)))
            .arg(Arg::new("language")
                    .help("The name of a language which should be added to your .gitignore file.")
                    .action(clap::ArgAction::Append)
                    .index(1))
    }

    #[tracing::instrument(name = "gt ignore", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        match matches.get_many::<String>("language") {
            None => {
                let languages = gitignore::list(core).await?;

                for lang in languages {
                    writeln!(core.output(), "{lang}")?;
                }
            }
            Some(languages) => {
                let mut original_content: String = String::default();

                let ignore_path = matches
                    .get_one::<std::path::PathBuf>("path")
                    .cloned()
                    .unwrap_or_else(|| std::path::PathBuf::from(".gitignore"));

                if let Ok(content) = tokio::fs::read_to_string(&ignore_path).await {
                    original_content = content;
                }

                let content = gitignore::add_or_update(
                    core,
                    original_content.as_str(),
                    languages.map(|s| s.as_str()).collect(),
                )
                .await?;

                tokio::fs::write(&ignore_path, content).await.map_err(|err| errors::user_with_internal(
                    &format!("Could not write to your '{}' file due to an OS-level error.", ignore_path.display()),
                    "Check that Git-Tool has permission to write to your .gitignore file and try again.",
                    err
                ))?;
            }
        }

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt ignore",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        if let Ok(langs) = online::gitignore::list(core).await {
            completer.offer_many(langs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::core::Config;
    use super::*;

    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn run_no_args() {
        let cfg = Config::from_str("directory: /dev").unwrap();
        let console = crate::console::mock();
        let core = Core::builder()
            .with_config(cfg)
            .with_console(console.clone())
            .build();

        let cmd = IgnoreCommand {};
        let args = cmd.app().get_matches_from(["ignore"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            console.to_string().contains("visualstudio"),
            "the ignore list should be printed"
        );
    }
}
