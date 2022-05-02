use super::async_trait;
use super::online::gitignore;
use super::Command;
use super::*;
use clap::{Arg, ArgMatches};

pub struct IgnoreCommand {}

impl Command for IgnoreCommand {
    fn name(&self) -> String {
        String::from("ignore")
    }

    fn app<'a>(&self) -> clap::Command<'a> {
        clap::Command::new(self.name().as_str())
            .version("1.0")
            .visible_alias("gitignore")
            .about("generates a .gitignore file for the provided languages")
            .long_about("This will manage your .gitignore file using the gitignore.io API to add and update languages.")
            .arg(Arg::new("path")
                    .long("path")
                    .help("The path to the .gitignore file you wish to update.")
                    .default_value(".gitignore")
                    .value_name("GITIGNORE")
                    .takes_value(true))
            .arg(Arg::new("language")
                    .forbid_empty_values(true)
                    .help("The name of a language which should be added to your .gitignore file.")
                    .multiple_values(true)
                    .index(1))
    }
}

#[async_trait]
impl CommandRunnable for IgnoreCommand {
    #[tracing::instrument(name = "gt ignore", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let mut output = core.output();

        match matches.values_of("language") {
            None => {
                let languages = gitignore::list(core).await?;

                for lang in languages {
                    writeln!(output, "{}", lang)?;
                }
            }
            Some(languages) => {
                let mut original_content: String = String::default();

                let ignore_path =
                    std::path::PathBuf::from(matches.value_of("path").unwrap_or(".gitignore"));

                if let Ok(content) = tokio::fs::read_to_string(&ignore_path).await {
                    original_content = content;
                }

                let content =
                    gitignore::add_or_update(core, original_content.as_str(), languages.collect())
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
    async fn run_no_args() {
        let cfg = Config::from_str("directory: /dev").unwrap();
        let core = Core::builder().with_config(&cfg).build();

        let output = crate::console::output::mock();

        let cmd = IgnoreCommand {};
        let args = cmd.app().get_matches_from(&["ignore"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            output.to_string().contains("visualstudio"),
            "the ignore list should be printed"
        );
    }
}
