use super::async_trait;
use super::online::gitignore;
use super::Command;
use super::*;
use clap::{App, Arg, ArgMatches};

pub struct IgnoreCommand {}

impl Command for IgnoreCommand {
    fn name(&self) -> String {
        String::from("ignore")
    }

    fn app<'a>(&self) -> App<'a> {
        App::new(self.name().as_str())
            .version("1.0")
            .visible_alias("gitignore")
            .about("generates a .gitignore file for the provided languages")
            .long_about("This will manage your .gitignore file using the gitignore.io API to add and update languages.")
            .arg(Arg::new("path")
                    .long("path")
                    .about("The path to the .gitignore file you wish to update.")
                    .default_value(".gitignore")
                    .value_name("GITIGNORE")
                    .takes_value(true))
            .arg(Arg::new("language")
                    .about("The name of a language which should be added to your .gitignore file.")
                    .multiple(true)
                    .index(1))
    }
}

#[async_trait]
impl<C: Core> CommandRunnable<C> for IgnoreCommand {
    async fn run(&self, core: &C, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let mut output = core.output().writer();

        match matches.occurrences_of("language") {
            0 => {
                let languages = gitignore::list(core).await?;

                for lang in languages {
                    writeln!(output, "{}", lang)?;
                }
            }
            _ => {
                let mut original_content: String = String::default();

                let ignore_path =
                    std::path::PathBuf::from(matches.value_of("path").unwrap_or(".gitignore"));

                if let Ok(content) = tokio::fs::read_to_string(&ignore_path).await {
                    original_content = content;
                }

                let content = gitignore::add_or_update(
                    core,
                    original_content.as_str(),
                    matches.values_of("language").unwrap_or_default().collect(),
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

    async fn complete(&self, core: &C, completer: &Completer, _matches: &ArgMatches) {
        match online::gitignore::list(core).await {
            Ok(langs) => completer.offer_many(langs),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::core::{Config, CoreBuilder};
    use super::*;
    use clap::ArgMatches;

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();
        let cfg = Config::from_str("directory: /dev").unwrap();
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .build();

        let cmd = IgnoreCommand {};

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
        }

        let output = core.output().to_string();
        assert!(
            output.contains("visualstudio"),
            "the ignore list should be printed"
        );
    }
}
