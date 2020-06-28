use clap::{App, SubCommand, ArgMatches};
use super::Command;
use super::*;
use super::async_trait;
use online::registry::Registry;

pub struct ConfigCommand {

}

impl Command for ConfigCommand {
    fn name(&self) -> String {
        String::from("config")
    }

    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name().as_str())
            .version("1.0")
            .about("manage your Git-Tool configuration file")
            .help_message("This tool allows you to easily make changes to your Git-Tool config file.")

            .subcommand(SubCommand::with_name("list")
                .version("1.0")
                .alias("ls")
                .about("list available config templates")
                .help_message("Gets the list of config templates which are available through the Git-Tool registry."))
    }
}
    
#[async_trait]
impl<F: FileSource, L: Launcher, R: Resolver> CommandRunnable<F, L, R> for ConfigCommand {
    async fn run<'a>(&self, core: &core::Core<F, L, R>, matches: &ArgMatches<'a>) -> Result<i32, errors::Error> {
        match matches.subcommand() {
            ("list", Some(_args)) => {
                let registry = crate::online::GitHubRegistry::from(core.config.clone());

                let entries = registry.get_entries().await?;
                for entry in entries {
                    println!("{}", entry);
                }
            },
            ("add", Some(_args)) => {
                println!("This has not yet been implemented");
            },
            _ => {
                println!("{}", core.config.to_string()?);
            }
        }

        Ok(0)
    }

    async fn complete<'a>(&self, core: &Core<F, L, R>, completer: &Completer, matches: &ArgMatches<'a>) {
        match matches.subcommand() {
            ("list", _) => {

            },
            ("add", _) => {
                match online::GitHubRegistry::from(core.config.clone()).get_entries().await {
                    Ok(entries) => {
                        completer.offer_many(entries);
                    },
                    _ => {}
                }
            },
            _ => {
                completer.offer_many(vec!["list", "add"]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::{Config};
    use clap::ArgMatches;

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();
        let cfg = Config::from_str("directory: /dev").unwrap();
        let core = Core::builder().with_config(&cfg).build();

        let cmd = ConfigCommand{};

        match cmd.run(&core, &args).await {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }
    }
}